---
id: plan:kinekino/20261029113510
title: 실제3분류 종단 · 다음 최소 source-only checkpoint
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제3분류 종단 · 다음 최소 source-only checkpoint

감사 수리 r3: r2의 ledger write 실패 때 outcome true/최종 Err가 달라질 수 있는 P2를 기록·보존하고, 전용 evidence 한 곳에 ledger 저장 결과를 포함한 판정을 두었다. 최종 source4 92D8E556…4FC24A1A09/stage same-endpoint-media-r3, 변경2/불변2·pure10 작성/directfmt4 통과/실제 실행0이다. outcome 자체 write 성공은 파일로 입증할 수 없으며 최종 반환 Result만 전체 완료 증거다. revision-r3/handoff.md를 따른다.

책임 분리 후속: PM이 전용 모듈2를 추가 배정해 원 source2/A22C…를 보존한 r2 exact4로 분리했다. 최종 source4 C9D94677…92FD5162F, stage runtime/development/same-endpoint-media-r2. fixture/request_evidence.rs와 same_endpoint/evidence.rs에 각각 순수 metadata 책임·회귀3/5를 두고 기존2는 조립/호출을 유지한다. directfmt4 통과/실제 시험0이며 revision-r2/handoff.md를 따른다.

최신 상태: PM이 정확2 구현을 배정해 독립 stage same-endpoint-media-r1/source2 A22C86BB…04BCB0A2로 동결했다. 신규 pure Rust8 작성·직접fmt 통과, Cargo/typecheck/시험·브라우저/미디어 실행0이다. runtime/coordination/same-endpoint-media-r1/handoff.md 및 before/source/delta를 따른다. 아래 최초 읽기·제안은 이력으로 보존한다.

2026-10-21 PM 읽기전용 배정. 관측 root `6e732c206a4839f2c26917782a527161e4eb035a`. 지정 테스트·helper·CLI assembly·기존 인계만 읽었다. broad cache/runtime 탐색·코드 변경·Cargo/shared·브라우저/미디어/다운로드/AI 실행0이다. 아래2경로는 제안이며 구현 권한은 후속 PM 배정이다.

## 현재 실제 경로와 legacy 최소 대상

진입은 `tests/jobs/same_endpoint.rs::actual_three_categories_to_owned_playback_on_one_endpoint`(계속 ignored)다. `tests/jobs.rs`에 등록돼 있다. 이미 `BoundedNativeOwner::try_start/try_stop_chain`→`tests/jobs/owned_process.rs::OwnedHost`→기존 owned-process Supervisor/Deadline/RecordCursor를 사용한다. 요청은 `Fixture::try_call_until`→`tests/support/owned_process_evidence.rs::run_process_until`, job/progress는 provider::try_terminal_until·playback_observation::try_wait_for_progress/capture_frame이다. 원 오류/request+host 두 owner·panic·증거 보존을 재사용한다.

따라서 **이 단일 게이트에 필수인 legacy consumer 추가 이관은0**이다. 과거13개 소비 목록은 전체 부채이며 별도 actual_provider/resolve_cli/native_episode를 순서대로 실행해 같은 host 게이트를 대신하지 않는다. 그 경로들의 legacy Host/NativeOwner/Fixture::call을 이 게이트의 선행조건으로 만들지 않는다. 신규 Supervisor/Job 상태머신·legacy13 일괄 Result 변환도 필요 없다.

CLI assembly는 `src/bin/player-cli/playback_host.rs`의 `--serve-playback --job-root ABS --catalog-root ABS --browser-profile-root ABS --mpv-executable ABS --mpv-sha256 SHA`다. 같은 PlaybackWorkspaceHost/Sessions/jobs/catalog/local endpoint 하나로 3분류를 순차 처리한다. CLI stop은 stdin EOF→LocalHost pipe/SDK 정리→원 workspace shutdown receipt다. ready는 transport 준비만 뜻하며 playback_verified=false다.

Fixture는 `JOB_EVIDENCE_DIR/same-endpoint-three-categories`를 새로 만들며 transition-jobs/transition-profile/catalog·원 CLI 사본/sha·logs/owned·logs/owned-requests를 보존한다. 임시 경로도 keep_evidence다. 별도 provider seed host나 offline catalog replay fixture는 필요하지 않다. 과거 실제 fixture media 파일/캐시를 현재 입력으로 추정하지 않는다.

## 분모·외부 provider의 관측 정정

기존 “37요청 compile-only”는 역사적 표기이며 실제37 성공 근거가 아니다. 현재 source 성공 경로는 분류당 fresh CLI14=3 job submit+3 fresh jobs.get+category/episodes 결과조회2+resolution inspect1+open1+capture1+close1+diagnostic1+closed-engine 오류조회1, 총 **42 CLI 요청**으로 정적 계수된다. 9개 job의 SDK poll과 progress SDK poll은 가변이며 이42와 별도다. ready/EOF는 JSON CLI 요청 수에 넣지 않는다. 실패 중단 시42를 모두 실행했다고 계상하지 않는다. 다음 checkpoint에서 실제 request label ledger로 분모를 확정한다.

category는 airing/completed/movies 결과 SHA→첫 검증 Series→첫 Episode→jobs episodes.resolve→inspect→같은 host playback.open→진행>1초→PNG/sha/디코더·instance 대응→close/worker receipt→diagnostic/NotFound까지 연결한다. manifest_verified는 media_verified/native_playback/decoded와 다르다.

현재 `src/domain/player_document.rs::PlayerDocumentLocation`은 **https://media.example.test/video/<32hex>**만 허용하고 frame.example.test은 명시 거절 시험에 있다. 원 요청의 “Frame-host/resolve”를 현재 지원 완료로 표시하지 않는다. SDK가 저장하는 player_document_url/raw_artifacts/source_descriptor SHA를 원 요청/응답과 연결해 실제 provider를 기록한다. 새 도메인을 발견해도 테스트에서 allowlist/URL 정책을 우회하지 않으며, Frame-host가 현재 필수라면 별도 SDK 계약/근거 배정이 선행한다. 이번에는 네트워크로 현재 사이트를 확인하지 않았다.

## 다음 source-only 정확2 제안

| 경로 | 최소 변경·인수 |
|---|---|
| `tests/jobs/same_endpoint.rs` | 성공경로42 label/명령의 유한 ledger와 실제 시도/완료/중단 분모·같은 endpoint/원 artifact/instance 대응을 명시. 원 provider_document_url/관련 raw 참조·SHA를 별도 관측에 연결하고 Frame-host 지원을 발명하지 않음. 기존 selection/core 유스케이스·job9·필수 worker 단언·ignore 유지 |
| `tests/jobs/fixture.rs` | 기존 additive try_call_until 증거에 label/command/request_id/원 endpoint/request SHA·expected/actual exit의 연결을 보강. spawn 전 실패·원 owner-bearing failure는 기존 형식에 연결하며 새 registry/counter/상태머신을 만들지 않음. legacy call/Host 및 기타 소비자 동작 불변 |

원 owned_process/playback_owner/provider/playback_observation/shared support 및 제품 SDK/CLI 코드는 보호한다. helper의 기능 변경이 실제 필요하면 경로별 근거로 PM에게 조율하고 조용히 exact2를 늘리지 않는다. SDK B/P1/P2·다음 UI48과 겹치는 제품 소스0이며 tests/jobs의 진행 중 담당 중첩은 구현 전 중앙이 직렬 확인한다. 원 same-endpoint r4 source8/F942…·실제32/no-media 증거와 현재 root before2를 각각 보존한다.

## 검증 인수와 실제 슬롯 경계

source-only는 별도 overlay/before2/source2/delta·Rust pure ledger/증거 연결 회귀·직접fmt·독립 읽기 인계까지다. 후속 중앙 슬롯에서 양 구성 strict/fmt 및 기존 bounded regular9·Rust fixture failure1·관련 영향만 최종 pin으로 확인하고 actual3category는 compile-only를 유지한다. source/helper 통과와 실제 사이트/재생을 분리한다.

기존 r4 고유32는 bounded regular9+지정 Rust Windows failure1+snapshot18+기존 jobs4다. 실제 no-media는 `owned_process::tests::no_media_jobs_host_uses_finite_request_and_owned_eof_without_fake_join`: 실제 CLI --serve-jobs→finite version→EOF, 원 host/request child0/Empty/forcedfalse와 제품 pipe/sdk joined를 구분했다. 실제 영상·브라우저·Frame-host/3분류는0이다. 원 증거는 runtime/verification/same-endpoint-owned-r1/handoff.md와 검증 SOT이며 새 baseline의 actual pass로 승계하지 않는다.

실제 종단은 별도 중앙 browser/network/trusted mpv media 배정 후에만 `cargo test … --test jobs same_endpoint::actual_three_categories_to_owned_playback_on_one_endpoint -- --exact --ignored --test-threads=1 --nocapture`로 실행한다. 사전 입력은 fresh JOB_EVIDENCE_DIR, JOB_CLI_EXE/소문자 JOB_CLI_SHA256, NATIVE_MPV_EXE/NATIVE_MPV_SHA256, SAME_ENDPOINT_JOIN_WIRING_REVISION과 JOB_EXECUTION_MS/JOB_OBSERVATION_MS/STOP_OBSERVATION_MS/TOTAL_OBSERVATION_MS/READY_OBSERVATION_MS/REQUEST_OBSERVATION_MS/CLEANUP_RESERVE_MS(모두 SAME_ENDPOINT_ 접두)다. pin은 그 슬롯의 실제 artifact/원 build receipt에 연결하며 여기서 경로·hash·예산 수치를 추정하지 않는다. stop>30초와 각 단계/cleanup의 유효성·원 lifetime의 min만 인수하고 임의 연장0이다.

인수는 실제3개 결과/원 URL·raw SHA·manifest와 media/진행/PNG/오디오 관측을 구분하고, 원42 CLI ledger·가변 SDK polls·9 job 완료·세 engine identity의 구분·같은 endpoint·close/LocalHost/제품 worker와 own family receipt를 각각 확인한다. timeout/오류/두 Incomplete/강제 정리는 성공·취소/ACK로 바꾸지 않는다. 최종 Drop kill-on-close best-effort·동기 FS/SDK의 hard deadline 한계·live stderr alias snapshot 한계를 유지한다. 보호/index/원exec terminal/PID creation identity/원 오류·증거와 명시 자원 반환을 기록한다. 실제 시각/청각 사람 품질 평가는 별도다.

기존에 본문을 읽은 modoo.today 계획/QA 스킬의 정확 범위·실측/미검증·원 오류/복구 분리 원칙을 재사용했다. SDK/domain의 provider 정책을 테스트로 대체하지 않는다.
