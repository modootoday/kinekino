---
id: plan:kinekino/20261029113757
title: Close 진단 통합 gate 준비
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Close 진단 통합 gate 준비

상태: source-only 준비. PM의 SDK r2 독립 읽기는 확정 P1/P2 없음이며 [후계 정본](../../../sots/verification/presentation/default-native-window-witness-source-review-2026-10-24.md)에 기록했다. 이 계획 자체는 공유 S 수정·Cargo·Run 배정이 아니다. Meari 자체 features window type gate 완료/인수와 중앙 heavy 배정 뒤에만 진행한다.

후속 배정: Meari 반환4986ED36/ED427EBE를 인수했고 PM이 통합 heavy를 배정했다. 실제 S는 이전 gate 원복 상태인 default_launch.rs7A8854/file_guard.rs부재이며 다른 SDK6 hash는 일치한다. 원 before/restored 기록과 일치함을 대조했다. PM 추가 승인으로 canonical file_guard.rs A71B92F4 new1 prerequisite를 정확히 일시 배치한다. 경계는 SDK changed1+prerequisite new1/Meari existing2 총4이며 종료 시 S 원7A8854 bytes/mtime와 file_guard 원부재를 복구한다. 전체 canonical sync0이다. 입장 중단/대조는 runtime/verification/default-native-close-integration-r1/02-source-admission-stop.md에 보존했다.

## 입력과 복구 경계

root 기준 abb63c8ed1778d185603b2ca62ada9eb016973dd, library f85a68df7a6a8fee239d9655895ee15d5eaa5510, lock 8F2C1D56C8A2C22D36D9E57A285D727C7EE9C7D2D1160902B66CC6630B042C67은 현재 준비 기준이다. 실행 직전 PM 승인 기준과 달라지면 자동 재기준화하지 않는다.

SDK는 E3DF2660 canonical-before8에서 EDEF73FF effective8로 default_launch.rs exact1만 overlay한다. Meari는 별도 preimage2에서 최종 인수된 runner.rs/window.rs exact2만 overlay한다. 아래 r2는 아직 인수 대기이며 최종 반환 뒤 경로와 활성 핀을 명시 교체한다. 각 소유자의 preimage·후보·복구 기록을 분리하고 전체 canonical 동기화는 하지 않는다. SDK 미실행 r1은 복구 기준으로 쓰지 않는다.

PM 선인계 후 위 Meari 8FFFA42E는 통합 사용 금지인 이전 후보로 분류한다. 현재 **인수 대기** 후계 r2 source2는 `ED427EBE2E939296D699DD19D7C11A4C4C617D2695DADDED5291E3E94722BD5F`, window leaf는 `FA2DACB5AC2392014DEBE6F8B3EBA607D314482EAFCA1D34D3FD1AA5198306FD`이며 runner는 이전과 같다. 현 Rust1.99에서 deprecated인 AtomicU8.fetch_update를 try_update로 바꾼 이름1 수리다. 동일 ordering/closure를 전달하는 설치 std wrapper 확인은 Meari/PM 보고이며 SDK 독립 검증으로 표시하지 않는다. 원 r1 strict101은 보존됐다. 보고 시점 r2 fmt/strict0·harness 빌드 중이므로 최종 반환의 소스 경로·핀·검증 결과를 인수한 뒤에만 활성 통합 입력을 교체한다. 이전 Rust 호환은 미검증이고 manifest/lock 변경0이다. 이 선인계는 overlay 실행권이 아니다.

기존 root 공유 S/T와 resource runner를 재사용한다. 새 target·다운로드·새 검증 프레임워크0이다. protected carry142/SDK23/기존 alias 및 known 예외를 이전 반환 기록에서 승계하고 변경 경로만 대조한다. alias 복구는 기존 보존 지침을 따르며 hardlink 대상에 덮어쓰지 않는다.

## 기존 공통 gate 순서

1. 배정된 단독 writer에서 정확 overlay3과 preimage를 확인한다. 기존 fmt, default strict, native strict를 같은 공통 명령/feature 계약으로 수행한다. 최초 오류를 보존하고 소스 수리는 새 핀과 구분한다.
2. 기존 native_product_launch no-run의 Cargo JSON으로 test executable producer를 고정한다. product/CLI도 해당 공통 producer JSON의 실제 산출물 경로·SHA·크기·생산 기준을 기록한다. 진단 product에는 인수된 Meari 후보가 포함돼야 한다. 옛 4224 product를 새 계측 이미지로 표시하지 않는다.
3. 새 고정 H의 목록을 읽어 예상28/ignored11과 대조한다. 기존 ordinary17 exact 목록을 재사용해 opt-in off로 실행한다. actual OS file-guard3과 N/C는 이번 CPU gate에 포함하지 않는다. 새 테스트명이나 임의 필터를 만들지 않는다.
4. receipts·원 오류·producer 핀·입력/보호 범위/복구 결과를 기존 반환 형식으로 남기고 슬롯을 반환한다. 전체 일반 gate 통과는 SDK 진단 IO fault나 opt-in 실제 실행의 입증이 아니다.

위 범위는 타입·strict·등록·기존 ordinary 회귀와 새 실행 이미지 생산을 입증한다. SDK write fault 주입과 off 출력 불변의 실제 OS 관측, Meari stderr 정체/오류, 진단 actual N은 별도 배정이다. 실제 N은 원 argv0·30/5·case1·같은 host/controller/operator 고정 경로 계약을 유지하며 새 Close/예산 연장/강제 종료를 자동 추가하지 않는다. C는 N 결과와 별도 지시를 따른다.

## Normal producer와 ordinary17의 정확 근거

기존 `runtime/verification/default-native-file-guard-r1/07-native-producer.stdout.jsonl` SHA256 `778027327FA362BB2D402B54B67CC141EAD0B4C59E48AD5AAB2E7B3294AA5D9B`를 읽었다. 해당 no-run JSON에는 target.kind=[bin], name=player/player-cli, **profile.test=false**, executable=각 native/debug/player.exe·player-cli.exe인 normal compiler-artifact가 모두 실제 존재하며 fresh=false였다. target.test=true는 Cargo target의 시험 가능 속성이고 profile.test와 구분한다. native_product_launch의 profile.test=true test artifact를 제품으로 사용하지 않는다.

통합 no-run의 **새 JSON**에서 위 normal 두 artifact를 다시 확인하고 새 후보/feature/producer 경로와 SHA를 고정한다. 과거 JSON은 생산 가능성 근거일 뿐 옛4224 이미지를 승계하는 근거가 아니다. 새 JSON에 normal 둘 중 하나라도 없으면 기존 공유 S cwd와 T 환경·resource runner conservative2에서 추가할 기존 명령1은 `cargo build -p kinekino-player --bins --features native-proposal-window --offline --locked -j 1 --message-format=json`이다. 이 fallback도 중앙 배정 범위에서만 실행하며 현재 실행0이다. normal producer 확인 없이 actual N으로 넘어가지 않는다.

ordinary17 exact 목록은 `runtime/development/default-native-window-r1/revision-r5-clippy/GATE-expected17.json`, SHA256 `FC29CAB6856D8215A00442DA660D549B99D443F17EFA7D1B52BC6402D80ED680`이다. 현재 읽기에서17개 모두 ignored=false/OS=false를 확인했다. 목록의 actual=0은 작성 당시 계획값이며 과거 실행 결과는 pure17 정본에 따로 있다. 새 고정 H에 각 name을 `--exact`로 실행하고 기존 명단 자체는 수정하지 않는다.

modoo.today code-change-review-verification 본문9f73865f/필수 verdict-table9d9619a8과 plan-authoring 본문31459a74/필요 plan.md091d2f8d를 이번 세션에 읽고 적용했다. 기존 검증 경로 재사용, 실제 관측과 미검증 분리, 원 소유/첫 오류 보존을 검증 범위에 반영했다.
