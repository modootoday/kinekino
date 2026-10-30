---
id: plan:kinekino/20261029113932
title: NoMedia frame 소비·host 반영 후속 — 설계 후보
status: active
created: 2026-10-29
updated: 2026-10-29
---
# NoMedia frame 소비·host 반영 후속 — 설계 후보

2026-10-21. 생산 코드 배정0이다. source13의 NoMedia factory/proof 구현과 own-window source2 fixture를 생산 기본 Viewer 배선 완료로 해석하지 않는다.

## full-turn gap 정정

Meari `NativeHostRuntime.after_dispatch`는 새 batch에서 `next_turn`을 호출한 뒤 host를 호출한다. committed surface scope는 전체 HostStamp를 비교한다. FrameSubmitted N을 own wake 뒤 Completion N+1에서 checked proof로 재소비하면 Stale이다. 앞선 stage2의 그 호출 순서를 정정한다. pending batch 재사용으로 turn이 증가하지 않을 수 있어도 이를 보장으로 사용하지 않는다.

현재 frame observer에서 원 checkpoint.advance→필요한 with_disabled_child/checked.with_parent 실소비→모든 checked/document/owner borrow 종료→실행 권한 없는 owned 결과 저장→own wake 순서로 한다. 다음 Completion은 결과를 반영하고 old proof/복사 snapshot/Constructed 상태를 새 turn 권한으로 승격하지 않는다. 새 geometry/visible 행동은 새 FrameSubmitted의 fresh proof가 다시 필요하다.

Meari checked_render는 document RefMut를 해제하고 commit read scope를 반환한 후 lifecycle.submitted를 호출한다. ParentBinding의 state/notice/observer take guard도 callback 전에 끝난다. flushing=true는 notice take/retry 재진입을 막지만 normal checked parent read는 막지 않는다는 source412 읽기 확인을 받았다. callback 안 host after_dispatch/try_reproject/driver 재진입은 금지한다. 실제 배선/OS 성공 근거는 아직 없다.

이후 [own-window r5 결과](../../../../runtime/coordination/ui-own-window-r1/revision-r5/HANDOFF.md)에서 별도 Rust fixture의 실제 observer N=1 proof 소비→borrow 종료/own wake→같은 full scope의 실제 Completion N+1=2 결과 반영, 원 hidden child/parent 정상 정리를 확인했다. 앞선 "성공 근거 없음"은 작성 당시 상태이며 **기본 제품 host 배선의** 실행 근거가 아직 없다는 범위는 유지한다. fixture 한 번의 정상 인과를 임의 사용자 입력/지연 전달/오류 branch 전체 보장으로 확대하지 않는다.

## 최소 변경 후보와 책임

| 후보 경로 | 책임·필요 이유 |
|---|---|
| `src/assembly/meari/quarantine_surface/frame_delivery.rs` 신규 | 원 request/registration/checkpoint·deadline/budget에 결속한 bounded frame 소비. 원 window/document/frame 불일치와 지연 전달 거절, 결과는 owned metadata만 |
| `src/assembly/meari/quarantine_surface/frame_delivery/tests.rs` 신규 | N-frame 실제 소비/N+1 owned 반영, stale/reentry/만료/close·budget 미갱신 제어 회귀 |
| `src/assembly/meari/quarantine_surface/mod.rs` | 위 책임 모듈의 최소 crate 경계 등록만 |
| `src/assembly/meari/player_shell_host.rs` | generic C에 맞는 opt-in frame consumer를 원 preparation.surface_slots/children/wake에 한 번 결속. 동일 coordinator 생성·반환, 기본 생성자 동작 유지 |
| `src/assembly/meari/player_shell_host/delivery.rs` | 기존 route observer와 NoMedia consumer를 조합. N-frame에서만 소비하고 borrow 종료 후 own wake. 기존 delivery failure/원 bounded notice 재전달 보존 |
| `src/assembly/meari/viewer_host/mod.rs` | host input에서 초기 Pending의 once-redraw 요청을 적용하고 Completion에서 owned 결과만 투영. 원 SDK callback/ACK/close 정책 유지 |
| `src/assembly/meari/viewer_host/tests.rs` | 기존 source/stamp/ACK/close 회귀와 새 owned 결과 반영/old proof 재소비0 |

이 7개는 opt-in 소비 경계의 최소 후보이며 확정 exact lease가 아니다. generic shell에 임의의 두 번째 shutdown executor나 별도 state policy를 넣지 않는다. typed frame consumer의 bind/consume/owned-result 경계는 원 registrar/coordinator를 입력받으며 bool/raw HWND/복사 snapshot을 권한 입력으로 삼지 않는다.

## 기본 Viewer 전환은 추가 범위

현재 `viewer_cleanup.rs`의 ViewerController는 ShellHostController<ViewerHost,ViewerNoChild>다. 단순 consumer 등록만으로 QuarantineChild를 원 coordinator에 넣을 수 없다. 기본 제품 배선을 진행하려면 `viewer_cleanup.rs`, `viewer_native_launch.rs`의 원 owner carrier/type 조정과 실제 선언 video node·NoMedia 요청 계약을 별도 확정해야 한다. start 화면에 video가 존재한다고 가정하거나 div로 대체하지 않는다. 엔진/visible/media/지원 flag/main 전환도 이 최소 계획에 포함하지 않는다.

요청 수명은 initial registration 시 원 source/window/request/semantic node·원 deadline·관측 한도를 고정한다. Pending/wake/frame 재전달은 같은 예산을 쓰며 신규 요청이나 세대 리셋으로 시간을 연장하지 않는다. 성공 frame 후 next-turn에 보존하는 것은 결과/진단이며 계속 유효한 surface 권한이 아니다. close/source 전환은 기존 child-first cleanup 소유자를 통해 정리하고 Drop/timeout을 Complete로 승격하지 않는다.

## 현재 소스 인계 상태 정정

2026-10-21 opt-in host exact7이 배정되어 r2 동결 및 PM 독립 읽기/Meari 지정 교차 읽기를 마쳤다. 앞선 생산 배정0·미확정 후보 표기는 설계 작성 당시 상태다. 현재 source7 SHA는 `93F142DB818648DADAEA4CEEED35C96885237B0A30BDA98D01B33E674D74804F`, Flow14+Viewer4 작성18/실행0이며 기본 Viewer 전환은 없다. 실제 인계는 [host7 r2](../../../../runtime/coordination/ui-frame-delivery-r1/revision-r2/HANDOFF.md), 공동 검증 준비는 [gate plan](../../../../runtime/coordination/ui-frame-delivery-r1/revision-r2/gate-plan.md)이다.

원 registrar/coordinator/preparation과 실패 carrier를 유지하는 typed opt-in이며 생산 설치 호출0/루트 제품 적용0이다. 결과 반영은 Completion에 한정한다. callback suffix가 남으면 mailbox/redraw를 보존하며 pending batch 재사용의 동일 N Completion도 original scope 검사 아래 허용한다. 처리된 notice 재전달은 publication만 복구하고 native proof를 다시 소비하지 않는다. 호출 전후와 발행 전 원 deadline을 검사한다. 통지가 전혀 없을 때 즉시 만료 알림/OS 호출 강제 중단은 보장하지 않는다. 실제 합성 observer 오류 재전달/Weak 소멸/생산 설치 경로 및 창 실행은 미검증이다.

## 최종 r6 공동 gate 인수

UI 최종 source7은 `53F85DA3DC1F38FCEB4A0AFB3A2F8C7C0DDC1FCE377CAEDFEC70C43CCC1909F8`다. 원 r2 뒤 미설치 API의 정확 항목별 dead_code 사유, cfg(test) owned closure lifetime capture, suffix fixture의 원 문서 generation 정합을 각각 독립 수리·읽기 인수했다. 정책/권한/단언/기본 Viewer 배선 변경0. [r6 인계](../../../../runtime/coordination/ui-frame-delivery-r1/revision-r6/HANDOFF.md) 및 [공동 검증 결과](../../../../runtime/quality-review/ui-sdk-union12-r1/RESULT.md)가 현재 정본이다.

양 all-target strict/fmt와 최종 같은 Native binary의 UI103(새18 포함)+SDK219+integration6=고유328 통과. root1441/snapshot1403·원 bundle/before/입력mtime 보호 차이0/index0, 원37receipt 전terminal(원실패101셋 보존)/기록74PID 부재를 확인하고 PM에 heavy/shared를 반환했다. 실제 생산 opt-in 설치·합성 observer 오류 재전달/Weak 소멸·추가 observer-test-support delivery 구성·GUI/시각/미디어는 여전히 미검증이며 기본 ViewerNoChild/main 전환은 별도 범위다.
