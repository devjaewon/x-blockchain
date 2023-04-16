#[macro_use]
extern crate candid;

use std::{
    env,
    process,
    fs::{File},
    path::{Path, PathBuf}, sync::Arc,
};

// https://docs.rs/anyhow/latest/anyhow/
// Rust 에서 에러핸들링을 직관적인 결과로 받아 처리하기 쉽게 돕는 크레이트
use anyhow::{anyhow, Result};

// https://docs.rs/dialoguer/latest/dialoguer/
// cli 환경에서의 사용자와 인터렉션을 돕는 라이브러리
use dialoguer::Confirm;

// https://docs.rs/candid/latest/candid/
// https://docs.rs/candid/latest/candid/types/principal/struct.Principal.html
// icp 에서 개념화 한 일반 ID 타입이다. (향후 확장성을 고려하여 설계되었음)
// 사용자 ID와 캐니스터 ID를 구분하지 않고 범용적으로 사용되며 0~29바이트의 불투명한 이진 blob이다.
use candid::{Principal};

// https://docs.rs/clap/latest/clap/
// Command Line Argument Parser 의 약자로
// cli 로 넘겨준 인자값을 파싱할 수 있고 도움말 등을 생성해주는 라이브러리다.
use clap::Parser;

// https://docs.rs/ic-agent/latest/ic_agent/
use ic_agent::{Agent, Identity, identity::{Secp256k1Identity, BasicIdentity}, agent::http_transport::ReqwestHttpReplicaV2Transport};

#[derive(Parser)]
struct Args {
    /// DIP-721 NFT 컨테이너 캐니스터 ID.
    canister: Principal,
    /// 발급될 NFT의 오너
    #[clap(long)]
    owner: Principal,
    // smart contract에 보낼 파일경로
    #[clap(long)]
    file: Option<PathBuf>,
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

    if args.file.is_none()
        && !Confirm::new()
            .with_prompt("정말로 업로드 대상이 되는 file 인자가 없습니까?")
            .interact()? // ? 는 result 값을 바로 불러오며, 에러 시에 바로 From::from 값을 이용해 변환하는 연산자이다.
    {
        println!("file 인자를 설정하여 다시 실행해주세요.");
        return Ok(());
    }

    let canister = args.canister;
    let owner = args.owner;
    let agent = get_agent().await?;

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
