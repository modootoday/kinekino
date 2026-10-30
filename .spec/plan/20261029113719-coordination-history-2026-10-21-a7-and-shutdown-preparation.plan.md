---
id: plan:kinekino/20261029113719
title: 현재 세션 조율
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 현재 세션 조율

2026-10-21 KST. 중앙 PM 소유. 전체 제품 목표는 진행 중이다. 실행 성공·기능 지원·실제 OS 동작·시각 일치·사용자 품질 승인을 구별한다.

## 현재 기준

- root HEAD: dc4453ec6c1ba03dcb9e46a4fe376309e5d1a601.
- Meari gitlink: 98f6a5c4c72ad07a40ac0eae7f7f113926609fae.
- root index/단계 커밋/의존성 인수는 PM 단독 소유, push0. source staging은 담당이 소유한다.
- shared snapshot: public-p3-parent-r1, SDK30·UI준비6·문서4까지 인수한 상태. 다음 overlay 전 전체 보호핀을 새로 만든다.
- 현재 heavy/shared snapshot은 SDK A7 실행 검증에 배정했다. 하나의 실행만 진행한다.

## 담당과 다음 결과

| 담당 | 현재 작업 | 실행 경계·다음 인수 |
|---|---|---|
| PM | 근거 인수·독립 리뷰·공유 파일·자원·단계 Git | 최종 source/receipt/PID/보호 SHA 확인 후 exact 경로만 커밋 |
| SDK·CLI | A7 실행 검증·B15 별도 소스 구현 | 단일 heavy/shared snapshot 소유. A7 고정 입력과 B15 혼합0, 실제제출false |
| UI | host7 통합 완료·실 viewer H/공통 startup·cleanup 범위 준비 | 별도 staging/추가Cargo0·OSGUI0 |
| Meari | queue harness 생산자22 시험·정적검사 완료 | 원후보/증거 보존, 추가Cargo0·UI 읽기지원 |
| 자막·스킵 | LocalHost 명시 shutdown·실제 worker join 계약6 구현 | 별도 staging/Cargo0. 스킵소비 원업무 유지 |

## 최근 인수한 제품 변경

| commit | 범위 | 실제 검증과 한계 |
|---|---|---|
| 87992b3 | Meari d669 gitlink | library17/기존 제품 소비자98pass. 실제 OS 미검증 |
| a8efdb6 | SDK30: surface 수명·복구·receive observation·API fixture | 전체 native627pass/29ignored, default523pass/24ignored, 양 strict/fmt. 실제 HWND/미디어 guard 제출 미인수 |
| f402e07 | view.start 준비6 | 고유19pass/ignored1·양 strict/fmt. source graph·원 owner·route 희망만 준비 |
| ca78ea6 | 실제 문서4·공통 viewing styles/assets | 고유24pass/ignored1·양 strict/fmt. CPU module/document/handle 구성이며 OS 창·재생 성공 아님 |

구성 사이 및 집중시험·전체시험 사이의 중복을 합산하지 않는다. 근거는 runtime/quality-review/sdk-native-combined-r1, ui-view-start-r1, ui-view-document-r1과 기능별 정본에 보존했다. SDK 원 compile/strict/full/fixture 실패4개와 parity·snapshot670ms 중첩 운영 위반은 보존하며 엄격 직렬 운영 성공으로 표시하지 않는다.

## 현재 수정·설계의 중요한 경계

UI host 원 observer는 borrow 실패에도 Ok를 반환해 마지막 frame notice가 소비되는 P2가 있었다. r2는 typed failure/Err→Paused와 원 observation의 명시 retry를 연결했다. 아직 실제 queue 회귀 전이다. Paused 중 같은 kind의 새 notice는 latest coalescing으로 옛 notice를 대체할 수 있으므로 exact notice 영구보존을 주장하지 않는다. generic H와 configured SDK/LocalHost/ExecutionOwner를 소유한 실제 viewer H의 생산·정리 연결도 남아 있다.

SDK A7은 공용 seconds→us 투영/guard private view/원 enqueue ControlReceipt/read-only ticket clone/result getter를 구현한다. pending consume·terminal 단일 소비·late ACK·owner 종료 뒤 조회의 실제 동작은 B/C registry/protocol 구현이 필요하다. 8RPC는 before reply→6속성 reply→after 요청의 순서를 지켜야 한다. revoke는 같은 fence의 in-flight count를 보존하며 logical write-commit과 물리 write를 구분한다. result registry는 Active 제거 뒤 원 ticket 조회를 유지해야 한다. 실제 고정 mpv/CLI/media 인수 전 seek 지원 false다.

## 자원·검증 정책

고정 resource-runner conservative2/BelowNormal/descendant Job, offline/locked/-j1/thread1 및 기존 캐시를 사용한다. exec가 session_id를 반환하면 원 handle을 끝까지 회수하고 receipt terminal을 확인한 뒤 다음 명령을 실행한다. timeout은 종료가 아니다. 사용자 ChatGPT/Terminal을 종료하지 않는다. runtime 전체 검색·중복 빌드·반복 실미디어 시험은 피한다.

Pages·VM·DPI·새 장시간 재생·AI 추론/다운로드·수동 GUI/시각 인수는 보류다. 컴파일과 in-memory 렌더 검증은 실제 창/재생/시각 승인을 대신하지 않는다.

## 다음 필수 게이트와 연결

[전체 제품 통합 완료 조건](full-product-integration-gates.md)을 유지한다. Source 세 분류 실제 CLI→Frame-host→동일 native host 재생, 실제 OP/ED, 다운로드·제작 executor, 기본 native 창·재생·close, 마지막 AI 제품 IO·사람 품질 검토가 남아 있다.

[Native shell](presentation/default-native-shell.md), [공통 guarded 계약](native-guarded-observation-contract.md), [스킵 소비](native-segment-skip-observation.md), [렌더링 비교](native-video-composition-comparison.md)를 따른다. 상세 단계·원 실패·인수 이력은 [SDK30·UI 문서 이전 조율 원본](coordination-history-2026-10-21-sdk30-and-view-documents.md)에 그대로 보존했다. 이 문서는 현재 배정과 최신 기준만 소유한다.


## 현재 후보와 실행 배정

Meari test-support candidate98f6a5c4c72ad07a40ac0eae7f7f113926609fae/source405 manifest1D43C0A7C47740E46A0CA6EDBA3952F5F9428B1D4C64D9F6DF7E236CBC1F15F5를 독립읽기 인수했다. exact5이며 기본off, 고정Created/GeometryChanged metadata만 발행하고 frame/window/parent권한은 만들지 않는다. 원queue·비파괴pending read·Paused/retry/latest-kind회귀2를 재사용한다. 생산자에게 단일heavy로 집중시험/관련window회귀·기본window/testsupport strict·fmt·최종정리를 배정했다. rootgitlink는 아직d669이며 소비자UI7 검증 전 자동채택하지 않는다.

UI host는원4에서production observer factory/deliverytests/Cargo opt-in개발feature를 추가한exact7 source-only범위를배정받았다. 원F622/r2를보존하고 같은실콜백+원queue/actualRefCellborrow로P2재발을검사한다. 단일notice재전달과Paused중동종latest교체를구분하고 syntheticprobe로route확정하지않는다. UI새Cargo는아직배정하지않았다.

UI r3 exact7 pin098B31D4F03E2720F5B088BFF0F1444B605F7586B18386D929F48077AEC03D5E는 독립읽기에서실productionobserver/actualRefCellborrow와원queue를행사하는2개재발시험이확인됐다. 원Ok삼킴으로되돌리면CallbackErr첫단언이실패한다. 기본off개발feature/lock0/7SHA0, authored총6·실행0이다. Meari생산자반환뒤소비자gate배정예정이며현재제품새Cargo0. SDK A7 source는runtime/coordination/sdk-guarded-a7-r1에작성돼별도독립소스감사를시작했다.

SDK A7 최종source7 manifest C84156393F2959FF5DDDCE68849B0CDE8E10130DD728A011B10DFFF2A44E0AB7를 독립읽기인수했다. 집중9는신규8+기존Unsupported1이며실행0이다. existing5.patch에등록mod2가포함돼 registration2와연속적용하면안된다. 다음실행슬롯대기중 원A7불변을보존하고 B13을별도source-only후보로작성하도록배정했다. 순차8RPC/동일fence/Active외부registry/예산·소비계약을준수하며 private issuer 접근을위한추가application경로는작성전보고한다. production권한false·C조립미완료·Cargo0를유지한다.


Meari생산자 gate는고유22pass(신규2/notice3/기존surface17), strict-window52255ms/testsupport29519ms/fmt2509ms0·6receiptterminal0/12PID부재·명시heavy반환으로마감했다. PM source405전체재해시delta0/현재PID0를확인했다. UI에new405+제품7의snapshot소비자gate를배정했다. rootgitlink는d669유지하며소비자검증후중앙채택한다. before원403·새2부재·명시delta5와UI7만합류하고기존보호를유지한다. SDK A7 source gate는이후슬롯예정이며B15source-only 구현은계속한다.

UI 소비자gate runtime/quality-review/ui-view-host-r3은before12(제품기존2/신규5+library기존3/신규2),root935/snapshot1293을보존하고librarydelta5·제품7만합류했다. source405전체SHA일치0,integrated7 68D26664CDB098A4940E3BFEC9A2ED40AB847E288958E7114399AD15ECBF79BD/원098B불변. host6 첫원session64071만진행하며고유30예상(host6+기존24)은실제로그로확인한다. rootgitlink/index/OS0유지다.


## 2026-10-21 host7 인수와 다음 병렬 배정

UI 소비자 actual30pass/ignored1·default/native/testsupport strict/fmt·10receipt0/20PID부재·원405/SDK30/제품보호불변·명시반환을 인수했다. PM은보호root935/snapshot1293대조후 Meari405를checkout·전체SHA확인하고gitlink exact1 commit2c0bd5b46991d7079d20fb54dfc8f59d9c2b188b로채택했다. submodule문서2는status표시가있으나gitdiff내용변경0·동결405바이트일치상태를보존했다.

제품source7은runtime/coordination/pm-host7-adoption에원root기존2/신규5를보존후exact바이트로인수했다. Cargo.toml은snapshotbefore와CRLF차이만정규텍스트동일을확인해line-endings.json에원SHA/참조SHA보존. 복사운영첫명령의루트파일빈부모경로오류는제품변경전에종료됐고부모가있을때만생성하도록수정했다. exact7(659+) commitdc4453ec6c1ba03dcb9e46a4fe376309e5d1a601, indexempty/push0다.

단일heavy/sharedsnapshot은SDK A7 원핀C841…0AB7 검증에배정했다. B15후보는별도소스로혼합하지않는다. UI는기존executionstartup공통추출/실ViewerHost/carrier·공통cleanup/thinlaunch 범위를LocalHost계약과조율한다. 자막담당은LocalHost server/executor/mod+shutdown/shutdown_tests/executor_tests exact6을별도source-only구현한다. admissionclose/accepted상태는동일Mutex로직렬화하고기존pipe/SDKworker의실join을분리해Pending원owner보존을검증한다. LocalHostDrop을SDK전체quiescent로대체하지않는다. 실제OS/positiveframe/close·재생/main·시각승인은남아있다.

## 세션 역할 갱신과 다음 source 검토

사용자 요청으로 프로젝트 5개 세션 이름을 현재 책임에 맞게 변경했다. 중앙 PM · 세션 조율·통합 검증 / SDK·CLI · 재생 상태·명령 안전성 / 플레이어 UI · Meari 통합·네이티브 실행 / Meari · Native/Web 기반·관측 지원 / 재생 구간·자막 · LocalHost 종료·수명 관리다.

SDK A7는 기존5 EOL 차이를 정규텍스트 비교한 뒤 원 frozen bytes로 공유 snapshot에 합류했다. Cargo 결과는 아직 인수하지 않았다. B15 r2 source13 5AC4491B93412300CDBD3B004A36BFEAE4E6C9697C6A7F973566CC63447F52B0/application2 1003EF5DFBD7371FF8AF7DEAA3E4D913CB8B51A71CCCD99445152FDC7A7992FD는 작성 시험22·실행0이며 별도 읽기 감사에 배정했다. 두 채널 전체 순서와 barrier 중 legacy write 가능성을 감사하며 production issuer/C 조립 완료로 보지 않는다.

LocalHost exact6 manifest CBF11D50925E7C5FFC18D0CE5EB73711E6705C21A0622F432235055A9916F690는 작성 시험8·실행0 상태로 독립 감사 중이다. Pending Drop은 detach로 join proof를 잃으므로 원 owner 보존과 pipe/SDK 양쪽 실join을 별도로 인수한다. SDK A7 실행 슬롯과 섞지 않는다.

UI에는 runtime/coordination/ui-viewer-production-preparation-r1/scope.md의 1단계 공통 startup 추출을 source-only로 배정했다. native_execution_startup/proposal wrapper/mod/tests 정확4 후보이며 기존 공개 API와 원 실행 tuple을 유지한다. cleanup driver·ViewerHost·thin launch는 이후 별도 경계다. LocalHost6은 감사·실행 전 소비 인수하지 않는다. 타 프로젝트 discrete-circuit Cargo는 사용자 작업이므로 종료하지 않으며 현재 자원 관측과 제한 프로필을 기준으로 SDK 담당이 실행 시점을 판단한다.

SDK A7 실행 중간 관측: guarded-default 원 receipt87622ms/terminal0 및 실제6pass, projection-default2423ms/terminal0 및 실제3pass를 PM이 원 로그에서 직접 확인했다. 집중 고유9만 확인된 상태이며 영향 회귀·default/native strict·fmt·보호 SHA·실 PID 종료·명시 슬롯 반환은 아직 인수하지 않았다. resource-read의 CPU18%/free23788424KiB 관측을 확인했으며 단일 SDK 제한 실행을 유지한다.

LocalHost exact6 독립 읽기 감사는 소스 해시 일치·확정 신규 버그 없음으로 인수했다. 기존 Drop의 동기 pipe join 제거는 호출 서명과 별개인 종료 의미 변화이므로 담당에게 기존 caller/drop/시험의 정확 경로와 명시 Pending 회수 필요 지점을 조사하도록 배정했다. source 인수는 실행 pass나 기존 caller quiescence 인수가 아니다. startup ready.send 실패 시 SDK handle detach도 실제 join 증거로 보지 않는다.

UI 공통 startup exact4 후보 pin3EA641C1D7827470BB838DD122E33DED88A06C81E177D9CAB66DF12932DA924C를 접수해 독립 읽기 감사에 배정했다(runtime/coordination/ui-native-execution-extract-r1). 원 proposal 공개 타입/함수 alias와 Debug label을 보존하며 작성 active3/ignored loop1·실행0이다. 이름 역변환 텍스트 동일은 컴파일/실행 증거와 구별한다. UI에는 별도 overlay에서 공통 cleanup driver 추출을 이어가도록 배정했다. bounded poll/sleep/budget/explicit acceptance dispatch만 공통화하고 proposal 승인·registry 정책과 원 carrier 반환을 유지한다. 두 후보 핀과 검증 경계는 합치지 않는다. LocalHost6 실행 인수 전 ViewerHost 구현은 대기한다.

독립 감사에서 B15 r2 P2 liveness 소스 반례를 확인했다. ipc.rs160–162 legacy write_all.await가 무기한 Pending이면 같은 worker의 guarded deadline/live/read도 멈춰 원 결과가 Pending으로 남는다. OS 재현 사실과 구별하며 r2 원본을 보존한다. SDK에 별도 r3 source-only로 bounded pinned legacy write/실패 연결 폐기/원 guarded terminal 및 조합 회귀를 배정했다. A7 gate에는 섞지 않는다. BarrierUpdate::Complete 표본을 버리는 현재 B만으로 동일 barrier observe→core→guarded submit이 완성되지 않으므로 C의 표본 전달·opaque issuer 발급/소비 시점 계약을 필수 잔여로 유지한다.

UI startup4 감사는 기계적 이동/공개 alias/해시 일치를 확인했으나 ignored loop 시험의 즉시 execution.close().unwrap() 가정이 정상 WorkerExitPending에도 실패할 수 있는 P2를 찾았다. 제품 종료 정책을 바꾸지 않고 기존 bounded poll로 Closed receipt를 기다리는 시험 수리와 별도 r2 핀을 담당에 배정했다. authored 시험은 여전히 실행0이다.

LocalHost caller-audit.md의 제품6·시험fixture8 경로를 인수했다. 직접 민감한 stale endpoint 시험과 CLI4 정상EOF/ready출력/read오류의 원 Pending·의존host 유지가 후속이다. 담당에게 CLI4+공통종료carrier/driver의 정확 경로·API·회귀·예산만료시owner보존 범위 제안부터 배정했다. GUI main/audio/fixture를 한 번에 확대하지 않으며 LocalHost6 실행 gate가 선행한다. 무한 join 또는 미완료 owner Drop 후 stopped 출력은 금지한다. SDK default-list는 receipt terminal=false/launcher114264-child49324·세션 active를 확인한 검증 대기이며 timeout을 종료로 보지 않는다. 집중9 이외 추가 pass 집계0이다.

UI startup r2 pinBAB56CBA81511A8432095D50DC8F5923D258E57BC5E21630173057598B8680A9와 cleanup exact4 pin0B0DE0954BB135C662EDFA8586C7C5DD137413207A9A7E4EA8EDDA21D1EA8248를 접수해 독립 읽기 감사를 배정했다. startup은 시험1의 즉시join가정만수리, cleanup은 generic driver/기존 proposal adapter/작성 fake4이며 실제pass0이다. 두 mod 등록은 각각 additive 병합해야 한다.

CLI scope 제안에서 기존 jobs/provider/native/workspace shutdown이 동기·소비형임을 확인했다. 따라서 전체 bounded shutdown을 새로 보장하거나 강제 process exit 정책을 추가하지 않는다. 담당에게 공통 원Pending+dependency+첫error 보존 carrier/비차단 poll driver/회귀 및 필요한 root등록의 exact4~5 source-only부터 배정했다. 예산만료·Partial·Panicked는 원owner 반환, 양정상join만 ReadyForDependencies이며 CLI4 wiring/u8 최종변환/프로세스 정책과 실제 dependency shutdown은 별도 필수 후속이다. 기존 동기shutdown을 공통driver 뒤에 숨기지 않는다. 실행 슬롯은 A7 다음 LocalHost6, 이후 UI 추출 검증 순서다.

UI startup r2/cleanup4 독립 감사는 새 확정 결함 없이 source gate 인수했다. 공통 startup 작성 active3/ignored1과 cleanup fake4는 실행0이며 실제 proposal adapter 기존 회귀가 필요하다. PM이 extracts-combined-registration.patch의 root read-only git apply --check exit0을 직접 확인했다. 두 원 exact4는 mod 공유로 고유7이며 gate 시 원파일별 핀과 additive 합류 통합핀을 별도로 보존한다. 앞선 합성 명령 exit1은 각각 검사 결과를 분리해 인계하도록 요청했으며 patch 실패로 단정하지 않는다.

다음 LocalHost6 및 UI추출 gate는 정확 대상/필터와 기존 알려진 목록을 사용하고 불필요한 전체 시험목록 재빌드를 피하도록 전달했다. CLIcarrier4는 unregistered source overlay로 작성하며 사용되지 않는 모듈을 allow로 숨기기 위한 root등록을 만들지 않는다. SDK A7 singleheavy 유지, 후속검증 슬롯 자동선점은 없다.

CLIcarrier 신규exact4 source-only manifest298CFA39D0905607F1C8C2D472A588507A00F132523315A554F76CE9A95753F3를 접수해 독립 읽기 감사에 배정했다. 미등록 host_shutdown/{mod,carrier,driver,tests}이며 작성4/실행0, 원LocalHost6 및 SDK A7와 섞지 않는다. 1회advance가원owner/Pending/Partial/Panicked·예산만료를반환하고양정상join만dependency회수가능하다는주장을감사한다. 실제 IO실패주입/CLI4 wiring/최종u8변환 및 dependency shutdown은 여전히후속이다.

SDK status-api-default 원로그 실제3pass/2625ms0을확인했다. session/control/status 필터는 guarded/projection과중복을제외해집계한다. native 구성회귀 receipt107992/24000 terminalfalse·실컴파일로그확인 상태로원handle관측중이며새시험중복실행0이다. C의동일barrier표본전달/opaqueissuer 최소2phase계약은별도read-only리서치에배정했다.

A7 default 관련회귀는집중9와중복제외고유30pass로확인했다(guarded6/session17/statusprojection3/control7/statusAPI3의중복집합제외). native구성session은원실행계속이며fulldefault/native완료나전체제품기능지원으로승격하지않는다.

CLIcarrier4 source감사에서현재advance호출경로는정상이지만 into_ready(self, publicstatus)가caller제공 Joined/Joined를신뢰하는P2캡슐화약점을확인했다. 담당에원r1보존·r2에서원Pending실join직접확인/미완료원owner반환·가짜status회수불가회귀를배정했다. source6와실행슬롯분리유지다.

C연결의최소2phase리서치를SDK에전달했다. 기존boundedchannel에서목표없는observe→원Sample+privateissuer/fence/generation/identity/deadlineguard→application기존core평가→같은guard소비submit로연결하고worker내skip정책/새queue/새표본묵시교체는금지한다. 관측포기/timeout/receiverdrop/cleanup/총자원예산과원ticket read/admission분리를후속정본에포함한다. 이는읽기설계제안이며현재productionauthority=false를바꾸지않는다.

A7 native session17/status3/router12/binding6 원로그pass, default와구성중복을제외한고유48명을확인했다. deferred_engine선택0은pass0이다. strict-default1290ms는Checking출력없음/작성mtime가과거fingerprint보다오래돼stale가능성이있어최종strict근거에서제외하고원receipt를보존한다. 담당은A7exact7bytes불변mtime갱신후strict재검사/새integratedpin을고정중이다. focused9의실제시험이름·양구성재컴파일근거와최종binarySHA를연결하도록요청했다.

CLIcarrier r2 manifestE65068BD8619C8A9F0A158FA5DD61580DD852109BA549FF8996A7BD08C39CCA4를재감사에배정했다. caller가준publicstatus를받는into_ready를없애고원privatePending을직접poll하는try_ready로Ready발급을제한했다. 작성기존4+신규1=총5/실행0이며미등록·source6불변이다. 최종문단의시험수표현불일치는담당에문서정정을요청했다.

A7 마감 중간 직접 대조: 기본 strict-r2 31268ms/native53671ms는 실제 Checking 후 terminal0, fmt3034ms terminal0이다. final difference 7종(root/snapshot/source7/integrated/library405/SDK27/defaultbinary)은모두빈목록이며 final-owned도빈목록이다. native lib artifact153961984bytes SHA F65FB62ACFEAA125F78889ABEA1EEC6151C2487588AFB499ABBEC26771B8CE74를확인했다. 담당의최종정본/handoff·명시반환전까지후속shared쓰기/실행은배정하지않는다. closeout.json이라는파일명은존재하지않아직접읽기실패했으며실제final-owned/final-receipts/차이파일로근거를확인했다.

CLIcarrier r2 독립재감사는원Pending실poll·Ready발급수리와작성5를확인해source인수했다. 실제실행/CLI wiring/dependency shutdown은미완료다. 원r1/source6불변및4사본hash일치,수정시험수문서정정도확인했다.

## A7 인수 완료 · 다음 LocalHost6 실행 배정

SDK A7 최종정본/handoff/closeout의명시반환을인수했다. 집중9·영향고유48, 양strict실검사31268/53671ms·fmt3034ms0, receipt17terminal0/기록37PID부재·보호차이0이다. PM이root before7의기존5/부재2와원SHA를직접확인하고runtime/coordination/pm-a7-adoption에원bytes를보존후검증된source7그대로복사·해시확인·diffcheck했다. exact7(+468/-42) commitfc22d8c88a6dd1b07d6196f5d1e42d2cb1f3ad5b,indexempty/push0이다. runtimeauthorized=false/guardNone/submitresultUnsupported 및B/C·실미디어잔여는유지한다.

단일heavy/sharedsnapshot을LocalHost6 담당에명시배정했다. 최신rootfc22d8c/Meari98f6·snapshot검증완료A7상태를기준으로before6/보호/A7·Meari405/UI7·SDK비중첩을확인후exact6만합류한다. 신규8+기존local_host9직접target·양strictfmt·실compile/시험명/CLIartifact·receipt/PID/소스보호closeout을수행한다. carrier4는혼합0이며rootlive/index0,명시반환전UI또는SDK새heavy0이다. SDK는B15r3독립source-only수리를계속한다.
