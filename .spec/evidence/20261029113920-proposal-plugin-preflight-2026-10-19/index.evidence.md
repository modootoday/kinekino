---
id: evidence:kinekino/20261029113920
title: 제안 연결 전 독립 plugin 검사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제안 연결 전 독립 plugin 검사

2026-10-19. SDK가 root jobs/activation을 수정 중이므로 root를 소비하는 전체 bridge 검사를 실행하지 않았다. PM에 독립 패키지의 별도 슬롯을 요청·배정받아 준비된 `player-ui-plugins`만 검사하고 반환했다. 최종 제안 인수를 이 작은 범위로 축소하지 않는다.

| 명령(packages/player-ui-plugins/Cargo.toml) | 결과 |
| --- | --- |
| cargo fmt --check | exit0 |
| cargo update --offline --workspace | exit0,새 registry package0,직접 serde 의존성 lock 동기화 |
| cargo test --offline --locked -j1 | exit0,기존6회귀,21.35초 |
| cargo clippy --offline --locked --all-targets -j1 -- -D warnings | exit0,4.39초 |
| cargo check --offline --locked --target wasm32-unknown-unknown --lib -j1 | exit0,2.60초 |

공유 target/-j1/BelowNormal을 유지했다. source/lock/hash/log는 runtime/quality-review/player-ui-proposals-r1/plugin-gate에 보존했다. 기존6회귀는 자막 codec/등록/지원오류/permission/schema 범위다. 새 TypedSdkPlugin의 generic body와 portable library 타입은 컴파일됐지만 root canonical proposal 타입의 연결·실제9·bridge7·CLI·브라우저 실행을 증명하지 않는다. 해당 full gate는 SDK의 성공 checkpoint/freeze 뒤 실행한다.

별도 읽기 preflight에서는 시험 workspace의 ParentDir 경로가 SDK의 fixed ordinary-root 계약과 맞지 않음을 확인했다. SDK guard를 완화하지 않고 test helper만 manifest의 두 단계 조상 경로를 쓰도록 수정했다. 원 prepared snapshot·수리 후 source/hash·실행 전 점검 사실은 source-prepared/source-preflight/preflight-note.md에 분리했다. 실패한 컴파일/실행이 있었다고 표현하지 않는다.
