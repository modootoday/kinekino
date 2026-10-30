---
id: evidence:kinekino/20261029114320
title: 카드 Jobs 입력 schema — R112
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 카드 Jobs 입력 schema — R112

카드 operation에 자막 변환 schema가 반환되던 분기를 실제 CardBundleRequest 계약으로 수정했다. API 정책 복제 없이 application/card_bundle/schema.rs의 공통 schema를 선택한다. 명시 카드 host의 실제 CLI 시험에 schema 조회를 추가했고, 이후 동일 여정의 카드 제출/typed 제출·replay·잘못된 frame·idempotency 부재·직접 RPC 거절·기존 자막·원 EOF 종료를 그대로 수행했다. 이번 시험은10개 CLI 요청이며 typed SDK 요청과 polling은 별도다.

입력은 private mirror runtime/development/source-sdk-integration-r1의 input-r112다. inventory SHA256 63B9957608AB0149B3F0B9789B6B002F3906AE32E60DDACA9651A55D9333758E,2358파일; SOURCE.md SHA256 39F6C76E4D32A667C42F214BFF7344454A4D0B6C40A50A1103405B9FF5E1FDF2. AGENTS SHA256 40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C. 이전3파일은 preimage-r111-card-schema에 보존했다.

로그 루트 runtime/verification/source375-datalab-b0-r1-logs:2017 fmt 최초 실패(시험 서식) 보존,2018 cargo fmt 종료0,2019 snapshot 종료0,2020 card feature no-run 종료0,2021 실제 CLI1시험 통과(6.91초,67filtered). fixed-r112의 CLI SHA256 B64CE0CD20BCACD4F18E86759D12A2AA04EF267C1AB40A6F134B28BFB3FEE7C5,Jobs 시험 exe 2E5A5E8852D392B16FDDB4217363D73E6120A18A3656A36A33FCB7D353C93102; PDB 함께 보존했다. 실제 증거는 runtime/verification/source-sdk-r112-card-cli-r1/card-bundle에 있다.

2022 card all-target clippy 종료0,2023 fmt 재검사 종료0,2024 기본 native-proposal-window all-target clippy 종료0을 확인했다. 양쪽 로그에 기존 외부 blitz-paint 경고1개가 있으며 제품 strict 통과를 의존성 경고0으로 표현하지 않는다. 2025 기본구성 gate no-run 종료0,2026 Product145/2027 Core18/2028 Native26/2029 OS1은 모두 종료0이다. 네 group-return의 execution_result/producer_integrity가 모두 Ok이며 동일 R112 SOURCE.md를 가리킨다. Meari는 기존622/624 manifest의 실제 시험을 이번에 재실행했으며 새 컴파일이라고 표시하지 않는다.

2030 정상 native-proposal-window의 player/player-cli 빌드 종료0. fixed-r112/native-bins에 exe/PDB를 보존했다. GUI SHA256 8643C586F5FA6A2796E207142B52B5A95FD80A3F7CDEEC61FCE7B562F6480C1E,CLI SHA256 3AC9A0F84C9BBD091C38A51944F293F54B8CECBDCDF479C2C6B56E941D4863B4. 카드 구성 CLI와 구분한다. 2031 사후 custody 종료0,current source content/identity unchanged files=2358을 확인했다. R112 통합gate 완료이며 UI/품질/전체 카드 인수로 확대하지 않는다. 원 AI9 NeedsRepair·사용자승인0/9,기존 소유권/WIP·보류 유지.

기존 적용 스킬 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100),code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 이어 적용했다. schema 구조 일치만으로 인수를 끝내지 않고 같은 원 host에서 실제 정상/거절/출력/종료를 실행했다.
