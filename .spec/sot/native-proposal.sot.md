---
id: sot:kinekino/native-proposal
title: 제품 Native proposal 실행과 종료
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 제품 Native proposal 실행과 종료

2026-10-20 제품 연결 5단계. [입력 계약](native-proposal-launch.md), [UI helper](../presentation/product-native-window-helper.md), [실행 검증 계획](../../../plans/player/sdk-proposal-window-startup.md)을 따른다. UI21/Native316/Core/generic 및 root Cargo/lock을 변경하지 않는다.

## 단일 조립

`launch::desktop_mode`는 유효한 feature-on Native 옵션을 `DesktopMode::NativeProposal`로 분기한다. feature-off는 기존 typed Unsupported를 유지한다. `assembly::meari::product_native_launch::prepare_native_launch`가 기존 옵션 파서·bounded 파일 adapter·application decode와 고정 스타일 자산을 worker 생성 전에 준비한다. main은 이 결과를 받아 기존 configured PlayerSdk/store/단일 ApplicationClientHandle 조립을 그대로 사용한다. LocalHost에도 같은 client.clone을 전달하고, Native 분기는 PreparedNativeLaunch::run에 그 client를 넘긴 뒤 기존 eframe 실행 함수로 넘어가지 않는다.

Native 실행은 기존 prepare_proposal_execution의 한 loop/startup/ExecutionOwner를 사용한다. UI prepare_product_window는 같은 client/owner/request를 소비하며 새 SDK/store/loop를 만들지 않는다. labels는 공통 Korean labels, viewport는 1180×760 dark, 일정 관측은 2~250ms/1024회, worker exit 관측은 5ms다. 이 값은 trusted assembly 설정이며 launch JSON에서 받지 않는다. 모델 추론/다운로드와 새 실행 파일·TSX 선택 옵션을 추가하지 않는다.

결과 객체의 notice 목록은 유한 NativeWindowNoticeKind별 최초 관측만 보존한다. 매 프레임을 메모리 Vec에 계속 쌓지 않는다. 개별 이벤트의 원 순서는 --record-validation stdout 스트림에 남기며, kind 목록을 전체 호출 횟수나 전체 이벤트 순서라고 해석하지 않는다.

## 실패와 소유권

ProductNativeOutcome::StartFailure는 원 ProposalExecutionFailure를 보유한다. Finished는 ProductCleanupOutcome와 창 notice 및 미실행 Native 권한을 소유한다. UI 조립 전 실패는 `unrun_native: Option<Box<(NativeLoopOwner,NativeStartup)>>`에 원 권한을 남긴다. 원 BuildFailure/NativeRunOutcome 전체도 ProductWindowOwners 안에 보존한다. Native 실행 이후 loop_owner가 None인 경우 새 loop로 자동 재실행하지 않는다.

ObservedController는 원 UI controller를 그대로 위임한다. window가 생기기 전 mount failure가 반환되면 사용자에게 표시할 retry UI가 없으므로 WindowStartupError::Mount로 runner에 반환하고 원 partial owner를 cleanup으로 인계한다. Native/Core의 OnWake/attach 정책 자체를 수정하지 않는다. factory/bridge/attach/initial/preparation 원인은 결과 보고와 소유 객체에 각각 남긴다.

product_native_cleanup::finish_owned_product는 UI OwnedProductCleanup을 2초/400회/5ms 예산으로 관측한다. poll 자체는 ACK하지 않는다. NeedsTerminalAcceptance에는 application의 명시 종료 정책으로 accept_initial_for_shutdown을 호출해 원 typed/raw 초기 결과를 격리 보존한다. Input 실패에 초기 포트가 없거나 수락할 수 없는 envelope는 Faulted와 원 오류를 반환하며 terminal을 버리지 않는다. Faulted/BudgetExhausted도 owner와 evidence를 함께 반환한다. 이 관측은 새 SDK executor나 대체 Native event loop가 아니다.

성공은 실제 반환 NativeRunOutcome.result Ok, SDKQuiescent의 Native Closed, 수락 오류 없음, FrameSubmitted 관측을 함께 요구한다. Native runner의 정상 result는 실제 owned window Destroyed 대기 계약을 사용한다. logical Closed notice/Drop/HeadlessClosed/NoDocument/ForcedIncomplete/Unconfirmed만으로 OS 정상 종료를 판정하지 않는다.

main은 owned outcome을 검사·기록한 뒤에만 최종 process 종료를 판단한다. 정상은 확인된 객체와 LocalHost를 명시 종료하고 반환한다. 실패는 원 오류·미정리 상태 보고 후 owned outcome을 유지한 채 명시 exit1로 process 경계를 닫는다. `result?`나 anyhow 변환으로 owner를 먼저 drop해 정리 성공을 암시하지 않는다. 비정상 process 종료가 추가 cleanup 완료를 뜻하지 않으며 Rust 권한 객체가 process 종료 후 복원 가능하다고 주장하지 않는다. 진단/receipt 보존과 메모리 owner의 수명은 별개다.

## 고정 제품 스타일과 자산

infrastructure::meari::product_styles는 원 tokens/components/studio/subtitles CSS 경로를 StylesheetProject에 유지한다. stylesheet_assets::rewrite 반환 CSS를 원 순서대로 결합해 Some(VerifiedAssets)와 함께 document에 전달한다. raw product_css join을 inline styles.css로 바꾸어 상대 URL을 해석하거나 자산 등록만으로 자동 rewrite된다고 가정하지 않는다.

세 Pretendard TTF는 기존 v1.3.9 SHA를 검사하며 원 license 두 개도 bundle에 포함한다. 모든 CSS 참조 아이콘/창 SVG 30개와 fonts3을 content-addressed manifest/blob으로 검증한다(고유33자산/URL참조34개). JSON/웹 메시지가 파일 경로를 선택하지 않으며 디스크·네트워크 fallback이나 WOFF2 대체를 추가하지 않는다. 각 원 CSS hash·재작성 참조·asset manifest와 실행 중 renderer asset report를 --record-validation 진단에 남긴다. delivered는 바이트 전달이며 폰트 선택/글리프 shaping/시각 일치의 증거와 구별한다.

## 검증 경계

원 회차·실패·수리·새 binary/입력/소스 핀과 최종 결과는 `runtime/quality-review/product-native-main-r1/`에 보존한다. 최종 handoff.md/receipt 기준 source13은 4F4AE9DCEB4A222F89B5730E3990C08CFE2DED6690F2A9144DF52B25376B0F0E, on player는 6610D2FFD801020882E62B46F8BE26244CB72202F1A91E972E3B8C34146FDC6F다. 이 source의 기본 전체440, on library371, 새 exe 정상6/오류5, trusted start/mount/deferred attach, 양 feature strict와 fmt가 통과했다. Native316 감사·UI21/고정10 동결 및 문서924 local link0을 기록하고 exact 담당 가계[] 확인 뒤 heavy를 반환했다. 예비 binary 실행을 최종 source의 새 binary 실행으로 승계하지 않는다. 반환 후 후속 library 변경은 이 checkpoint에 소급하지 않는다.

실제 player.exe의 6입력은 cards/highlights/cue correction/document correction과 repair unselected/selected의 시작·프레임·정상 close/join·원 store 불변을 검사한다. 새 exe에서 편집/수락 버튼을 실제 조작했다는 증거가 아니다. 기존 CLI73/OS9를 새 launcher 근거로 합산하지 않는다. 실제 요청 오류 및 SDK 초기 오류/잘못된 revision은 원 진단과 SDK/Native 상태를 분리한다. trusted SDK start/mount/deferred attach 실패는 Rust 포트 시험이며 같은 실패를 실제 exe 옵션으로 주입했다는 뜻이 아니다.

정확 child PID의 전체 top-level HWND/title/class/visibility/owner/parent/style을 보존하고 확인된 제품 root에만 WM_CLOSE를 보낸다. 처음의 visible2 단언 실패는 제품 root와 제목 없는 Winit Thread Event Target을 구분하지 못한 시험 가정이었다. 동일 winit-win32 0.31.0-beta.3 event_loop.rs의 THREAD_EVENT_TARGET_WINDOW_CLASS/전용 callback/CreateWindowExW와 actual HWND metadata를 대조했다. 원2관측과 최초 정상 process/SDK 결과는 그대로 보존한다. 보이지 않는 IME도 기록하며 auxiliary 관측을 삭제하지 않는다. FrameSubmitted 이후 제품 root가 보일 때까지 전체30초 deadline 안에서만 기다리고 강제 종료는 실패로 남긴다.

전체 제품 CSS를 사용한 Native renderer 기술 probe는 세 TTF 전달/rejected0/failures[] 및 chrome token pixel을 검사한다. 이것을 전체 화면/아이콘 paint·한일 shaping/weight 선택·13화면 1:1·물리 입력 인수로 확대하지 않는다. 사용자 흰 창 관측과 특정 fixture의 동일성도 단정하지 않는다. 기존 AI·DPI·VM 보류와 전체 제품 목표 미완료 상태를 유지한다.
