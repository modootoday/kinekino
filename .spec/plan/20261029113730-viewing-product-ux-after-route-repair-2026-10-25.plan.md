---
id: plan:kinekino/20261029113730
title: 원 review 전환 수리 뒤 Viewing 일상 UX 최소 후계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 review 전환 수리 뒤 Viewing 일상 UX 최소 후계

현재 first-layout 실제 GUI는 기술용 μs·숫자 음량·수신 처리/조회 버튼과 상세 SDK 표시가 많아 승인된 최종 플레이어 UX와 거리가 있다. 기능 연결/71개/실미디어 성공을 시각·사용성 인수 완료로 승격하지 않는다. 아래 표는 초기 후속 설계이며, 최신 별도 구현 후보는 [일상 transport 계약](../../sots/player/presentation/viewing/everyday-transport-contract.md)의 source16 ECE5632F894D03F3361BA2CB2D8C12B05EC546AB549302A328DEFB51B30BBCBE다. 명시 고급 접기·사람용 시간/음량·공통 아이콘 연결과 회귀 소스가 준비됐으나 rustfmt/check 외 타입·시험·GUI는 미실행이다. 원 진단/선임 gate와 native retirement 수리는 별도 핀으로 보존한다.

우선순위는 정상 실제9 open→원 consumer 정리→새 inspection/mount/retained proof/frame 활성화 결함을 먼저 수리·검증하는 것이다. FocusChanged와 route 실제 정지 단계를 혼동하지 않는다. bounded private phase 진단은 원 policy/result를 바꾸지 않으며 해결 증거가 아니다.

| 후속 책임 | 정확 소유 경로 | 최소 변경과 gate |
|---|---|---|
| 일상 transport | src/presentation/web_ui/viewing/native-transport-bar.tsx, read-panel.tsx | 재생/일시정지·이전/다음·현재/전체 시간·위치·음량·회차·전체화면을 일상 조작으로 묶는다. 원 SDK callback/admission을 유지하고 disabled 이유를 간결하게 표시한다. 진단 도구 나열을 일상 transport로 인수하지 않는다. |
| 기술 상태 접기 | read-panel.tsx, src/assembly/meari/viewer_read_actions/mod.rs, viewer_callbacks/lifecycle.rs, viewer_read_projection/mod.rs, player_shell/document.rs | 기존 조회·ACK 복구·상세 오류·μs/퍼센트 수동 입력은 고급 상태 영역으로 명시 접기. 기존 controlled local intent와 typed props로 구현하며 임의 details/새 browser DOM 기능 지원을 가정하지 않는다. 내부 continuation8은 끝까지 원 owner가 보유한다. |
| 자동 진행·오류 | viewer_host/automatic_progress.rs, viewer_callbacks/{progression,catalog,episode_transition}, viewer_read_projection/mod.rs | 원 bounded host timer/동일 ticket acceptance/ACK/불확정 재제출 금지 그대로 사용. 기술 poll/read/inspect/commit 단계를 사용자가 누르게 하지 않는다. 정상 상태/대기/실패·취소·복구 필요를 구분하고 일부 응답을 전체 성공으로 표시하지 않는다. |
| 시간 표시 | viewer_read_projection/native_playback.rs 및 presentation 시간 표시 전용 모듈 | 공통 SDK의 원 position/duration μs를 presentation Rust에서 시:분:초로 포맷한다. 미관측/연결 해제는 00:00 성공값으로 만들지 않는다. 긴 영상의 hour 누락·overflow·끝점 반올림을 회귀한다. 값 객체/SDK 단위 정책은 변경하지 않는다. |
| seek 범위 입력 | native-transport-bar.tsx, viewer_read_actions, viewer_callbacks/native_playback.rs, viewer_read_projection/native_playback.rs | 실제 duration/connected/원 instance 및 pending 상태로 활성화한다. drag draft는 로컬 값이고 원 committed observation과 별개다. 확정 입력만 playback.seek로 연결하며 move마다 무제한 SDK 제출하지 않는다. cancel/route/instance 변경은 드래그 초안을 폐기하고 늦은 확정은 거부한다. |
| 음량 범위 입력 | 위 transport/native adapter | 표시 draft와 원 volume observation 구별. 100 기본 초안은 실제 관측이 아니다. 범위/키보드/commit/cancel은 검증된 Meari 계약만 사용하고 SDK volume validator/오류를 우회하지 않는다. |
| 공통 디자인 토큰 | src/presentation/web_ui/styles/{tokens,components,viewing}.css | 기존 spacing/font/color/border/radius/control/transport/volume 토큰 사용. hover/focus/disabled/selected 간격과 명칭을 공통 컴포넌트로 맞춘다. footer clipping 때문에 실제 버튼이 잘리지 않도록 실제 좁은 폭/긴 번역을 검토한다. |

## Meari range 재사용 판단

기존 range 후계의 정확 source pin·전임·typed value/change/commit/cancel·pointer/keyboard/capture/focus/instance 교체·controlled replay 지원과 실제 검증 수준을 Meari 담당에게 문의했다. 회신 전에는 range54 같은 과거 미선택 패킷을 현재 소스에 합성하지 않는다. 새 대형 API/별도 JS 정책·검증 도구도 만들지 않는다. 현재 native μs/volume Text callback을 range 지원으로 표시하지 않는다. 재사용 가능 시 제품 어댑터와 검증된 최소 dependency closure만 PM이 중앙 합성한다.

## 수리 후 일괄 검증

공통 SDK/CLI 기능은 기존 gate로 검증하고 GUI는 표시·입력·focus·키보드·캡처·정상 native effect 정합성만 본다. 원 Root/source graph, ACK/late instance, host timer, live-entry retained owner, 실제 원본/후보 표시를 우선 유지한다. range가 검증되면 draft/commit/cancel/원 observation·duration 변화·late input·route close tests를 Rust로 추가한다. 마지막에 실제 승인 시안 확대 비교와 같은 정상 창의 전체 layout/반응형·고급 접기·시간·입력·오류를 묶어 검토한다. 사용자 AI 품질 판정/실제9 수정·저장은 개발 시험이 아니다.

미완료 범위: 최소화/최대화/닫기 제품 chrome 비연결, 실제 target hover 상세, range/슬라이더 UX, 시안 일치, 실제9 원문/후보 비교 GUI와 사용자 승인. 시간 부족을 이유로 목표를 축소해 완료라고 표시하지 않는다.

기존 적용 modoo.today defect-triage-evidence/test-fixture-and-mock-build에 따라 원 실패·미지원·미검증·실제 승인 경계를 유지한다. 새 스킬/새 구현의 적용이나 실행 성공을 주장하지 않는다.
