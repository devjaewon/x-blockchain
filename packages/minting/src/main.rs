use std::{
    process,
    path::{PathBuf},
};

// https://docs.rs/anyhow/latest/anyhow/
// Rust 에서 에러핸들링을 직관적인 결과로 받아 처리하기 쉽게 돕는 크레이트
use anyhow::{Result};

// https://docs.rs/dialoguer/latest/dialoguer/
// cli 환경에서의 사용자와 인터렉션을 돕는 라이브러리
use dialoguer::Confirm;

// https://docs.rs/candid/latest/candid/
// https://docs.rs/candid/latest/candid/types/principal/struct.Principal.html
// icp 에서 개념화 한 일반 ID 타입이다. (향후 확장성을 고려하여 설계되었음)
// 사용자 ID와 캐니스터 ID를 구분하지 않고 범용적으로 사용되며 0~29바이트의 불투명한 이진 blob이다.
use candid::Principal;

// https://docs.rs/clap/latest/clap/
// Command Line Argument Parser 의 약자로
// cli 로 넘겨준 인자값을 파싱할 수 있고 도움말 등을 생성해주는 라이브러리다.
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// DIP-721 NFT 컨테이너 캐니스터 ID.
    canister: Principal,

    // smart contract에 보낼 파일경로
    #[clap(long)]
    file: Option<PathBuf>,
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

    println!("실행이 완료 되었습니다.");

    return Ok(());
}
