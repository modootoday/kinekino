---
id: plan:kinekino/20261029113900
title: R2 후속 fixture — source-only 후보 동결
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R2 후속 fixture — source-only 후보 동결

2026-10-24 후속 배정 결과: 기존9/new2에 viewer_native_launch.rs test-only 최초 mount 전 source seam1을 추가해 기존10/new2 exact12를 private 동결했다. runtime/development/ui-live-frame-fixture-r1/source12.json SHA5245F605D60D7DA5DDFD859707A9DCDFB6E135B022AE93F2CB3940B51F586A69, before12 E2FAF3ACD161E7D76913C258C109840EB70F549ADA042C69906A32E81CDAE79A. 작성시험13/leaf12 fmt·check 통과/compile0/list0/actual0이다. 상세 source/delta/predecessor·관측 한계는 같은 경로의 HANDOFF.md에 동결했다. b942 preimage를 보존하고 dd27 SDK 채택이12경로와 비겹침임을 기록했으며 임의 재기준하지 않았다. canonical/S/T/Cargo/OS/GUI 변경0, 독립 읽기·타입 실행 배정 전이다.
2026-10-24 UI source-only 정본. [Meari의 원 fixture 대조](../../../../packages/meari/.spec/evidence/reviews/r2-minimal-fixture-proposal-2026-10-23.md)를 전부 읽어 연결했다. 실행 권한은 execution-leases.md만 소유한다. 이 계획은 추가 코드·Cargo·S/T·창 권한을 부여하지 않는다.

## 선행 상태

[R1 실제 gate](../../../sots/verification/presentation/live-entry-r1-2026-10-24.md)는 source3FD29712의 고유23 통과 후 rootaa88706에 채택됐다. R2 원6A0A는 f4/private547 계보로 보존하고, successor1 C5BD3B83776EA4D149422AC460FE19B98E526985C656C1A580B37960252ECB8E는 최종 R1 fixture 수리를 R2 tests.rs 한 leaf에만 상속했다. 다른 제품11 raw는 불변이며 이는 최초 successor 동결 시점 기록이다. R2 본체는 이후 source5007ED의 고유74 headless 근거와 캐시2 예외를 포함해 검증·반환한 뒤 rootb942901에 채택됐다([R2 정본](../../../sots/verification/presentation/live-mount-r2-2026-10-24.md)). 현재 후속 fixture12의 compile/actual은0이다. 기본 chooser·R3/R4와 시각 인수는 완료되지 않았다.

먼저 독립 source 검토 및 명시 gate에서 새 root/preimage·typed 코드·실제 목록/선택된 시험을 확인한다. 기존 R1 실제 통과를 R2 통과로 합산하지 않는다. 추가 source/fixture 경로는 아래 exact 제안의 별도 배정 뒤에만 작성한다.

## 동결 후보 — 기존10/new2, 실행0

| 경로 | 책임 |
|---|---|
| viewer_native_launch.rs | cfg(test) 정적 source·Text callback reference/props를 configured 후·ProductFrame/최초 mount 전에 함께 준비. 기본·Jobs 경로와 원 오류 owner 보존 |
| viewer_native_launch/no_media.rs | cfg(test) trusted live choices를 원 ProductFrame 준비에 전달. 기본 None 보존 |
| viewer_native_launch/no_media/tests/end_to_end.rs | sibling child 모듈 등록만 |
| viewer_native_launch/no_media/tests/end_to_end/live_retained.rs (신규) | 원 supervisor의 task별 exact child/mode·원 cleanup/inner record |
| viewer_native_probe.rs | opt-in live 책임 모듈 등록. Jobs/기본 mode·bounded trace 유지 |
| viewer_native_probe/live_retained.rs (신규) | 실제 source 입력·entry/return proof·관측·원 close phase |
| player_shell_host/delivery.rs | 원 borrow 종료 후 실제 notice를 opt-in probe에 전달 |
| viewer_host/live_entry.rs | 첫 AwaitingArm에서만 원 startup route notifier의 counted/fail-once decorator·arm boundary |
| viewer_host/mount_proposal.rs | 실제 commit 뒤 cfg(test) proof-retainer 오류 carrier 경계 |
| viewer_host/return_viewing.rs | 별도 Viewing proof-retainer 실패/원 release·close 경계 |
| viewer_host/live_entry/tests.rs | 실제 arm borrow·receipt 재생0·notifier·미관측 입력 차단 영향 |
| viewer_host/mount_proposal/tests.rs | 기존 typed seed를 공유하고 원 client만 seed. 별도 owner/SDK 복제0 |

모든 경로는 src/assembly/meari/ 아래다. 원 source11 제안에 한정 pre-mount seam1을 별도 승인받았으며 원 NoMedia supervisor/Jobs fixture는 수정하지 않았다. 기존 supervision.rs/jobs_retained.rs와 libraryb532 변경 제안0이다. 실제 타입·원 fixture에서 추가 요구가 나오면 경로/이유를 다시 제안한다.

## 분리할 gate

1. **headless typed 연결:** 원 SDK create/import 결과→원 host-held 명시 target→같은 inspect/terminal/ACK/release→공통 panel과 원 오류/partial cleanup. 수동 export·모델 초기 route와 실 renderer export·frame context를 구별한다.
2. **실제 양성:** Card/Highlight/Correction route를 각각 귀속한다. cue/document correction의 서로 다른 등록 경로까지 이어받으려면4 exact child를 사용한다. 원 NoMedia WindowsFamily/Supervisor의20초+정리2초·EOF/familyEmpty/forcedfalse/inner schema를 유지한다. 실제 Some DTO의 document/ticket을 원 opaque proof와 맞추고 library validator를 통과한 entry observed 뒤 다음 활성 projection, 현재 scoped 입력/결과 표시, 별도 Viewing commit/proof와 실제 ViewingSome/observed, 원 close를 연결한다. 같은 SDK Save/Accept/CAS 기능을 GUI에서 전부 재검사하지 않는다.
3. **arm/notify:** 실제 observer borrow Busy의 첫 exact receipt와 같은 proof 보존, 현재 원 intent로만 명시 rearm·재inspect/재attach/재commit0을 확인한다. route notifier의 fail-once typed 오류는 wake_error이며 arm Busy가 아니다. 성공은 원 startup notifier에 위임하고 duplicate notify0/단발 redraw·close 우선/late wake 무효화를 구별한다. 다른 SDK wake로 진행했으면 실패한 notify가 성공했다고 표시하지 않는다.
4. **postcommit proof 오류 모델:** 현재 공개 API로 즉시 발급 지점의 실제 Busy를 안전하게 강제할 seam이 없다. cfg(test) retainer가 실제 commit 뒤 한 번 Err를 반환하는 제품 carrier 시험을 제안한다. 원 receipt/bridge/proof_error·첫 HostEffects를 유지하고 역사 proof 재발급/입력/재commit 없이 원 close한다. entry와 return은 별도 carrier다. 이는 실제 Meari Busy나 실제 제출 양성이 아니며 같은 통과 수로 합산하지 않는다. 새 projection/driver drop/forged receipt/opaque proof mint로 오류를 만들지 않는다.

실제 Some나 checked CPU submission도 OS compositor readback·사람 키보드/IME·디자인 시안 전체 일치가 아니다. 가상 모니터/DPI/VM 보류와 사용자 demo 소유를 유지하며 별도 승인 없이 수동 GUI를 재개하지 않는다.

기존에 본문을 읽은 modoo.today test-fixture-and-mock-build를 success/error/fault knob·원 owner/setup/body/teardown·source/model/actual 분리에, frontend-first-journey를 원 typed/common component와 사용자 도달성 구분에 적용했다. 실제 작성 코드·시험은 private fixture12에 있으며 이 계획의 과거 제안과 현재 source-only 동결을 구분한다. 실행 근거는 아직 없다.


실제 입력 결과 후보는 projected controlled props와 renderer editor.raw_text를 원 controller commit 뒤 같은 current projection의 실제 frame에서 함께 비교한다. 입력용 try_reproject 후 old entry proof를 재인증하지 않는다. 복귀는 현재 원 host typed request이며 source navigation callback/R3 도달성 인수가 아니다. notifier의 원 inspect continuation은 그대로 두고 entry/return arm의 호출·delegate 시도·Ok·오류 snapshot을 구분한다. 실패 notifier가 성공 wake였다고 계상하지 않는다. 원 owner 미정리 실패는 보존/명시 누수 fallback이며 정상 cleanup/누수 없음 증거가 아니다.
