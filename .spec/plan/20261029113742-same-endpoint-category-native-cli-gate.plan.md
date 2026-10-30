---
id: plan:kinekino/20261029113742
title: 세 category → native 재생 동일 endpoint CLI gate 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 세 category → native 재생 동일 endpoint CLI gate 제안

**컴파일·통합 인수 후속(2026-10-21):** root05ae548d 기준 원 r2 exact6만 합류해 jobs no-run44394ms0, 기존 safe job1/synthetic provider1의 고유2pass, 양 strict/fmt0을 확인했다. 새 ignored 종단1은 metadata만 확인하고 실행하지 않았다. source6불변/보호root968·snapshot1330·CLI11/B15/Viewer9/LocalHost6/A7/Meari405/binary delta0, receipt7 terminal0/기록14PID 부재를 확인하고 슬롯을 반환했다. [검증 정본](../../sots/verification/application/same-endpoint-harness-compile-2026-10-21.md)을 따른다. 아래의 Cargo0/source-only는 작성 당시 이력이다. supervisor/실제 종단·실 mpv/필터 동일성의 미검증은 유지한다.

**최신 r2 수리(2026-10-21):** 독립 감사의 P1(추출 helper의 미정의 artifact)을 opened.resolution_artifact_id로 고쳤다. 원r1 보존/변경1·나머지5 불변, source6=70B6C45B313201790B8247BD63CF76E4BAD2FF267E1E24D368B9FB85BE7D1BB5이며 runtime/coordination/sdk-same-endpoint-r2/handoff.md를 따른다. 직접 fmt/check·diff0, Cargo/실행0이다. readiness read_line과 stop 후 read_to_string은 원 helper의 무제한 동기 대기이므로 전체 subprocess 수명 bounded를 주장하지 않으며 실 gate 전 외부 supervisor 검토가 필요하다. embedded 룰 digest도 시험 build 값으로만 기록한다.

**소스 작성 후속(2026-10-21):** PM이 원 exact5와 provider.rs 추가1을 배정했고 별도 staging에서 exact6을 작성·동결했다. source6 D3AED09699907B1DC7CC64881193C0F40E0A3D4B75288BB7A28A27D091598A8B, 신규 ignored 시험1/실행0, direct fmt/check·diff0, Cargo/browser/HTTP/media/shared/root/index0이다. runtime/coordination/sdk-same-endpoint-r1/handoff.md가 인계 정본이다. 원5 계획은 같은 디렉터리 original-exact5-plan.md로 보존했고 아래는 계획 당시 이력이다. provider::terminal45초 wrapper는 유지하고 단일 poll 본문을 terminal_until로 추출해 새 명시 관측 deadline을 받는다. CLI 종료 wire 단일 정본을 소비하며 wiring 실인수 전 실행0이다.

2026-10-21 PM read-only 배정. 읽기 기준 root324c3e0b41ae30aad8349bd161f80a14d45e0144다. Rust 시험 source 구현/변경0, Cargo·브라우저·HTTP·미디어·다운로드·공유 snapshot 실행/변경0이다. B15 r3 동결과 C2단계 계약은 별도 범위다.

## 관측된 현재 경계

`src/bin/player-cli/playback_host.rs`는 --serve-playback에서 하나의 PlaybackWorkspaceHost와 PlayerSdk에 jobs/catalog/shared sessions/skip/transitions를 이미 조립한다. 새 host·production 명령·SDK 유스케이스를 만들 이유는 확인하지 못했다.

기존 `tests/jobs/native_episode.rs::actual_resolve_and_owned_native_lifetime`은 먼저 resolve_cli 시험을 호출한다. resolve_cli는 외부 EPISODE_SEED_ROOT/SHA의 episodes 결과를 읽고 별도 provider host에서 resolve한 뒤, native_episode가 그 catalog를 새 --serve-playback host에 넘긴다. 따라서 원 시험은 resolve→native의 보존 자료 연결을 다루지만 category부터 같은 endpoint의 전수 종단 근거는 아니다. series 시험도 외부 category seed를 사용한다. 기존 근거는 지우거나 새 종단 성공으로 재명명하지 않는다.

## 최소 Rust 시험 exact5 후보 · 작성 전 배정 필요

| 경로 | 책임/기존 재사용 |
| --- | --- |
| tests/jobs.rs | 새 same_endpoint 모듈과 공통 playback helper2 등록만 추가 |
| tests/jobs/same_endpoint.rs(신규) | 단일 ignored 시험에서 원 host/endpoint로 세 category를 순차 수행하고 단계별 artifact 참조·결과를 보존 |
| tests/jobs/playback_owner.rs(신규) | native_episode의 기존 NativeOwner 시작/EOF/실제 child wait/실패 보존을 추출. 기존 --serve-playback을 재사용하며 새 host 구현을 만들지 않음. 원10초 stop 예산/고정jobs≥2 assertion은 새 join 계약에 맞춰 명시 관측 예산·기대 작업 수 입력으로 분리 |
| tests/jobs/playback_observation.rs(신규) | 기존 observe_until·진행/PNG capture 검증 부분을 공통 Rust helper로 추출. identity/position·PNG bytes/SHA·크기 검사를 한 곳에서 공유 |
| tests/jobs/native_episode.rs | 추출 helper를 소비하도록만 변경하고 기존 seed/resolve별도host/transition·options·pointer 시험 의미를 보존 |

Fixture::named/call/request는 기존 tests/jobs/fixture.rs를 그대로 사용한다. provider::terminal의 bounded job 대기, CatalogResults/EpisodeResolutionResults의 검증된 archive 읽기와 EndpointClient를 재사용한다. fixture.rs·provider.rs·resolve_cli.rs·series.rs를 바꾸거나 기존 ignored 시험을 호출해 새 host를 숨기지 않는다. 추가 helper/경로가 필요하면 작성 전에 보고한다.

## 같은 endpoint의 단계와 성공 근거

한 fresh evidence root 아래 job/catalog/browser profile root와 하나의 --serve-playback process를 만든다. ready의 원 endpoint·host PID·CLI/engine SHA를 먼저 고정한다. 요청별 원 endpoint와 고유 request/idempotency key를 로그에 남기며 세 category 사이에 host/profile/catalog를 재시작하거나 다른 endpoint로 대체하지 않는다. 실행 중 원 source/archive가 변하면 성공으로 덮지 않는다.

airing/completed/movies를 순차 처리한다. 각 category의 jobs.submit(catalog.search)→terminal job→output SHA→같은 endpoint catalog.search(result_artifact_sha256)를 연결한다. 원 순서에서 PageLocation/ResourceKind 검증을 통과한 첫 Series entry를 선택하고 index·URL·부모 결과 SHA·전체 응답을 남긴다. 빈 결과/Series 부재는 그 category의 명시 실패이며 성공할 때까지 다른 항목을 무작위 재시도하지 않는다.

선택 Series URL로 같은 endpoint의 jobs.submit(episodes.list)→terminal/output SHA→episodes.list(result_artifact_sha256)를 수행한다. 원 episodes 결과의 첫 검증된 Episode entry와 부모 결과 SHA를 남기고 jobs.submit(episodes.resolve)를 호출한다. 완료 job의 resolution artifact ID/body SHA·source descriptor·manifest 참조를 기존 archive 계약으로 대조하며 같은 endpoint의 episodes.inspect_resolution으로 읽는다. 외부 CATALOG_SEED/EPISODE_SEED를 읽거나 seed/resolve용 host를 새로 만들지 않는다. 파일 기반 hash 대조는 보조 검증이며 business command를 다른 endpoint로 우회하지 않는다.

같은 endpoint에서 playback.open을 호출한다. 접수 receipt를 재생 성공으로 취급하지 않고 원 engine instance/resolution artifact가 일치하는 관측에서 connected/idle=false/알려진 video size·duration과 position 진행을 대기한다. 원 PNG capture의 engine/resolution·bytes/SHA·decode 가능한 양수 크기를 대조한다. frame_time_exact/color_parity_verified는 기존 false 의미를 유지하며 실제 시각 품질/정확 PTS/음성 출력 성공으로 확대하지 않는다.

각 category 후 playback.close의 engine/IPC/stdout/stderr 종료·diagnostics 완전성 및 원 instance의 부재를 확인하고 다음 category로 간다. 이전 instance/resolution을 새 category로 remap하지 않는다. 마지막 EOF는 business 작업 종료 뒤에만 요청한다. 세 category 전부의 단계별 성공과 원 host 종료가 있어야 이 gate를 성공으로 표시한다. 한 category 실패는 원 오류/부분 자료를 보존하고 전체 성공에서 제외한다. cleanup은 성공 assertion 전 수행한다.

## 확인된 종료 계약 공백 · production 별도 후보1

현재 playback_host.rs는 skip.begin_close→**drop(local)**→host.shutdown→playback_host_stopped 순서다. 새 LocalHost::begin_shutdown/PendingLocalHost::poll/is_quiescent 계약을 아직 소비하지 않는다. Drop/EOF/process exit나 workspace native/provider receipt만으로 pipe와 accepted SDK worker의 명시 join을 증명할 수 없다.

PM에 선보고한 production 소비 공백은 LocalHost 담당의 CLI root/runner/host/carrier exact10 독립 source-only 배정으로 처리 중이다. SDK는 해당 파일을 수정하지 않는다. 배정 정책은 원 Pending+dependency+첫 오류 보존,30초 join 관측 예산, pipe/SDK 양 join 뒤 기존 dependency의 동기 shutdown이다. dependency shutdown까지 전체 bounded라고 주장하지 않는다. timeout/panic은 원 owner가 살아 있는 상위 boundary에서 진단하고 nonzero abnormal/stopped 출력0으로 구분한다. 새 cleanup driver/SDK/명령 제안이 아니며 실 subprocess 검증은 아직 없다. 이번 exact5 시험 후보에 이 production 경로를 합류하지 않는다.

harness 실행 전 해당 wiring이 인수된 CLI revision/SHA를 고정해야 한다. 출력에 그 근거가 없으면 quiescence 항목은 미검증이고 종단 전체 성공으로 승격하지 않는다. process 종료 관측만 있는 낮은 수준의 결과는 별도 표시한다. 기존 NativeOwner의10초 EOF timeout을 그대로 복사해 새30초 join 예산을 선제 실패시키지 않는다. 테스트의 별도 supervisor 관측 예산과 production join 예산을 구분하고, supervisor 초과/강제 종료는 abnormal failure로 남겨 전체 정리 성공으로 바꾸지 않는다. 실패/Drop에서 소유한 child만 다루며 원 receipt·확인된 자식 수명 미완료와 남은 process를 보존한다.

## 필터 정책과 실행 제한

provider capture의 embedded rule policy(adblock::engine(""))와 GUI 요청 필터의 filters.local.txt 추가 규칙을 같은 정책으로 가정하지 않는다. gate는 provider의 원 blocking 설정/embedded 규칙 revision 또는 digest·requested/blocked 수치를 가능한 원 capture 근거로 기록한다. blocked=0은 해당 관측에서 차단 집계0이라는 뜻이며 광고 없음/필터 미설치/GUI 동등 정책 중 어느 것도 단독 증명하지 않는다. 해당 capture가 정책 식별을 충분히 제공하지 않으면 미관측 항목으로 남기고 새 production 계측 필요성을 먼저 보고한다.

제안 시험명은 `same_endpoint::actual_three_categories_to_owned_playback_on_one_endpoint`이고 #[ignore]에 실제 Windows WebView2/network·trusted mpv/media·중앙 실행 슬롯 요구를 명시한다. 나중에 배정된 정확 test binary에 이 이름의 --exact --ignored --test-threads=1만 적용한다. 전체 --ignored 실행은 하지 않는다.

명시 조건은 fresh 절대 JOB_EVIDENCE_DIR, 고정 JOB_CLI_EXE/JOB_CLI_SHA256, 절대 NATIVE_MPV_EXE/NATIVE_MPV_SHA256, 원 provider/network 접근과 native 영상/창·capture 실행 권한, 기존 conservative2 runner와 소유 프로세스 기록이다. 모델/다운로드/DPI/VM은 합류하지 않는다. 성공·실패 모두 원 request/response/exit·job/artifact/PNG·시간표본·종료 receipt와 PID 관측을 보존한다. 기본 cargo test는 이 시험을 실행하지 않으며 source/compile gate와 실제 종단 실행은 별도 배정이다.
