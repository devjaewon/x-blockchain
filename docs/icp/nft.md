# IntertComputer에 NFT 민팅하기

> 이번 문서에서는 Rust와 DIP-721 표준 토큰을 민팅해볼 예정
> - 레퍼런스 1: https://internetcomputer.org/docs/current/samples/nft
> - 레퍼런스 2: https://www.youtube.com/watch?v=1po3udDADp4

## 시작하기 전에

1. NFT를 IC에 배포하는 방법에 대한 트랙, 그리고 배포된 NFT를 지갑인증과 연관하여 관리해볼 트랙 2트랙으로 나눠서 진행해본다
2. 3가지 캐니스터로 나눠서 구현을 해볼 것이다
    - NFT 캐니스터: DIP-721 표준에 맞게 토큰을 관리하는 캐니스터
    - Asset 캐니스터: NFT를 디지털 미디어 형식에 맞게 표현하는 캐니스터
    - Wallet 캐니스터: NFT에 대한 소유권을 관리하는 캐니스터

