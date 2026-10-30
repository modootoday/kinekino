---
id: evidence:kinekino/20261029113732
title: PM 통합 검사 이력 — 2026-10-25
status: active
created: 2026-10-29
updated: 2026-10-29
---
# PM 통합 검사 이력 — 2026-10-25

원 현재 상태 문서의 전체 기록을 보존한다. 아래 시점별 판정은 역사적 근거이며 최신 상태는 [현재 검토 빌드](../../../plans/player/current-review-build.md)를 따른다. 상대 링크만 새 위치에 맞춰 조정했다.

# 사용자 검토 빌드 — PM 현재 상태

현재 기준: 2026-10-25, source267-r3의 영향27 통과·root77fd593 정본 반영 완료. 실제 GUI의 AwaitingFrame/Slot(NotVideo) 이후 native retirement·새 Viewing 등록의 공통 계약 후계를 구현 중. 이 문서는 현재 판정과 실행 배정을 관리한다. 아래 과거 검사 기록은 후속 결과와 함께 읽으며, 원 증거는 담당 verification 정본과 [실행 이력](../../../plans/player/execution-leases.md)에 보존한다.

## 목표와 마감

goal 생성 2026-10-24 22:45:31 KST, 12시간 마감 2026-10-25 10:45:31 KST. 목표는 active이며 아직 미달성이다. 정적 검사·합성 시험·정상 창 종료를 실제 영상 재생이나 사용자 품질 승인으로 계상하지 않는다.

최신 실행 판정: source267-r3 `720AA4F17C15955A2A7FC3ECBCC15CF72FA3DD024DFAFE43EBB09929E033A137`는 fmt/type/strict Clippy/no-run 및 영향27을 모두 통과했다. 각3 작업 strict state·원 SDK scratch 편집/저장/오류/채택/출력 판정·이력과 초기 mount·원 ignored6·raw 변조 거부·graph·feedback를 포함한다. 원 후보 판정의 합성 UI 상태는 실제 SDK 원 후보 persist 인수와 구분한다. 보호418 `B3DED9C42B915DD285769F61C30D17F8895A1F9C8A59EAEF48363991D2277C6B`, source 복원 실패0·새 HEAD9f4/0844 불변·WIP/EOF/writer0. 정상 player `C5F5F60FFD593645218921CC4A7FCB64FED17CF1C7D3CB16B3BEFD37CE86AD30`의 GUI 재검증을 UI에 배정했다. 실제9는 읽기만 하며 사용자 품질 승인은 아직0이다. 별도 Viewing UX 후계는 이 후보에 섞지 않는다.

최신 GUI 결과: PID95768에서 실제 Correction 원시 모델 JSON과 새 Studio UI를 표시했고 UIA에 이전 Viewing controls는 없었다. 그러나 검은 영상 추정 영역이 상단을 가리고, 단계는 AwaitingFrame에서 멈췄다. 물리 child bounds는 미관측이므로 잔류 child로 단정하지 않는다. 최종 오류는 `Surface(ParentMediaSurfaceError(Visible(Input(ParentClient(Access(Slot(NotVideo)))))))`, frame15/scene15·admission_revoked true·original_engine_retained true다. Activating/Active0, 실제9 쓰기0·재open0. 원 AltF4 후 PID 종료·정상 OS exit·cleanup complete·worker/dispatcher 정리를 확인했다. 이전 video surface/consumer 해제와 새 Studio frame 관측의 수명 경계를 SDK·UI·Meari가 소스 조사 중이다. NotVideo 검사 무시나 가짜 프레임으로 통과시키지 않는다.

마감 경과 시점에 사용자 검토 가능한 통합 빌드는 제공하지 못했다. 당시 신규 통합은 fmt/type 통과, strict Clippy 한 곳 수리 후 재검사 대기였고, 신규71 회귀·실제 재생·GUI 인수는 미완료였다. 사용자에게 지연과 미완료를 보고했다. 마감을 임의 연장하거나 성공 기준을 축소하지 않으며 원 목표의 미완료 작업을 계속 수행한다.

## 현재 선택과 검사

- 검증된 최근 선택은 r5 공급251, SHA `C0057D4BAC392077A58DF38C32259998E0803E78424BB8F39D1A203202343144`이다. r4에 원 Root binding 검사 순서 수리만 추가했다.
- 같은 Root·blitz-paint·meari-native Native 기능 조합의 fmt, all-target type, strict Clippy `-D warnings` 모두 통과했다. Clippy 종료0/59.508s.
- r5 fmt/type/strict Clippy, Root/jobs·Paint/Native 시험 빌드, 영향20/20 회귀 통과. 이어진 실제 airing1은 44.518초 Ready timeout: clip은 Rectangular로 정상 인식됐지만 후행 unknown paint 범위 때문에 visibility/plane Unknown, 활성화 실패, 실제 재생과 provider jobs 모두0이다.
- 반환 기준 보호386 `B4AC429910FDB53497AD8EF77E84189941DEE064862718E8F2B90D512A9727F5`: 원 창 unforced exit0/family Empty/EOF, 복원 불일치0. 커밋·push는 현재 배정하지 않았다.
- 다음 통합은 r5 + 제품 창 효과13/영역 clip `C56A71F077AC6A6424C90021AAC600786E9A74741E974B2D4D09BDE350CC5E26` + Meari 입력8-r2 `451BD6369A36C4A10EAC8EA574D7EA61F7BC6BD1E4B2D4F57CA84FE964C28D8C` + Unknown paint4-r3 `348BE20A3405C1484A3E287AD83A75C66BFD158005ECBCB85383ACF7FA166532`. 현재 소스 후보이며 컴파일·실행 성공으로 계상하지 않는다. SDK에 단독 heavy slot을 배정했다.
- 위 후보의 합성 manifest는 [최종260](../../../../runtime/development/product-review-next260-r1/coherent260-supply.json), SHA `388533C554F22523EC2BB4D8FF50312E589D4871EC99F05D087BBE5C00DA3493`으로 동결됐다. 선행 바이트가 동일한 상속 행과 직접 후계를 구별했으며, 실제 검사 결과는 별도 기록한다.
- 최종260 첫 검사: fmt0, type101/63.048초. 테스트용 `ViewerHost` 리터럴 두 곳(`mount_card/tests/capture_readonly.rs:35`, `mount_card/tests.rs:637`)에 신규 `window_ui` 필드가 누락됐다. Clippy·시험 빌드·71 회귀·actual은 실행하지 않았다. 원본 복원 실패0/자손 writer0/원386 보호 불변으로 반환했다.
- UI 최소 후계 [fixture2](../../../../runtime/development/product-window-ui-fixture-init-r1/source2.json), SHA `17A2790D9E4E2CDF640D53820D0A11EA7A62BCCCDBFC94C8686DC5D17FF1BC9A`: 각 리터럴에 `window_ui: Default::default()` 한 줄을 추가하고 원 assertion을 보존한다. 새 후계 통합 후 동일 검사와 조건부 actual1을 SDK 단일 슬롯에 배정했다. 이 후계의 컴파일 성공은 아직 미확인이다.
- fixture2를 포함한 r2는 fmt0/type0(67.078초), strict Clippy101(37.119초)이다. 원인 한 곳 `viewer_host/window_ui.rs:305`의 collapsible_if가 lib/lib-test에 중복 보고됐다. 후속 시험·actual0. UI는 원 평가 순서와 `armed.take()` 조건을 유지한 let-chain 후계 [lint1](../../../../runtime/development/product-window-ui-timer-lint-r1/source1.json), SHA `EBFA4089FCFEB9D30A140AE669164407903A4D71C4BE4F0AD037E1FD6A858176`을 제공했다. 이 후계의 중앙 검증은 아직 미완료다.
- lint1을 포함한 최종260-r3 SHA `1448D972F9A7C024F5ED35821783A8956239CF8A3DEF0BB4852E7BD6513E2A95`는 실제 fmt0/6.620초, joint type0/52.552초, keep-going strict Clippy0/66.144초다. Root/jobs 시험 빌드가 실행 중이며, 다음 joint 시험 빌드·고정71 회귀·실제 airing1은 아직 성공 근거가 없다.
- 후속 실제 결과: Root/jobs no-run0/360.636초, joint no-run0/60.481초, 고정71 receipt 모두 terminal/exit0(71 pass, fail0). 신규 창 효과4, 입력/창 명령, clip/Unknown 관측과 timer 회귀를 포함한다. CPU·Scene/RGBA·정책 모델의 통과이며 실제 OS Ready·영상·소리·사용자 품질 승인을 입증하지 않는다. 원본 복구·producer pin 확인 뒤 같은 소스의 실제 airing1을 이어가도록 배정했다.
- **새 실제 airing1 통과**: 원 GUI-owner/동일 SDK·CLI exact 시험 exit0/36.15초, Ready activated/frame callbacks3, provider jobs3 completed. 실제 mpv native 관측 1920×1080 h264/aac·audio_channels2·position213278→1214278μs와 frame PNG를 확보했다. 시험 volume0이므로 사람이 들은 소리의 증거는 아니다. FinalV2 cleanup_complete/normal_os_exit true, first_error null, faults0/unack0, 보호395 SHA `9E9DCBAF3BE7DA02958C49F70137DA49E7B83BA6525446DE4B6AB7BB7AF7ED95`, WIP·EOF·writer0 확인. [실제 airing 정본](../application/product-review-next260-r3-actual-airing-2026-10-25.md). 같은 후보의 completed→movies→adjacent 검증을 직렬 배정했다. 이 성공을 세 카테고리·수동 창 컨트롤·AI9 품질 승인으로 확대하지 않는다.

| 목표 흐름 | 현재 근거 | 남은 인수 |
|---|---|---|
| 공통 SDK/CLI·네이티브 경로 | source1448 실제 세 카테고리 탐색·제공자·네이티브 재생 및 정리 통과 | source267 정적·27회귀와 실제 재생 근거를 구별, native route 후계 영향 검증·실제 연결 |
| 자동 시청 UI | source1448 원 GUI-owner/동일 SDK·CLI 이전·다음 왕복 실제 통과 | 정상 GUI 사용자의 표시·입력 정합성·최종 UX |
| clip·Native timer | source1448 정적·71 회귀·실제 Ready/재생 통과 | 정상 GUI의 간헐적 capture 불일치와 실제 시각 인수 구별 |
| AI 검토·재진입 | 실제9 입력·목록/선택 GUI 관측, raw graph·오류·취소 회귀 통과 | 비교 화면의 props 수리는 27회귀 통과, 실제 AwaitingFrame 전환 수리·사용자 편집/판정/이력 GUI 인수. 사용자 승인0 |
| 컨트롤·창 UX | source1448 CPU 회귀 및 실제 전체화면·패널/유휴 숨김·포인터 복귀 관측 | 커서 OS 가시성·시각 일치·최종 drag/time UX. μs 입력/개발 버튼을 최종 PotPlayer UX로 계상하지 않음 |
| Git·배포 인계 | 기본 거부 공개 경계·root77fd593 및 Meari0844 검증소스 정본 커밋 | native route 후계 검증·명시 경로 채택, 정상 실행 안내 |

### 실제 후속 완료 및 GUI 슬롯 이관

동일 source1448의 completed·movies·adjacent 원 GUI-owner/SDK·CLI 시험도 통과했다. completed와 movies는 각각 1280×720·audio2·재생 시간 증가, adjacent는 engine1→2→3 및 next/previous 각 1920×1080·audio2·재생 시간 증가를 기록했다. adjacent exact0/70.90초/jobs5, 이전 engine·IPC·stdout·stderr 종료, FinalV2 clean/error null/normal exit 및 원 child unforced exit0를 확인했다. 최종 보호398 SHA `10A8E37A5BF7A6A906E5206584725FDBF1508D55CC43BB9D458D45879CDA2CE3`, root/WIP5 불변·streams4 closed·writer0다. 자동 반복 재시도0/새 빌드0이며 음성 청취·GUI 수동 컨트롤 성공을 주장하지 않는다.

PM은 정상 player `E4B866F874351D79EE964009A45AE2C447B79DAE6F1EBBA17A08008C9D9D728F`의 단독 GUI 슬롯을 UI 담당에게 배정했다. 주 모니터 기본 placement·기존 argv와 별도 normal-product-review-r1 jobs/catalog/browser-profile을 사용한다. DPI/가상 장치 변경 없이 전체 GUI 레이아웃·입력·fullscreen·panel·idle/cursor·actual9 표시를 묶어서 검토하며, 실제9 사용자 판정/편집/저장에는 쓰지 않는다. 검증 namespace 기능 시험과 GUI 표시 검토를 구분한다. SDK는 추가 heavy 실행 없이 정본과 명시 경로 채택 계획만 준비한다.

정상 GUI 원 PID61036에서 첫 전체 레이아웃, 실제 borderless 3440×1440 확장과 전체화면 해제 라벨, 패널/유휴 컨트롤 UIA 항목 숨김을 관측했다. 직접 ActionRecord 로그·커서 OS 가시성·AI9 화면은 아직 미확인이다. 도구 즉시 출력의 검정 화면과 저장 PNG의 정상 레이아웃 사이 불일치가 있어 실제 client 검정 결함으로 확정하지 않는다. 최초 PM view_image 검정 관측과 담당의 후속 정상 파일 관측을 함께 보존하며, capture-custody-1.json의 SHA/mtime/FI와 원 반환 ref를 기준으로 구별한다. 저장 파일명만으로 결함을 확정하지 않는다.

후속 GUI 관측: 포인터 입력 뒤 transport 복귀와 실제 proposal9 목록·문서/항목 선택까지 확인했다. 선택 제안 열기 후 fullscreen 복귀는 관측됐지만 ManualRepair 원문/후보 화면은 표시되지 않았다. 원 AltF4 정상 종료, PID/child 종료, Final cleanup_complete/normal_os_exit true·unack0. trace first_functional_error는 `Surface(ParentMediaSurfaceError(Visible(FocusChanged)))`, frame4/scene4이나 발생 시점·분기는 미확정이다.

소스 조사에서는 최초 transition의 keyboard/active/foreground/capture 4-tuple을 이후 `observe_visible`에서도 영구 동일 요구하는 계약 공백을 발견했다. 정상 사용자 focus 변화 거부 가능성과 실제 child focus 탈취를 Final만으로 구분하지 않는다. SDK에 생성/transition 부작용 구간 before-after 불변 검사와 장기 관측의 child focus 비소유 검사를 분리하는 최소 후계 및 네 필드별 양성/음성 회귀를 배정했다. 원 parent/process/thread/scope/geometry 가드는 유지한다. UI는 AI9 화면 전환의 원 cleanup→consumer 교체 경로를 별도로 조사하며, 이 수리를 실제 GUI 실패의 확정 원인으로 선계상하지 않는다.

다음 중앙 후보261 SHA `2CBED494686E613BFA93299EB2CD538F25085567E33B1F4055541E521716A113`: 검증된1448 + Focus3 `333D3854A7D64DCE8EF8071C6AA0BAFD31A8C82041556177A349391C3F9116FC` + private route 진단2 `29FB5EA599FA6E527D5953851F9CB646C6B7173E3D3AA0617B4054C079F9C890`. 진단은 원 정책·오류 결과를 유지하고 최대64회/각768B의 단계·generation·오류 종류만 기록한다. 중앙 `product-review-focus261-checks-r1-gate`가 시작됐으며 fmt/type/strict/no-run/영향25(SDK13+기존UI11+진단개인정보1)를 검증한다. 검사 성공은 아직 미확인이고 이전1448의 실제4 성공을 새 후보에 복사하지 않는다.

후속 결과: 후보261 fmt0/7.225초·type0/60.780초·strict Clippy0/49.617초. 시험 빌드가 진행 중이다. 영향 시험은 case 읽기 전에 기존 live_entry 소유권·취소·오류 보존6을 추가해31개로 확정했다(`00-exact31.json`; 기존 `00-exact25.json` 파일명은 역사적 이름이며 실제 행31). 이전71 전체나 별도 Cargo 실행을 추가하지 않는다.

후속 검증 결과: Root/jobs no-run0/244.306초·joint no-run0/17.129초·원 영향25 pass. 추가 live_entry6의 첫 호출은 `--ignored` 누락으로 실행되지 않았고 성공으로 계상하지 않았다. 원 조건 확인 후 같은 바이너리의 별도 continuation에서 첫 항목 pass, 두 번째 `one_original_sdk_inspection...`은 `node slot requires exact declared component props`로 실패했다. 나머지4는 미실행이다. 최종26 pass/1 fail/4 미실행이며 사용자 GUI 재검증은 인계하지 않았다. 원 closeout complete/setup·cleanup failure0, 보호402 `A0CBEC15E090922806289FB938F1B5171122C48F372C282A97F3F8B8684EBC91`·WIP/EOF/writer0 반환을 확인했다. UI가 strict props 계약을 유지하며 원 live mount와 raw-review 공통 조립의 차이를 수리한다.

제품 원인: retained live mount만 `compose_editor_slot`을 거치지 않아 raw-review props를 strict typed editor에 직접 전달했다. fixture assertion이 오래된 문제가 아니며, 호출 한 곳을 기존 공통 helper→root_props로 연결했다. 수리1 SHA `D038186C2D6A9B65705791710E97C3D5322D5ABF8F7034E2B9B41D8AB384638A`, 최종 source261 SHA `60DFB2E3B8926C0EEC5FE6B03FD5C91D4B8302DDD9CDF31D03CC6BEFADDCD2B9`. `product-review-mount261-checks-r1-gate`에서 fmt/type/strict Clippy는 통과했고 Root/jobs no-run 진행 중이다. 영향11은 원 ignored live_entry6·raw 조립/기본 Root/start4·ManualRepair graph1이며, 각각 실제 실행 여부를 확인한다. 이전 Focus13/Native71의 변경 없는 시험을 기계적으로 반복하지 않는다.

mount261 후속 완료: Root/jobs no-run0/196.136초, 영향11 모두 실제 pass(ignored6는 각각 `--ignored`로 실행). 원 source111existing/150absent 복원 실패0·WIP/root 동일·streams closed·writer0, 보호406 `CBFA85B9AD41D7D9DE80D28DBA59BB1CEFDCC810CBAED86A1FF383DE04686C51`. 새 정상 player `6D14B657310368662A36FE684D876ED9DA582EF019FAD8BF7FEC493412A80C00`/CLI `E828A993DB8857BB79469D4C030AC97BA8A7F19EA248FBD1526870C8F534351D`의 원 producer를 인수해 UI에 정상 GUI route1 재검증을 배정했다. 기존 실제9 저장소는 읽기만 하고 사용자 판정·수정·저장에는 쓰지 않는다.

같은 후보의 정상 GUI 재검증은 원 PID58524에서 scene18까지 진행했고 이전 FocusChanged는 관측되지 않았다. 그러나 실제9 ManualRepair 열기는 `Draining→Starting→Inspecting→Inspected→Failed(Callback::Adapter)`로 실패했다. AwaitingArm/Frame/Activating에는 도달하지 않았다. 기존11 pass를 실제 ManualRepair 성공으로 확대하지 않는다. UI에 실제 ManualRepair와 기존 Highlight 양성 fixture의 차이를 합성 scratch SDK 경로에서 재현하고, mount prepare/bind/commit의 정적 오류 종류를 콘텐츠 없이 세분화하도록 배정했다. 원 actual9 판정·수정·저장0, 재open0이다.

ManualRepair 후계3 SHA `38EAA2A5AD4E6B9A1D4F5FB247A28A68FF97E11E2E3B87442A20B24BDA8816B7`는 미소비/미선언 repairMode·canEdit·revisionLabel·historicalReferenceLabel·historicalVerificationLabel의 생성 경로를 정리한다. 선언 확장·광범위 props 필터·strict guard 완화는 하지 않는다. 같은 원 SDK의 합성 NeedsRepair 문서를 사용해 Correction/Card/Highlight 각각 current inspection·ACK·release·원 mount receipt/proof까지 검증하는 시험을 추가했다. 진단은 mount Prepare/BindSession/Commit/MountBridge 및 정적 allowlist 오류 코드만 기록한다. 소스 fmt만 확인됐으며 중앙 fmt/type/strict/no-run/신규 all-task 재현+기존 영향 검증을 배정했다. 이 후보의 컴파일·실행 성공은 아직 미확인이다.

후속 source262 `9DFB56A10982D97D454A4A32BA078BFD0ACF317DC38A7A9BF11F6B59E1224E6D`는 fmt/type/strict/no-run 통과 후 첫 신규 재현 시험에서 실패했다(0 pass/1 fail/15 미실행). 외부 오류는 `Invalid(native consumer host failed; typed error retained)`이고 새 시험의 직접 expect가 원 Rig 오류 수집 경로를 우회해 내부 원인은 미관측이다. closeout complete/setup·cleanup failure0, 보호410 `E51676A00105BDEA8FEC97378420C210CCAE911A1816EA6A518060BE469850C0`·WIP/EOF/writer0 반환. [CPU 정본](../application/product-review-manual262-cpu-2026-10-25.md). 동일 바이너리 재시도0/GUI 추가0이다.

다음 UI 후계는 ManualRepair 전용 명시 props builder·bindings와 전용 feedback 투영을 책임별 모듈로 분리한다. typed editor 전체 props 상속 후 임의 삭제를 반복하지 않으며 선언된 필드를 직접 구성한다. 공유 feedback의 다른 소비자는 유지하고, ManualRepair는 선언된 resultLabel에 revision·결과·오류를 표시한다. 초기 noDraft뿐 아니라 각 작업 working draft·저장·오류·채택·판정·이력 상태의 실제 NodeSlot 계약과 원 Rig 오류 수집을 시험한다. 현재 소스 작성 중이며 컴파일/실행0, 실제9 쓰기0이다.

Git 체크포인트: 검증된 Meari 최종21 경로를 담당이 정본에 채택하고 `0844f6c24caf224617ca8abbb2dd75695a971b99`로 커밋했다. PM은 root의 `packages/meari` gitlink만 stage하여 `9f4a277`로 반영했다. 원 root HEAD3fc6455→9f4a277, library5893d44→0844f6c. docs WIP2·AGENTS/application 모듈 WIP·비공개 경계는 보존했고 push0이다. 정상 GUI/AI 인수 미완료를 commit 본문에도 명시했다. 이후 중앙 admission은 새 HEAD와 canonical library bytes를 인수하며 이전 root state와의 차이를 무단 변경으로 오판하지 않는다. Root 제품 소스 채택은 아직 미완료다.

전용 후계 구현 인계: [source10](../../../../runtime/development/product-manual-repair-projection-r1/source10.json), SHA `8A88587A8E3EA8F9E85C3DE4B3B7991E7EAF408D49E58E00EC0DCA598E8D4C3D`. projection/bindings/feedback 책임을 분리하고 초기·재투영 모두 명시 props를 사용한다. 기존 typed 전체 props 상속·알려진 키 삭제를 중단하고 공통 output 근거·이력·확인/재시도 값을 실제 TSX 표시와 입력으로 연결했다. 새 strict NodeSlot 회귀는 각3 작업의 실제 SDK scratch 편집·저장·stale/validation 오류·현재 revision·채택·출력 판정/이력을 다룬다. 원 후보 판정의 합성 UI 상태는 실제 SDK 원 후보 persist 인수와 구분한다. 원 Rig 오류 수집도 복구한다. 파일 fmt0/컴파일·시험·GUI0이며 중앙 통합 검증과 Meari 소유권/타입 독립 읽기 검토를 배정했다.

현재 중앙 최종267 SHA `12294FDFD027B4137F6ADEA0023171FBFD4E090BB95646BFE8AD693296579377`/영향27이다. 新 HEAD9f4a277/0844f6c와 D 선택 Meari36의 해시 일치·C preimage 차이를 별도 입장 근거로 인수했다. 준비 스크립트의 count406→410 전역 치환이 Cargo.lock 기대 해시를 바꾼 오류는 소스 적용·Cargo 전 중단했고 원 lock과 모든 pin을 확인해 기대값만 수정했다. 원 설정 오류 기록을 보존하고 명시 재admission 뒤 실행했으며 제품 실패로 계상하지 않는다. 포맷 통과/타입 진행 중, Meari 독립 검토의 필수 결함 없음은 실행 통과 근거로 대체하지 않는다.

이후 초기267은 신규 IntentPins fixture의 없는 source 필드로 type E0560 중단/복원했다. 실제 계약으로 시험 생성자만 수정한 후계 `10ECF98FF63CDF8A65E125CA97A56E443D4FD462C008B4030BD4C12AD573035B`를 합친 source267-r2 `39E0FC7E023906C1E42934647493ABFD3FC3D699EFC0D2D755D00A6E8127A880`는 fmt/type/strict/no-run 통과 후 12 pass/1 fail/14 미실행이다. 신규 상태 회귀와 초기 마운트는 세 작업 모두 통과했고, 13번째 graph 시험의 exact callback 기대 목록에 새 Text2/Action1이 빠져 실패했다. 기대값만 정확3 추가한 후계 `D5F40468E2FC1071785C9E2523363395B2331A28A33DE207E1816A92D2FCB5EE`를 원 선언·실 바인딩과 대조했으며 strict equality는 유지한다. 보호414 `EC2F719DB6A0EA360B5E87F864E4CCAB04250B2A92D1CF08F6167F16DF17F5A6`·WIP/EOF/writer0 반환 후 다음 검증을 배정했다. 현재 GUI는 재실행하지 않았다.

추가 확인: 실제 AI highlights·같은 media identity·영상 PTS·현재 SDK 구간 재생을 함께 입증하는 기존 표본은 확인되지 않았다. 실제9의 range=null을 임의 채우지 않는다. [표본 연결 감사](../ai/highlight-media-provenance-gap-2026-10-25.md)에 기존 ASR/음악 출력과 제작 클립의 한계 및 필요한 후속 검증을 기록했다. 이는 하이라이트 구간 재생 인수의 개발 미완료이며 사용자 승인 대기로 처리하지 않는다.

## 보존할 실패와 근거의 한계

원 통합251 회귀36은 각1회 **32 pass / 4 fail**이었다. 실패는 fixed raw 모듈 중복으로 인한 raw 재진입/기본 Root/start3건과 Paint fixture 생성자1건이다. 수리 전 통과32를 최종 r4의 실제 실행 성공으로 바꾸지 않는다.

- raw 수리 A7174FE9: 정확 sibling2 경로와 전체 선언이 동일할 때만 공유하며, 변조·다른 모듈 충돌 거부를 유지한다. 기본 제품도 Correction/Highlight 초기화에서 같은 분기를 사용한다.
- Paint fixture 수리 86710CD0: 원 leading LineTo와 거부 assertion을 유지하고 Kurbo 공식 Shape slice를 사용한다. 생산 clip 인식 함수와 Native 시험 바이트는 불변이다.
- 생산 clip 수리 0772D728: finite·unique·positive·축 정렬 사각형만 open4/closed5로 인식한다. 렌더 명령·stroke 경로와 비직사각형 거부를 바꾸지 않았다.
- 실제218은 높이871→258 변경만 관측했고 visibility Unknown/plane Unsupported·clip Unsupported로 Ready 미도달이었다. 원창은 unforced exit0/family Empty/EOF로 반환됐으며, expected jobs3/실제0 거절은 cleanup fault가 아니다.
- CPU HTML/Scene 관측은 원 OS 프레임 제출이나 실제 영상의 증거가 아니다. 실제 사용자 AI9 판정·품질 승인0이며 개발자가 대신 기록하지 않는다.

## 담당과 다음 배정

1. SDK: 위 최종 후보의 정확한 leaf·선행 관계를 합성하고 fmt/type/strict Clippy/관련 회귀 뒤 새 소스의 원 GUI-owner defaultViewing 실제 airing 첫1회를 검증한다. 실패 시 같은 바이너리 자동 재시도 없이 오류를 인계한다.
2. UI: 감사에서 확인한 필수 누락을 private 후계로 구현한다. 기존 typed host/plugin/Native 창 효과와 owned timer를 사용해 전체화면·패널 토글·idle 컨트롤/커서 복원을 연결한다. 동결 r4·중앙 캐시·GUI를 임의 변경하지 않는다. 근거는 [GUI source 감사](../presentation/product-r4-gui-acceptance-source-audit-2026-10-25.md)다. 드래그 seek/volume slider도 미지원이며 μs 텍스트 입력을 최종 PotPlayer UX로 완료 표시하지 않는다.
3. Meari: 새로운 정확한 library 오류가 나오면 소유 leaf 최소 후계로 지원한다. 추가 독립 빌드·API 확장·registry 변경은 하지 않는다.
4. Media: [실제9 정상 실행 입력](../../../../runtime/coordination/ai-review-input-r1/NORMAL-PLAYER-AI-REVIEW-INPUT.md)을 최종 검증된 player 절대 경로·producer pin으로 연결한다. 사용자 저장소에 개발 판정을 쓰지 않는다.
5. 실제 CLI 기능 인수 뒤 정상 GUI의 표시·입력·오류·원본/후보/이력 연결을 확인하고 사용자 테스트 절차를 인계한다. 검증된 변경만 PM 직렬 stage/commit, push하지 않는다.

빌드·미디어·추론 슬롯은 하나이며 CPU2/jobs1/BelowNormal·C warm cache를 유지한다. 독립 소스 작업만 병렬이다. ChatGPT·Terminal·무관한 앱, DPI·VM·Pages·새 추론/다운로드·새 장시간 시험은 건드리지 않는다. 마감이 지나도 미완료를 완료로 바꾸지 않는다.

## 검증 소스 정본 채택 checkpoint — 2026-10-25

PM이 source267-r3 `720AA4F17C15955A2A7FC3ECBCC15CF72FA3DD024DFAFE43EBB09929E033A137`의 non-Meari 231행 SHA 전부를 확인하고 canonical에 반영했다. 변경 211경로(신규127)를 root `77fd593e3c937c684c16427ae0735830623218da`로 커밋했다. 제목은 `Integrate SDK playback routing and strict Studio review projections`다. 기존 AGENTS, application/mod.rs의 별도 줄바꿈 WIP, Meari 문서 WIP 및 기존 미추적 파일은 보존했다. application/mod.rs는 ui_observation 모듈 추가만 index에 반영했다. 비공개 경로 stage0, cached whitespace 검사0, push0. 경로별 원본 SHA와 preimage는 `runtime/coordination/root-adoption267-r3/`에 보존한다.

이 커밋은 검증된 소스의 정본 채택이며 새 검증 실행이 아니다. 앞선 fmt/type/Clippy/27 회귀와 실제 GUI AwaitingFrame/Slot(NotVideo) 실패를 그대로 적용한다. 전체 제품 또는 사용자 품질 인수 완료가 아니다. SDK/UI의 route retirement 후계는 private로 분리 중이며 다음 gate는 새 root HEAD와 canonical preimage를 다시 캡처해야 한다. C snapshot의 복원 대상은 D canonical과 별도로 검증한다.
### 다음 단독 CPU 검증 슬롯 배정

PM은 SDK에 native retirement 통합 후계의 단독 빌드·시험 슬롯을 배정했다. SDK final pin·UI 공동 계약 검토·coherent manifest 전체 SHA와 새 root77fd593/Meari0844·기존 WIP·C snapshot preimage/absent 경계 확인이 선행 조건이다. 전제 충족 후 추가 PM 답변 대기 없이 기존 C warm cache/locked offline/jobs1/CPU2/BelowNormal에서 fmt/type/strict Clippy/no-run 및 기존27+신규 정책/UI/coordinator/late-registry/redraw 영향 회귀를 수행한다. 전제 불일치는 stage 전 중단하고, 실패는 원 오류와 원본 복원을 보존한다. GUI·실제 미디어·추론·다운로드·canonical/index 변경은 이 배정에 포함하지 않는다. 현재는 배정이며 실행 성공 기록이 아니다.
### 저자원 병렬 UI 범위 — 소스 구현만

UI는 retirement 후계와 별도로 고급/문제 해결 접기·사람용 시간 표시/입력·음량 표시를 구현한다. 이 후계를 최종 seek/volume slider 인수로 계상하지 않는다. PM은 Meari에 기존 A7 조사와 최소 terminal commit/owner continuity 수정의 private 구현을 배정했다. canonical0844와 기존21 정본은 동결하며 queue/drain 제한 확대·strict guard 완화·동시 Cargo/GUI 실행을 금지한다. Meari/UI가 typed range callback 소비 계약을 직접 맞추고 exact source pin·선행 SHA·지원/미검증 표를 인계한다. 현재 retirement gate와 별도 후계이며 완료 근거가 아니다.
### UI retirement 최종 최소 overlay 인수

UI 후계 선택은 source6 `CEC577294E02BF61FB109F045C1BDAF8F51766885F084020D5FDF513827F049C` → guard2 `8ECEBD3464B447FACF8DC51BD6FADDF6446893AEF973FC1137DFF095C4BF4C5C` → redraw4 `21D26B522A5E7A96AA28ECE74D0BF34BC12B3DF1281390D89DBD7A73ECE52FC6`다. PM은 마지막 manifest4와 각 source SHA 불일치0을 확인하고 HANDOFF 본문을 읽었다. 원 one-shot take_for가 실제 true인 경우에만 pending return에서 redraw를 전달하며, 없으면 기존 bounded retry를 유지한다. 두 ViewerHost fixture의 새 필드/explicit headless 초기화를 포함한다. 신규 ignored relay는 원 queue 1회·replay0·inspection0을 검사하도록 작성됐으며 실제 OS frame 증거가 아니다. SDK에 동일 통합 후보로 동결·검사 착수를 지시했다. 현재 formatter 이외 Cargo/시험/GUI 성공은 아직 없다.
SDK native retirement source15 `0A1A02200E8469967BE218DC16D417D62B08BBFB46182908271FA669ECF7192F` 생성 후 PM이 전행 SHA 불일치0을 확인했다. 공통 application route_media와 원 registry rearm·native consumer·redraw 연결을 포함한다. SDK는 UI6→guard2→redraw4와 합성해 이미 배정된 보호 재캡처/타입/회귀 gate를 진행한다. source15 존재와 formatter 통과는 Cargo 또는 실제 GUI 성공을 의미하지 않는다.
PM 독립 합성 preflight: `runtime/coordination/native-retirement-joint-preflight-r1/coherent-supply.json`, SHA `0E18EC864964C13D7D8C07EE938C639200669EA4B9C5D03044C5DF9D6A779B45`, unique275행. 선행267→SDK15→UI6→guard2→redraw4 순서로 선택하고 input-pins.json에 각 manifest SHA를 보존했다. 전 source SHA 불일치0. 이는 검증 입력 조립이며 staging/Cargo/GUI 실행은 아니다. SDK에 같은 행 선택의 독립 대조 또는 해당 manifest 사용을 허용하고 원본 보호·복원 이후 기존 단일 gate를 진행하도록 인계했다.
### Route275 중앙 자동 검증 실제 착수

SDK exec69111, gate `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/product-review-route275-checks-r1-gate`. SDK 합성 SHA `8B0BC4B684905D22BEB382780C4CEE96997A10CA93E966221508FD2C530A83F3`와 PM 독립0E18의 relative/SHA 275행은 일치한다(메타데이터 표현 차이). 입장·canonical77/WIP checkpoint·보호418·preimage275·staging275 파일을 PM이 직접 확인했다. exact50은 기존27+정책6+coordinator3+UI모델2/ignored3+late registry4+redraw신규1/기존2+원 parent-result-read2다.

PM 직접 관측: fmt receipt terminal/exit0/6.600초. type receipt는 아직 terminal=false이며 cargo50732/rustc27520/resource-runner29404가 실행 중이다. effective CPUs2/affinity3/BelowNormal·descendant job 적용을 확인했다. 아직 type/Clippy/no-run/50회귀/실제GUI 성공은 아니다. 동일 실행만 추적하며 새 heavy 실행은 배정하지 않는다.
Route275 후속 직접 관측: 07-type receipt terminal=true/exit0/80.146초와 cargo Finished 로그를 확인했다. 타입 검사는 통과했으며 strict Clippy·시험 빌드·50회귀·GUI 인수는 아직 미확인이다. 같은 gate의 후속 runner48856이 존재하므로 재시작하지 않고 기존 실행을 추적한다.

Route275 첫 gate 중단: fmt0/6.600초, type0/80.146초, strict Clippy101/70.111초. PM이 stdout compiler-message를 직접 읽어 신규 coordinator fixture retirement/tests.rs의 register closure4곳(130/191/251/292)에서 result_large_err(Err208bytes)만 확인했다. no-run/50회귀/normal bins/GUI 실행0. 17-result.json의 restored_existing113/absent162, restore_failures0/protected_changes0을 직접 확인했다. SDK에 Probe payload의 간접 소유로 크기를 줄이는 private 최소 후계와 같은50 회귀 재실행을 배정했다. lint allow·원 assertion 완화·제품 API 변경을 허용하지 않는다. 최초 후보와 실패 증거는 보존한다.
Route275-r2 재검증 착수: fixtureBox1 manifest `1CEE25859C3FB3D79ED73F2FEA75BEA18574442783B135582D4188D90BFE3210`, payload `42B825EA8A54C5EB07529C2D5DCA8661BB90925311F51BE98A12BA3FA7AF03CF`를 PM이 직접 확인했다. 같은 PlaybackSurfaceChild를 Box-owned Child에 두고 NativeChild 호출을 위임하며 API/기존 단언/lint 정책은 유지한다. SDK coherent275-r2 `4F64105FAD82F62CFC252F2CAC21A01908CED6DEF4B492D4E47C73FF915D7947`, exec11485. fmt receipt exit0/7.087초, type는 terminal=false이며 runner7596/cargo44476/rustc40972를 직접 관측했다. 원50·Root77fd/Meari0844·보호418/WIP 입장 유지, 성공시 정상 bins 조건부 배정 유지. r1 실패와 r2의 성공 여부는 별도로 기록한다.
후속 ViewingUX source16(ECE563)과 route275-r2(4F6410)의 소유 leaf를 PM이 대조했다. 직접 중복0/source SHA 불일치0이며 audit는 runtime/coordination/native-retirement-joint-preflight-r1/viewing-ux-overlap.json에 보존한다. 이는 파일 충돌 사전검사이며 결합 타입/기능/GUI 성공이 아니다. 현재 route 검증 후보는 그대로 유지하고 UX의 새 실행은 아직 배정하지 않는다.

Route275-r2 직접 관측: type0/63.713초, strict Clippy0/60.866초. 신규 fixture의 result_large_err는 이 후보에서 재발하지 않았으며 root no-run receipt가 실행 중이다. 원50 이후 동일 소스 정상 bins/보호 반환에 이어 기존 실제 airing/completed/movies 각1·adjacent왕복1의 조건부 단독 실행을 SDK에 배정했다. native consumer/activation/registry 변경이 initialReady·이전 engine 정리에 영향을 줄 수 있어 같은 후보의 재검증이 필요하다. 기존 validation namespace·짧은 sample·volume0을 사용하고 GUI/추론/다운로드/장시간 시험/실제9 사용자 판정은 제외한다. 각 단계 실패 시 중단·원 오류 보존, 자동 반복 재시도 없이 최소 후계로 진행한다. 기존1448의 실제 성공을 현275-r2의 성공으로 복사하지 않는다.
### PM 큐 한도 해석 정정

PM이 window/forwarding.rs의 0..32를 Native per-turn drain 한도라고 UI에 전달한 것은 잘못된 해석이다. Meari 담당 지적 후 본문을 재열람해 early_notifications_flush_once_and_require_bound_before_ready 시험의 반복 횟수임을 확인했다. 실제 core host_callbacks/document.rs::drain은 전체 pending.drain(..)이며 native_host/runtime.rs는 원 PendingTurn에 Vec를 유지하고 IntentAdmission::Capacity에서 tail을 보존한다. 근거 없는 실행 한도32 요구를 철회하고 UI/Meari 정본·인계를 정정하도록 전달했다. queue count1024/byte4MiB 계약은 실제 코드에 존재한다. 이 정정은 정책 변경이 아니며 현재 route275-r2 소스·실행 후보에는 잘못된32 한도를 도입하지 않았다.
### 왕복 이후 회차 전환 슬롯 위험에 따른 실행 순서 조정

SDK 추가 소스 검토에서 OriginalAdmission::revoke가 router.close_transition_admission을 호출하고 shared TransitionSlot::close_admission이 영구 closed가 되는 결합을 발견했다. PM도 admission.rs 본문을 읽어 호출을 확인했다. 현재 exact50은 실제 shared transition-slot의 Studio 왕복 이후 previous/next 허용을 입증하지 않으므로 완료로 계상하지 않는다.

현275-r2 gate는 동결 상태로 끝까지 실행·복원한다. 다만 현후보 정상 bins/actual4 후속 배정은 보류하고, SDK에 원 parent 최종 종료와 명시 route retirement의 admission 수명을 구분하는 최소 private 후계를 배정했다. 단순 closed=false나 전역 guard 제거는 허용하지 않는다. same shared-slot/router·새 lease 왕복 후 previous/next, old issuer/ticket 거부 및 finalclose 영구 거부를 함께 검증한다. 이후 후계 자동 검증 통과→normal bins→actual4→정상 GUI 순서를 유지한다. 이는 소스 근거의 미검증 위험이며 현재 실제 왕복 실패를 관측했다는 주장은 아니다.
Route275-r2 root/jobs no-run receipt를 PM이 직접 확인했다: terminal=true/exit0/312.409초. 시험 실행 파일 생성 성공이며 50개 회귀의 실행 성공은 아니다. 회차 전환 슬롯 위험의 별도 후계 때문에 정상 bins/actual4는 여전히 후계 검증 이후로 이관한다.

Route275-r2 exact 첫 실패: fmt/type/strict Clippy/root no-run 통과 후 exact47 pass, 48번째 `native_retirement_pending_blocks_original_current_inspection_until_same_ticket_completes` exit101. stderr의 positive fixture panic과 retained typed `ReadConsumerRetry(5ms)` → runtime Invalid(native consumer host failed) 전달을 PM이 직접 읽었다. 49~50 미실행, normal bins/actual/GUI0. 단순 fixture 실패 기대/오류 묵살로 통과시키지 않고 production 원 runtime의 bounded pending/wake 계약과 대조하는 UI 최소 후계를 배정했다. SDK transition-slot 후계와 함께 공동 계약 검토·새 coherent로 검증한다.

최종20-slot-return 직접 확인: source275 restored, root state identical, WIP5 unchanged, streams closed, writers0. Cargo 변경16은 원 producer 귀속으로 분류되고 fixed4를 포함한 보호422 SHA `1ACFC0F2D4D874C7EAC82F63F9006DE52386185230AE97B8A8D2EA7D58A39102`로 반환됐다. 원본 복원 실패0이며 이 반환은 시험 성공을 뜻하지 않는다. 실패 후보·시험 stderr·원 소스는 보존한다.
Route283-r1(독립283B936) 실제착수/타입중단: fmt0/6.461초,type101/33.993초. 신규 admission/tests/route_retirement.rs48/52가 private new_with_scope/guarded_command/HostScope::testing에 접근한 E0603/E0624 3건을 PM이 compiler JSON에서 직접 확인했다. 원본116기존/167부재 restore0/protectedchange0/exact0. SDK에 기존 허용 생성 경로를 쓰는 testleaf 최소후계와 같은56 재검증을 배정했다. 생산 visibility 확대·assert 약화0. 원 후보/실패 기록을 보존한다.

Route283-r2 B06 실제 결과: fmt0/6.670초,type0/62.431초,strict Clippy0/55.055초,no-run0/286.978초. 52개 exact pass 뒤 53번째 retirement_cancels_only_original_attempt...의 기존 old transition handle 오류가 NotFound인데 새 시험은 RevisionConflict를 기대해 실패했다. PM은 원 require_pending과 시험 본문을 읽어 pause ticket 충돌과 이미 정리된 transition handle 조회를 구분했다. 원 실패를 보존하고 오류 기대값 하나만 맞추는 최소 후계를 SDK에 배정했다. 반복 재컴파일을 줄이기 위해 미실행54~56을 같은 B06 고정 producer로 각1회 별도 진단하도록 배정했으며, 실패53 재시도/Cargo/staging0, 전체56 성공 주장0이다.

20-slot-return 직접 확인: source283 restored/root state identical/WIP5 unchanged/streams closed/writers0. 보호426 SHA96631DDB62B68550092793B24048AD7ED816A68695BD44B44DA3CEC788CBAF60, actual0. 보호 증거는 시험 성공과 별개다.
Route283-r3 DDBF9AC73959260E66735AC9C4CD92D5215ED37F1F187CB291D54849D5B3F8B7: fmt0/7.276초,type0/51.266초,strict Clippy0/43.178초,no-run0/97.168초 및 exact56 전체 통과. PM이 00-exact56과13-results를 대조해 expected56/observed56/unique56, 각 receipt terminal/exit0와 stdout의1pass/fail0/ignore0를 직접 확인했다. mismatch0. 17result first_error null/source116기존+167부재 복원실패0. 최종 Cargo 산출물/보호 반환은 SDK가 마감하고 이후 같은 소스 normal bins/actual4로 진행한다. 실제 OS/정상GUI/user quality 성공은 아직 아니다.