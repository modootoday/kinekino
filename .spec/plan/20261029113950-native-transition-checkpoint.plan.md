---
id: plan:kinekino/20261029113950
title: 회차 전환 등록·인수 checkpoint
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 회차 전환 등록·인수 checkpoint

실행 순서는 [회차 전환 계획](native-episode-transition.md)의 중앙 예약을 따른다. 이 문서는 준비·등록 경로와 인수 요구의 기록이다. 중앙 반환 뒤 등록 및 첫 전체 검사와 직접 교체 fault 검사를 실행했다. 현재 관측과 미완료 경계는 [실행 정본](../../sots/verification/application/native-transition-cli-2026-10-19.md)에 분리한다. 준비 파일은 모두 runtime/coordination/native-transition 아래, hash는 source-sha256.txt다. root SDK 기준 해시는 api-mod-baseline-sha256.txt이며 등록 전에 대조했다.

## 준비본 → 등록 경로

| 준비 .pending.rs | 제품/테스트 경로 |
| --- | --- |
| transition_module | src/application/playback_session/transition/mod.rs |
| adjacency | src/application/playback_session/transition/adjacency.rs |
| service | src/application/playback_session/transition/service.rs |
| resolve_job | src/application/playback_session/transition/resolve_job.rs |
| slot | src/application/playback_session/transition/slot.rs |
| coordinator | src/application/playback_session/transition/coordinator.rs |
| native_replacement | src/infrastructure/mpv/session_engine/replacement.rs |
| owned_startup | src/infrastructure/mpv/owned_startup.rs; trusted launch 및 engine open/replace의 recovery 소유 연결 필요 |
| owned_launch | src/infrastructure/mpv/owned_launch.rs |
| session_engine | src/infrastructure/mpv/session_engine.rs 변경의 비교 후보; 전체 덮어쓰기 금지 |
| process_output | src/infrastructure/mpv/process_output.rs 변경의 비교 후보 |
| diagnostics | src/infrastructure/mpv/diagnostics.rs 변경의 비교 후보 |
| combined_host | src/infrastructure/playback_workspace.rs |
| api | src/application/api/episode_transition.rs |
| schema | src/application/api/episode_transition_schema.rs |
| client | src/application/api/client/episode_transition.rs |
| cli_host | src/bin/player-cli/playback_host.rs |
| actual_next | tests/jobs/native_episode/transition.rs |
| actual_driver | tests/jobs/native_episode.rs 변경의 비교 후보; 전체 덮어쓰기 금지 |
| api_mod | src/application/api/mod.rs 변경의 비교 후보; 전체 덮어쓰기 금지 |

등록 부모는 playback_session의 pub mod transition, session_engine의 mod replacement, infrastructure의 Windows playback_workspace, API의 두 모듈, client의 child/공개 확장 trait, bin의 Windows playback_host다. runner의 serve-playback 분기/중복 mode 거부와 global COMMANDS의 transition.observe/commit/release3개도 연결한다. 기존 API 변경은 다른 담당의 저장소/프로젝트/제안 builder 수정을 보존하고 필요한 필드·generic 전달·분기만 적용한다. runtime 경로를 제품에서 include하지 않는다.

## 공유 영향과 입력 동결

root application/API/native infra와 player-cli 조립·native actual이 바뀐다. Meari나 UI TSX는 이 checkpoint에서 수정하지 않지만 UI 소비가 root SDK를 컴파일하므로 공유 입력이 새 checkpoint가 된다. UI 명시 반환 후 소스 snapshot을 만들고 SDK gate가 끝날 때까지 새 UI 소비를 시작하지 않는다. 이전 r3 bin·원문·실패 staging은 유지한다. 제품 표시 이름·DPI·VM·새 모델 추론/다운로드는 변경하지 않는다.

## 최소 강한 gate 원문

1. 등록/type 검사 뒤 준비 Rust8개를 실행한다. 추가로 unsupported target(명시 Empty Referer)/잘못된 결과·다른 target·다른 job/attempt·stale old instance/artifact가 old engine을 닫지 않는지 검사한다. old-close 실패·old 종료 후 new-launch 실패·launch 중 EOF·새 cleanup 실패는 별도 fault 주입으로 소유 상태/오류 receipt를 확인한다. spawn_process 초기화 실패의 child/worker 정리 결과 무시도 해결한다.
2. cargo fmt --check, cargo test, cargo clippy --all-targets -- -D warnings와 GUI/CLI bin build를 수행한다. root 테스트 수/ignored 수는 현재 로그에서 다시 집계한다. 정상 실행을 timeout으로 재시작하지 않는다. 단일 계열·두 logical CPU·BelowNormal·offline/locked/-j1·단일 테스트 thread·기존 cache를 사용한다.
3. CLI/GUI/harness/runner/Cargo.lock/mpv SHA를 고정한다. 새 증거 디렉토리와 새 job/profile root를 사용하고 저장된 실제 episodes.list를 재검증한다. 초기 resolve는 fresh job/attempt와 원문·wire/manifest 근거를 보존한다. 목록의 서로 다른 인접 두 회차를 선택하고 URL/제목 산술로 대상을 만들지 않는다.
4. 같은 combined host에서 CLI next 시작→resolve 중 old instance/artifact/무오류 유지→중복 시작 RevisionConflict와 전후 완전한 jobs ID 목록 불변→resolve 완료→CLI commit→old engine/IPC/stdout/stderr 종료 확인→새 instance/artifact와 기대 target descriptor 일치→30초 내 독립 영상·오디오 채널/시간1초 이상 진행→새 PNG의 hash/bytes/engine/artifact/dimensions/상한/전체 decode+finish를 검사한다. caption 번인과 엔진 subtitle overlay를 구별하며 exact PTS/color parity는false다.
5. 소비 handle 조회/재commit은 NotFound, old instance control은 RevisionConflict이며 새 instance를 보존해야 한다. 이전 회차 방향도 새 resolve와 같은 강도로 인수해 두 방향을 모두 확인한다. 현재 helper는 방향별 공통 검사를 제공하며 상위 driver에서 Next/Previous 두 leg와 최종 EOF를 연결해야 한다. 옵션1.25→1.0·관측 audio/video disable/복원/PNG·4controls와 close/reopen/EOF/post-host Cancelled도 유지한다. 마지막 combined EOF에서 native/slot/jobs/browser 종료 receipt와 소유 프로세스 부재를 확인한다.

실제 harness의 exact case와 --list 결과는 driver 등록 뒤 고정한다. 현재 actual_next는 helper이므로 단독 실행 성공으로 gate를 대체하지 않는다. 기존 native 실제 case를 확장하거나 별도 ignored case를 등록하되 위 assertion을 모두 연결하고 해당 고정 harness를 runner로 실행한다. deadline·해상도·전체 PNG 검증을 완화하지 않는다.

## 인계 결과 형식

모든 최초 실패/수리/최종 receipt, 고정 SHA, source snapshot 범위, 실제 target descriptor, 두 회차 PNG·관측, old/new 소유 종료와 프로세스 부재를 정본에 연결한다. master 저장 완료·job 완료·process 생성·재생 관측·청취/사람 품질 승인을 분리한다. 선택된 두 회차 성공을 전체 회차/사이트 완전성이나 전체 제품 완료로 확대하지 않는다.
