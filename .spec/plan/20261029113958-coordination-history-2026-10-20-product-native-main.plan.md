---
id: plan:kinekino/20261029113958
title: 현재 세션 조율 상태
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 현재 세션 조율 상태

2026-10-20 KST. 현재 배정·검증 입력·잔여 순서만 기록한다. 이전 원문은 [typed close까지의 이력](coordination-history-2026-10-20-typed-close.md)에 보존했고 원문/사본 SHA E6433C2536B167B934C40A9ACEE41F1DB5EC653B9E16050D73DDF0C661BD0EF0 일치를 확인했다. checkpoint 성공은 제품 전체 완료나 사람 품질 승인이 아니다.

## 담당과 실행 소유

### 현재 배정: SDK 제품 실행 checkpoint 인수 → Meari P1a 구현

SDK 정본 마감도 인수했다. [제품 Native 실행 정본](../../sots/player/application/native-proposal-execution.md)과 `runtime/quality-review/product-native-main-r1/handoff.md`를 PM이 실제 읽고 원 receipt/최종 source·binary와 대조했다. 최초 실패·수리·예비/최종 binary·실제 exe와 trusted Rust 포트 시험·미검증 시각 범위가 구분돼 있다. SDK의 다음 배정은 기존 `native-resolution-handoff.md` 계획에 mpv owned session/NativeChild의 실제 공개 포트와 parent/resize/focus/close 수명 계약을 읽기 전용으로 준비하는 것이다. 빌드·재생·소스 변경은 하지 않는다.

SDK가 heavy 슬롯을 명시 반환했고 `owned-processes-observed.json=[]`를 인계했다. PM은 기록된 receipt 34개의 launcher/child PID가 생존하지 않음을 독립 관측했고, 최종 실제 6실행의 개별 exit0·forced=false·closed_windows=1·child_waited=true를 확인했다. 최종 source13 inventory는 `4F4AE9DCEB4A222F89B5730E3990C08CFE2DED6690F2A9144DF52B25376B0F0E`이며 독립 재해시 차이0이다. 최종 실행 파일 SHA256은 `6610D2FFD801020882E62B46F8BE26244CB72202F1A91E972E3B8C34146FDC6F`로 직접 대조했다. SDK는 남은 정본·handoff 문서만 마감한다.

| 담당 | 현재 배정 | 실행·변경 경계 |
|---|---|---|
| Meari | P1a typed node slots/children 실제 구현·검증 | 단일 heavy 소유. 내부 Core/Native/Web의 필요한 범위만 before/after 보존 후 변경. root Cargo/SDK/UI 고정 입력 변경 금지 |
| UI | 원 StudioFrame의 실제 caller/source graph와 presentation 상태 조립 작성 | 원 source와 신규 파일 책임을 구분. 새 빌드·GUI는 Meari 반환 뒤. 공개 slot ABI를 임의 복제하지 않음 |
| SDK | 최종 실행 checkpoint 정본·handoff 마감 | 추가 실행·제품 소스 변경0. 다음 호출은 중앙 배정 뒤 |
| AI | 기존 조사 근거 보존 | 새 다운로드·추론·사용자 품질 검토는 제품 연결 뒤 |

최종 실제 gate는 `final-actual-six`27042ms/exit0(6개 실제 실행), `final-actual-invalid`12581ms/exit0(5개 오류 실행), trusted mount2524ms·attach2484ms·SDK start48ms 각 exit0이다. 각 receipt의 exact test 하나를 내부 실행 수와 혼동하지 않는다. attach의 Native Unconfirmed와 원 runtime/recovery 보존은 정상 OS 종료가 아니다. 최종 on Clippy18717ms/exit0·fmt1534ms/exit0, Native316 감사 unchanged 및 링크924/정본 broken0/upstream 보존4도 확인했다. 전체 UI 시각13·새 exe 편집/채택 상호작용·미디어 child 통합은 여전히 미완료다. 아래 중간 배정·예비 binary·진행 중 표현은 이 checkpoint 이전 이력이다.

최종 소스 재검증 관측: `final-on-lib`114634ms/exit0, 371pass·6ignore; `final-on-test-build`64169ms/exit0이다. PM이 `final-input-source-inventory.json`의 13파일을 실제 재해시해 차이0을 확인했다. 진단 notice는 유한 종류별 최초1회만 보관하도록 바뀌었으며 원 validation 이벤트 스트림과 FrameSubmitted 성공 조건을 유지한다. 이는 이벤트마다 Vec가 커지는 새 진단 코드의 메모리 경계 수정이고, 이전 실제 binary의 검증을 새 binary에 상속하는 근거는 아니다. 새 binary 실제 gate·최종 인계는 진행 중이다.

최신 종료 관측: SDK `test-off`는 동일 원 실행이 끝까지 진행해 350953ms/terminal=true/exit0으로 종료했다. `strict-off`18743ms/exit0, `fmt-final`1544ms/exit0도 원 receipt로 확인했다. 중간 stdout 한 줄의 반복 관측을 정지로 단정하지 않았으며 재시작하지 않았다. 최종 Box 변경이 반영된 새 binary 실제 gate·최종 source 핀과 슬롯 반환은 여전히 SDK 소유다.

후속 라벨은 P1a=node slots/children 기반, P1b=InspectorDock 및 필요한 Boolean controlled/Dialog를 포함한 공통 프레임, P2=navigation capture/router·탭이다. 이전 P1/P2 분류 중 Boolean을 P2로 둔 표현은 이 구분으로 정정한다. UI 공급 요구는 required named4각1+children1, 누락과 명시 empty 구별, caller lexical owner 및 같은 identity의 편집/state/useId/callback 보존이다. 명시 화면 교체에서는 이전 export 폐기·stale 거부와 무관 shell 보존을 검증한다. 동일 owned subtree 중복 삽입과 일반 null/iterable 확대는 현재 제품 요구가 아니며 명시 진단을 허용한다. 공개 타입·한도·port는 Meari가 소유한다.

P1 선행 의존성 읽기 검토: Meari의 `studio-frame-slots-callbacks-2026-10-20.md`는 현재 ReactNode 타입/값 부재, JSX scalar Text lowering, local component children의 명시 거부를 원 소스에서 확인했다. Action/Text forwarding 및 일부 bool predicate/Dialog 동작은 이미 존재한다. 따라서 SDK 반환 후 최소 typed node slot/children 보완을 Meari에 먼저 배정하고 UI가 같은 원 StudioFrame을 소비한다. Boolean host callback·navigation scalar capture·controlled Dialog는 P2와 분리한다. 이번 관측은 새 parser/런타임 시험 통과가 아니며, 정확 caller graph·상태/키/콜백 소유권과 예산·stale 거부 계약을 먼저 고정한다. 직접 세션 교환 권한 해석 때문에 검토 자료는 PM이 UI에 중계했다. 지금은 SDK의 live test-off family만 실행하며 UI/Meari source 변경·빌드는 없다.

최신 후속 배정: SDK의 `test-on-lib` 원 receipt는 terminal=true/exit0/52739ms이며 371pass·6ignore다. `strict-on-r2`도 terminal=true/exit0/20210ms다. 첫 Clippy 지적 후 unused Native 권한 묶음을 Box로 옮긴 소스 변경이 있어, 이전 binary의 실제 6입력 성공을 최종 소스에 자동 상속하지 않는다. SDK가 최종 binary·필수 검사·보호 핀·owned 프로세스 및 슬롯 반환을 마감한다.

다음 구현은 UI 담당의 P1 단일 PotPlayer 공통 프레임이다. [후속 UI 계획](presentation/meari-consumer-gaps.md)의 P0~P4와 전체 13화면군을 유지한다. 현재 연결은 개별 proposal 패널이며 전체 shell/router/video child가 아니다. UI와 Meari 담당에는 실제 StudioFrame/inspector의 ReactNode slots·children·typed callback 공개 API를 읽기 전용으로 함께 대조하도록 배정했다. 오래된 unfinished 문구만으로 현재 미지원이라 단정하거나 frame 복제·flatten으로 우회하지 않는다. SDK 슬롯 반환 전 새 빌드·GUI·동결 코드 변경은 하지 않는다. 흰 기능 fixture를 최종 UI 디자인이나 시각 인수로 표시하지 않는다.

현재 배정은 SDK 제품 연결5단계다. UI4 최종 인계의 변경21 inventory535EC6B64BF0737C3C19384620E4EBD856FE982DCD963A77610330FEFBC855A9와 고정10 inventory58E4C98A8ED0A2FD62EA72C1CFAB57ACBF89A8CAB485689C793A64338CCA818B를 PM이 독립 재해시해 차이0으로 확인했다. canonical-final143376ms/0·30pass, 최종 root/consumer fmt2797/3182ms0, owned-processes-observed=[] 및 명시 슬롯 반환을 인수했다. UI 최종 정본과 runtime/quality-review/product-native-window-helper-r1/handoff.md를 따른다. 아래 UI4 진행 중 표현은 이전 회차 이력이다.

SDK는 기존 configured client/store·단일 execution/Native loop로 main Native 분기와 staged guard 해제를 함께 구현하고 실제 새 player.exe로 검증한다. UI21·Native316/Core는 동결한다. 제품 visuals는 빈 기능 fixture가 아니라 고정 CSS와 검증된 폰트/아이콘 자산을 연결한다. Meari 읽기 조사 packages/meari/.spec/evidence/reviews/native-product-css-font-boundary-2026-10-20.md에 따라 원 stylesheet 경로를 보존해 URL rewrite 후 CSS를 전달해야 하며 Some(VerifiedAssets)만으로 자동 rewrite된다고 가정하지 않는다. 실제 렌더러 지원·시각13·물리 입력·AI 품질은 여전히 별도 미완료다.

SDK5는 runtime/quality-review/product-native-main-r1/before에 main/launch/assembly/infra 부모를 보존하고 실제 UI 인계를 읽어 구현을 시작했다. PM은 product_styles.rs 후보의 원4 CSS StylesheetProject→rewrite, 폰트3·아이콘30의 manifest/blob·SHA/길이/MIME 및 폰트 라이선스 구성을 읽었다. 이는 소스 검토이며 자산 전달·글리프/weight·레이아웃·실제 제품 실행 통과 근거가 아니다. StartFailure 원 owner와 UI ProductWindowOwners/CleanupEvidence를 유지하는 조립 outcome은 SDK가 구현 중이다. process 종료 이후 Rust owner 복원이 가능하다고 주장하지 않고 비정상 종료 진단과 메모리 소유 한계를 구분한다.

SDK5 최초 feature-on lib/player check는45342ms/terminal0이었다. 이후 main의 입력 준비를 assembly 호출로 정리했다. PM 검토에서 pre-run 실패의 unused Native loop/startup 보존을 확인했고, 현재 Finished.unrun_native에 같은 권한을 유지하며 원 오류/cleanup 미완료와 함께 반환한다. focused 첫100907ms/101·3pass1fail은 running 미ACK가 TerminalAckPending를 먼저 반환하는데 assembly의 명시 수락 오류를 BudgetExhausted로 예상한 시험 오류였다. SDK 정책을 바꾸지 않고 UI bounded poll과 assembly 수락 거부/원 owner·raw Arc 보존을 분리한 focused-r2는72497ms/0·4pass다. 기존manifest/mpv 회귀2개를 포함하므로 새 기능4개로 세지 않는다. 직접 UI 예산 검증을 production assembly 반복 예산 소진 증거로 확대하지 않는다. 실제 제품 CSS 전달·새 exe 실행은 아직 미검증이다.

styles-native-r2는69421ms/terminal0·2pass였다. PM은 원 로그와 단언에서 전체 제품 CSS를 실제 Native renderer의 작은 probe 문서에 전달하고 TTF3개의 콘텐츠 경로 delivered>0/rejected0/failures[] 및 chrome 토큰 pixel[25,27,31,255]를 확인했다. asset33/reference34 rewrite 시험도 포함한다. 첫 styles-native5558ms101은 시험의 Viewport 이동후사용 오류로 clone 수리했으며 원 로그를 보존했다. 이는 실제 renderer의 자산·대표색 증거이고 전체 product 창·글리프 shaping/weight·아이콘 pixel·시각13 성공은 아니다. SDK는 새 player.exe의6case/입력오류/off 및 정확 child PID 소유 창 정상종료를 Rust fixture로 준비 중이다. 사용자/다른세션 창을 조작하지 않는다.

실제 exe gate: build-on-test134247ms/0, actual-invalid15024ms/0(원 입력 오류/off 및 초기 오류·stale 경로), actual-six 최초11351ms/101은 visible창2개 단언 실패였다. 제품 exit0/SDK join과 단일창 실패를 분리 보존했다. 새 fixture 메타데이터에서 제품 Window Class/title와 Winit Thread Event Target/빈 title가 구분되며 hidden IME owner/parent도 기록됐다. 모든창을 기록하고 정확PID의 제품root만 닫도록 시험을 정리한 actual-six-r2는28499ms/terminal0·exact1pass다. PM은6개(cards/highlights/cue/document correction/repair selected/unselected)의 각 exit0·forcedfalse·closed_windows1·child_waitedtrue를 직접 확인했다. binary SHA256은2fa356ed0e67eae38f6a7c6d5a8444144312bf9a968c8e53d54be0288a5fe3d6다. 이는 실제 제품 startup/frame/정상종료/store불변의6입력 검증이며 새exe 편집·채택 상호작용·시각13 완료는 아니다. 최종fmt/test/clippy/보호핀/owned·인계는 아직 남아 있다.

trusted-mount2680ms/terminal0·exact1pass와 trusted-start66593ms/terminal0·exact1pass를 PM이 원 결과로 확인했다. 전자는 trusted CSS 실패 유발로 원 factory cause/ConsumerMount/SDK join·Native NoDocument를 보존하며 일반 exe 입력의 임의failure injection 기능이 아니다. 후자는 SDK 시작 거부 뒤 같은 client/Native loop/startup 재사용과 pre-run UI 입력 실패 시 unused Native 권한 보존을 검사한 Rust unit이다. 실제exe6입력 성공과 같은 범위로 합산하지 않는다. 현재SDK가최종회귀·정적검사를마감하며UI는소스/빌드/GUI변경없이현재패널과승인단일창13화면의미연결·후속파일소유계획을정리한다. AI/VM/DPI/Pages는보류유지다.

등록3단계는 최종 인수했다. 8파일 inventory B2A69A908163CC3282E36848A67FA25716FDD3221A2C1EFD14B68F64BFB11CAE를 PM이 현재 파일과 독립 대조해 차이0을 확인했다. focused off/on 각15pass·기존 launch14pass·fmt1712ms/0·strict26420ms/0, 담당 PID 재관측 owned[]와 명시 슬롯 반환을 확인했다. 근거는 runtime/quality-review/product-native-launch-input-r1/handoff.md와 [입력 정본](../../sots/player/application/native-proposal-launch.md)이다. 아래 등록3단계의 인수 전 표현은 중간 이력이다.

현재는 UI 제품 연결4단계다. UI 세션의 active turn과 실제 소스·계약 읽기를 확인했다. checked 선택 변환, 명시 DocumentRepair Selected/Unselected, production Correction entry, 고정 startup helper, 원 오류·owner·명시 terminal 수락·bounded cleanup 결과 보존을 담당한다. SDK8와 Meari316은 동결한다. main 실제 Native 분기 및 staged guard 해제는 이 인계 뒤 SDK가 같은 checkpoint에서 수행한다. 제품 실행·물리 입력·전체 화면 설계 인수는 여전히 미완료다.

정확한 변경 경계는 [UI4 계획](presentation/product-native-window-helper.md)을 따른다. PM은 초기 props와 실제 mounted ReviewSession 양쪽에 같은 full reference를 설치해야 한다는 공백과 production correction의 선언만 있던 원문 선택·수동 시작 controls를 직접 소스에서 확인했다. 현재 선택 변환·초기/live 선택 전달·최소 controls 소스는 작성 중이며 compile/test/실제 실행 성공 근거가 아니다. 실패 정리는 SDK quiescence/join과 Native Closed/ForcedIncomplete/미확인을 분리한다. 원 owner의 중복 소유·중복 ACK 또는 새 loop fallback을 허용하지 않는다.

UI4의 추가 계약 변경: 기존 ExplicitPanelSelection.item_index:usize는 repair의 명시 no-index를 표현할 수 없었다. UI가 Option<usize> 정규화를 소유하며 일반 item은 Some(checked), repair는 None만 허용하고 target/실제 task와 초기 handoff·factory 양쪽에서 교차검사한다. dummy0·첫 항목 fallback·None만 보고 repair 추측은 금지한다. 기존 caller의 index 의미는 Some(원값)으로 보존한다. PM은 SDK 담당에 변경 중인 계약을 미리 인계했고 SDK는 최종 UI 타입·시험 핀을 받은 뒤 조립한다. 이는 현재 구현 배정이며 최종 API/회귀 통과 인수가 아니다.

UI4 첫 root feature-on lib check는16628ms/exit101이었다. PM은 원 stderr에서 PluginScope를 ReviewScope 자리에 전달한 타입 오류2개와 deprecated atomic 메서드 경고1개를 확인했다. 담당이 원 소스/로그를 보존하고 UI 범위만 수리한 check-r2는9687ms/terminal0이며 원 stderr의 Finished를 대조했다. runner는 affinity3/2CPU·BelowNormal·descendant Job이었다. 이는 타입 검사 성공이며 실제 시험 본문·strict·OS·새 player.exe 실행 성공이 아니다. 후속 회귀와 최종 source pin 인수 전까지 UI4는 진행 중이다.

UI4 product-first는137485ms/terminal0으로 새 actual SDK headless8개가 통과했다. PM은 원 stdout의8pass/0fail/0ignored와 stderr의 실제 test 실행을 확인했다. 카드·하이라이트·cue 교정의 index1 수정/저장/채택과 명령5개 순서, repair Selected fullref/Unselected 및 원 source/generated 보존, invalid selection의 원 terminal 보존·명시 종료 수락·예산 초과 후 같은 owner 회수, 설정 실패의 SDK 호출0, 반환된 Unconfirmed/ForcedIncomplete를 OS Closed로 승격하지 않는 검사를 포함한다. 마지막 항목은 구성된 반환 outcome의 headless 의미 시험이며 실제 OS 강제 종료 증거가 아니다. 이후 추가 수정/회귀가 진행 중이므로 이 실행을 최종 소스 인수·strict·실제 제품 실행 성공으로 상속하지 않는다.

UI4 consumer-r2는462587ms/terminal0·63pass/0fail/9ignored로 PM이 각 test-result와 원 receipt를 확인했다. 기존 canonical55에 새 product helper8이 포함된 합계이며 product-first8을 다시 합산하지 않는다. factory의20개에는 모순 선택 거부가 포함되고 helper에는 headless owner 재인계 시 HeadlessClosed 유지 단언이 추가됐다. OS9는 명시 제외되어 이번 실행의 OS 성공이 아니다. consumer-r2-owned.json=[] 뒤 root-on 단일 실행(launcher49332/child88268)이 시작됐고 현재 terminal=false다. 원 실행의 관측 timeout은 실패나 재실행 근거가 아니다. root·strict·fmt·OS 영향·최종 핀과 슬롯 반환은 미완료다.

후속 root-on도102587ms/terminal0으로 종료했다. PM은 제품 lib366pass/5ignored와 나머지 workspace lib16+1+0+16+5+10을 원 로그에서 확인했다(합계414pass/0fail). 이는 --lib feature-on 범위이며 root 전체 binary/integration/실제 제품 실행 성공이 아니다. 이후 product-r3 단일 실행이 시작됐고 새 OS fixture2는 작성 중이다. strict/fmt·OS 실제 실행·최종 핀/인계는 여전히 미완료다.

후속 product-r3는3204ms/101의 OS fixture import 누락이었다. product-r4는54341ms/101·9pass1fail로 context 오류 기대 Unsupported가 실제 canonical InvalidArgument와 달랐다. PM은 proposals execute의 idempotency/timeout 거부 정책과 direct SDK code/details 대조로 수정된 시험을 확인했다. SDK 정책/요청 context 변경은 없다. product-r5는53405ms/0·10pass이며 OS2는 이 실행에서 ignored다. 실제 개별 OS 정상 frame/close/join은2039ms/0·exact1pass, attach 원 실패1534ms/101 뒤 deferred Startup::Attach의 정확 오류·recovery 소유자 및 SDK join/Native Unconfirmed 보존을 검사한 os-attach-r2는18806ms/0·exact1pass였다. PM은 원 로그와 실제 단언을 읽었다. 두 OS 시험도 empty CSS/assets 기능 경계이며 실제 player.exe·시각 인수가 아니다. 현재 consumer strict 단일 실행 중이며 최종 핀·전체 인수는 미완료다.

consumer strict50978ms/0과 root strict의 원 실패10801ms/101(collapsible_if1건)·같은 조건 let-chain 수리 뒤 root-on-strict-r2 32040ms/0의 원 결과를 PM이 확인했다. validation-source-inventory.json의21파일은 PM 독립 재해시 차이0이며 inventory SHA256 C37E78C86709961B7FA01BD81E9F4329813B7ABD4DB332FF6F6A6C67F20316C5다. 이는 현재 검증 입력 핀이고 최종 인계 확정과 같지 않다. root-off 원 실행은 아직 terminal=false이며 추가 context 관측·off 영향·fmt 및 최종 반환은 별도 인수한다.

root-off는326273ms/101로 기존 jobs::actual_cli_explicit_host_and_sdk_share_durable_jobs_without_gui의 forced-submit read_response/client_deadline Timeout(execution_unknown·자동재시도false·실제CLIexit10/기대0)에서 실패했다. 원 로그/receipt/종료 가계[]를 보존했다. 원 temp fixture는 기존 Drop 정책으로 자동 삭제되어 당시 publication/store 사후 근거가 없으며 성공/실패를 추정하지 않는다. 같은 uncertain 요청은 재전송하지 않았다. 별도 namespace·JOB_EVIDENCE_DIR의 jobs-r2 exact1은10127ms/terminal0으로 통과했고 PM이 원 stdout을 대조했다. 이는 최초 원인 규명이나 일시 자원 문제 확정이 아니다. UI가 새 fixture 전체 root-off-r2를 단독 실행하고 SDK는 코드/빌드 없이 transport/publication 진단에 필요한 근거를 읽기 검토한다. timeout·SDK정책 완화는 없다.

SDK 읽기 전용 진단에 따르면 read_response 진단은 frame 쓰기 완료 뒤 응답 deadline 초과까지 입증하며 서버 dispatch/접수/발행 성공은 입증하지 않는다. submit의 prepare/source blob 생성은 admit 이전이므로 blob만으로 접수 성공을 판단하지 않는다. 향후 재현에서는 독립 endpoint/fixture label, submission key·normalized request hash·job ID·journal revision/attempt·publication 및 검증 receipt와 host dispatch/write_response/read_ack trace를 연결해야 한다. 반복 request_id jobs-fixture 하나만으로 상관관계를 확정하지 않는다. timeout 확대·unknown 요청 재전송 근거는 없다. 실패 시 원 fixture 보존과 최소 Rust 관측의 보강은 별도 후속이며 이번 UI가 임의로 transport 정책을 수정하지 않는다.

새 fixture 전체 root-off-r2는151151ms/terminal0으로440pass/0fail/23ignored를 확인했다. PM이 모든 test-result와 원 receipt를 읽었으며 앞서 실패했던 jobs 항목도 포함한다. 이전426과의14차이는 launch 입력/옵션/adapter 신규 회귀이며 UI4의 새 기능440개라는 뜻이 아니다. 최초326273ms101과 원인 미확정 한계는 유지한다. 현재 consumer-off 단일 검증이 진행 중이며 최종 context/포맷·source 인계와 명시 슬롯 반환은 미완료다.

consumer-off도496570ms/terminal0·96pass/0fail/9ignored로 PM이 각 결과와 원 receipt를 대조했다. 기존 off93 대비3 증가는 선택 변환/모순 거부 등 영향 회귀이며 canonical on63과 중복 구현 시험을 신규 기능으로 합산하지 않는다. partial path 소비의 poll_shutdown 미사용 경고는 별도 handoff 종료·부분 실패 정리 회귀로 검증해 해소하는 중이다. 후속 consumer-off-ports 단일 실행이며 최종 context 전달·포맷·소스 핀/owned/슬롯 반환은 미완료다.

사용자는 일부 Native 창의 흰 배경·흩어진 배치를 관측했다. 해당 실행/창의 정체는 미확정이다. PM이 확인한 native_pending_window_sdk fixture는 css:String::new()/assets:None을 주입하므로 제품 테마·폰트 검증 근거가 아니다. 제품 tokens.css에는 dark color-scheme/Pretendard 규격이 존재한다. 기능 fixture의 무스타일 입력과 제품 CSS/asset 주입 후 렌더러 호환성은 구별하며 후자는 아직 미검증이다. UI 담당에 관측을 인계했다. 현재 창을 승인된 PotPlayer 제품 디자인 완성으로 표시하지 않는다.

SDK는 코드·빌드 없이 [후속 main 검증표](sdk-proposal-window-startup.md#다음-main-연결-checkpoint의-검증표-4단계-ui-인계-전-준비) 14행을 준비했고 PM이 원문을 검토했다. 동일 client/store와 단일 backend, worker 전 입력 실패, feature-off, 실제3task 및 repair 명시 선택/미선택, SDK 시작·mount·attach 실패, cleanup incomplete, actual join/Destroyed를 구분한다. UI cleanup 포트의 정확 타입은 아직 확정 전 후보이며 새 player.exe 검증은 미실행이다. 문서의 승인 후 표현은 기존 사용자 권한 내 담당 인계·슬롯 배정 조건이며 추가 사용자 허가 대기를 뜻하지 않는다.

등록3단계 launch-off1449ms/0의14pass와 focused-on77317ms/0의15pass도 PM이 원 결과를 확인했다. on/off 집중15는 새 입력·옵션·어댑터14개와 기존 manifest1개를 합친 실행이며 서로 독립 새 기능30개로 합산하지 않는다. 명시 DocumentRepair Selected DTO는 full ref를 그대로 보존하지만 현재 initial_panel의 Some 거부 분기는 후속 UI가 공통 correction projection과 맞춰 구현할 잔여다. 미선택/첫 항목 fallback으로 대체하지 않는다. strict/fmt와 최종핀/반환은 별도 인수한다.

등록3단계 runtime/quality-review/product-native-launch-input-r1/focused-off는58481ms/101·12pass1fail이었다. 미선택 unit variant가 추가 source_ref를 허용하는 실제 입력 결함을 발견했고, launch 전용 Unselected{}로 최소 수리하며 단언을 유지했다. focused-off-r2는37171ms/0·15pass로 PM이 원문 대조했다. tagged 선택/미선택·full ref/문맥/decimal 경계·중복/unknown JSON·64KiB 실바이트·실제 junction ancestor/final reparse 거부가 포함되며 기존 manifest 회귀1개도 포함된다. 새 기능15개로 세지 않는다. 후속 기존 launch/off-on·adapter 영향 회귀와 fmt/clippy·핀/문서/반환은 인수 전이다.

등록3단계의 staged guard를 PM이 승인했다. 현재 main은 DesignPreview 여부만 분기하므로 새 Native mode만 반환하면 기존 eframe으로 잘못 진행할 수 있다. plain 옵션/DTO parser는 구현하되 desktop_mode는 feature-off의 미지원과 feature-on의 아직 미연결 상태를 명시 Unsupported 진단으로 구분해 차단한다. 후속 main의 Native 단일 분기 구현과 이 임시 guard 해제를 같은 checkpoint에서 수행한다. 이는 중간 입력의 fallback 방지이며 제품 Native 실행 지원 완료나 최종 지원 범위 축소가 아니다.

| 담당 | 현재 상태 | 다음 책임 |
| --- | --- | --- |
| PM | 최신 SDK/UI 검증·입력 인수 | 단일 슬롯·계약·증거·후속 인계 관리 |
| SDK | 제품 연결5단계 배정·단일 heavy 소유 | main Native 단일 분기·staged guard 해제·제품 CSS/assets·새 exe 실행 검증 |
| UI | 제품 연결4단계 최종 인수·21파일 동결·슬롯 반환 | SDK 계약 질의 대응·후속 제품 스타일/전체 시안 게이트 유지 |
| Meari | Native checkpoint 최종 인수·슬롯 반환·소스 동결 | 소비 gate 동안 API 질의 대응, 추가 Core/Boolean/HMR 변경은 후속 조율 |
| AI | 결과·출처·품질 계약 유지 | 제품 연결 뒤 마지막 검토, 새 추론/다운로드 보류 |

Rust 통일·단일책임·모듈화·계층 구조를 코드와 .spec에 적용한다. GUI/CLI는 같은 SDK/application/domain을 소비한다. 기능은 CLI 우선, GUI는 실제 OS·입력·시각 증거 범위만 검증한다. 사용자 ChatGPT/Terminal은 유지한다. Pages·VM·DPI·실제 시스템 절전·새 장시간 재생 시험은 건드리지 않는다.

[자원 규칙](resource-budget.md)의 중앙 Rust runner affinity3/2논리CPU·BelowNormal·descendants Job, offline/locked/-j1·단일 test thread·전용 target을 유지한다. UI4 명시 반환 후 단일 heavy는 SDK 제품 연결5단계로 이관했다. UI21·Native/Core/generic은 동결하며 SDK는 본인 launch/main/assembly 및 필요한 trusted visuals adapter만 변경하고 공유 계약은 담당과 조율한다. consumer 역의존이나 Native lock 복사는 금지한다. timeout은 종료가 아니며 원 handle·실제 프로세스·terminal을 대조한다. runner 종료만으로 자식 회수가 보장되지 않으므로 정확 가계를 확인한다. 빌드를 겹치지 않는다.

## UI 최종 소비 인수

등록2단계 최종11 inventory215B94167313293829674568B504154EC4349A27A67DE9E92F446D9DF4D73332를 PM이 독립 재해시해 차이0을 확인했다. canonical OS9 각각 exact1pass/terminal0·회차별 가계[]와 최종owned[]·정적3종(root41336/on40074/off33864ms 각0)·fmt2종(7304/3297ms 각0) 및 명시 슬롯 반환을 인수했다. 원 실패·수리·중단/자원 위반은 별도 보존했다. [등록2단계 정본](../../sots/verification/presentation/product-native-registration-step2-2026-10-20.md)과 runtime/quality-review/product-native-registration-step2-r1/handoff.md를 따른다. 다음은 SDK plain launch DTO/adapter이며 제품 main/launch 실행·물리 입력·13화면 인수는 미완료다. 아래 등록2단계 중간 기록은 이력이다.

등록2단계의 기존 독립 소비 경로 off-consumer-r2도403442ms/0·93pass/0fail/9ignored를 PM이 원문 대조했다. 기존91에 공개 용량 경계2가 추가된 수치다. canonical on55와 root 고유UI31 의미 이동 및 off93의 원 기록을 분리해 보존하며 중복 시험 수를 합산하지 않는다. root-on-strict-first가 현재 단일 실행이고, 후속 consumer on/off strict·fmt·canonical OS9·최종핀/owned[]·명시 반환은 아직 미완료다.

등록2단계 root-off-test-first는331397ms/0·426pass/23ignored/0fail을 PM이 원문 확인했다. 이 원 실행이 끝나기 전 UI가 별도 off consumer 작업을 시작해 단일 family 규칙을 위반했다. 소스 변경은 없었고 새 runner 종료 후에도 자식이 남아 정확 consumer 명령/가계를 확인해 추가 회수했다. overlap-final-owned.json=[]와 원 root terminal0 및 전체 cargo/rustc/link/runner 조회의 잔류 없음을 PM이 확인한 뒤에만 후속 고유 consumer 회차를 허용했다. 원 기능 결과와 자원 위반은 분리하며 겹친 경과시간을 성능 근거로 쓰지 않는다. runner 종료만으로 descendants 회수가 보장된다고 가정하지 않는다. 현재 후속 off 소비/strict/fmt/OS9 및 최종 인수는 미완료다.

등록2단계 canonical on-consumer-r2는46280ms/0·55pass/0fail/9ignored를 PM이 원문 대조했다. off 예상93과 차이38은 consumer 단위24 및 중복 initial14의 root 이동이며 기능 삭제가 아니다. root-on-lib-first277786ms/0은 제품lib347pass/5ignored와 workspace lib합계395pass를 확인했다. 고유 initial7은 presentation::web_ui_adapter::proposals::initial_panel::tests 이름으로 각각1회 통과했다. 제품 루트의 신규 UI31 의미를 consumer55와 숫자만 중복 합산하지 않는다. 현재 root-off-test-first가 실행 중이며 off 소비 호환·strict/fmt·canonical OS9·최종핀은 인수 전이다.

등록2단계 on-compile-r2는433825ms/0으로 실제 root façade를 사용하는 전체 시험 실행 파일 생성을 통과했다. 후속 on-consumer-first12534ms/101은 신규8entry 시험이 동일 field 입력의 의도된 coalescing을 무시해(9,0)을 관측한 실패다. UI는 원 소스/로그를 보존하고 실제 두 field 교대 입력으로 시험만 수리했다. on-capacity-r2 23391ms/0의2pass는8수용/1보류·commit없음→재개9/0·commit과 registry8슬롯·중복없는 처리를 검사한다. PM이 원문과 실제 단언을 확인했으며 canonical 전체 on-consumer-r2는 아직 실행 중이다. 기존 제품 정책이나 동결 UI19본문을 시험 통과용으로 변경한 근거가 아니다.

후속 등록2단계는 runtime/quality-review/product-native-registration-step2-r1에서 별도 추적한다. UI는 root presentation façade와 consumer native-proposal-window 전달·canonical re-export를 등록 중이다. 첫 locked compile657ms/101은 consumer lock의 root→meari-native 간선 누락이었다. before/원 실패를 보존한 offline resolver2733ms/0의 diff는 간선1줄뿐이며 새 package0·기존 identity 변경0을 PM이 원문 대조했다. 고유 on-compile-r2는 아직 실행 중이다. private 조립 필드는 공개 getter로 승격하지 않고 feature-off 기존 단언을 유지하며 feature-on의8 pending/8entry와9번째 suffix 재개·8192 aggregate bytes 공개 행동 회귀로 매핑한다. 시험 수 변동은 root와 consumer의 중복 모듈 재배치와 구분하고, 실제 완료는 별도 결과로 인수한다.

19파일 inventory26335921B555B8732B3609F9D22A7AD7DDA93B9F5E3D4998111381818774F35C를 PM이 독립 재해시해 차이0을 확인했다. full-final82040ms/0은91pass/0fail/9ignored이며 중복 initial7을 새 기능으로 세지 않는다. 같은 소스 strict3507ms/0·fmt5438ms/0, 실제 OS9각 exact1pass(최종6과 guard/partial/commit3), final-owned[]·명시 슬롯 반환 및 최종 정본을 인수했다. 근거는 runtime/quality-review/ui-native-pending-r1/final-summary.json과 [소비 검증 정본](../../sots/verification/presentation/native-pending-window-sdk-2026-10-20.md)이다. 제품 player.exe 등록·물리 입력·13화면 시각 인수는 여전히 미완료다. 아래 소비 중간 기록은 회차별 이력이며 현재 배정은 위 표가 기준이다.

## SDK 최종 인수

제품 등록1단계의 별도 근거는 runtime/quality-review/product-native-registration-step1-r1이다. optional Native feature·Windows 의존성·전역 paint patch·self alias 및 infra/assembly 부모 등록을 SDK가 완료했다. 원 resolver8300ms/0, 부모 mod 정렬 수리 뒤 fmt-r2 4065ms/0, feature-off root test348318ms/0(426pass/0fail/23ignored), strict38836ms/0, feature-on lib check59225ms/0을 PM이 확인했다. lock 기존555 identity 보존·추가136·제거0과 off/on normal tree, UI19 불변·owned[]·명시 슬롯 반환을 인수했다. 5파일 inventory36914414B13F3C05D0B1EE32F3983203DEE533BB85071CF9D7DB3201B7545A8B는 PM 독립 재해시 차이0이다. [1단계 정본](../../sots/verification/application/product-native-registration-step1-2026-10-20.md)을 따른다. 기존 완료 입력 E195와 제품 등록 후 새 입력을 같은 핀으로 취급하지 않는다. 다음 heavy는 UI의 presentation 부모 등록·canonical consumer 연결이며 main/launch는 후속이다.

[공유 계약](../../sots/player/application/api/sdk-execution.md), [typed close 검증](../../sots/verification/application/sdk-close-progress-2026-10-20.md)을 따른다. 증거: runtime/quality-review/sdk-close-progress-r1.

- 최종 SDK/parent/assembly/Cargo inventory: E1959DC775A0EBA92619F83AB26EB48672367298CD5441D80D66B265DA298BF5.
- SDK 세부 소스 전후: 6F8EA80821549619B4219E465E1FFC2FAF67EA0E1440E600B30215FA86CE7044 동일.
- PM이 기존 경쟁 barrier42345ms/0·1pass, 집중50392ms/0·17pass, strict60656ms/0, root401449ms/0·426pass/23ignored, 고정UI consumer409776ms/0·83pass/0ignored, fmt1451ms/0·final-owned[]·명시 반환을 원문 대조했다.

poll_close는 TerminalAckPending/WorkerExitPending/Faulted(Terminal/WorkerJoin)/Closed를 구분한다. loop_finished·마지막 notify는 join ACK가 아니다. 실제 join 뒤에만 Joined/Closed이며 원 오류 Arc와 request/response를 보존한다. legacy close 성공·NeedsInput details·join 오류, 권한·동시 정확1join·test thread panic 반복 오류를 검증했다. CLI wire/명령·proposal 정책·dependency/manifest 변경은 없다.

기존 actual CLI73은 [S1](../../sots/verification/application/sdk-execution-s1-2026-10-20.md)의 원 evidence/binary 이력이다. 이번에는 root의 실제 CLI/typed·capability parity를 영향 근거로 사용했다.73을 새 바이너리 실행으로 승격하지 않는다. 부분 provider identity와 전체 inventory도 구분한다.

## UI 소비 인수

[Pending 검증](../../sots/verification/presentation/pending-document-sdk-2026-10-20.md), [연결 계획](presentation/typed-async-sdk-s1.md)을 따른다. UI16 inventory5b6515510b154805e5d8d199fdf778f3d439486a0fd6f7a15113def151d17a2f, generic5 f469a872ece03c8c41ab4f2a534658a97cba0e94de2a7aa2227c7f05cdea8d2b다. 최신 SDK에서도 실제 해시와 고정83 회귀를 재검증했다.

실제3task는 같은 PlayerSdk/ProposalFiles/ExecutionOwner·Native5와 원 TSX ModuleProgram/InteractiveDocument를 소비한다. callback queue→guard 밖 edit→inspect→명시 accept→inspect→명시 close/join, UI thread 밖 SDK 실행을 확인했다. 두 번째 항목의 실제 수정 저장, 미선택/다른 필드·source refs·원문 ID/text/PTS·pins·generated raw 보존, acceptance output 일치·자동 적용false를 검증했다. 초기 준비/ACK·canonical 오류·queued 취소·명시 선택·부분 mount 복구와 Immediate/bootstrap도 유지한다.

Core 수용 뒤 ACK 실패·완료 기억 소거 시 close가 Stale로 막힌 결함을 실제 재현·수리했다. cached ACK drain→남은 pending 취소로 mutation1/ACK1·원 결과 보존·반복 close 추가 ACK0을 확인했다. 원 실패·수리·최종 UI83/strict/fmt·사본/diff는 runtime/quality-review/ui-pending-document-r1에 있다.

이는 headless 기능 소비다. 제품 런처 등록·실제 SDK→OS wake/자동 close·물리 입력/IME·시각 일치·13화면은 아직 미완료다. 데이터 ACK는 화면 표시나 사람 품질 ACK가 아니다.

## Native 최종 인수와 제품 연결 배정

Native 상세 중간 실패·수정·인수 원문은 [Native 인계까지의 조율 이력](coordination-history-2026-10-20-native-startup.md)에 보존했다. 정리 전 원문/사본 SHA256은81B70EF24AB3A6851EE397B80BAA31E1EAE5BA4784FD37A528D0AB8B8AC76504로 동일했다. 범용 API/검증의 정본은 [Meari 검증](../../../packages/meari/.spec/evidence/verification/native-startup-scheduling-2026-10-20.md)과 [ADR0015](../../../packages/meari/.spec/adrs/0015-ui-thread-document-host-port.md)다.

- 최종 inventory2827a5a6b4b902ce00b4f1acbb1631b157c5c93dd41dea5ca50521753b63132a의316개 현재 파일을 PM이 직접 재hash해 차이0을 확인했다. Native 외부·Cargo/lock 차이0, owned[]와 명시 슬롯 반환을 인수했다.
- final-headless68437ms/exit0·88pass(56+14+1+17)/각0fail·0ignored, final-strict14176ms/exit0, final-doc3603ms/exit0·3pass, fmt0을 원문 대조했다. 기본 구성62pass와 window-only cfg 수정 후 default-strict21154ms/exit0은 별도 근거이며 중복 합산하지 않는다.
- 새 startup OS5를 최종 동일 핀의 final-os-*에서 각각3795/1854/1456/2175/1731ms·exit0로 재확인했다. 실제 doc0 마지막 알림 후 timer→join1·추가 입력0, 정상 resize/child-first close/실제 HWND 제거, pre-View 원 오류/owner 보존, post-View 실패1/명시 정리, Ready 재투영 전후 frame stamp 보존을 포함한다. 기존 child OS36428ms/exit0·정확1pass도 확인했다.
- 실제 발견한 종료 후 추가 wait와 HWND 파괴 전 반환 결함을 수정했다. 원 실패/소유 PID 회수/수정 전 소스/최종 diff는 runtime 근거에 보존했다. frame 최초 실패는 pre-loop baseline 오류였으며 원 Ready 자극을 복원한 r3만 인수했다.
- Closed notice는 논리적 host/lease 제거다. strict 정상 반환은 실제 Destroyed를 기다린다. FrameSubmitted는 render 입력 HostStamp와 checked submission이며 compositor 표시·SDK ACK·사람 품질 승인과 별개다. Ready는 surface 준비로 모든 실행에서 첫 제출 이후임을 보장하지 않는다.

현재 제품 연결 배정은 [UI 정확 계획](presentation/typed-async-sdk-s1.md)과 [NW01–08](presentation/native-sdk-wake-gate.md)을 따른다. UI는 기존 same SDK/owner/actual scope/immutable raw·prepared Arc로 window_startup·typed close·typed guard 재관측·실제3task 소비를 구현한다. SDK는 consumer Native window feature/lock, 얇은 notifier와 startup 소유 bundle 후보 소스를 작성해 UI에 인계한다. 이 단계는 consumer integration path 검증이며 root Native 의존성/부모 export/production launch는 다음 필수 단계다. 기본 eframe/main 교체와 전체13화면 완료로 확대하지 않는다.

[SDK 조립 후보](sdk-proposal-window-startup.md)를 PM이 인수했다. inventory9CA03C7A223B7CF5332FE4CAEBEB035BD36F8E00CDB8A8A32BC6CC7668B6579C의5파일 현재 차이0. consumer manifest EF84652B…71BC/lock E5C8BB86…04D0, offline resolver3246ms/exit0와 diff를 확인했다. SDK는 후보를 동결하고 UI에 같은 소스/alias 소비를 인계했다. 이는 rustfmt 파싱·소스 인계이며 typecheck/test/strict/OS는 아직 미실행이다. UI는 이제 인계 조건을 충족해 본인 구현 입력 고정 뒤 단일 heavy 검증을 시작할 수 있다.

UI는 위 SDK5를 직접 재hash한 뒤 window_startup/attach_observed, typed parent close의 원 ExecutionCloseProgress 보존, completion guard 출처 전달, 기존 의미 assertion의 support 추출과 실제3task OS 후보를 작성했다. runtime/quality-review/ui-native-pending-r1에 before/의존 입력을 보존하고 compile-first를 시작했다. PM이 launcher83712/child24280·terminal=false·2CPU/BelowNormal/descendant Job receipt를 확인했다. 신규 window graph 컴파일 중이며 아직 새 typecheck/test/OS 성공이 아니다. SDK 원 Fault Arc·kind/worker/unack와 outer Err/registration 실패를 구분하도록 소스 검토 의견을 반영했으나 동작 근거는 이후 시험에서 인수한다.

NW04/05 실제 마지막 SDK notifier 경쟁을 위해 SDK 소유 조립 함수에 trusted notifier wrapper의 additive seam을 배정했다. 기본 함수는 identity wrapper로 같은 경로를 유지하고 생성 실패의 wrapped Arc는 Failure가 보존해 같은 Arc로 재시도한다. 외부 payload/late setter/추가 worker·업무 정책은 없다. 현재 compile-first terminal을 UI가 확인·인계한 뒤 SDK가 해당 파일만 수정하고 새 핀을 넘긴다. 빌드 중 입력 변경은 금지하며9CA 후보는 이전 입력으로 보존한다. 실제 Native notifier 위임 후 정확 최종 알림만 barrier로 막아 WorkerExitPending→추가 알림/입력 없이 timer join을 검증하는 계획이며 아직 실행 증거는 아니다.

compile-first의 terminal=true/exit0·583412ms와 --no-run executable 목록을 PM이 원문 확인했다. 실제 테스트 본문은 실행하지 않았고 OS 후보의 미사용 Arc import 경고1건이 남았다. UI가 원 소스/로그를 보존하고 SDK에 wrapper 수정 시점을 인계했다. SDK 수정 후 새 pin/경고 수리를 포함해 다시 검증하며 첫 컴파일 성공을 최종 strict·실제 동작 성공으로 확대하지 않는다.

SDK wrapper 인계본의 5파일 inventory SHA256은 22E1F7C92A17A6B23C36184F4158A51CCA88F15AA70239F109A76BEE8E2453ED다. PM이 현재 파일을 독립 재해시해 차이 0을 확인했다. UI focused-first 원 receipt는 terminal=true/exit0·69741ms이며, 두 실행 파일의 결과는 각각 7 pass/6 ignored 및 10 pass다. 공통 initial_panel 시험이 중복 포함되어 있으므로 독립 기능 17건으로 합산하지 않는다. 실제 OS 6건은 이 실행에서 제외되었으며 전체 회귀·strict·실제 OS 및 제품 player.exe 등록은 아직 완료 근거가 없다. SDK/Meari 입력은 동결하고 UI가 동일 단일 heavy 슬롯에서 후속 검증을 계속한다.

이전 제품 동결 입력은 SDK E1959DC…8BF5의23파일/UI5b6515…7a2f의16파일/generic f469a8…d2b의5파일이 각각 현재 소스와 차이0임을 PM이 인계 직전 대조했다. 이제 승인된 UI/조립 후보만 변경하며 새 before/after 핀으로 검증한다. Native/Core/generic/SEG/CLIP/CSS/테마와 SDK application/domain은 계속 동결한다. Meari 신규 라이브러리 변경은 소비 gate 인수 뒤 별도 조율한다.

## 독립 UI 후보와 후속

잔여 OS3경계의 원 결과도 PM이 대조했다: guard-first57561ms/124는 전체 stamp 동일성의 잘못된 단언과 barrier 대기 종료로 실패했고 원 소스를 보존했다. owner/definition/document revision/raw 불변과 turn_sequence+1을 구분한 guard-r2는39374ms/0, partial-first2986ms/0, commit-first7366ms/0이며 각 exact1 test 통과다. trusted commit 확장의 첫 검사 E0282는 Option 타입 명시로 수리하고 strict-commit-r2는3507ms/0이었다. 이력 OS6과 새3을 같은 최종 입력으로 고정하기 위해 full-final 및 필요한 OS 재확인·최종 fmt/pin/owned[]를 UI가 수행 중이다. 아직 제품 등록을 시작하거나 최종 소비 인수 완료로 승격하지 않았다.

잔여 소비 인수의 renderer commit 경계는 기존 Native invalidprops/staged cleanup 회귀만으로 동일 async terminal 수용 후 실제 commit 거절·복구를 증명하지 못한다. PM은 window_bridge의 기존 reproject_with apply(export) 안에 UI 소유 trusted 최소 commit seam을 배정했다. 기본은 기존 Native commit 위임이며 시험만 commit 전 오류1회와 명시 retry 후 실제 commit을 검증한다. Native/Core/SDK 정책·직렬화 입력·전역 setter·성공 후 실패 위장은 추가하지 않는다. 원 OS6 입력을 보존하고 새 후보의 영향 회귀·strict/fmt·raw/old export/stamp/임시 등록 정리·중복 SDK edit/ACK 없는 복구를 별도로 인수한다. 이는 구현 승인과 미검증 계획이며 성공 근거가 아니다.

PM이 UI의 실제 OS6 원 receipt와 각 test 본문 결과를 대조했다. os-doc0-prepared-first43576ms/0, doc0-error1808ms/0, closed-notifier1664ms/0, correction5725ms/0, cards3113ms/0, highlights3149ms/0이며 각 별도 process의 exact test1개가 통과했다. doc0는 실제 SDK 마지막 알림 경쟁·원 terminal과 typed join receipt를, 창3종은 정규화 callback을 통한 동일 SDK 편집/채택 의미를 검증한다. 실제 guard 해제·commit 실패·부분 같은 owner 복구의 잔여 매핑과 최종 핀/영향 회귀/owned[]/슬롯 반환은 인수 전이다. 물리 keyboard/IME·13화면 시각 일치·player.exe 기본 등록의 증거가 아니다.

UI consumer full-first의 원 receipt와 실행별 test result를 PM이 확인했다: terminal=true/exit0·252870ms, 합계91 pass/0 fail/6 ignored다. 기존83 의미 기준에 registry guard/provider Busy 구분 회귀1개와 새 실행 파일에 중복 포함된 initial_panel7개가 더해진 수치다. 독립 신규 기능8개나 OS 통과로 합산하지 않는다. 후속 strict/fmt 및 실제 OS6은 별도 게이트이며, 제품 실행 경로 등록은 아직 다음 배정이다.

후속 strict-first는68404ms/exit101로 OS 시험의 collapsible_if1건을 검출했다. UI가 원 소스/로그를 보존하고 조건을 정리한 strict-r2는13062ms/0이었다. doc0 관측 보강 후 strict-r3는3639ms/0, fmt-first는3840ms/0으로 PM이 원 receipt를 대조했다. 새 시험은 notifier 총2회·최소3회 지연 관측·실제 WorkerExitPending{loop_finished:true} 이후 release·원 raw Arc·반복 Closed receipt를 검사한다. SDK 내부 join 호출 수 자체를 공개 API로 직접 관측한 증거로 확대하지 않는다. 실제 OS 첫 실행은 os-doc0-prepared-first이며, 이 기록 시점에는 컴파일 중 terminal=false다. 보강 전 full-first와 후속 소스의 근거 범위를 분리하고 최종 핀과 영향 회귀를 별도 인수한다.

SEG 신규 src/presentation/web_ui/studio/segments/inspector.tsx SHA1d4b0da7b0aaba755cfdf29ecc59aa2a56f223596fc67eaccc576038783cdf49를 PM/Meari가 대조했다. 명시 include graph·bounded scan 뒤 route/export/import 등록 없이 분리했고 기존16·공통 TSX/CSS/tokens를 유지했다.

OP/ED 설정·Switch2·재생 옵션·최근 receipt/undo·분석 metadata·범위/회차 저장은 controlled props/callback뿐이다. JS 판정·시간 계산·저장·demo 정책은 없다. 원본/crop·요소표·해시·정렬/치수/색상/i18n 미검증은 runtime/coordination/ui-seg-inspector-s1/ownership.md와 [SEG 시각 계약](../../sots/player/presentation/studio/segments-visual.md)에 보존했다.

checked/direct Boolean host callback은 미지원이며 boolean state/input.target.checked와 구분한다. uncontrolled/defaultChecked·disabled mock으로 축소하지 않는다. [Meari 독립 후속](../../../packages/meari/.spec/plans/native-document-callbacks.md)로 추적한다. parser/compile/실제 렌더·SDK 저장/13화면 완료 증거가 아니다.

CLIP 신규3파일 source-only 후보를 인수했다: src/presentation/web_ui/studio/clips/recommendation-item.tsx, src/presentation/web_ui/styles/shorts.css, assets/design/icons/source-link.svg. runtime/coordination/ui-clip-recommendation-s1/candidate-pins.json SHA는F15E24827194A39CDC3F0BFA327D251D94BC6835AC64BDFE74C0C67C73EC2BE0다. PM이 실제 TSX/CSS/SVG·관측 파일을 읽고 공통 토큰/마스크 규격과 포함 선택·preview 분리를 대조했다. 원본/4배 crop·선행 요소표·graph·미검증은 같은 경로의 ownership/preflight에 보존했다. controlled Boolean은 SEG와 같은 미지원 후속이며 Action으로 우회하지 않았다. 공통 파일·라우팅/@import·SDK 정책·빌드/GUI 변경0을 유지한다. parser/asset 해결/실제 시각·전체 CLIP 기능 완료는 아니며 Native 인수 후 제품 OS 조립이 우선이다.

## 남은 통합 순서와 전체 범위

1. Native 구현→pure/호환→Windows 기술 수명·최종 API/핀·소유 정리·슬롯 반환 인수.
2. UI가 같은 SDK owner/registry/document로 production notifier·startup·Busy guard 해제 kick·typed join 대기 bounded 재관측 조립. 실제 SDK→UI→OS에서 terminal ACK·projection commit·worker join·parent 제거를 각각 관측.

   소비 gate 뒤 [제품 등록 경계 검토](../../evidence/reviews/product-native-registration-boundary-2026-10-20.md)에 따라 root optional Native feature·부모 모듈·명시 시작 분기를 SDK/UI가 나눠 등록하고 player.exe 실제 시작·종료를 별도 검증한다. root가 bridge tool을 역의존하지 않는다. path module/alias 소비 성공을 루트 등록 성공으로 확대하지 않는다.

   SDK의 [제품 등록 사전안](sdk-proposal-window-startup.md)을 PM이 검토했다. 제안 경계는 optional Windows native-proposal-window feature, 전역 paint patch의 resolver 영향 분리, feature-gated self alias와 canonical root façade, bounded local plain launch DTO 및 단일 client/loop다. SDK는 Cargo/lib/infra/assembly/launch/main, UI는 presentation façade·원문 모듈 등록과 입력 변환을 맡는 순서로 준비한다. 정확 schema·한도·옵션 충돌·미지원 오류와 실패 owner 회수는 구현 시 검증해야 한다. 소비 gate 최종 인수 전 구현/manifest 변경은 시작하지 않으며 명시 시작 옵션은 아직 지원 API가 아니다.
3. [HP01–HP10](historical-import-pending-integration.md)의 강한 취소·게시 reservation, project async UI, invalid terminal 명시 복구·원 후보 decision·품질 표시 구현·검증.
4. 승인13화면·세부 탭/상태·공통 아이콘/Pretendard·router/plugin과 미디어·자막 화자 분리·음악 OP/ED·다운로드·스냅샷·카드·숏폼·2D/3D 에셋 전체 범위 유지. 부족한 Meari 기능은 Rust checkpoint로 연결. [mpv 배포안](mpv-package-boundary.md)은 권고이며 submodule/설치기 구현이 아님.
5. HF 모델·실제 미디어의 사람 품질 검토는 제품 연결 뒤 AI 담당과 마지막에 진행. 현재 추론/다운로드 재개0.

SDK의 [S2 후속 감사](../../evidence/reviews/sdk-historical-cancellation-boundary-2026-10-20.md)를 PM이 인수했다. 최초 배정 후보는 media-jobs의 cleanup 미완료 typed owner와 기존 executor 호환 회귀다. proposal 게시와 durable job complete는 별도 저장이며 실제 출력 영수증·중간 실패·restart 재시도 복구 계약이 아직 없다. 이 공백을 구현 완료로 취급하지 않으며 현재 SDK/UI 동결·Native 단일 슬롯을 유지한다.

UI의 [제품 연결 사전 계획](presentation/typed-async-sdk-s1.md)과 SDK의 조립 읽기 검토를 인수했다. startup wake→얇은 notifier→기존 client의 ExecutionOwner 1회→InitialPanelStartup/controller→동일 loop run 순서이며 실패 시 같은 loop/startup/execution owner를 회수한다. notifier 오류는 기존 SDK notification_error에 보존하고 mailbox/thread/late setter를 복제하지 않는다. consumer는 기존 Native dev-dependency에 window feature만 명시하는 경계를 우선 검증한다. root는 Native 의존성과 presentation 등록이 아직 없으므로 production optional feature/export/launch는 별도 SDK 배정이 필요하다. 이는 인수한 후속 설계이며 새 API·제품 실행 성공이 아니다.

history3문서 링크15개 수리 이력은 runtime/quality-review/sdk-document-links-r1에 있다. 현재 조율 문서 축약 뒤 기존 Rust 도구로 다시 검사한 check-links38006ms/0·912문서/local broken0 결과를 PM이 원문 확인했다. runtime/quality-review/typed-close-coordination-links-r1에 이전/현재 보고서를 분리 보존했고, 원본 보존 upstream 참조4개는 별도 미해결 분류다. 문서 의미나 기능 완료를 링크 통과로 대신하지 않는다.

이력: [typed close까지](coordination-history-2026-10-20-typed-close.md), [Pending](coordination-history-2026-10-20-pending.md), [provenance](coordination-history-2026-10-20-provenance.md), [이전 인계](coordination-history-2026-10-20.md), [회차·커서](transition-cursor-coordination-history-2026-10-19.md).

