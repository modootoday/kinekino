---
id: evidence:kinekino/20261029114057
title: "r4 정상 제품 GUI 인수: 소스 연결과 실행 증거"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# r4 정상 제품 GUI 인수: 소스 연결과 실행 증거

기준: `runtime/development/product-review-final251-r4/coherent251-supply.json`, PM/SDK 전달 pin C55F9511. 이 문서는 해당 선택 소스와 선택되지 않은 canonical 보완 소스를 읽은 기록이다. 현재 소스를 수정하거나 Cargo·GUI·OS·사용자 저장소 쓰기를 실행하지 않았다. 최종 전체 SHA/실행파일은 중앙 인계 정본을 따른다. 그래프 등록, CPU scene, CLI 결과, 원 OS Ready와 실제 사용자 품질 승인을 구분한다.

중앙 전달 관측: r4 Root/Paint/Native joint fmt/type/all-targets strict clippy exit0. 영향20 테스트는 이 문서 작성 시 진행 중이며 통과를 선계상하지 않는다. 과거 원251 자동UI3·Native16 통과는 각 원 입력의 결과다. 원251 raw/Root/start 그래프 실패, Paint malformed fixture 실패는 후계가 있다고 원기록에서 지우지 않는다.

## 실제 연결표

| 사용자 항목 | 현재 선택 소스 경로 | 판정과 인수 조건 |
|---|---|---|
| 재생·일시정지 | ViewingReadRoot → NativeTransportBar onPlay/onPause → onNativePlay/onNativePause → viewer_callbacks/native_playback.rs CONTROLS → typed playback.play/playback.pause → codec SdkPort → 원 ExecutionAttachment | 소스 연결 있음. 원 엔진 관측/admission/미수락 ticket 조건으로 비활성화한다. 재생 중 실제 영상·소리·pause 상태 반영은 정상 player에서 별도 관측 필요. |
| 위치 탐색 | native-seek-position 문자열 입력 → onNativeSeekDraft(로컬 입력) → 이동 버튼 onNativeSeek → DecimalU64 position_us 파싱 → NativePlaybackProjection.control → playback.seek 공통 SDK | μs 텍스트 입력+명시 적용만 있다. 디자인 시안의 드래그 가능한 진행 막대·프리뷰를 구현 완료로 표시할 수 없다. 범위·엔진 정책은 SDK 소유. 실제 탐색 결과/오류는 미검증. |
| 음량 | native-volume-value 입력 → onNativeVolumeDraft → onNativeVolume → SetVolume/typed playback.set_volume → SDK; 관측 volumePercent 별도 | 시작 음량 기본100은 사용자 입력 초안이며 관측값이 아니다. 슬라이더/음소거 토글 구현 없음. 실제 적용·오류·관측 갱신은 GUI 미검증. |
| 이전/다음 | EpisodeNavigation → onPreparePrevious/onPrepareNext → episode_transition callback → playback.previous/next → 원 submitted transition/job → automatic_step observe/jobs.read/commit → 새 원 엔진 관측 | SDK 인접 회차 정책을 사용하고 UI가 URL/인덱스를 계산하지 않는다. 취소 latch는 자동 commit 금지, 불확정 begin/commit 재전송 없음. 소스·headless 근거와 실제 회차 교체를 구분. |
| 회차 목록 | ViewingReadRoot aside 안 CatalogBrowser RadioGroup → onCatalogAiring/Completed/Movies/Series/Episode → 원 catalog SDK/자동 inspect/open | 상시 표시 영역이며 열기/닫기 토글 패널이 아니다. 재생 중 임의 회차 교체 atomic API가 없어 해당 회차 선택은 비활성화한다. 이전/다음 지원을 임의 선택 지원으로 확대하지 않는다. |
| 전체화면 | 제품 NativeTransportBar에 callback/버튼 없음. canonical components/transport-bar.tsx 아이콘은 disabled=true. Meari window/actions.rs에는 WindowAction::SetFullscreen → winit window.set_fullscreen(Borderless) 구현 존재 | **제품 연결 누락. CSS boolean 전체화면도 없음.** 라이브러리 기능 존재를 제품 창 효과 지원으로 계상하지 않는다. |
| 컨트롤 자동 숨김 | NativeTransportBar는 항상 렌더됨. 선택 Rust/TSX에서 자동 숨김 상태·활동 시간·포인터 입력·관련 callback 참조 없음 | **누락.** 자동 진행용 NativeHostTimer는 카탈로그/회차 continuation용이며 컨트롤 숨김 타이머가 아니다. |
| hover·커서 복원 | 제품 측 cursor visibility/window-effect 소비 및 hover/idle 상태 전이 없음 | **누락/실제 창 미검증.** 구현이 없는 상태를 hover 커서 버그 수정으로 주장할 수 없다. |
| AI 원본/후보 읽기 | 원 ManualRepair rawReviewOnOpenPackage/OpenItem/ReadSource/ReadCandidate → 원 raw_review_state/NodeLease queue → reviews.inspect/read_text_asset 공통 SDK | 원 package SHA와 item 코드를 명시 입력한다. proposal ID에서 package를 추측하지 않는다. 실제9 CLI 읽기 성공은 GUI 표시 성공이 아니다. |
| AI 수정·저장·채택 | ManualRepair onSourceChoice/onStartManual/onDraftChange 등 → repair_panel ReviewAction → 원 proposal session/use cases → onSave/onAccept | 빈 수동 수정 버퍼·원 CAS/저장·채택 정책 유지. 정상 제품 GUI 표시/입력·오류 상태는 미검증. 개발자가 실제9 수정·채택하지 않는다. |
| AI 명시적 원결과 판정·이력 | ManualRepair raw verdict 선택+메모+onRecord → reviews.record_decision, onHistory → reviews.decisions | 원결과 판정과 채택 출력 판정은 별개. 저장/채택/CLI 성공을 사용자 품질 승인으로 변환하지 않는다. 실제 사용자만 판정한다. |
| 저장 후 typed 재개방 | Native12 등록 + FrameComposition owned ReviewWorkspace(editor, RawReviewPanel) → package/item 재입력 → 원 read/history callback | raw11+typefix+graphfix 후계의 소스 연결. sibling은 **읽기/이력 전용**이며 판정 기록 버튼 없음. 초기 ManualRepair 판정 write와 구분. 실제 재개방/이력 GUI 인수 미완료. |

## 구체 소스 책임

- 표시/입력: `src/presentation/web_ui/viewing/{read-panel,native-transport-bar,episode-navigation,catalog-browser,player-frame}.tsx`, 공통 `components/transport-bar.tsx`는 별도 비연결 placeholder다. 원 video semantic slot은 native surface 대상이며 HTML video 자체의 재생 구현이 아니다.
- typed callback/SDK: `src/assembly/meari/viewer_callbacks/{native_playback,codec}.rs`, `episode_transition/{callbacks,codec,automatic,state}.rs`, `viewer_read_projection/{mod,native_playback}.rs`, `async_sdk_port.rs`. UI는 capability/admission 결과를 표시하고 기능 정책을 복제하지 않는다.
- 실제 창 효과 후속 소유: 제품 UI 담당은 NativeWindowActions를 소비하는 얇은 presentation/host 어댑터·명시 callback·오류/관측 표시를, Meari 담당은 기존 window action 계약/수명·native 적용을 소유한다. fullscreen 요청/실제 상태/거부/복원은 구분해야 한다. 플랫폼 API를 TSX나 presentation에 직접 호출하지 않는다.
- 자동 숨김/커서 후속 소유: 제품 presentation에 활동·포커스·컨트롤 hover·창 이탈/복귀 상태 모델과 원 host timer 수명을 설계하고 native 효과 계약의 실제 지원을 확인해야 한다. OS cursor 복원 API를 현재 지원한다고 가정하지 않는다. 드래그·키보드 입력 중 숨김 방지, pointer 진입/이동·포커스 복귀 시 표시, 닫기·전체화면 해제 시 커서 복원을 시험한다. 이번 범위에서 새 구현/큰 range 플러그인 추가 없음.
- 실제 누락 항목은 최종 완료 판정에서 제외한다. 사용자의 전체 시안 일치 요구가 해결되었다고 표시하지 않는다.

## 정상 player 확인 순서

1. 중앙 SDK가 공급한 **동일 선택 소스 정상 player.exe 절대경로·전체 SHA·원 producer/source pin·필수 자산 custody·최신 보호/복구 기록**을 받는다. 예전 player/libtest/preview로 대체하지 않는다. 중복 player를 띄우지 않으며 실제 실행/창 배치는 PM 자원 배정에 따른다. 이번 정상 사용자 인수는 사용자에게 보이는 주 모니터의 기존 창 기본 배치를 우선하며 DPI·가상 장치 설정 변경은 하지 않는다. 가상 모니터가 명시된 별도 recipe는 해당 범위만 따르며 이번 인수의 선행 조건으로 승격하지 않는다.
2. Media `runtime/coordination/ai-review-input-r1/NORMAL-PLAYER-AI-REVIEW-INPUT.md`와 `USER-REVIEW-HANDOFF.md`의 원 argv/저장소를 사용한다. 정상 바이너리 placeholder를 임의로 채우거나 미지원 자동 선택 flag를 만들지 않는다. native-viewer/record-validation과 원 jobs/catalog/browser-profile/review/proposal root 분리를 유지한다. 이 문서 작성 중 실행·디렉터리 생성·해시 재계산/자산 복사 없음.
3. 먼저 원 Native 창/video slot/엔진 연결의 실제 결과·오류를 관측한다. Ready/cleanup만으로 미디어 표시를 승인하지 않는다. 실패하면 원 오류/로그/창 상태를 보존하고 재생 제어를 성공으로 건너뛰지 않는다.
4. 카테고리→작품→회차를 명시 선택해 자동 준비 진행/오류·취소 표시를 확인한다. 엔진 연결 후 재생/일시정지, 유효 μs 위치 입력+이동, 음량 입력+적용을 각각 확인한다. 공통 SDK/CLI 기능 시험을 GUI에서 반복하지 않고 입력→동일 SDK 연결/관측 표시 정합성만 확인한다. 이전/다음은 원 회차 존재·작업 상태에 따른 활성/거부·새 title/instance 표시를 본다.
5. 회차 목록이 상시 aside이며 fullscreen/auto-hide/cursor restore가 누락된 현재 상태를 기록한다. 없는 버튼/콜백을 다른 GUI 자동화나 OS 키 조작으로 대신해 인수하지 않는다.
6. 같은 정상 player의 원 제안 chooser에서 `actual9-review-r1-gemma270-correction`을 명시 선택한다. 목록/원 document/item inspect를 거쳐 ManualRepair를 연다. 실제 ID가 없으면 연결 오류이며 scratch나 다른 root로 대체하지 않는다.
7. 원 자료 코드 `659b3cb6757259d8e770f6d3c96c4a77dafbab43c6ea540bcc7e0c58e4eb0197`, 후보 `gemma270-correction`을 입력한다. 자료 열기→후보 선택→원 요청/원문 읽기→모델 원결과 읽기 순으로 proposal ID/item/model/기준/내용 표시를 확인한다. 다른8개는 Media 단일HANDOFF를 따른다. range9null·영상 PTS/하이라이트 구간 미검증 유지.
8. 개발 확인은 표시·입력·오류 정합성까지만 진행하고 실제9 판정/수정/저장/채택은 수행하지 않는다. 이후 사용자가 명시적으로 원문 선택→빈 수정본 시작→수정→저장, 원결과 verdict+메모→기록→이력을 확인한다. 채택 출력 품질은 실제 채택 출력이 있을 때 별도다.
9. 저장 후 창을 다시 열 때 typed 패널 옆 read-only 원본/후보 이력 sibling 존재를 확인하고 같은 package/item을 재입력해 읽는다. 자동 연결/자동 판정으로 대체하지 않는다. 사용자 직접 종료 원칙과 원 worker 정리/오류 보존을 따른다.

적용 근거: 기존에 읽은 modoo.today software-qa/defect-triage-evidence 및 software-qa/test-fixture-and-mock-build에 따라 소스 계약/합성 graph/CPU 관측/실창/실미디어/사용자 판정을 분리했다. 새 스킬 이름만을 근거로 실행 성공을 주장하지 않았다.

## r4 뒤 필수 누락 구현 후계 — 아직 미검증

PM 명시 배정으로 `runtime/development/product-viewing-window-effects-r1/source13.json` 759F4279934DC8E51753C80030F04FC6B097420779A997DD92961CEAF1DD989C가 별도 작성됐다. Meari cursor/input source8 08CCDCCB 의존이며, 실제 전체화면 약포트 요청/관측·오류와 회차 로컬 토글, 실제 활동 scope/sequence 및 누적 focus/inside를 사용하는 3초 idle 숨김·입력 복원·종료 정리를 연결한다. 원 r4의 누락 표는 r4 관측 사실로 유지한다. 이 후계는 포맷만 확인됐고 타입/테스트/실창은 0이므로 기존 누락을 검증 완료로 바꾸지 않는다. 초기 focus/enter 미관측과 DOM target hover 부재 제한은 원 HANDOFF에 명시했다.

Media 배포 입력 최신 전달 pin: NORMAL-PLAYER-AI-REVIEW-INPUT 80FBD9DFFF44B1AE237938AF7ABF05A80182B03A666BCDB38D1CD6A453BC4081, USER-REVIEW-HANDOFF 668A329DFB916566CF580D4DD534151FE3B20DF7B21EF69FAF46B0E898AB0C5D. 기존132/211 한계와 새 read-only 재개방 후계를 분리한 문서 갱신이며 정상 exe/SHA/자산 custody 미공급·실 GUI/사용자 승인0은 유지한다.

## 창 효과 통합 후 정상 player 검토 순서

중앙 선택 예정은 제품13-with-clips C56A71F077AC6A6424C90021AAC600786E9A74741E974B2D4D09BDE350CC5E26 + 입력8r2 451BD6369A36C4A10EAC8EA574D7EA61F7BC6BD1E4B2D4F57CA84FE964C28D8C + Unknown paint4r3 348BE20A3405C1484A3E287AD83A75C66BFD158005ECBCB85383ACF7FA166532다. 이 목록은 배정 사실이며 통합 검증 결과가 아니다. 중앙 fmt/type/strict/회귀 결과와 해당 정상 player provenance가 공급된 뒤 아래 창 고유 검토를 한다. 실제 재생 미완료·GUI 승인0은 유지한다.

1. **전체화면 요청**: 엔진 제어와 독립인 정상 Viewing 창에서 전체화면 버튼을 누른다. 원 host local intent가 현재 원 scope로 SetFullscreen을 요청하고 버튼이 pending 동안 비활성화되는지 본다. 요청 수락/Issued만으로 성공을 기록하지 않는다. 현재 label은 관측된 상태를 따르므로 요청 직후 해제 label로 선반영하면 안 된다.
2. **전체화면 관측**: 원 ActionRecord가 Observed이고 원 snapshot borderless_monitor_coverage=true인 상태와 실제 창이 같은 모니터를 채우는지 확인한다. 이 제품은 exclusive video-mode 전환이 아니라 borderless다. 다시 눌러 실제 false/복귀를 확인한다. scope 변경/Unsupported/BackendFailed는 오류 표시 대상이다. 5초 미확정은 자동 재전송 없이 중단하며 실패를 관측 성공으로 바꾸지 않는다. OS 키 조작을 제품 버튼의 성공 증거로 대신하지 않는다.
3. **회차 패널**: 회차 아이콘으로 기존 CatalogBrowser를 닫고 다시 연다. 해당 표시 클래스만 바뀌고 같은 원 카탈로그 선택·진행 중 작업·취소·ticket 소유권이 유지되어야 한다. 닫기를 SDK 작업 취소/리셋으로 해석하지 않는다. 재생 중 임의 회차 교체 미지원 정책은 그대로다.
4. **유휴 숨김**: 실제 전체화면 관측이 있고 원 창이 focused/inside로 실제 관측됐으며 회차 패널이 닫힌 경우에만 마지막 실제 활동 후 3초를 센다. 3초 전 숨김/hover 즉시 커서 소실은 실패다. 포커스·진입 미관측 None은 true로 추정하지 않고 숨김하지 않는다. 정상 창 모드/열린 패널/포커스 이탈/포인터 이탈/다른 route/종료 중에는 숨기지 않는다.
5. **실제 입력 복원**: 숨김 뒤 원 창에서 실제 포인터 이동·버튼·wheel·key 또는 focus/enter 사건을 발생시킨다. library의 cursor 복원과 제품 controls 표시가 반영되는지 본다. 2.9초에 새 입력을 주고 이전 3초 completion이 와도 새 입력 기준 3초 전 다시 숨지 않는지 확인한다. pending hide 뒤 늦게 들어온 실제 입력은 그 hide를 StaleRequest로 무효화해야 한다. synthetic input API/HostTurn Input 가정만으로 성공을 기록하지 않는다.
6. **정리·오류**: 포커스 이탈/route leave/close 시 컨트롤 표시와 커서 복원을 확인한다. route leave는 원 fullscreen 해제를 요청하며 원 SDK 작업 소유권을 폐기하지 않는다. close는 원 timer owner만 취소하고 library가 해당 창에서 숨긴 커서를 최종 복원한다. 닫힌/대체 scope의 늦은 callback·timer는 다른 창에 효과를 주면 안 된다. 사용자 직접 종료 원칙을 유지하며 임의 추가 player 실행은 금지한다.
7. **남은 표현 한계**: 현재 API에는 DOM target/좌표별 hover 정보가 없다. 특정 컨트롤 위에 가만히 머무는 동안 숨김을 계속 막는 상세 동작은 이 소스가 입증하지 않는다. 실제 검토에서 관찰하고 필요 시 정확한 target 관측 계약의 최소 후계를 별도 작성한다. μs 탐색 입력과 음량 입력은 그대로이며 드래그 timeline/slider 구현 완료로 확대하지 않는다.

제품 신규 Rust 테스트 정확 이름과 범위:

| 테스트 | 입증 범위 | 입증하지 않는 범위 |
|---|---|---|
| assembly::meari::viewer_callbacks::tests::window_ui::original_window_intents_preserve_sdk_tickets_and_reject_input_after_close | 원 callback export/queue/drain, local panel toggle, fullscreen request flag, SDK ticket 불변, release 후 늦은 입력 거부 | 실제 native request/OS fullscreen/cursor |
| assembly::meari::viewer_host::window_ui::tests::activity_rearms_idle_deadline_and_old_completion_cannot_hide_early | 실제 코드의 3초 판정 함수와 새 활동 이후 이전 deadline 무효 | 실제 timer delivery/window input |
| assembly::meari::viewer_host::window_ui::tests::close_is_idempotent_cancels_owned_timer_and_prevents_late_intents | 빈 host 모델의 idempotent stop, timer/arm 없음, restore 의도 | 실제 armed timer Drop·실 cursor 복원; 원 timer 회귀와 창 검토가 별도 필요 |
| assembly::meari::viewer_host::window_ui::tests::idle_is_blocked_after_focus_loss_route_leave_close_or_panel_open | 원 can_idle의 route/close/fullscreen/focus/inside/panel 조건 | 실제 focus event가 전달됐다는 증거 |

테스트 목록 등록/실행 결과는 중앙 SDK가 확정한다. 현재 문서 작성자는 신규4를 실행하지 않았다. Meari cursor Applied/실fullscreen Observed 분리, pending hide의 activity sequence 검증, pre-Ready 원 event 보존, scope/close/stale tests와 원 timer11은 별도 의존 회귀다.

### source1448 정상 binary 후속 인수

PM 전달 정적/71개 및 Airing 원 owner 재생 성공 뒤 Media가 정상 player E4B866F874351D79EE964009A45AE2C447B79DAE6F1EBBA17A08008C9D9D728F와 CLI1CE00072의 원 source1448 producer22 metadata/FI를 인수했다. 최신 실행 argv packet BC525908B4C677CBB9CCD45D18C4C1FC484C675DAECA5930BD05DEA4AC832644, USER-REVIEW-HANDOFF 5FCD5F4CB02A0D5AA5D8091C01E3FB903A9547F1ED1B5C5ADE9BACE690697D35를 따른다. 이전 미공급 문구는 그 시점의 관측으로 남긴다. 현재는 정상 exe placeholder가 채워졌으나 normal-product-review-r1의 jobs/catalog/browser-profile 미존재3개에 PM 생성·소유권 배정이 남았다. 정상 GUI 입력/실제9 표시·사용자 품질 승인0은 유지한다. 단일 [후속 계획](../../../plans/player/product-normal-gui-review-after-cli1448-2026-10-25.md)으로 이어간다.

후속 준비 확인: PM이 위 라이브 경로3개를 직접 생성해 정상 GUI 원 owner 전용으로 배정했고 작성자가 존재만 확인했다. 경로 부재 조건은 해소됐다. actual adjacent 반환 뒤 PM의 추가 실행 배정 대기이며, GUI/사용자 승인0은 그대로다.
