---
id: evidence:kinekino/20261029114259
title: SDK source 통합 격리 입력 준비 — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SDK source 통합 격리 입력 준비 — 2026-10-27

## 실제 관측

원 제품 DFB0 입력을 custody verify-current로 확인했다. 원 2244files content/identity unchanged, exit0다. 원본은 읽기만 했고 `runtime/development/source-sdk-integration-r1`에 TSV에 지정된 파일만 복사했다. 캐시·Git 저장소 전체를 복사하지 않았다.

복사본의 `packages/media-source-host`와 `packages/owned-process`에 선택 private R36의 src/Cargo.toml을 적용했다. 두 manifest의 standalone `[workspace]`를 제거해 같은 제품 workspace에 참여시켰다. 복사본 root에만 media-source-host path dependency/parent-host feature를 추가했다. 따라서 서로 다른 같은 이름/버전 path package를 한 그래프에 중복 삽입하지 않는다. 원 제품과 R36 manifest/source는 수정하지 않았다. 원본의 tests는 복사 상태이고 R36 OS16 test suite를 이 문맥에 이식·실행한 것은 아니다.

복사 preimage 최초 명령은 여러 Cargo.toml을 같은 디렉터리로 복사해 basename이 충돌했다. 이후 원본에서 product-Cargo.toml/product-Cargo.lock/media-source-host-Cargo.toml/owned-process-Cargo.toml의 명시 이름으로 다시 보존했다. 선택 preimage는 이 명시 이름 파일이다. 첫 무효 metadata 옵션 `--output-version`의 exit1도 관측했다. 수정된 no-deps metadata는 exit0이나 전체 graph 증거로 사용하지 않는다.

## 의존성 해석

1028 전체 플랫폼 metadata는 offline 상태의 android-activity0.6.1 부재로 exit101이었다. 원 stderr/receipt는 격리 문맥 runtime/verification에 보존했다. Android를 설치하거나 offline을 해제하지 않았다.

1029 `cargo metadata --offline --locked --format-version 1 --filter-platform x86_64-pc-windows-msvc --manifest-path <격리 root>/Cargo.toml`은 exit0/1130ms였다. 실행기 conservative2/CPU2/BelowNormal·C-only target `native/source-sdk-integration-r1`/공통tmp를 사용했다. stdout SHA256 `78DAB50821EB01F9840CAADAFDEB3402C8A3D8629F146F594F8FB63E1D01C352`. 이것은 Windows graph resolution 성공이며 컴파일·타입·기능 성공이 아니다.

격리 lock SHA256 `955DEBA0338A3A32ED64D6B2A500C192CD55517336FE531D6EED374D0B2916A5`는 새 그래프의 lock이다. 원 제품 lock은 `B2E3D540665FFC8E68D24D0A6C531D2B983658FB19B5ABBA525659BA1ACC371D`로 유지한다. 원 DFB0/190/bins 성공을 이 격리 입력 성공으로 승계하지 않는다. 아직 격리 전체 입력 snapshot/파일 식별 고정은 하지 않았다.

## 다음 구현 계약

현재 제품은 `kinekino_player` library와 public `application::download::{SourceOpener,DownloadSession}`를 제공한다. 별도 기능 정책을 복제하지 않고 infrastructure의 process-backed adapter로 연결한다. SourceOpener.open 이전에 원 owner를 보유하고 오류/unwind에도 남기며, opener와 session이 동일 owner를 공유한다. Pending은 EOF로 바꾸지 않는다. 실제 Metadata 이후에만 response_length를 노출하고 독립 trusted digest가 없으면 trusted_sha256=None을 유지한다. 원 execution/current/attempt 검사와 한 번의 replay admission을 생략하지 않는다. poll_close는 같은 원 host/IPC/raw/final receipt의 완료를 확인해야 하며 단순 drop을 ACK로 취급하지 않는다.

원 browser issuer/current grant는 아직 없다. 테스트 fixture는 실제 browser replay 권한이 아니고 registry의 기본 Unsupported를 바꾸지 않는다. 다음은 adapter 구현·입력 snapshot·fmt/type/strict 및 실제 원 SDK 계약 시험이다. 정상 HTTP Open/데이터·CLI download 성공은 미검증이다.

modoo fixture/mock 및 postcondition 지침을 재사용해 graph metadata/실제 컴파일/OS/제품 기능의 증거 범위를 분리했다. source375 private overlay·원 WIP/소유권/첫 실패와 추론/다운로드/수동GUI/WARP/push 보류를 유지한다.

## SDK adapter 후속 — 실제 구현/타입 확인, 시험 빌드 진행

격리 복사본 `src/infrastructure/download/process_source.rs`에 실제 제품의 SourceOpener/DownloadSession 구현을 추가했다. Arc/Mutex로 같은 ParentTransfer·ReplayAdmission·sink/원 deadline을 opener와 session이 공유한다. open은 spent를 먼저 설정하고 current 검사→bootstrap→원 admission.consume→Open→Metadata 후 session을 반환한다. pending read는 1ms bounded poll로 같은 control/current를 확인하며 EOF로 바꾸지 않는다. raw error는 ParentTransfer에 남고 SDK에는 고정 비민감 메시지만 반환한다. read_owned는 기존 opaque checkpoint cancellation 경로를 소비한다. trusted_sha256=None이다. 실제 ReplayAdmission 구현과 제품 registry는 추가하지 않았다.

실제 core 순서를 추가 대조해 정상 경로의 session.close_owned가 opener.poll_close보다 먼저 호출됨을 확인했다. R1의 단순 pending 오류를 R2에서 고쳤다. session은 같은 원 cutoff 안에서 같은 close owner를 구동한다. opener.poll_close는 요청 cutoff로 원 deadline을 단축할 수 있으며 새 예산을 만들지 않는다. mirror owned-process에 constrained_end, mirror ParentTransfer에 control_pending 읽기 getter를 추가했다. sink는 원 facts와 원 ParentTransfer 참조를 받아 원 IO/host/오류를 함께 기록할 수 있다. transport receipt는 시도 전에 attempted를 설정하고 실패 원 ApiError를 보존하며 재시도/ACK하지 않는다. 원 root/R36에는 이 변경을 적용하지 않았다.

|입력/로그|범위와 실제 결과|
|---|---|
|input-r1 /1030|2279files; SOURCE.md1836882C53974EA53B27793FE2A68A4C9D82FC3C3AA61A626ED6F4F51C700042; inventory63BC205F; `cargo check --locked --offline -j1 --lib` exit0/2m29s. 시험 및 R2 수정의 성공 아님|
|input-r2 /1031|2280files/inventory8887D732; `clippy --lib --tests -- -D warnings` 최초 collapsible_if 실패 보존|
|input-r3 /1032|2280files/inventory1ED2925B66A80FEB89FA18215D0120700A33D8FD6F6D1076116039B4FAC07864; 같은 library/tests strict exit0/38699ms|
|input-r3 /1033|`cargo test --locked --offline -j1 --lib --no-run --message-format=json` 시험 producer 빌드 시작. 이 기록 시점 unified exec61255는 실제 live이며 terminal/성공 미확인. 동일 handle을 polling하고 terminal 전 다른 heavy 빌드를 시작하지 않는다|

input-r1~r3는 각각 소스 작성 뒤 검사 중 읽기 전용 snapshot으로 기록했으며 검사 전 고정 완료라고 표현하지 않는다. post-check content/identity 확인은 아직 남는다. tool의 SOURCE.md 제목 'canonical'은 generic 출력이며 이 private 입력의 canonical 승격이 아니다.

새 exact ignored test2는 `infrastructure::download::process_source::tests::rejected_sdk_open_is_spent_and_session_close_drives_original_no_child_cleanup` 및 `infrastructure::download::process_source::tests::transport_receipt_failure_is_not_retried_or_acknowledged`다. 실제 원 Windows prepared IPC/evidence handles를 만들지만 launch 전 권한 거절 또는 close만 수행한다. 첫 시험의 수동 생성 session은 cleanup-only fixture이며 정상 admission/Metadata를 입증하지 않는다. 아직 시험 실행 성공으로 계상하지 않는다. 원 evidence는 C tmp의 sdk-source-* 경로에 향후 보존한다.

후속 필요: 전송 close 뒤 후보.publish의 check_origin은 active transport current와 구별해야 한다. 원 retained browser origin/attempt 권한을 검사하는 별도 ReplayAdmission 계약을 작성하고 실제 SDK 순서와 대조해야 한다. 현재 기본 check_origin이 closing 상태를 거절하므로 정상 발행 인수는 아직 불가능하다. launch/queue/read의 callback current 전달 범위도 추가 감사한다. 정상 HTTP/제품 다운로드/CLI·AI/GUI 완료가 아니다.
