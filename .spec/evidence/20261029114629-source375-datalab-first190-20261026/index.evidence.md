---
id: evidence:kinekino/20261029114629
title: 새 호스트 source375 첫 190 gate — Product 68 실패
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 새 호스트 source375 첫 190 gate — Product 68 실패

2026-10-26 build-host. 선택 source375 A09B51ACBC77645FD12D8488B23021173780851E267A8BA5103D34F827294D39, exact190 계획082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464. [B0/초기 B1](../../../evidence/environment/source375-datalab-b0-20261026.md)의 원375 입력·preimage와2103파일 private stage를 사용했다. canonical overlay/stage/commit/push0이다.

## 실제 결과

### 최신 후계 실제 결과 (원375와 구분)

**최신r9 B2/B3 gate완료:** final50Product145/51Core18/52Native26/53OS1은동일r9입력SHA2E6BFA809A8554EBBCB82611997B16C075F28BD231558A76CA516F3341FB216B로각실제등록/ignored·exact190/190·producer integrity Ok이다. 원375/각후계pass합산0. Mearifmt54/strict55 exit0,제품은최종제품소스의fmt39/strict38 exit0이며r9추가는Meari시험fixture뿐이다. normalbins56 actualCargo build exit0/470ms,profile.test=false/bin타깃·freshtrue로정상player/CLI기존산출물일치를확인했다. 원CargoJSON/CLI57version·58capabilities oktrue및EXE/PDBhash/bytes/FileID·UTC초단위관측은 B2-B3-RETURN-r9.md에기록했다. 실제GUI실행/시각·AI품질/전체기능인수완료가아니다. 현재writer/heavy0·canonicalwrites0·index불변이며다음은candidate경로별채택검토/F1기능연결이다.

**OS 후계r9:** 직전턴은r8 Native26통과/OS1 phase3실패를확보한progress였다. 소스대조에서fixtureConsumer.next_close_observation=After5ms인데runtime.new의schedule_limits=None임을확인했다. scheduler는명시예산없는timed관측을거부하는계약이다. privateMeari시험fixture만with_schedule_limits(min5ms/max5ms/observations1)로조립했다. 제품scheduler/guard/deadline10s/검사변경0,independentcanonicalMeari WIP변경0. 원OS실패EXE/PDB는failure-os1-bin-r8에독립보존·44의SHA대조했다. 7번째delta/원본/부모chain은 INPUT-R9.md SHA2E6BFA809A8554EBBCB82611997B16C075F28BD231558A76CA516F3341FB216B에고정한다.

45 OS no-run exit0;46 OS 실제등록/ignored·exact1pass(0.10s)·무결성Ok로영향수리확인. 이는r9의1개pass이며r8의189와합산하지않았다. 같은r9 final no-run47 root/48 core/49 native 모두exit0(49=33844ms),50Product부터전체190재검증을시작했다.47/48/49 CargoJSON을각그룹producer정본으로쓴다. 정상bins/전체제품완료는아직계상하지않는다.

**r8 최신189pass/OS1fail:** no-run42 exit0/271431ms. Native26 gate43 실제등록/ignored·exact26/26·producer integrity 모두Ok. OS1 gate44 실제등록/ignored확인후exact1 failed101/10.02s,outerexit1/12005ms,producer integrity Ok. 이름actual_window_user_veto_preserves_coalesced_completion_action_and_timer_owner,stderr`user close OS dispatch deadline phase 3` atwindow_user_close_os.rs:313. Source probe phase0/1 user-veto/action/coalesced wake 및phase2 one-shot timer를지나phase3 두번째allow=true request_user_close뒤HostLifecycle::Closed 대기에서실패했다. 이를원인확정이나정상cleanup성공으로확대하지않는다. deadline/guard변경·재시도0. 현재Native/OS producer terminal이고추가Cargo writer0,정상bins미실행이다. 사용자요청목표별현황/CLI·GUI완성률분모제한은current-review-build.md의목표기준표에기록했다.회귀189/190=99.5%는전체제품완성률이아니다. 배포는현재목표에서제외하고AI결과/GUI시각후속을유지한다.

**r8 최신163/190:** Core no-run40 exit0/68848ms, gate41은실제Core18등록/ignored와exact18/18pass·producer integrity Ok. Product145(gate37)+Core18(gate41)은동일r8입력receipt로163pass이며Native26/OS1은미실행이다. native no-run42는 --features window/--lib/range_input/source_callbacks/window_user_close/window_user_close_os/locked/offline/jobs1/JSON·C/meari-native를단일producer로시작했다. 아직terminal/27시험/정상bins성공을계상하지않는다. canonical/MeariWIP/두HEAD·두lock·src/application/mod.rs hash·빈index 불변을확인했다.

**최신 r8 Product145/145 통과:** r7에서typed callback assertion은통과한뒤같은110시험이module projection의class선택자오류(None vs Some(true))로실패했다. 실제JSX는className이며native markup에서만class로정규화하므로fixture selector만className으로고친r8을고정했다(INPUT-R8.md SHAF9840E5E0B4D7FC4FC389352CB3AF42304DB8999B8B4FAE60A14BEB0FE21029A). no-run36 exit0/18152ms; gate37은145실제등록/ignored대조후exact1~145전부pass,group execution_result/producer_integrity모두Ok이다. source receipt와casePID·metadata전후·EXE/PDBhash는37원로그다. 서로다른후계의pass를합산하지않았다. strict38/fmt39도exit0이다. 원A09 67pass/68fail은별도보존한다. 현재Core no-run40을단일C/meari-native cache로실행중이며Core18/Native26/OS1의실제pass는아직0,정상bins/AI/UI인수미완료다.

r6/r7 후속: 직전goal턴은 r5의77pass/78fail과실제누락props를확보한progress다. r6는capture caption fixture에생산경로의captureCanReturn guard값을추가하고live핸들을보관해폐기후거부검사를유지했다. 원r5 EXE/PDB를 failure-078-bin-r5에독립보존·원producer SHA대조했다. no-run32 exit0/61666ms, Product33은109pass/110fail/남은Product35+Meari45미실행이다. 실제145등록/ignored·producer integrity Ok.110은actual_owned_start_screen_is_removed_and_restored_without_replacing_video_or_submitting_sdk의 component JSON 문자열contains(onCancelTransitionJob) assertion에서exit101/208ms였다. r7는실제 module projection typed callback table의reference.binding 존재검사로대체한다. text/video node/scope/noSDK·hidden/fullscreen/startscreen 검사는유지하며production변경0. 원r6 EXE/PDB는 failure-110-bin-r6에독립보존·hash일치했다. 각입력/6delta/부모chain은 INPUT-R6.md(EC03AEDCB3C25DE09AF81BB5393E794E742B93E497B6E5B3814F9E35B4106634), INPUT-R7.md(DA8E571285E3B52F40121A0072A03805A313E43FE7FFDCC790D9B84FE41B3163)에고정했다. 현재r7 no-run34실행중,후계성공은원375성공으로대체하지않는다.

- r4 root no-run28 exit0/73389ms. gate29의 실제145 등록/ignored 일치, exact1~76pass/77fail101(199ms)/Product68+Meari45미실행. 원source375 67pass/68fail을 변경하지 않는다. metadata/PID 증거와 최종hash integrity Ok이다. 77은 product_card_callbacks_edit_accept_retry_and_retain_original_ack_evidence에서 hidden diagnostics의 onReadStatus 활성화를 시도해 strict input이 거부했다. r4 실패 EXE/PDB는 failure-077-bin-r4에 독립 보존·SHA대조했다.
- r5는 시험의 실제 onToggleDiagnostics click/drain을 먼저 수행한다. hidden/disabled guard 완화·props강제0이다. 입력 receipt runtime/development/source375-jobs-recovery-fix-r1/INPUT-R5.md SHA184AF49D4CD2AA35CA69E0ADE15C30A33DCB8F01907E88A81EB07CF3D80D9257. 네번째delta와 before/after 및 부모r4의3delta를 유지한다. no-run30 exit0/50339ms.
- gate31은 r5의 실제145등록/ignored를 확인하고 exact1~77pass 뒤78fail101/251ms에 멈췄다. 나머지Product67+Meari45미실행이다. r4/r5/원375의 성공을 합산하지 않는다. 이름은 capture_caption_factory_and_original_export_reject_invalid_foreign_stale_retired_and_closed_inputs이며 capture_surface/caption_contract_tests.rs:21에서 node slot exact declared props 불일치다. primary failure=true, cleanup complete=true/worker stopped/unack0이다. PID/helper/source-receipt 및 metadata와 group integrity Ok는31로그에 보존한다.

다음 읽기 조사: capture_surface의 production projection은 model.props+captureCanReturn(실제 navigation_guards)+inputs.props를 editor node_slot에 전달한다. 해당 caption test fixture는 model.props+inputs.props만 재구성해 captureCanReturn을 빠뜨린다. 또한 surface.stop 후 새 original resolve를 시도하는 후속 test 경로가 보이므로, 원 실패를 보존한 뒤 실제 생산 props 계약 및 live 때 캡처한 queued handle의 폐기 후 거부를 유지하는 최소 test delta를 검토한다. 현재 추가수정/78재실행0이며 재시도는 새 입력 pin 뒤에만 한다. Core/Native/OS no-run·45시험·정상bins·AI결과/UI시각은 여전히미완료다.

- root/Meari fmt·type·strict Clippy 모두 exit0. 필수 Rust crates만 사용자 승인으로 확보했고 원 lock 두 개는 불변이다. 모델/미디어/HF 다운로드·추론·수동 GUI·WARP 변경0.
- 10-root-norun은 원 package/lib+jobs/native-proposal-window/locked/offline/jobs1/JSON producer로 exit0,515995ms였다. 원 Cargo JSON 산출물에서 root libtest와jobs EXE/PDB를 고정했다.
- Product145의 실제 --list 단일등록과 ignored 속성을 모두 대조했다. Core18/Native26/OS1의 실제 등록·실행은 아직0이며190은 고정 계획 수량이다.
- 실제 Product exact1~67 pass, 68번째 exit101/1failed/0passed. 이후 Product77개와 Meari45개는 미실행이다. 전체190 통과·정상bins gate 성공으로 합산하지 않는다.
- 과거 r4의30번째 actual_catalog_root_factory_and_owned_callbacks_accept_nonempty_rows_and_captured_values는 이번 원 source375에서 통과했다. 원 r4 실패를 삭제하거나 소급 성공으로 바꾸지 않는다.

## 첫 실패와 보존

시험: assembly::meari::viewer_host::jobs::route::product_route_tests::jobs_caption_factories_reject_missing_types_and_original_exports_reject_foreign_stale_retired_and_closing.

원 stderr는 product_route_fixture.rs:481에서 `Original route did not settle`, elapsed3232ms를 기록했다. fixture closeout은 complete=true/primary_panic=true/setup_panic=false/cleanup_panic=false. 이는 시험 실패이며 정리 완료와 구분한다. group-return.txt는 producer_integrity=Ok(())를 기록했다.

원 근거: runtime/verification/source375-datalab-b0-r1-logs/11-product-exact-r1의 producers-before.tsv·등록/list/ignored·exact001~068 stdout/stderr/receipt/passed·group-return. outer resource receipt는 launcher32388/child25640/CPU2/priority16384/terminaltrue/exit1/12860ms. 초기 helper 내부 per-case receipt는 명령/exit/elapsed를 기록하지만 실제 case별 PID와 metadata 원값을 별도 직렬화하지 않는 한계가 있다. 코드의 매실행 metadata 동일성 검사·그룹 전후 hash를 그 누락과 구분하며, 후계 gate 도구에서 보완해야 한다. 시험을 통과하지 않고 exit0만 있는 경우는 거부한다.

실패 libtest EXE SHA24C25DBD61E64C552BD1B01FF4B4C182F11D4E337A8887906E63EDC78ED57324/185364480B, PDB SHA7EBD69DFDEB64792925928816290E1FE3FC794683C3E4981DFFBF9DA3BC2212A/1022308352B. 원본은 native/debug/deps/kinekino_player-608b31f95d06993b.*이며 같은 SHA의 독립사본을 failure-068-bin-r1에 정확2파일만 보존했다. 대형 통과 target 전체 복제나 삭제0이다.

## 한정 진단

처음 실패를 보존한 뒤 동일 원 EXE에 RUST_BACKTRACE=1을 지정한 **명시적 진단1회**를 실행했다. 자동 성공 재시도나 gate 통과 재계상은 하지 않았다. 12-failure-068-backtrace.*에 exit101/같은panic·정리완료를 보존했다. stack은 Rig::enter line626의 첫 EntryPhase::Armed 대기에서 실패했음을 보여준다. jobs.list/read 뒤 두 번째 대기나 caption assertion까지 도달한 실패가 아니다.

읽기 조사: fixture Clock.now=Duration::ZERO·tick은 NativeHostRuntime.after_dispatch(Completion)이며, product route는 Requested→Draining→Committing→Armed를 갖는다. ReadConsumerRetry는 실제 runtime에서 Invalid+typed retry로 반환하므로 단순 ‘고정clock이 원인’으로 확정하지 않는다. 현재 tick.unwrap는 이 경우 별도 panic이어야 한다. route의 first_error/recovery/phase·pending/원 callbacks 상태를 같은 deadline과 guard를 유지한 진단 후계에서 관측해야 한다. 생산코드 타이머 신설·deadline 연장·Busy/strict guard 완화·가짜 Armed 주입은 하지 않는다.

## 다음 작업

### 복구 export 후계 조사/수리

후속 실제 결과: r3 build19 exit0/54230ms, exact20은 원068명칭으로 running1/1passed/0failed/0ignored 및 closeout complete=true였다(319ms). 후계의 영향시험1개이며 원375/전체190 성공이 아니다. r3 strict26은 새 nested-if lint로 exit1, r4가 해당if만 접은 뒤 strict27 exit0(17.83s)였다. 기존 dependency fetch_update deprecated 경고는 별도 유지한다. r4 입력 SHA7BD06EA4C87A727F416898BD96A57A02A10096E94D7A36E3BDFCBA73958B00A6. r4 full root/lib+jobs no-run28을 시작했다.

gate helper 보완은 fmt25/test21(7pass)/strict22/build23 모두 exit0으로 자체검증했다. SOURCE_RECEIPT 필수인 새 CLI이며 source receipt 원문·SHA와 case별 PID/spawn/terminal·EXE/PDB metadata before/after를 기록한다. 이 receipt는 호출자의 closure 진실성을 자동 보장하지 않음을 README에 명시했고, PM의 실제 입력 고정과 구분한다. 기존 gate 증거를 덮어쓰지 않는다.

직전 goal 턴은 진단14의 실제 오류와 상태를 확보한 progress였다. 이번 턴은 소스 계약 대조로 read_props의 확대된 transport/catalog/window callback refs와 RecoveryExport의 오래된 부분 export가 불일치함을 확인했다. private stage에만 RecoveryExport::viewing을 추가해 host-generated top-level typed callback refs를 local export하되 admission keys에는 추가하지 않는다. 따라서 rendering 가능성과 recovery 입력 허용을 분리하며 SDK capability 등록/owner·lease·stale guard 완화0이다. 원 canonical 및 독립 Meari source 수정0.

후계 r1의 build15 exit0/60295ms, exact16은 Requested timeout을 지나 실제 caption 검증에 도달했지만 helper의 폐기 export resolve에서 exit101/325ms 실패했다. 후계 r2는 유효할 때 원/복구 핸들을 보관하는 test delta를 추가했다. build17 exit0/64207ms이나 exact18은 새로 추가한 `paint_recovery 뒤 독립 export도 resolve 불가` assertion에서 실패했다. 그 가정은 잘못됐다: 별도 export lease는 독립이며 paint는 모든 export를 무효화하지 않는다. 이 추가 assertion만 제거한 r3를 고정했다. 기존 foreign/stale/duplicate/retired/closed admission 검사는 전부 유지하며 explicit recovery owner invalidate 뒤 resolve 거부 검사도 유지한다. r1/r2 실패를 보존하고 통과로 계상하지 않는다.

후계 입력/각 delta SHA와 before/after는 runtime/development/source375-jobs-recovery-fix-r1/INPUT.md, INPUT-R2.md, INPUT-R3.md에 있다. r3 receipt SHA1F857E2F7800BC24DE77A646FCA618FD70FA6E5D79D2A4A20605681CBDCB8D91. 진단 fixture는 원 bytes로 복원했다. 원 A09 67pass/68fail과 후계 결과를 혼합하지 않는다. gate helper는 명시 source receipt SHA·사본과 case PID·실제 metadata before/after를 기록하도록 수정 중이며 도구 자체 검증 전 새 전체190에 쓰지 않는다.

### 진단 후계 r1 관측

필수 의존성 최소 설치에 대한 사용자 승인을 유지한다. 13-jobs-diagnostic-build는 locked/offline/jobs1로 exit0/53838ms, 14-jobs-diagnostic은 CPU2/BelowNormal에서 exit101/3211ms였다. 진단은 private stage의 product_route_fixture.rs에 timeout 상태 출력만 추가했으며 deadline/guard/판정은 그대로다. before SHA195817C4A88E8B1BE9303540EDE6DEB013B8FEDD721A04171CE244835965B6EE, diagnostic SHA175B50D622D9AB144DA733F85C38292FF2AB8DBFDC5CB9244AE4ED9108518398. 두 파일은 runtime/development/source375-jobs-route-diagnostic-r1에 보존한다. 이 stage는 이제 A09 원pin과 구분하는 진단 후계이며, 원375 성공으로 계상하지 않는다.

실제 timeout에서 entry=Requested/recovery_live=false/document=false/completed=false/returning=false, first_error와 host_error 모두 Callback Adapter `Apply("callback reference is not exported")`였다. 정리는 complete=true였다. 따라서 단순 대기시간 부족으로 확정하거나 timeout을 늘리지 않는다. 다음 조사는 Requested의 recovery view callback export/binding 계약이다. canonical source, 독립 Meari WIP, 원 source375와 최초 실패 EXE/PDB는 그대로 보존한다. Core18/Native26/OS1·정상bins·AI 결과 검토·UI 시각 비교는 여전히 미완료다.

원 source375/실패산출물·root/Meari WIP를 그대로 보존하고, 읽기 원인분석→필요한 진단/최소수리 delta와 새 입력 pin→영향시험→전체190 재검증 순서다. 원pin의67pass/68fail과 후계 결과를 별도로 표시한다. 첫 실패를 고치기 전에 나머지를 합성 성공으로 돌리지 않는다. 이후 정상 bins·실제 AI 결과 검토·수정/판정/이력 및 UI 시각 인수를 전체 목표에 유지한다. 사용자 품질 승인 대기를 개발 미완료의 차단 사유로 쓰지 않는다.

스킬: build-cache-correctness-audit의 source/producer별 실패·부분결과 보존과 shared-checkout의 타 WIP/index 보존을 적용했다. 새 gate 도구는 private Rust이며 원 historical PS recipe는 읽기 참조만 하고 실행하지 않았다.
