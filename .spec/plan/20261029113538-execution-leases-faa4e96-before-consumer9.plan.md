---
id: plan:kinekino/20261029113538
title: 현재 실행·편집 배정
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 현재 실행·편집 배정

중앙 PM 소유, 2026-10-21. 기준 root `faa4e9637974364567e8831e10a3da221eb5338e`, Meari `22fe0d952f5d03d28147d12fcce5f1914fe26bbc`.
이 문서가 현재 권한의 단일 기준이다. 과거 배정은 [보존 이력](history/execution-leases-through-c485b74.md), 실제 인수·commit은 [현재 조율 ledger](current-coordination.md), 전체 완료 조건은 [통합 게이트](full-product-integration-gates.md)를 따른다. 배정 범위는 의향 보고나 goal 저장 오류 때문에 다시 승인 대기하지 않는다.

## 현재 담당

| 담당 | 허용 작업 | 실행 제한 |
|---|---|---|
| 미디어·검증 도구 | harness8r4 최종32·양strict/fmt·보호 인수/반환·root626dca0 채택. C 소비 계약 정합 검토/후속 준비 | 새 코드/Cargo/shared/root/index/브라우저/매체/추론0. C 소비 정확 배정 전 구현0 |
| SDK·CLI | P0 계약5 독립 읽기 인수·strict-first 실제 SDK199 예정·최종 보호/반환 gate | **단일 heavy/shared 소유자**. root/index/CLIflag/OS0. P1/P2·소비/전역출력 연결은 별도 단계 |
| 플레이어 UI | own-window6r5 실제86·양strict/fmt·보호 인수/반환·rootfaa4e96 채택. 생산 opt-in host7 source-only | Cargo/shared/root/index/OS/수동GUI/브라우저/미디어/VM/DPI0. 기본 Viewer/main 전환은 추가 단계 |
| Meari | source412의 프레임 통지/borrow/권한 소비 계약 읽기 검토·UI 직접 인계 | 새 API/코드/Cargo/shared/OS0 |
| 중앙 PM | 증거 인수·정확 root staging/단계 commit·gitlink 채택·배정 변경 | exact paths만. push0 |

## 현재 실제 CLI gate

입력은 `runtime/coordination/same-endpoint-owned-r1/source8.json` pin `27B27068841F3D2D9742C6EA5F90ACA87468C566D1D7386171A2EF041B9ABC25`다. PM 직접 계획 검토·독립 읽기에서 actual8/before기존7 일치·새 확정 P1/P2 없음을 인수했다. [gate 계획](../../../runtime/coordination/same-endpoint-owned-r1/gate-plan.md)의 원본과 [이관 계획](../runtime/same-endpoint-owned-process-migration.md)을 따른다. runtime 상대 링크가 환경에서 해석되지 않으면 저장소 루트의 위 경로를 사용한다.

정확8은 tests/support/owned_process_evidence.rs, tests/jobs.rs, tests/jobs/{owned_process.rs,fixture.rs,playback_owner.rs,provider.rs,playback_observation.rs,same_endpoint.rs}다. 최신 B11 반환 snapshot·원 UI13·Meari412의 최종우선 baseline과 before8을 대조하고 exact8만 적용한다. 원 helper4 중 공유1의 additive Deadline 변경 외3은 보호한다. 기존 legacy13은 미이관이다.

실제 fresh player-cli 원 build path/hash·소문자 JOB_CLI_SHA256을 고정하고 no-media 요청/EOF 및 신규regular9, 기존snapshot18, 기존jobs4를 수행한다. actual3category·frame media는 compile-only다. 중복 adapter8/과거 producer pass를 신규 고유 수에 더하지 않는다. stderr alias는 시점별 snapshot이며 live writer가 아니다.

regular 성공 뒤 기존 Rust fixture `owned-process-test-child.exe` 293376bytes/SHA `4C71114DF2DC85A43270A8739BE629B016320DDFAC454A8F4E51D9BF57649BC7`와 원 producer-r4 provenance를 재대조한 경우만 지정 ignored Windows failure1을 실행한다. 본인 process family만 대상으로 하며 GUI/브라우저/미디어 실행 권한이 아니다. 부재/변경 시 새 build하지 않고 미검증을 보고한다.

원 runner SHA `EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D`, conservative2/CPU2/affinity3/BelowNormal, 기존 snapshot/target·offline/locked/-j1/thread1을 유지한다. 각 원 handle/receipt terminal 뒤 다음 실행으로 진행한다. timeout은 종료가 아니다. 원 실패·수리 revision·raw/제품 receipt·request/host 두 Incomplete owner·보호/PID/index와 명시 반환을 보존한다. 오류 객체 최종 Drop의 kill-on-close best-effort를 Empty/제품 worker join으로 표시하지 않는다.

후속 새 gate는 입력 핀/fmt 뒤 구성별 all-target strict를 먼저 수행하고, 통과한 같은 소스로 최종 libtest/실제 CLI를 한 번 빌드·실행하는 순서를 우선한다. 낮은 비용의 type/lint 오류를 긴 링크·실행 전에 수리하기 위한 순서이며 시험을 생략하지 않는다. 이후 소스가 바뀌면 영향을 받는 strict/실행을 다시 인수한다. 이미 진행 중인 gate를 이 순서 때문에 재시작하거나 중복 실행하지 않는다.

## UI source-only와 프레임 인과

base a2d10fb 이후 원2 before/pin을 보존한다. 최초 tests.rs가 약900줄로 늘어 역할이 섞여 PM이 기계적 책임 분리4를 허용해 정확6으로 갱신했다. 원 NativeLoopOwner/registrar/coordinator·실 frame·NoMedia hidden disabled child·원 cleanup/remaining0을 검증하는 ignored fixture1 및 필요한 cfg(test) 조립/기록 포트만 작성한다. 기존pure회귀·fixture 상태·host 조립·외부 supervisor를 계층별로 분리하고 새source6/before를 고정한다. partial C/full request 실제 관측과 제어 주입 한계를 분리하고 추가 경로는 근거로 조율한다.

full HostStamp는 다음 turn에서 바뀐다. FrameSubmitted observer의 원 turn에서 checkpoint.advance/child proof를 검증·소비하고 borrow 종료 후 권한 없는 owned 결과만 저장/ownwake한다. Completion은 결과만 반영하며 old proof를 재사용하지 않는다. observer 호출 전 document/ParentState borrow 해제는 source412 읽기로 확인했으나 실제 실행은 아직 아니다. host/driver 재진입·stale 검사 완화0. parent37의 driver 밖 접근을 제품 turn 인과의 증거로 승계하지 않는다.

inner15초는 새 작업 차단/본인 close 요청이며 hard process exit 보장이 아니다. 기존 owned-process Supervisor로 원 libtest child를 감싸는 outer20초/cleanup reserve2초를 사용한다. 강제 종료는 GUI 정리 성공이 아니다.

원source2·기계분리r2·semanticr3를 보존하고 최종 source6 `D37BF16B80F386DA2DAC8E48639E6C17A02B3CFCA5B4B788ECE6BDD6D27B1515`와 인계 문서를 PM이 직접 읽었다. source412 교차 읽기에서 새확정P1/P2가 없고 원N-frame→실Completion exactN+1/fullscope·ownwake 기록의 시험 공백 수리를 인수해 UI에 다음 단일 gate를 배정한다. root626dca0/미디어최종source8·B11·UI13·Meari412 최종우선 baseline을 보호한 뒤 정확6만 적용한다. default/native all-target strict와 fmt를 먼저 인수하고 같은 최종핀 Native libtest를 새로 컴파일해 pure1+원84 영향범위의 실제명을 인수한다.

위 단계가 통과하고 같은 source/binary/hash가 유지될 때만 ignored own-window exact1을 명시 실행한다. parent가 같은 libtest의 정확 child 모드를 원 Supervisor로 감독하고 본인 parent/child·원 registrar/slot/frame/Destroy/remaining0·원 raw/receipt/실패 owner를 관측한다. parent composite1만 고유 수에 넣고 inner결과를 중복 합산하지 않는다. timeout/forced·0selected·typed실패는 성공이 아니며 원실패와 새수리pin을 보존한다. observer/host 실제 생산 연결·영상/시각/focus·VM/DPI는 이 검증으로 완료되지 않는다. 최종 source/before/보호·binary/모든 원handle terminal·프로세스identity/PID재사용·index/명시 반환을 마감한다.

## 공통 보호

root/index/gitlink는 PM만 변경한다. 담당은 독립 overlay/source pin·before·최종 상대 핀을 보존한다. 다른 담당 코드·옛 Cargo/lock·library/docs 전체를 복사하지 않는다. 실제 지원 flag·legacy 거절·원64/readcell/Weak 수명/원8KiB·first error를 보존한다. 새 필수 경로는 책임과 이유로 조율한다.

SDK P0는 [C0 계약](../../sots/contracts/playback/native-result-export/index.md) 및 [정확 계획](native-result-export-c0-producer.md)을 PM이 직접 읽고 배정한 source-only5다. playback_session/{native_result_export.rs,mod.rs}, 신규 native_result_export/{codec.rs,metadata.rs,projection.rs}만 편집한다. strict v1 lookup codec과 권한 facade를 구분하고 원 legacy/default Unsupported·구조체 literal 호환을 유지한다. scope37/ref54 및 envelope/item 수치와 escaping/pretty/header/newline 경계는 설계값이며 작성 회귀·실제 인수 전 검증 완료로 표시하지 않는다. 기존 원row/actual target stamp·C8 전달·CLI/공유출력 구현은 이 단계에 섞지 않는다. 독립 overlay/before/source5 pin·직접fmt/회귀작성·읽기 인계까지 수행하고 Cargo 실행은 아직 없다.

최종 source5 `EE8B1724C873A8F9D8A2AC44178CD81C1341A878DD9035CAF7C7A1B8434A90B0`는 독립 읽기 actual5 길이/해시·기존2신규3·새확정P1/P2 없음으로 인수해 다음 단일 gate를 배정한다. gate-plan/최신 rootfaa4e96·own-window 최종source6/integrated6·media8/B11·Meari412 최종우선 baseline 및 before5/새3부재를 고정한 뒤 추가 승인 대기 없이 실행한다. exact5만 적용하고 default/native all-target strict+fmt를 먼저, 같은 최종핀 Native fresh binary로 원SDK184+신규15 예상199를 실제명으로 인수한다. default runtime0/OS0이며 원실패·수리pin/출력·보호/binary/원receipt/PIDidentity/index·명시 반환을 보존한다. 실제 GUID/Weak issuer/row/target packet/consume삭제전검증/pretty·최종framing cap·CLI는 후속이며 P0 성공으로 승계하지 않는다.

own-window 최종r5 source6 8A98F971…A6C06B9는 제어85+실OS parent composite1·양strict/fmt·15receipt terminal(13zero+원101둘)/31PID부재·보호delta0로 인수하고 rootfaa4e96에 채택했다. 실제 원N1→CompletionN2/fullscope·원등록/partial/disabled/Destroy/remaining0·nested family Empty/forcedfalse는 실제 fixture 증거이며 메인 UI/영상의 성공은 아니다.

UI 다음 source-only 정확7은 quarantine_surface/{frame_delivery.rs,frame_delivery/tests.rs,mod.rs}, player_shell_host.rs, player_shell_host/delivery.rs, viewer_host/{mod.rs,tests.rs}다. [host 후속 계획](presentation/native-surface-host-follow-up.md)의 opt-in frame 소비·원 deadline/budget·같은 coordinator·기존 route observer 조합·owned 결과 반영만 구현한다. default 생성자/ViewerNoChild와 SDK callback/ACK/close/원 shutdown 정책을 유지한다. 기본 Viewer 전환과 영상권한·실제 main 연결은 추가 범위이며 새 executor/복사 snapshot 권한/검사 완화로 대체하지 않는다. 독립 overlay/before/source7·직접fmt/회귀작성/읽기 인계까지 수행하며 실행은 아직 없다.

Pages·VM·DPI·새 장시간 재생·AI 추론/다운로드·수동 GUI 검토는 보류다. ChatGPT/Terminal·다른 앱/프로젝트를 종료하지 않는다. 개별 read-only 분석 결과나 작성 회귀를 실제 실행·시각 일치·사용자 품질 승인으로 표시하지 않는다.
