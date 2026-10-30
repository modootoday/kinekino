---
id: plan:kinekino/20261029113552
title: CLI host 종료 carrier·driver 범위 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# CLI host 종료 carrier·driver 범위 제안

상태: active. 2026-10-21. PM source-only 범위 제안 배정이다. 코드 작성·실행 슬롯 배정이 아니다. LocalHost source6 원핀 CBF11D50925E7C5FFC18D0CE5EB73711E6705C21A0622F432235055A9916F690은 유지한다.

중앙 후속 결정: 전체 CLI bounded 종료/새 강제 exit 정책은 추가하지 않는다. 아래 exact9 중 신규 공통4를 **미등록 source-only**로 먼저 작성하도록 배정했다. overlay는 `runtime/development/local-host-cli-carrier-r1`이며 등록은 CLI4 wiring과 함께 후속 배정한다. 한 번의 `advance(owner, deadline)`는 기다리지 않고 Ready 또는 Pending/Partial/Panicked의 동일 owner를 반환한다. Ready는 LocalHost 양 정상 join만 의미하고 dependency 성공을 뜻하지 않는다. 원 first error를 변경하지 않는다. 아래 abnormal exit 제안은 채택된 정책이 아니며 구현하지 않았다. Cargo/공유/root/index0이고 source6 gate가 우선이다.

독립감사 P2 후속: r1의 caller status+debug_assert를 받는 into_ready는 실제 join 이전 회수 우회를 허용했다. r1을 보존하고 r2 E65068BD8619C8A9F0A158FA5DD61580DD852109BA549FF8996A7BD08C39CCA4에서 원 private Pending을 직접 poll하는 try_ready(self)로 좁혔다. 미완료에는 같은 owner와 원 status를 반환한다. 신규 가짜 공개 status 회귀1을 포함 작성5/실행0, 직접 fmt0이며 source6는 불변이다. 인계는 `runtime/coordination/local-host-cli-carrier-r2/handoff.md`다.

## 324c3e0 이후 CLI4 배선 후속 제안

후속 PM 결정과 작성: root boundary의30초/abnormal 정책을 중앙이 채택했고 exact10 source-only를 승인했다. `.spec/adrs/player/adr-cli-host-shutdown-observation.md`에 정책과 미검증을 분리했다. r3 source10 7303621465FF709A04450AF590FCB14708BB35641919D5F01FFDB7F2DE6211F1을 별도 overlay에 동결했다. host4의 typed owner는 runner까지 전달하고 process exit는 runner의 최상위 branch 하나만 사용한다. 공통/runner 작성11·실행0/directfmt0이며 실제 subprocess 인수 전이다. 종료 DTO는 [단일 wire 정본](../../sots/player/application/local-host-cli-shutdown-wire.md)으로 고정하고 PM/SDK에 공유했다. 아래 '정책 차단'은 이 결정 이전 이력이며 현재 미검증은 구현·subprocess gate다. source6·carrier r2·live/root/shared/index는 유지한다.

현재 중앙 heavy 소유자는 SDK B15다. 아래 UI 소유 표현은 작성 당시 이력이며 이 담당의 실행 배정이 아니다. PM이 stderr eprintln! 실패의 panic 가능성을 독립 감사에 추가했으므로 원 r3 핀을 보존하고 결과 전 코드 수정0을 유지한다.

감사 후 P2 수리: r3의 stderr eprintln! 경로2가 첫 오류/code 반환 전에 panic할 수 있음을 확인했다. 별도 r4 exact10 917F0927CC962CD700C8713F9152ED0DED703CCCCAD7D45A79EB98C5B79B090A에서 driver/runner/tests3만 수정했다. 실제 production helper에 Write를 주입하고 let_=writeln best-effort로 처리하며 BrokenPipe 회귀2를 추가했다. 총 작성13/실행0/directfmt0이고 나머지7/DTO/원r3는 불변이다. `runtime/coordination/local-host-cli-wiring-r4/handoff.md`를 따른다. 위 '감사 결과 전'은 이전 상태이며 현재 독립 재감사 대기다.

LocalHost6의 실제 양 join 계약은 commit324c3e0b41ae30aad8349bd161f80a14d45e0144로 제품 인수됐다. carrier r2는 여전히 미등록·작성5/실행0이다. 아래는 후속 **제안**이며 코드 배정/최종 종료 정책 채택이 아니다.

최소 영향 후보는 고유10경로다: 기존 `src/bin/player-cli.rs`, `src/bin/player-cli/runner.rs`, CLI host4, 신규 carrier r2의 `host_shutdown/{mod,carrier,driver,tests}.rs`. GUI main/audio/fixture·LocalHost6·하위 dependency API를 합치지 않는다. 이10을 한 번에 구현 승인으로 보지 않으며 최종 boundary 미결정을 먼저 해소한다.

1. 각 host의 serve는 LocalHost 생성 뒤 ready write/flush/read의 첫 오류를 저장하고 EOF와 동일 carrier 경로로 보낸다. setup 실패는 owner 생성 전 오류로 구분한다. native/playback은 기존 skip.begin_close 후 original NativePlaybackHost/PlaybackWorkspaceHost와 skip owner를 한 D bundle에 유지한다. provider/jobs도 원 host를 D로 유지한다. 원 error를 shutdown JSON으로 덮어쓰지 않는다.
2. CLI private run 결과는 기존 `Result<(),ApiError>→u8` 조기 축약 대신 **완료 결과 또는 원 소유 종료 상태**를 반환하도록 제안한다. 구체 enum/이름은 caller 배정에서 정하며 공용 application/SDK API로 올리지 않는다. runner는 네 host의 서로 다른 D를 보존하는 owned variant만 전달하고 임의로 같은 trait object에 넣어 자기 Drop/동기 shutdown을 숨기지 않는다.
3. LocalHost Pending/Partial/Panicked가 남으면 원 owner와 첫 오류를 root boundary까지 move한다. budget_expired는 그 원 owner에 붙은 관측이며 정상 code/error code로 줄여 return하지 않는다. 첫 오류가 있어도 incomplete owner를 버리고 그 오류 code만 반환하는 방식은 금지한다.
4. ReadyForDependencies에서는 원 D와 첫 오류/LocalHost receipt를 회수한다. 기존 dependency.shutdown을 실행한다면 **기존 동기·소비형 단계임을 호출부에 그대로 노출**하고 LocalHost budget으로 bounded라고 보고하지 않는다. 이 단계에서 실패 뒤 D가 사라지는 기존 API 제약도 남는다. owner-preserving dependency 종료가 필요한 요구까지 이10으로 해결했다고 주장하지 않는다.
5. 최종 `u8` 확정은 LocalHost 양 정상 join과 dependency의 기존 종료 결과가 관측된 뒤만 허용한다. 원 error가 있으면 그 code/details를 유지하고 stopped 성공은 출력하지 않는다. EOF·오류 없는 전체 확인에서만 기존 stopped payload를 낸다. 출력 실패도 성공으로 바꾸지 않는다.

**root boundary의 남은 정책 차단:** 현재 player-cli main은 runner::run의 u8을 즉시 ExitCode로 반환한다. Pending owner가 올라온 경우 정상 return은 원 owner를 Drop하므로 허용할 수 없다. 새 무기한 재관측 loop/강제 process exit/전역 저장·누수 중 어느 것도 이 제안에서 채택하지 않는다. 따라서 pending을 가진 최종 run 결과와 main 보관/재관측·중단 동작이 중앙에서 결정되기 전에는 CLI4 전체 배선을 완료 가능한 범위로 승인하지 않는다. 독립적으로 가능한 일은 오류를 모으는 host 소유 bundle/owned 결과 서명의 source-only 검토 및 carrier 실제 검증 준비이며, 현재 제품의 u8 경계를 반쪽 교체해 정상 종료처럼 보이게 하지 않는다.

인수 시나리오는 host4 각각 정상EOF/readywrite/flush/read 오류, 이미accepted 작업·deadline 경과에서 D Drop0/첫 error 전필드 불변, 반복poll과panic에서도소유반환, 실제양join뒤만D회수, dependency 종료단계의원실패·성공출력분리다. 최종 u8 정책이 미결인 동안 실제 CLI EOF 통합 성공을 주장하지 않는다. 현재 UI heavy이므로 Cargo/root/index/shared0이다.

## Outcome — 결과

CLI native/playback/provider/jobs의 정상 EOF, ready write/flush 오류, stdin read 오류를 공통 종료 carrier로 모은다. 원 LocalHost Pending과 의존 host/skip owner를 함께 보관하고 양 worker join 이전에 폐기하거나 stopped를 내보내지 않는다. GUI main/audio/fixture는 이번 제안 범위에서 제외한다.

## Authority — 담당

| 범위 | 담당 |
| --- | --- |
| source6 gate·후속 exact 승인·실행 자원 | 중앙 PM. LocalHost6 gate 우선 |
| CLI4와 공통 carrier/driver | 이 세션의 배정 후보 |
| dependency host의 비차단 종료 API | 해당 owner와 PM 별도 배정 필요 |
| 예산 수치·CLI abnormal 종료 허용 | PM 결정. 이 제안으로 고정하지 않음 |

## Evidence — 읽기 근거

`runtime/coordination/local-host-shutdown-r1/caller-audit.md`의 제품6/시험8 인벤토리 중 CLI4만 좁혔다. runner의 각 host run은 현재 u8을 반환하고 player-cli main은 ExitCode로 끝난다. 따라서 미완료 owner를 run 내부에서 버리고 u8을 반환하면 보존 계약을 만족하지 못한다.

추가 읽기에서 **전체 dependency 종료를 CLI9만으로 bounded라고 주장할 수 없는 차단**을 확인했다. `packages/media-jobs/src/infrastructure/worker_host.rs:144,176`은 소비형 shutdown에서 실제 worker.join을 호출한다. `browser_provider_jobs.rs:130`은 먼저 jobs.shutdown을 기다리고 그 후 browser를 최대10초 관측한다. `playback_workspace.rs:92`도 소비형 shutdown이며 native/transition/provider를 정리한다. `mpv/session_engine.rs:159`, `mpv/mod.rs:245`에는 lock·worker join 경로가 있다. LocalHost SDK가 끝났다고 이 하위 worker들이 끝난 것은 아니다. 공통 CLI driver가 이 호출을 그대로 실행하면 그 외부 예산을 강제할 수 없다.

적용 스킬: modoo.today `modootoday/spec-authoring/plan` 및 `plan-authoring` 본문으로 관측/미확정·단계/복구를 분리했다. `code-change-review-verification` 본문/참조로 소비형 shutdown의 동작과 '전체 종료 bounded' 주장을 구분했다. Rust 정책과 소유권을 유지한다.

## Unverified — API·정확 범위 제안과 차단

**exact9 후보**: 기존5 `src/bin/player-cli.rs`, `src/bin/player-cli/{native_host,playback_host,provider_host,job_host}.rs`; 신규4 `src/bin/player-cli/host_shutdown/{mod,carrier,driver,tests}.rs`. root cli 파일은 Windows 모듈 등록만 한다. runner.rs의 u8 dispatch는 바꾸지 않는 안이며, 미완료는 아래 명시 abnormal 경계에서 처리해야 한다. LocalHost6/SDK B15·GUI 공용 경로 수정은 없다.

책임/API는 CLI private 후보이며 공용 SDK의 경쟁 인터페이스가 아니다:

- `ClosingHost<D>`: 원 PendingLocalHost, 원 dependency bundle D, 첫 ApiError Option, 종료 계기(EOF/ready write/flush/read 오류), 마지막 join 관측을 소유한다. D는 각 host의 기존 소유자 묶음(native/workspace/provider/job와 필요한 skip owner)이며 새 정책 객체가 아니다.
- `poll(&mut self)`: source6의 poll을 전달한다. join을 재구현하지 않으며 Pending에선 D를 꺼낼 수 없다. 두 정상 join 후에만 `ReadyForDependencies<D>`로 원 D를 옮길 수 있다. 이것은 전체 shutdown 성공이 아니다.
- `drive_until(owner, deadline)`: monotonic deadline 동안 bounded poll을 수행하고 종료 상태와 **동일 owner를 함께 반환**한다. 새 worker/timer·block_on·무한 join은 추가하지 않는다. 고정 sleep/재시도 정책을 공통 application 기능으로 만들지 않는다. deadline은 CLI 종료 관측 예산이며 SDK 작업 취소나 이벤트 freshness TTL이 아니다.
- 결과는 Pending(owner, snapshot), Partial(owner, observed_error), Panicked(owner, worker_join_status), ReadyForDependencies(D, first_error, local_receipt)로 구분한다. Partial/Panicked도 이미 join한 handle을 다시 처리하지 않고 남은 owner를 유지한다. pipe가 panic했어도 SDK Pending이면 D를 해제하지 않는다.
- 예산 만료는 Timeout 관측이며 원 error가 있으면 대체하지 않는다. 원 ApiError code/message/retryable/details와 첫 failure의 exit code를 유지하고, join 상태·추가 오류는 별도 CLI 진단에 둔다. 기존 error.details를 shutdown JSON으로 덮어쓰지 않는다. 원 오류가 없을 때의 종료 오류 exit code는 기존 ErrorCode 매핑을 사용한다.
- ready output 실패 뒤에도 carrier를 만든다. writer가 실패하면 추가 stopped 출력을 강제하지 않고 원 I/O 오류를 유지한다. stdin 오류도 정상 EOF와 같은 close 절차를 시작한다. LocalHost 생성 이전의 설정/조립 오류는 아직 LocalHost owner가 없다는 사실을 별도 취급한다.

**예산 만료 시 owner 보존 정책**: library형 driver는 owner를 가진 상태로 반환한다. CLI top boundary는 이를 u8로 줄이며 Drop해서는 안 된다. 추가 관측 배정이 없는 경우, owner가 같은 stack frame에 살아 있는 동안 bounded best-effort 진단 후 `process::exit(nonzero)`라는 명시 abnormal 경계를 택하는 안이다. 실행되지 않은 Drop/join을 성공으로 부르지 않고 stopped/host_lifetime_ended=true를 출력하지 않는다. 새로운 상시 대기 thread/전역 leak로 우회하지 않는다. 실제 exit 적용은 PM 승인 전이며 테스트 중 process::exit를 직접 호출하지 않는다.

**dependency 차단 처리**: ReadyForDependencies 뒤 기존 소비형 shutdown을 무조건 호출하는 코드를 이 driver에 감추지 않는다. 전체 CLI 성공 경로를 실제로 bounded하게 하려면 각 원 host가 소유권을 보존한 채 시작/관측할 수 있는 별도 종료 경계가 먼저 필요하다. 구체 SDK API는 이 문서가 신설하지 않는다. PM이 현재 dependency shutdown을 별도 비bounded 단계로 인수할지, 원 담당에게 nonblocking owner-preserving 계약을 배정할지 결정해야 한다. 후자 없이 exact9 전체를 완료 가능한 구현이라고 표시하지 않는다.

## Phases — 완료 조건

1. 이 exact9/API 제안과 dependency 차단을 중앙에 인계한다. source6 gate를 우선하며 아직 코드0이다.
2. 배정 후 generic carrier/driver의 Rust 회귀: accepted SDK barrier→ready/read 실패 또는 EOF→새 dispatch0→양join 전 D Drop0→release→양join→D 원본1회 이동; deadline0/짧은예산은 동일 owner Pending, pipe panic/sdk Pending은 Panicked와 owner 유지, 반복poll/close 원 admission 중복0, 원 ApiError 불변, 출력 실패로 stopped0. fake dependency는 Drop 카운터만 사용하고 새로운 업무 정책을 구현하지 않는다.
3. CLI4는 각각 ready write/flush/read 실패와 EOF를 Rust 주입 Read/Write로 확인한다. 기존 request/response wire와 성공 stopped payload는 양 LocalHost join 및 dependency 성공이 실제 인수된 경우만 유지한다. dependent shutdown이 Pending/오류이면 성공 출력0이다. 실제 subprocess abnormal exit는 별도 자원 배정에서 owner 생존 진단과 비정상 종료 코드를 검증한다.
4. 중앙 슬롯에서 집중 회귀·기존 CLI host/LocalHost 회귀·default/native strict/fmt를 수행한다. dependency 종료 계약과 전체 caller 연결이 인수되지 않으면 남은 범위를 명시한다. 모델/다운로드/GUI 실행은 없다.

## Rollback — 복구

이번 제안은 source6를 수정하지 않는다. 배정 후 carrier 연결이 미인수면 새 CLI 연결을 합류하지 않는다. 단순 drop(local)로 복귀하고 quiescence 성공을 표시하는 방식은 복구로 인정하지 않는다. 원 소스/진단·실패 이력을 보존한다. abnormal process exit는 되돌릴 수 없고 그 전에 실행 중 작업 완료를 보장하지 못한다는 사실을 명시한다. root/index/commit은 중앙만 소유한다.
