---
id: plan:kinekino/20261029113856
title: 원문 비교·수정·별도 채택·저장 재열기 — 기본 제품 경로 잔여
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원문 비교·수정·별도 채택·저장 재열기 — 기본 제품 경로 잔여

## 현재 기준

2026-10-24 root c9c48d20abddb110d37a810c08c3520c10e955fc / Meari f85a68df7a6a8fee239d9655895ee15d5eaa5510의 읽기 감사다. PM은 A14, D5, launcher7, B composite3를 채택했다. 아래 actual은 각 검증 정본의 고정 binary/입력에 한정하며 이번 문서 갱신에서 Cargo/OS/수동 클릭/모델 실행0이다. SDK가 fileguard 실증 후 정상 기본창 N/C를 우선하며 UI는 새 기능을 작성하지 않는다.

이 문서의 2026-10-23판에 있던 “무인자는 항상 Player/eframe”, “NativeViewer에는 교정·하이라이트 진입이 없다”는 당시 설명이다. 현재 기본 모드와 기본 chooser의 source 경로를 설명하는 사실로 사용하지 않는다. 기존 구현을 다시 만드는 작업도 제안하지 않는다.

## 현재 조립과 입증 경계

`src/application/launch.rs::desktop_launch_plan`은 native feature가 있는 일반 무인자 실행을 NativeViewer로 선택한다. 명시 browser/validation placement, preview, NativeProposal 및 feature 없는 빌드의 구분은 기존 typed plan에 남아 있다. main은 명시 adapter로 구성된 하나의 client/LocalHost를 `product_viewer_launch::run_product_viewer_with_project`에 전달한다. root·project 미지정은 영속 기본 store 선택이나 자동 scan/첫 record 선택을 뜻하지 않는다.

project=None도 `viewer_native_launch::prepare_product_viewer_launch`의 원 ProductFrame/choicesNone→default chooser를 사용한다. chooser의 명시 list/document/item/Open은 원 SDK inspection/ACK와 같은 host의 `live_entry`로 이어진다. target route는 Card=`studio.cards`, Highlight=`studio.shorts.recommendations`, Cue/DocumentCorrection=`studio.subtitles.correction`이다. 기존 bootstrap/factory/공통 품질 패널·controller를 소비하며 별도 SDK나 Native loop를 생성하지 않는다. source VIEW 복귀는 `return_viewing`과 원 retained proof/새 Viewing frame·fresh query/key를 따른다.

| 근거 | 이미 입증한 범위 | 이 근거로 확대할 수 없는 범위 |
| --- | --- | --- |
| launcher7 | feature별 mode/project typed plan과 headless caller·원 동일 SDK 전달 | 실제 noargs 제품 process/정상 창·렌더·기능 동등성 |
| 기본창 pure17 | 원 owner/error/log·순수 pin/admission 계약, 실제 등록17 통과 | 실제 fileguard/pipe/N·C/WM_CLOSE/worker 종료. 현재 SDK 후속 소유 |
| D 실제 source3 | default choicesNone, 세 작업의 명시 chooser→현재 항목→실제 entry frame→VIEW→새 frame/chooser→다른 문서 항목 | 일반 player.exe 무인자 실행·물리 pointer/IME·Save/Accept 기능 검사·미디어 |
| A 표시12 | 공통 revision/result/error DTO, 세 task 표시 연결과 typed bridge·영향 회귀 | 실제 SDK 실패를 일으킨 증거·실제 renderer/OS 입력·사람 승인 |
| B3 + D Card1 | 원 persisted Card draft2, Highlight/DocumentCorrection draft3/output3를 같은 proposal로 두 차례 source 재열기. read-only list/inspect6, 실제 editor/ref/revision/result 표시, 원 cleanup | 이번 UI에서 수정/저장/채택을 발생시킨 여정, 동시 writer·실시간2→3 전환, 모델 품질·사람 결정 |
| 이전 C21/명시 NativeProposal | 고정된 세 패널 원본 DOM/controlled input·저장/명시 채택 및 controller/공통 품질 계약의 과거 실제 근거 | 최신 기본 셸/noargs·미디어 결합으로 소급. 다른 binary의 성공 수 합산 |

D의 Cue→DocumentCorrection 두 번째 진입은 correction source/ref 선택 연결의 근거다. 자동 복구 모드 `DocumentRepair`를 기본 live entry에서 지원한다는 뜻은 아니다. `TrustedLiveChoices::from_trusted_local`은 해당 target을 명시 거부한다. 복구 UI 전체를 새로 만들거나 guard를 풀지 않는다.

## SDK 지원과 GUI 고유 관측

| 잔여 항목 | 재사용할 공통 SDK/CLI 계약 | GUI에 남은 정확한 관측·조립 범위 |
| --- | --- | --- |
| 원문 비교와 수정 | Cards front/back/source_refs, Highlights title/source_refs, Cue/DocumentCorrection의 typed source_ref·draft와 원 edit/CAS/accept/receipt. SDK·CLI 실제 수입/기술 edit/accept/restart 기록 및 이전 GUI 근거가 있음 | 최신 기본 ProductFrame에서 사용자가 선택한 동일 source/ref/editor에 실제 입력이 도달하는가. D/B는 현재 선택/표시를 입증하지만 세 task 전체의 물리 편집 입력은 미검증 |
| 별도 저장·별도 채택 | 기존 edit와 accept 유스케이스/expected revision·receipt·ACK/취소 정책을 유지 | 원 Text/Action callback이 같은 worker로 연결되고 pending/dirty/confirmed/error를 현재 문서에 표시하는가. B의 accepted leaf는 원 scripted accept이며 이번 UI 클릭이나 사람 승인이 아님 |
| 저장 재열기 | 원 persisted revision/current inspect/acceptance receipt와 read-only SDK 지원 | B는 선택한 고정 상태의 같은 proposal 재열기 완료. 기본 제품에서 직전에 편집한 세션을 닫고 이후 실행으로 재진입하는 연속 여정·동시 revision 변경을 추가 입증한 것은 아님 |
| 일반 noargs | typed NativeViewer 기본 선택·원 main/동일 SDK 조립 구현과 headless 근거 있음 | 실제 기본창 정상 N/C·정확 PID/HWND/원 종료·SDK join은 SDK 담당 다음 순서. root 미지정 상태는 자료 없는 상태이며 UI가 임의 root를 생성하지 않음 |
| 실제 미디어 결합 | 재생·저장·변환·자막 기능은 각 공통 SDK/CLI 지원표와 실제 미디어 근거를 그대로 사용 | 동일 native video child/재생 수명과 chooser/editor/route 복귀의 동시 호환. NoMedia B/D는 실제 mpv/video/PTS/seek/자막 적용을 입증하지 않음 |
| 오류·재시도 | 기존 typed SDK 오류·CAS 충돌·취소와 원 owner/ACK/cleanup. A는 오류 입력에 대한 표시 계약, 기존 scoped bridge에는 rejected commit owner 보존 시험이 있음 | 최신 기본 셸에서 실제 실패가 현재 editor/first error를 보존하고 허용된 명시 동작으로 다시 진행되는가. 내부 HostRetry/cleanup retry는 사용자용 기능 재시도 성공이 아님. 무조건 자동 재시도 정책을 추가하지 않음 |

실제 동영상 구간 자르기·미지원 모델 ranges/reason·PTS·SRT 출력·영상 자막 적용은 proposal title/ref/acceptance receipt와 별도 계약이다. SDK 미지원 항목을 GUI adapter로 지원 표시하지 않는다. 품질 판정 capability/Deferred 테스트도 사람 품질 승인이나 모델 품질 성공으로 바꾸지 않는다.

## 다음 최소 제안과 소유권

후속 읽기·정확 입력 제안은 [기본 chooser 입력 seam](default-chooser-input-seam-proposal.md)에 분리했다. SDK N/C는 실제 실행의 선행 인수이며 UI source 읽기·계획 작성은 비충돌 범위에서 병렬로 준비한다. 후보 입력4와 조건부 entry 복구1, 아직 SDK 오류 seam이 확인되지 않은 범위를 구별한다.

1. **SDK N/C가 선행이다.** SDK는 `tests/native_product_launch/default_launch.rs` 및 그 control/window/observation/owner/file_guard 모듈을 소유한다. 원 고정 BIN/byte pin·fileguard 실증 뒤 실제 normal noargs N과 명시 root C의 원 owner/창/종료를 검증한다. UI가 이 경로에 별도 launcher/프로세스 제어를 만들거나 같은 N/C 검사를 반복하지 않는다. SDK 반환 정본을 인수하기 전 baseline/자원을 가정하지 않는다.
2. **그다음 UI source 입력의 좁은 연결 관측을 제안한다.** 후보 경로는 기존 `src/assembly/meari/viewer_native_launch/no_media/tests/end_to_end/live_retained.rs`와 `src/assembly/meari/viewer_native_probe/live_retained.rs`다. 먼저 기존 입력 fixture와 C21 coverage를 대조해 빠진 task/input/ref/표시/route 항목만 정확 명단으로 제안한다. 원 renderer Text callback/현재 scope·same client/worker·별도 Save/Accept Action의 연결과 표시·focus를 검사한다. SDK CAS/파일 publication/accept 기능 검사를 다시 구현하거나 test 기대값을 느슨하게 하지 않는다. production 조립 변경은 아직 필요하다고 판정하지 않았다.
3. **관측으로 실제 누락이 확인될 때만 exact production 경로를 제안한다.** 현재 조사 대상은 UI 소유 `viewer_host/live_entry.rs`, `viewer_host/viewing_chooser.rs`, `viewer_host/return_viewing.rs`와 `presentation/web_ui_adapter/proposals/window_bridge.rs`다. 이것은 승인 source 목록이 아니다. 기존 controller/bridge 재사용이 원칙이며 수정 필요 호출·최초 오류·정확 경로를 먼저 PM/SDK와 확인한다. application launch/domain/usecase 정책은 SDK 소유다.
4. **미디어 결합은 별도 후속이다.** UI의 `src/assembly/meari/viewer_native_launch.rs`와 `product_viewer_launch.rs`는 입력 소유권 조립 후보이고 Media/SDK가 native child/playback adapter와 실제 미디어 fixture를 소유한다. 동일 child/SDK/host 반환·재생 수명·route 전환 계약을 조율한 뒤 최소 한 지원 미디어의 GUI 고유 결합 관측만 제안한다. media 처리 기능 인수·benchmark·자원 사용은 Media/SDK 근거를 재사용한다. 현재 새 child/worker/기능 코드 작성0이다.

우선순위는 SDK fileguard→정상 기본창 N/C→UI 잔여 입력/표시 연결 제안→별도 배정된 미디어 결합이다. SDK의 기본창 테스트 소유권과 UI의 NoMedia fixture는 경로가 다르지만 같은 원 launcher/owner·고정 bin·공유 S/T 및 실행 lease에 의존하므로 동시 heavy는 금지다. 이후 SDK source 채택 때 fresh baseline을 다시 인수한다. 수동 클릭·스크린샷·시각 인수·새 GUI 기능 확장·모델 실행은 이번 제안의 선행 gate도 배정 범위도 아니다.

## 근거와 적용 판단

- [기본 launcher 실제 검증](../../../sots/verification/presentation/product-gui-launch-2026-10-24.md), [기본창 pure17](../../../sots/verification/presentation/default-native-window-pure17-2026-10-24.md), [SDK 기본창 후속 계획](default-native-window-reachability.md).
- [D 실제 source 여정](../../../sots/verification/presentation/default-chooser-journey-d-2026-10-24.md), [A 공통 표시](../../../sots/verification/presentation/three-task-review-feedback-a-2026-10-24.md), [B persisted 재열기](../../../sots/verification/presentation/persisted-review-reopen-b4-2026-10-24.md).
- [SDK 제안 CLI/저장](../../../sots/verification/application/proposals-2026-10-19.md), [공통 proposal API](../../../sots/player/application/api/proposals.md), [C21/품질 표시 과거 근거](../edited-output-quality-decision.md), [명시 NativeProposal 실행](../../../sots/player/application/native-proposal-execution.md).

같은 세션에서 본문을 읽은 modoo.today `modootoday-free/design-delivery-plus/frontend-first-journey`의 원 공통 컴포넌트/typed port 재사용과 실제 reachable/미입증 구분, `modootoday/software-qa/test-fixture-and-mock-build`의 원 owner·고정 fixture provenance·시점별 검증 분리를 적용했다. 이번 변경은 이 기존 계획 정본의 stale 설명 교정과 후속 제안뿐이며 제품·테스트 코드를 작성하거나 실행하지 않았다.
