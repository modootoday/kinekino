---
id: plan:kinekino/20261029113758
title: 제품 NativeWindowHost bridge r2 gate 완료
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 NativeWindowHost bridge r2 gate 완료

2026-10-20 KST 확정. [실행 정본](../../../sots/verification/presentation/native-window-bridge-2026-10-19.md)에 신규 bridge·factory·panel과 기존 R3/CardHL 회귀를 실제 SDK/store/NativeHostRuntime으로 인수했다. OS 창·root 등록 완료는 아니다.

| 항목 | 결과 |
|---|---|
| 초기 선택 | 75719ms101·15pass/2fail, panel 미실행. 초기 factory와 bridge 별도 budget의 불일치 |
| 최소 수리 | 실제 factory PanelBudget 하나를 private Mounted→Parts→Bridge로 이전. 별도 bridge budget 인자 제거·assert 완화0 |
| 수리 후 선택 | --no-fail-fast19pass/0fail/0ignored·23050ms0 |
| 최종 전체 | 재투영 뒤 예산 불변 검사 포함46pass/0fail/0ignored·119312ms0 |
| strict/fmt | 21757ms0 / 310ms0, 동일 최종 source·lint 수리0 |
| 입력 경계 | SDK API96a9…e51a. 외부 SDK/Meari delta0·manifest0, 소유 프로세스[]·슬롯 반환 |

근거는 runtime/quality-review/player-ui-native-window-bridge-r2/{gate-r1,retry-budget}다. 원 실패·before source3개·수리 diff·최종 source5/pins를 보존했다. final-owned inventory4b2c97…9c50과 준비 r2 c609a2…9b4f를 구별한다. r1 원본과 준비 단계의 [과거 문서](history/native-window-bridge-preparation-2026-10-19.md)도 보존한다.

## 인수와 남은 통합

단건 영구 거부/누적 TextCapacity, 실제 shared registry 포화·검증·terminal 후 명시 Completion 및 실제 SDK 호출 counter, 동기 재투영 receipt·stale/foreign·typed close/parent 재시도·forced incomplete를 headless로 확인했다. [admission 정본](../../../sots/player/presentation/shared/proposal-native-admission.md)을 따른다. 초기와 후속 큐 예산·registry 한도는 factory가 실제 사용한 값 하나를 이전한다.

다음은 SDK concrete parent lease/assembly·실제 명시 선택을 제품 root에 연결하고 NativeWindowDriver와 backend terminal wake·사용자 종료를 한 창 수직으로 인수하는 것이다. 기존 eframe loop/공유 GL import는 해결된 것으로 표시하지 않는다. 새 GUI/cargo는 별도 중앙 슬롯에서 배정한다.

fresh CLI/project/provenance, 전체13화면과 시각1:1·폰트/자산·가상 모니터, 커서·오디오/실제 청취·seek stall 원인은 각각 별도 후속이며 이번 결과를 그 완료로 확대하지 않는다.
