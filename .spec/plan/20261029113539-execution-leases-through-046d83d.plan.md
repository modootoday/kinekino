---
id: plan:kinekino/20261029113539
title: 현재 실행·편집 배정
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 현재 실행·편집 배정

2026-10-21 중앙 PM 소유. root `046d83d6fe22b348836e520cb951517268edf9ac`, Meari `22fe0d952f5d03d28147d12fcce5f1914fe26bbc`.
이 문서만 현재 권한을 정의한다. [직전 배정 이력](history/execution-leases-faa4e96-before-consumer9.md)의 완료된 실행 지시는 재실행 권한이 아니다. 결과는 [인수 ledger](current-coordination.md), 전체 범위는 [통합 게이트](full-product-integration-gates.md)를 따른다.

| 담당 | 현재 권한 | 제한 |
|---|---|---|
| SDK·CLI | P1 최종219 인수·root채택. P2 전달5 source-only | Cargo/shared/root/index/OS0 |
| 플레이어 UI | opt-in host7 독립 source-only 구현·회귀작성·인계 | Cargo/shared/root/index/OS0. default Viewer/main 전환0 |
| 미디어·검증 도구 | 출력8 독립읽기 인수·strict-first 실제 회귀 gate | 유일한 heavy/shared 소유자. root/index/OS/실CLI0 |
| Meari | source412 프레임·borrow·권한 계약 읽기 및 UI 직접 지원 | 코드/API/Cargo/shared/OS0 |
| 중앙 PM | 정확 증거 인수·root staging/단계 commit·소유권 조정 | exact paths만, push0 |

## SDK P0 단일 실행

**최신 전환:** P1 ABC598 원핀/수리0·최종219·양strict/fmt·20receipt terminal0/40PID부재·보호delta0로 인수·반환해 root046d83d에 exact7/882+37- 채택했다. 아래 P1 실행배정은 종료 이력이다. SDK 다음 source-only 정확5는 parent_binding.rs,deferred_engine.rs,parent_router/{mod.rs,tests.rs},session_engine.rs다. expected opaque issuer Observe만 enabled 원Weak확인으로 좁게 전달하고 일반 standalone issuer/무인자router Unsupported를 유지한다. 명시원routehandle/epoch/binding capture·sameadmissiontoken preenqueue1·postbind0·옛ref currentroute retarget0을 검증한다. 독립before/source5·직접fmt·회귀작성·읽기인계까지, 추가필수경로는 근거조율한다.

**현재 P1 실행 배정:** source7 `ABC598812F754442AC2ADC08DD1941111384D31FC00A7E7BC64D44F23A2751BD` actual7/before·samecell/원Weak/target/삭제전origin·lock/charge 독립읽기에서 새 확정 P1/P2 없음으로 인수했다. rootbf98/소비9r2 반환 baseline과 library412를 보호하고 정확7만 합류한다. gate-plan 기록 뒤 추가승인 대기 없이 양all-target strict+fmt 먼저, 같은최종핀 Native SDK 예상213(기존199+신규9+skip소비추가5) 및 integration6=예상219를 실제명으로 인수한다. 신규 engine3 별도filter를 누락하지 않는다. default runtime/OS0·미완성UI/출력6 합성0. 최종 source/before/비입력/library/CargoLock/binary·원receipt/PID신원/index·명시반환을 보존한다. 아래 P0 종료 시점의 heavy없음 및 P1 source-only는 과거 상태다.

**현재 정정:** P0 아래 절차는 최종r3 고유199·양strict/fmt·21receipt terminal(원101 둘 포함)/42PID부재·보호delta0로 인수·반환됐고 root3032faa에 정확5/1402+ 채택됐다. 현재 heavy/shared 소유자는 없다. 재실행 권한이 아니다.

SDK 다음 source-only는 [생산 계획](native-result-export-c0-producer.md)의 P1 정확6이다: guarded_command/{exports,results,scope,issuer}.rs, application/playback_session/guarded.rs, infrastructure/mpv/session_engine.rs. root3032 기반 독립 before/source6·직접fmt·회귀작성·읽기 인계까지 배정한다. GUID1회/실패캐싱·원Weakissuer·원row/ref/64/8KiB·samecell preenqueue·실packet target stamp·metadata·legacyrow비노출·consume삭제전scope/row/origin검증·원lease/lock순서를 유지한다. P0/C8전달3/소비9/CLI/공유출력 수정0. 실제 지원false를 유지하고 추가 필수 경로는 근거로 조율한다.

원P0 및 기존 guarded의 issuer 경계를 직접 읽고 P1을 정확7로 확대했다. 추가1은 application/playback_session/native_result_export.rs의 crate-private SourceV1::matches_issuer(defaultfalse)와 facade require_issuer뿐이다. 원 Weak ptr 신원 확인 및 mismatch pre-observe0 회귀를 포함하고 공개 mint/nonce 문자열 권한 대체0을 유지한다. root3032 before7·별도 delta를 보존하며 다른P0 helper/mod는 수정하지 않는다. 위 정확6 문단의 전체 P0 수정0 제한 중 이 추가1만 예외다.

[P0 계획](native-result-export-c0-producer.md), [C0 정본](../../sots/contracts/playback/native-result-export/index.md), `runtime/coordination/sdk-native-export-c0-p0-r1/gate-plan.md`를 따른다. 정확5는 playback_session/{native_result_export.rs,mod.rs}, 신규 native_result_export/{codec.rs,metadata.rs,projection.rs}다.

원 r1 EE8B1724…34A90B0와 before5·strict101 둘을 보존한다. r2 codec 쌍 순회1, r3 시험 불필요 참조1 기계 수리의 최종 후보는 `runtime/coordination/sdk-native-export-c0-p0-r3/source5.json`, SHA256 `F536F17BC4F654930502704A3D282D74351F7F7CC3DCE10E6B088AD6EF694E3B`. 독립 읽기 actual5/delta·불변3·의미/권한 일치·새 확정 P1/P2 없음은 실행 통과가 아니다.

own-window protected-snapshot→integrated6-r5→Meari412 최종우선 baseline을 사용하며 root와 snapshot이 같다고 가정하지 않는다. B11 중 변경2를 선언하고 나머지9/UI6/media8/library412/Cargo/lock/비입력/root before2·신규3부재를 보호한다. default/native all-target strict를 먼저, 같은 최종핀 Native fresh binary로 예상199(기존184+신규15/helper2 포함)을 실제명/선택수로 인수한다. broad18의 기존3 중복 합산0. fmt·최종 source/binary/receipt/프로세스 신원/index 대조 뒤 즉시 반환한다. default runtime/OS0. 실제 issuer/row/packet target/삭제전 consume/C8/wire/CLI/지원flag는 후속이다.

## UI source-only7

[호스트 후속 계획](presentation/native-surface-host-follow-up.md)의 정확7: quarantine_surface/{frame_delivery.rs,frame_delivery/tests.rs,mod.rs}, player_shell_host.rs, player_shell_host/delivery.rs, viewer_host/{mod.rs,tests.rs}.

같은 registrar/coordinator/request/deadline/budget으로 opt-in NoMedia를 연결한다. FrameSubmitted 원 turn에서 checked proof를 소비하고 borrow 종료 후 권한 없는 owned 결과만 저장/ownwake한다. Completion은 결과만 반영하며 stale proof 재사용·검사 완화·driver 재진입0. 기존 route observer·SDK callback/ACK/close/shutdown·기본 생성자/ViewerNoChild를 유지한다. 추가 main/영상 경로는 별도 배정이다. 독립 overlay/before/source7·직접fmt·회귀작성·읽기 인계까지다.

## C0 소비 source-only9

**현재 출력8 실행 배정:** B35290B4BB2DEACC3B2A5E0BCDAFB5A291E1F42DE12E728DAE0B2DD09A61F53E actual8/before·오류순서/공통writer/할당전cap 독립읽기와 PM handoff/gate-plan을 인수했다. 최신P1 반환 protected→integrated7→library412 baseline과 root046d83를 보호하고 exact8만 적용한다. 양strict/fmt 먼저, 같은최종 Native library60+integration6+선택CLIunit7=예상73를 실제명으로 인수한다. broadCLI의OSfixture/실CLI/미디어/default runtime0. 기존선언overlap2·원bundle/소비7/P0P1/UI/library/docs/CargoLock·원receiptPID신원/index/명시반환을 보존한다. gate-plan 최신baseline 고정 뒤 추가승인 대기 없이 실행한다. UI/P2 미완성 합성0.

**최신 상태:** 아래 소비 gate는 최종r2 source9 5DCE6851…E319901 고유55·양strict/fmt·7receipt terminal/14PID부재·보호delta0로 인수·반환 후 rootbf98c4e에 채택됐다. 현재 heavy/shared 소유자는 없다. 재실행 권한이 아니다.

미디어 다음 source-only 정확6은 application/api/{capabilities.rs,mod.rs}, application/local_host/mod.rs, infrastructure/local_host/{server.rs,client.rs}, bin/player-cli/runner.rs다. [생산/공유 계획](native-result-export-c0-producer.md)의 전역등록·공통bounded compact/pretty/fallback·98299JSON/98304wire/header/newline·client할당전cap을 책임별로 연결한다. 기존4MiB/타명령정책과 공식hostconstants를 유지한다. 독립 rootbf98 overlay/before/source6·직접fmt·회귀작성·읽기 인계까지다. SDK P1/C8/소비9/CLI조립2/registry activation 수정0, 실제지원false 유지. 추가 필수 경로는 근거로 조율한다.

출력 범위는 정확8로 확대했다. 추가2는 application/skip_segments/host/{mod.rs,dispatch.rs}의 공통 crate-private strict preflight 재사용 경계다. 전역 신규4 등록 뒤 default UnavailableSkipHost에서도 shape→context→availability 순서를 지키며 parser/DTO 정책을 복제하지 않는다. 기존 legacy 순서·실제지원false·activation0을 유지하고 default malformed/context/Unsupported 회귀를 작성한다. 원 소비9 최종핀/실행증거와 rootbf98 before추가2·별도delta를 보존한다. 위 소비수정0 제한은 이2만 예외다.

**현재 실행 배정:** source9 `9161B39814AECCA9DF477F67105A625385939F45B4C7B40305DE80A05CA99159`와 before기존8/신규1·원오류순서/권한/actual9 독립감사 및 PM 직접 인계를 인수했다. gate-plan 기록 뒤 추가승인 대기 없이 root3032/P0 최종 protected-snapshot→integrated5-r3→library412 baseline에서 exact9만 합류한다. default/native all-target strict+fmt 먼저, 같은 최종핀 Native fresh libtest skip_segments 예상31+native_result_export18 및 integration native_skip_cli6=예상55를 실제명으로 인수한다. default runtime/OS/실CLI0. SDK P1/UI 미완성 소스 합성0. 최종 입력/보호/receipt/PID신원/binary/index 대조와 명시 반환을 보존한다. 아래 source-only 문단은 작성 단계 이력이며 이 gate만 실행 예외다.

[소비 계획](native-result-export-consumer-c0-review.md)의 표9: skip_segments/native_port.rs, host/{registry,contracts,dispatch,schema,mod}.rs, api/client/skip_segments.rs, skip_segments/service.rs, tests/native_skip_cli.rs.

P0 최신 공개 v1 실제 서명/핀을 SDK와 직접 조율한다. bounded binding/state 및 선택1 metadata getter를 같은 잠금/요청에 재사용하고 current_required·evaluate/cancel 오류 순서·원 SDK Ok reference를 보존한다. 독자 codec/map/권한발급·Unsupported 우회·dummy 성공0. SDK 생산6/C8전달3/CLI조립2/공유출력6 수정0. 생산/wire 미연결·지원false를 구별한다. 독립 overlay/before/source9·직접fmt·회귀작성·읽기 인계까지다.

## 공통 자원·보호

runner SHA `EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D`, conservative2/CPU2/affinity3/BelowNormal, 기존 public-p3-parent-r1/native cache·offline/locked/-j1/thread1을 유지한다. 원 handle terminal 뒤 다음 실행으로 가며 관측 timeout만으로 재시작하지 않는다. 원 실패/수리/raw·제품 receipt/Incomplete owner/PID 신원·재사용을 보존한다. CPU Job 제한은 제품 종료·worker join 증거가 아니다.

root/index/gitlink는 PM만 변경한다. 전체 overlay/Cargo/lock/library/docs 복사0. 추가 필수 경로는 책임·근거로 조율한다. 작성 회귀·읽기 감사는 실행 성공이 아니며 좁은 fixture 성공은 실제 제품·시각 품질 승인이 아니다. Pages·VM·DPI·새 장시간 재생·AI 추론/다운로드·수동 GUI 검토는 보류한다. ChatGPT/Terminal·다른 앱/프로젝트를 종료하지 않는다.
