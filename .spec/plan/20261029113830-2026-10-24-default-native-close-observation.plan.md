---
id: plan:kinekino/20261029113830
title: N 첫 close 실패의 최소 관측 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# N 첫 close 실패의 최소 관측 제안

Status: active · 계획/private 소스 검토 단계. 실행 슬롯은 반환7559366D, 원 실패는 [SOT2F742932](../../../sots/verification/presentation/default-native-nc-first-2026-10-24.md)에 보존했다. 아래 후속 승인 private 후보 외 공개 코드·예산 증가·Close/Run/C·새 host/API 구현0이다.

후속 승인 범위는 Meari existing runner.rs/window.rs의 opt-in coarse 관측 exact2와 SDK default_launch.rs의 누락 durable parent witness exact1뿐이다. 아래 port.close 상세/제품 viewer_host는 후속 후보로 보류한다. SDK는 private `runtime/development/default-native-window-witness-r2-buffered/` EDEF73FF(변경1/비수정7)를 작성했고 leaf fmt만0, Cargo/type/Run0이다. 일반 off 동작을 유지하고 on일 때만 버퍼링→원 lifetime Outcome 구성 후 기록하므로 writer 오류는 Close나 first error를 바꾸지 않는다. 자세한 핀/한계는 해당 HANDOFF에 둔다.

## Outcome / Authority

다음 **별도 승인 진단**에서 WM_CLOSE 선택/게시 → CloseRequested 수신 → host close 진입/진행 중 어느 경계까지 도달했는지 최소 기록으로 구분한다. SDK는 gate/제품 host, Meari 담당은 library runner/window를 소유한다. opt-in 수집 sink의 연결 방식은 Meari 담당 검토 후 고정하며 추가 path/API가 필요하면 먼저 정확 제안한다. 이 문서는 구현·재실행 승인이 아니다.

## Evidence / Unverified

확인된 사실은 원 N Deadline, close_attempted=true, product+conhost Active(2), raw0bytes, 같은 product creation의 Meari Native 창/응답 가능, PM Abort1이다. 원 선택 HWND 본문·callback 수신·첫 close status/마지막 progress는 없다. Winit의 비Redraw event_buffer 때문에 `driver.take()`만 보고 동기 유실 원인으로 단정할 수 없다. visible이 ready/checked-frame보다 앞서는 경계도 원인 확정 근거가 아니다. LocalHost/ExecutionOwner 정체 역시 미확정이다.

## Phases — 후보 exact4

| 정확 경로 / 소유 | 기존 지점에 제안하는 관측 |
| --- | --- |
| tests/native_product_launch/default_launch.rs / SDK | 기존 window.evidence의 HWND/PID/thread/class/title/visible/top-level을 최초 선택 때 durable 보존. 원 close 호출 전/반환 결과를 기록하되 새 조회/WM_CLOSE0 |
| packages/meari/packages/meari-native/src/infrastructure/window/startup/runner.rs / Meari | 기존 window_event 진입의 CloseRequested/WindowId·driver 존재·startup/attach 상태. queue 전달/미수신을 driver 상태와 구분하며 별도 event replay0 |
| packages/meari/packages/meari-native/src/infrastructure/window.rs / Meari | 이번 승인: 기존 event/window coarse scalar 관측. 첫 port.close HostCloseStatus 등 상세는 이번 구현에서 보류. 기존 schedule/queue/retry 동작 불변 |
| src/assembly/meari/viewer_host/mod.rs / 제품 SDK | **이번 보류:** 기존 close/poll_unrun_shutdown 반환·last_close_progress는 첫 계측 뒤 진입 후 정체가 확인될 때만 검토. Callback/Endpoint/Execution/Card/Jobs Pending과 Complete를 새 poll 없이 읽을 후속 후보 |

PM/Meari 검토 뒤 위 관측의 opt-in sink만 연결한다. N argv0·원30/5·case1·owner/guard·Close 재시도 금지는 유지한다. 기존 private raw/evidence를 사용하고 새 worker/전역 파일/renderer capture/공용 SDK API를 만들지 않는다. 최초 각 경계1개+progress 변화 최대16개·총16KiB 이하와 overflow 표시를 후보로 두며, 동기 FS flush를 이벤트마다 추가하거나 원 callback 오류/반환값을 바꾸지 않는다. 마지막 기록은 **마지막 관측값**이지 종료 직전 상태를 보장하지 않는다. 제한/누락/IO 오류는 진단 미입증으로 남긴다.

판정은 수신 기록 없음→미수신/계측누락 구분 보류, 수신+host mapping/close 진입 있음→첫 status와 마지막 progress에서 다음 원인을 조사하는 정도다. 둘 다 원인 확정이나 정책 수리 근거로 자동 승격하지 않는다. 아직 latency/실행은 not measured다.

## Rollback

제안 단계 source 변경0이다. 승인 구현 시 기존4 before/raw/delta/pin을 분리하고 비관측 정책·원 실패·known image/host는 보존한다. 어떤 관측도 새 Close/driver replay/무조건 shutdown/예산 증가를 정당화하지 않는다. 진단 실패는 최초 case 오류를 대체하지 않으며 원 owner는 기존 retained/명시 정리 계약을 따른다.

modoo.today plan-authoring31459a74/참조091d2f8d·fixture a393f15f의 확인/미검증·원 자원/수명 분리를 적용했다. 독립 조사와 기존 source getter/반환 경로를 읽었고 확정 결함을 새로 주장하지 않았다.
