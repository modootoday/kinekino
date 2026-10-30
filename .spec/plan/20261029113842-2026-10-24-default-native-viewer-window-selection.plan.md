---
id: plan:kinekino/20261029113842
title: 실제 viewer 창 선택 최소 수리 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 viewer 창 선택 최소 수리 제안

상태: 읽기 전용 제안, 소스/Cargo/OS0. 진단 N 실패와 승인 정리는 [반환 정본](../../../sots/verification/presentation/default-native-diagnostic-n-first-2026-10-24.md) D1ACD4BF/slotDDD25064에 보존했다. Media C2 heavy 중이며 구현은 별도 인수·배정 후다.

## 확인된 결함과 생성 계약

window.rs43의 accepts는 같은 PID/nonzero thread/visible/top-level/nonempty class만 검사한다. 실제 N에서 `Winit Thread Event Target`/빈 title이 이 조건을 통과해 HWND59314770에 게시됐다. cached winit-win320.31.0-beta.3 event_loop.rs982/1037은 내부 target의 class와 WS_VISIBLE|WS_POPUP 및 TOOLWINDOW/NOACTIVATE 계열 ex-style을 확인한다. target callback2588~2622에는 WM_CLOSE→CloseRequested 변환이 없고 user callback1338~1340에만 있다. SDK도 해당 원문을 직접 읽었다. 이 선택 결함은 확정 P2이며 전체 shutdown 결함이나 원 비계측 N의 동일 원인은 아직 확정하지 않는다.

실제 제품 생성은 src/assembly/meari/product_viewer_launch.rs231의 WindowOptions::default → Meari driver.rs254~276의 validate/caller-title/viewport/initial-hidden → resume_window의 set_visible(true)다. 현재 custom class/role/owner API는 없다. cached backend lib.rs482~491의 기본 class는 `Window Class`, owner=None/skip_taskbar=false이고 window_state.rs304~329는 일반 nonpopup의 caption/sysmenu/taskbar style을 만든다. 제목은 application.rs의 공개 표시 옵션이며 단독 identity가 아니다. 제품명 문자열을 새로 하드코딩하지 않는다.

cached window.rs816~817은 WindowId::from_raw(self.hwnd() as usize)다. 따라서 **이번 pinned Win32 backend에서만** child trace62134260과 선택59314770이 다른 실제 창임을 source와 연계할 수 있다. 임의 WindowId 숫자를 신뢰 입력으로 받아 OS 권한으로 사용하지 않는다. 실제 viewer의 class/style/owner는 N1에서 직접 조회하지 않아 아직 OS 관측 미입증이다.

## 정확 변경 경로 — 기존2, 신규0

1. `tests/native_product_launch/default_launch/window.rs` — preimage1BC901669295EADCDFDC9D9C74B47132AC1CB7069873CAF7BC39C551F3AE8F9B. private 기본 viewer 선택 계약과 snapshot 판정/유일 후보 선택·게시 직전 검사를 이 어댑터에 둔다. find(pid) 호출과 원 poll/deadline을 유지하므로 default_launch.rs 변경은 필요 없다.
2. `tests/native_product_launch/default_launch/tests.rs` — preimage4E1A981FFCF33B2567EABEA0DCE9A748232EB0EFADC0C8E7D2AABA4118F3F6D5. 기존 foreign/hidden/child/zero-handle/thread/class 회귀를 유지·보강하고 아래 실제 반례/유일성/소멸 회귀를 순수 snapshot으로 추가한다.

Meari/product/SDK application/domain/argv/config/public API 변경0이다. SDK witness EDEF와 Meari ED427은 기존 통합 기준으로 유지하고 이번2만 별도 preimage/raw/delta/핀을 만든다. S의 원7A8854와 prerequisite fileguard부재 복구 계약은 다음 gate에서도 분리한다.

## 제안 판정

원 Supervisor의 process handle 기반 PID/creation/image live 검증을 유지한다. 그 원 PID의 Win32 관측에서 내부 event target은 명시 배제하고, 검토한 backend의 application class `Window Class`/visible/unowned root/non-child/유일성을 positive 조건으로 사용한다. 현재 default viewer의 caller title은 같은 WindowOptions::default 값에서 가져와 생성 경로와 상관시키며 표시명 literal을 추가하지 않는다. title/class는 원 process 권한을 대체하지 않는 보조 조건이다.

style/ex-style과 parent/owner를 bounded snapshot/evidence에 보존한다. 현재 default decorated nonpopup 역할과 맞는 caption/sysmenu·nonpopup, TOOLWINDOW/NOACTIVATE 배제는 검토된 생성 경로의 보강 조건으로 제안한다. OS 관측 전 이를 실제 스타일로 표시하지 않는다. 임의 custom popup/undecorated Meari 전반의 지원 계약으로 확대하지 않는다.

helper만 있으면 eligible0/None으로 돌아가 원10ms poll/원 deadline 안에서 기다린다. hidden viewer도 기다리며 helper+visible user는 user만 선택한다. eligible user가2개면 원 ambiguous error를 유지한다. 선택 뒤 같은 HWND/PID/thread/class/역할·caller-title의 재확인에 실패하거나 창이 소멸하면 게시 전에 원 typed 오류를 반환한다. IsWindow만으로 identity를 대체하거나 실패 뒤 재선택/추가 WM_CLOSE를 하지 않는다. 정상 maximize 등 무관한 전체 style bit 변화는 identity로 오인하지 않도록 역할 mask와 raw evidence를 분리한다.

Probe는 현재 root/id/revision/sha256인 configured SDK 저장소 검사 계약이고 GateSpec에는 별도 창 selector/신뢰 HWND 입력이 없다. 여기에 임의 HWND/웹 메시지·새 CLI 인자를 추가하지 않는다. 강한 renderer-origin role/handle 계약은 향후 공통 typed host 확장으로 별도 검토하며 이번 test adapter 수리에 묶지 않는다. 다음 actual에서 trusted backend mapping으로 선택 HWND와 renderer-created window 관측을 상관시키기 전 실제 viewer CloseRequested 커버리지를 완료로 표시하지 않는다.

## 필수 순수 회귀·다음 검증 경계

- internal-only→None, internal+hidden user→None, internal+visible user→user만. 같은 title을 위조한 helper도 거절한다.
- 유효 user2→원 ambiguity 오류. foreign/hidden/child/owned/toolwindow/popup/wrong class·caller-title을 정상 user로 세지 않는다.
- 선택 뒤 PID/thread/class/역할/handle 변경 또는 소멸→게시 거절. 원 identity 조회 오류는 같은 Failure로 전달하고 성공으로 치환하지 않는다.

기존 window_admission 시험을 보강하고 새 순수 시험은 위3책임 단위로만 추가하는 안이다. 새 실제 OS 시험·검증 프레임워크0, 등록 수 증가는 구현 명단에서 명시 pin한 뒤 실제 list로 검증한다. 다음 별도 CPU gate는 기존 fmt/양strict/no-run/새 H 목록/기존17+새3를 재사용한다. 그 뒤 actual N1은 별도 배정이며 원30/5·case1·argv0·원 owner/first cause 유지, C/추가 Close/자동 Abort/기한 연장0이다.

이번 세션에서 읽은 modoo.today code-change-review-verification9f73865f/필수 verdict-table9d9619a8, fixture a393f15f의 실제 반례·원 오류 보존·소스/실측/미검증 구분을 적용했다. 정확 출처 전체 SHA는 parent witness 후계 정본에 유지한다. Meari의 독립 근거는 packages/meari/.spec/evidence/reviews/native-diag-n-window-selection-2026-10-24.md에 연결한다.
