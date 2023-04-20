#[macro_use]
extern crate candid;
#[macro_use]
extern crate thiserror;

use std::{
    env,
    process,
    fs::{File, self},
    path::{Path, PathBuf}, sync::Arc, collections::HashMap,
};

// https://docs.rs/anyhow/latest/anyhow/
// Rust 에서 에러핸들링을 직관적인 결과로 받아 처리하기 쉽게 돕는 크레이트
use anyhow::{anyhow, Result, Context, bail};

// https://docs.rs/candid/latest/candid/
// https://docs.rs/candid/latest/candid/types/principal/struct.Principal.html
// icp 에서 개념화 한 일반 ID 타입이다. (향후 확장성을 고려하여 설계되었음)
// 사용자 ID와 캐니스터 ID를 구분하지 않고 범용적으로 사용되며 0~29바이트의 불투명한 이진 blob이다.
use candid::Principal;

// https://docs.rs/clap/latest/clap/
// Command Line Argument Parser 의 약자로
// cli 로 넘겨준 인자값을 파싱할 수 있고 도움말 등을 생성해주는 라이브러리다.
use clap::Parser;

// https://docs.rs/ic-agent/latest/ic_agent/
use ic_agent::{Agent, Identity, identity::{Secp256k1Identity, BasicIdentity}, agent::http_transport::ReqwestHttpReplicaV2Transport, AgentError};

use sha2::{Sha256, Digest};
use types::*;

mod types;

#[derive(Parser)]
struct Args {
    /// DIP-721 NFT 컨테이너 캐니스터 ID.
    canister: Principal,
    /// 발급될 NFT의 오너
    #[clap(long)]
    owner: Principal,
    // smart contract에 보낼 파일경로
    #[clap(long)]
    file: PathBuf,
}

#[derive(Deserialize)]
struct DefaultIdentity {
    default: String,
}

// https://docs.rs/tokio/latest/tokio/attr.main.html
// tokio::main 속성을 이용해서 main 함수를 다중 스레드 async로 돌린다.
#[tokio::main]
async fn main() {
    if let Err(e) = rmain().await {
        eprintln!("{}", e);
        process::exit(1);
    }
}

async fn rmain() -> Result<()> {
    let args = Args::parse();

    let canister = args.canister;
    let owner = args.owner;
    let file = args.file;

    let agent = get_agent().await?;

    // DIP-721을 지원하고 있는지 확인하기
    let res = agent
        .query(&canister, "supportedInterfacesDip721")
        .with_arg(Encode!()?)
        .call()
        .await;
    let res = if let Err(AgentError::ReplicaError { reject_code: 3, .. }) = &res {
        res.context(format!(
            "canister {canister} does not appear to be a DIP-721 NFT canister"
        ))?
    } else {
        res?
    };
    let interfaces = Decode!(&res, Vec<InterfaceId>)?;
    if !interfaces.contains(&InterfaceId::Mint) {
        bail!("canister {canister} does not support minting");
    }

    // Agent 인터페이스에 맞는 민팅 Payload 생성
    let mut metadata = HashMap::new();
    metadata.insert("locationType", MetadataVal::Nat8Content(4));

    let data = fs::read(&file)?;
    metadata.insert("contentHash", MetadataVal::BlobContent(Vec::from_iter(Sha256::digest(&data))));

    let content_type = mime_guess::from_path(&file).first().map(|m| format!("{m}"));
    let content_type = content_type.unwrap_or_else(|| String::from("application/octet-stream"));
    metadata.insert("contentType", MetadataVal::TextContent(content_type));

    let metadata_part = MetadataPart {
        purpose: MetadataPurpose::Rendered,
        data: &data,
        key_val_data: metadata,
    };

    // 민팅 요청 보내기
    let res = agent
        .update(&canister, "mintDip721")
        .with_arg(Encode!(&owner, &[metadata_part], &data)?)
        .call_and_wait()
        .await;

    let res = if let Err(AgentError::ReplicaError { reject_code: 3, .. }) = &res {
        res.context(format!("canister {canister} does not support minting"))?
    } else {
        res?
    };

    let MintReceipt { token_id, id } = Decode!(&res, Result<MintReceipt, MintError>)??;

    println!("Successfully minted token {token_id} to {owner} (transaction id {id})");

    return Ok(());
}

async fn get_agent() -> Result<Agent> {
    let user_home = env::var_os("HOME").unwrap();
    let file = File::open(Path::new(&user_home).join(".config/dfx/identity.json"))
        .map_err(|_| anyhow!(".config/dfx/indentity.json 을 읽는데 실패했습니다."))?;
    let default_identity: DefaultIdentity = serde_json::from_reader(file)?;
    let pemfile = PathBuf::from_iter([
        &*user_home,
        ".config/dfx/identity/".as_ref(),
        default_identity.default.as_ref(),
        "identity.pem".as_ref(),
    ]);
    let pem = std::fs::read(pemfile).map_err(|_| {
        anyhow!("default identity 를 불러오는데 실패 했습니다.")
    })?;
    let identity = get_identity(&pem).map_err(|_| {
        anyhow!("default identity 를 불러오는데 실패 했습니다.")
    })?;

    let agent = Agent::builder()
        .with_transport(ReqwestHttpReplicaV2Transport::create("http://localhost:4943")?)
        .with_arc_identity(identity)
        .build()?;

    agent.fetch_root_key().await?;
    Ok(agent)
}

fn get_identity(pem: &[u8]) -> Result<Arc<dyn Identity>> {
    match Secp256k1Identity::from_pem(pem) {
        Ok(id) => Ok(Arc::new(id)),
        Err(e) => match BasicIdentity::from_pem(pem) {
            Ok(id) => Ok(Arc::new(id)),
            Err(_) => Err(e.into()),
        },
    }
}
