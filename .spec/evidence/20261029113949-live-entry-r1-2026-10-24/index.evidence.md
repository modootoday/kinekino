---
id: evidence:kinekino/20261029113949
title: R1 명시 live 선택·원 inspect 연결 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R1 명시 live 선택·원 inspect 연결 검증

## 기준과 입력

실행 배정은 execution-leases.md의 R1 단독 heavy다. root `579f5db0778a9d48f2eb2afc251c1f232cca7de3`, Meari `b532fe0eed06d8526b755b11bc36416070244ef7`, Cargo.lock SHA256 `E469A7204B9DC2AC82AC729E36FD8AC41449E1C382E9167A28A00A45C45789A2`를 사용했다. SDK 반환 S5,287개와 bytes/mtime 차이0을 확인하고 채택된 제품14/SDK10/task3/scoped9를 유지했다.

원 source9 `5479218AD0639821E23130827E0D6A11C34AAE02C559B414182F4B5A2881C607`의 기존7/신규2를 canonical/S에서 확인했다. before9는 `09F9540B145A0F0297C3244135BB70594A5F30348EDD3CB281AEA08CC52190E6`다. 아래 네 fixture revision은 모두 같은 `live_entry/tests.rs` 한 leaf만 바꾸며 다른8은 원본과 같다. 최종 검증 입력은 repair4 effective9 `3FD29712AF6A6483A621CDE3F55A44EC161693E94689B54741B3D054B09A8684`다. R2 source12는 합성하지 않았다.

## 원 실패와 최소 수리

원 자료는 `runtime/verification/ui-live-entry-r1/`에 보존한다. 원60 runner receipt는 모두 terminal이며 56개 exit0, 다음4개 exit101이다. 전체 이력을 전부 성공으로 표시하지 않는다.

| 원 실패 | 별도 승인 fixture revision과 원 자료 |
|---|---|
| 02-strict | unused Value, 없는 EndpointClient::connect, &str→RouteId 변환 오류. repair1 source6AC02C37은 기존 new/원 route_target(ticket)의 typed 값을 사용했다. |
| 10-live-1 | Rig.input의 지역 CallbackBindings가 drop되어 원 lease가 무효화됨. repair2 source730C5777은 원 수동 export owner를 Rig에 보존하고 cleanup 완료 후에만 해제한다. 제품 guard는 정상 거절했으며 변경하지 않았다. |
| 18-live-r2-1 | 첫 close poll에서 즉시 Closing을 기대함. repair3 source80B6F1C0은 원 endpoint-first progress를 보존하고 기존 end3초를 연장하지 않은 채 같은 owner가 Closing에 도달하는지 관측한다. 원 hold/inspect1/단언은 유지한다. |
| 28-live-r3-3 | 명시 NotFound 음성 case의 원 runtime 실패를 tick이 무조건 unwrap함. repair4는 명시 negative1/2에 한해 원 HostAccessError와 typed last_host_error를 보존하고 entry.error와 Arc 동일성, exact stale/SDK NotFound terminal 상관을 확인한다. 양성 오류는 계속 실패하며 error clear/복구/가짜 receipt는 없다. |

각 private repair 디렉터리의 delta/before/effective manifest·patch/HANDOFF와 PM 검토를 별도로 남겼다. 원 실패03개 실제 fixture는 cleanup complete=true였고 원 실패 payload와 complete Rig carrier가 보존됐다. 기존 시험/단언 삭제, allow 억제, 제품·SDK·Native 정책 변경은 없다.

## 최종 산출물과 실제 결과

`32-executable-artifacts.json`과 `32-fixed-bin.json`의 producer는 `kinekino_player`, `src/lib.rs`, profile.test=true, native-proposal-window + native-observer-test-support, fresh=false다. 고정 `bin/live-entry-r1-repair4-tests.exe`는 174,561,280B, SHA256 `DAD4721D344E738E26CE299CBE0598C7EDF3628DFFBC5C9073170BFB79FEA4B5`다. 종료 후 producer/fixed SHA를 다시 확인했다. 이전 CC6BFE14/C4B901D1/CFC4599A 고정본도 보존했다. root player/CLI alias는 실행 증거로 사용하지 않았다.

최종 revision의 `29-fmt-r4`, `30-strict-r4`, `31-build-r4`가 통과했다. strict 범위는 `cargo clippy -p kinekino-player --all-targets --features native-observer-test-support -- -D warnings`, no-run은 같은 feature의 `cargo test -p kinekino-player --lib --no-run --message-format=json`이다. 모든 Cargo 호출은 locked/offline/j1이며 원 runner CPU2/BelowNormal(16384)/Job 상속/thread1을 유지했다. 입장과 build/test 직전 fresh RAM/VRAM/C 여유를 기록했다.

`33-list` 실제 등록1,131개, `34-ignored-list` 제외61개다. `final23-results.json`은 다음 **고유23 passed / 0 failed**만 연결한다. 등록 수나 예전 revision의 통과를 중복 합산하지 않는다.

- 35 및36–41: R1 ignored7을 exact/literal --ignored로 각각 실행해 모두1 passed. 원 SDK/store→LocalHost→EndpointClient→같은 ExecutionOwner/ViewerHost에서 명시 선택·중복/unknown/foreign·Jobs 양방향 배타·suffix·원 inspect/terminal/ACK/release·stale/NotFound/close를 확인했다.
- 42: `cargo test -p kinekino-player --lib --features native-observer-test-support assembly::meari::viewer_host::live_entry::tests::trusted_table_preserves_explicit_identity_and_rejects_empty_overflow_and_repair -- --exact --test-threads=1`의 일반1 passed.
- 43/44/52–57: 기존 일반8 passed. Card callback2와 원 close6의 등록·소유/endpoint-first·ACK/worker·오류 계약 회귀다.
- 58–64: 기존 ignored7을 exact/literal --ignored로 각각 실행해 모두1 passed. captured readonly guard2와 Jobs route/중복/반환/continuation5다. Jobs 원 closeout도 complete=true다.

집계 정정: 45–51은 오케스트레이션의 가변 플래그가 전달되지 않아 각각 **0 passed / 1 ignored**였다. exit0만 보고 이7개를 실제 통과로 먼저 보고한 집계를 정정했다. 원 기록과 `impact15-results.json`은 보존하고, 실제 실행 근거는58–64로만 귀속한다. 최종23 집계는 이 미실행7개를 포함하지 않는다.

whole library/workspace/CLI/doc/나머지 ignored 및 Native-loop startup/OS 창/GUI를 반복 실행하지 않았다. 기존 close 계약 시험의 simulation/원 ExecutionOwner 근거를 OS endpoint/window physical close로 확대하지 않는다.

## 입증 경계

fixture는 원 HostCallbacks의 수동 admission export를 보유한다. 이는 Native document renderer의 export와 동일한 소유자라고 주장하지 않는다. 초기 Viewing은 RouteHostObservation 모델 설정이며 실제 frame 관측이 아니다. 실제 SDK inspect/ACK/release와 원 실패·정리를 증명하지만, Inspected는 editor mount/편집/Accept/actual Some checked frame/Viewer 사용자 진입 인수가 아니다.

TrustedLiveChoices::from_trusted_local은 신뢰된 Rust caller 계약이며 create/import provenance를 타입으로 증명하지 않는다. 후속 제품 조립이 원 결과/owner/scope/generation을 보유하고 capture-derived selection을 전달하지 않아야 한다. R1은 captured guard를 제거하지 않으며 목록·권한 발급·기본 launcher 정책을 만들지 않는다.

## 보호와 반환

R1 exact9를 시험 후 baseline으로 복원했다. S 기존5,287개 bytes/mtime 차이0, 제품14/SDK10/task3/scoped9 유지다. 새54개는 `S-new.json`의 원 fixture store 산출물(ui-live-entry30/ui-current-card-mount-store4/ui-jobs-product-route20)이며 원 실패/시험별로 보존했다. lock 파일 잔존을 열린 writer로 해석하지 않는다.

root tracked1,357/library493/fixed90/index2의 bytes 및 기록된 mtime 차이0, alias4의 현재 SHA/FileID/mtime 차이0이다. 새 고정본4도 각 원 producer SHA와 일치한다. T exe574개 중 변경1개는 위 library producer이며 각 revision Cargo JSON/고정본으로 귀속했다. 나머지573개 bytes/mtime 차이0이다. knowncache2는 원 SHA/mtime를 유지하지만 과거 미귀속이 해결된 것은 아니므로 all_cache_protected=false다. 복원한 S를 현재 캐시 producer가 재빌드했다는 주장은 하지 않는다.

모든 원 exec/runner 세션 종료와 출력 EOF를 회수했고 final-process-observation.json은 빈 배열이다. 담당 process/session/handle/writer/cleanup 대기0으로 반환한다. canonical/index 편집·stage/commit/push0이며 exact9 채택은 PM 소유다.

## R2와 스킬

R2 source12 `6A0A5864E175D5CDC35EC1F4AC493E55F6A646F3A3A588F4DEEFEC7E07C230D0`/raw12는 불변·합성0·compile0·actual0으로 보존했다. 최종 R1 fixture 수리를 상속하는 후속 R2 private revision은 별도 배정이며 R2의 실제 Some/typed task/입력/별도 Viewing proof/실 notifier·proof fault 검증을 대신하지 않는다.

앞서 modoo.today에서 검색하고 본문을 읽은 `modootoday-free/design-delivery-plus/frontend-first-journey`를 원 typed port/공통 owner와 사용자 경로·미지원 구분에 적용했다. `modootoday/software-qa/test-fixture-and-mock-build`를 원 SDK/endpoint/terminal/ACK·revocable export·setup/body/cleanup carrier와 source/model/actual/revision 분리에 적용했다. PowerShell은 파일·해시·시각·프로세스 메타데이터와 Rust 도구 실행만 담당했다.
