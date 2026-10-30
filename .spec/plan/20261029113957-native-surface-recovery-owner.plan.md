---
id: plan:kinekino/20261029113957
title: 원 native owner의 recovery 보유 경계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 native owner의 recovery 보유 경계

2026-10-20 설계 후보다. 원 probe r3 source11 SHA D9B11EADCC2EE7666D0774BCDC8145A2DA776ACE7DD49F3E30B053FAE2E07E02는 변경하지 않는다. 이번 문서로 production 조립·새 cleanup driver·Cargo/OS 실행을 승인하지 않는다.

## 해결할 소유 문제

현재 probe의 명시 release는 실패 시 같은 owner를 유지하지만 caller가 그 owner를 Drop하면 best-effort 이후 원 lease/slot/class 재시도 상태를 잃을 수 있다. 따라서 오류를 반환하면서 caller에게 보존을 요청하는 API만으로 production 경계를 닫을 수 없다. callback userdata 제거 실패의 raw Arc는 그대로 보존하며 추측한 from_raw로 회수하지 않는다.

## 최소 보유 방식

trusted UI host가 UI-thread 전용 recovery registry를 strong ownership으로 가진다. OS 생성 전에 checked generation과 capacity를 확보하고 원 owner의 보관 위치를 예약한다. probe 생성의 성공 또는 partial failure를 그 위치에 바로 저장한다. 등록이 실패하면 원 입력/partial owner 전체를 반환하며 원 자원을 Drop해서 성공으로 바꾸지 않는다.

호출자에게 원 owner를 꺼내 주는 take/remove API는 두지 않는다. registry가 계속 owner를 소유하고 호출자에게는 private generation key만 준다. key는 ready/표시 권한이나 raw HWND가 아니며 stale key를 새 owner로 remap하지 않는다. 명시 release는 기존 owner.release를 한 번 호출해 typed 결과와 원 진단을 기록한다. registry 자체가 NativeChild 또는 두 번째 cleanup driver가 되지는 않는다. 이후 production proxy는 유일한 lifecycle 호출 경로 안에서 같은 entry를 사용해야 한다.

상태는 Reserved, Owned, ReleaseFailed, Unobserved, Released로 구별한다. ReleaseFailed/Unobserved에서 entry와 원 owner를 유지한다. class 해제 실패는 같은 class_registered 상태를 이용해 명시 재시도하고, callback detach 실패는 복구 미지원 진단을 보존한다. 재시도 횟수와 key generation은 checked 증가하며 자동 retry loop를 만들지 않는다. Released는 실제 원 destroy/detach/class receipt가 성공한 경우뿐이다. NotCreated는 별도 생성 실패 정리 결과이며 정상 창 Destroy receipt가 아니다.

registry 종료는 pending entry가 있으면 typed shutdown-incomplete와 **원 registry 소유권 자체**를 반환한다. host는 그 보유자를 계속 유지하고 정상 parent/host 종료를 승인하지 않는다. 강제 프로세스 종료까지 자원을 보존한다는 보장은 하지 않는다. registry 최종 Drop만으로 성공을 발급하거나 안전한 회수가 안 된 owner를 정리 완료로 바꾸지 않는다. host의 오용 Drop 방지까지 포함한 실제 guard 구현/검토가 production 인수의 선행 조건이다.

## engine 연결 전 추가 조건

production에서는 surface만 중앙에 남겨도 outer가 원 engine/unique parent lease를 먼저 Drop할 수 있다. 그러므로 engine+surface+원 cleanup attempt를 보유하는 기존 PlaybackSurfaceChild 전체를 중앙 entry가 소유하고 lifecycle proxy가 그 유일한 entry를 구동해야 한다. engine Complete 뒤 surface만 retry하는 기존 정책은 재구현하지 않는다. 현재 probe는 원 input을 보유하므로 unique lease를 engine에 동시에 넘길 수 없으며, probe→등록된 engine/surface 조립 이전의 별도 ownership 분리 계약이 필요하다. 이 문서는 그 분리를 구현했다고 주장하지 않는다.

## 제안 소스 범위와 검증

최초 probe 보유 정책만 작성한다면 `src/assembly/meari/playback_surface/mod.rs` 등록1, `recovery/{mod,registry,tests}.rs` 신규3, exact4 후보가 최소다. opaque key·checked capacity/generation·원 owner 보유/실패 shutdown 반환을 generic fake owner로 검사한다. 실제 probe/root 조립/production proxy는 별도 범위이며 원 r3 수정0이다. PM 범위 확정 전 소스를 생성하지 않는다.

의미 있는 Rust 회귀는 release 오류/Unobserved 뒤 동일 owner 잔존, stale/foreign key 호출0, class-only retry 때 engine 재시작0, 미완료 shutdown이 registry를 반환, 완전 receipt 전 remove0이다. 실제 callback/Destroy/parent loss 증거와 seam 회귀를 섞지 않는다. 실제 GUI/OS 검증과 메모리 회수는 여전히 별도 gate다.
