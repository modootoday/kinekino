---
id: adr:kinekino/20261105191533
title: 설정 창 제품 caption(aw-window-chrome)
status: active
created: 2026-11-05T19:15:33Z
updated: 2026-11-07T14:58:00Z
rel:
  governs: [crates/kinekino-settings-ui]
  related: [adr:kinekino/20261030040215, adr:kinekino/20261031055110, sot:kinekino/settings-ui-design]
---

# 설정 창 제품 caption(aw-window-chrome)

## Context

[설정 Meari 창](20261030040215-settings-window-meari.adr.md) Consequences는 OS 프레임을 쓰고, 제품 공통 caption(`aw-window-chrome`)은 플레이어 셸과 맞출 후속이라고 적어 두었다. 플레이어는 이미 장식 없는 창에 앱 제목 줄을 쓴다([플레이어 창 조작](20261031055110-player-window-controls.adr.md)). 로드맵 M2-6 / UI-5는 설정 창도 같은 제목 줄 규약을 쓰라고 한다. 권고안(2026-11-06): OS 프레임을 끄고 플레이어와 같은 `aw-window-chrome`으로 통일한다.

## Decision

1. **제목 줄.** `aw-settings-header`를 플레이어식 `aw-window-chrome`으로 바꾼다. 왼쪽은 brand 아이콘 + 제품명 `kinekino` + 문맥 `설정`, 오른쪽은 caption 버튼 최소화 / 최대화(복원) / 닫기(`aw-caption`, `aw-icon-minimize` 등 공통 클래스).
2. **장식.** 창이 생긴 뒤 `shell_provider.set_window_decorations(false)`를 한 번 호출한다(플레이어 `caption.rs`와 동일).
3. **끌기·두 번 클릭.** `.aw-window-chrome` 배경(버튼 제외) 왼쪽 누름은 `drag_window()`. 400ms 안 두 번 누르면 최대화/복원을 `shell_provider.set_window_maximized`로 토글한다.
4. **닫기 id.** caption 닫기는 `#windowClose`(footer `#close`와 충돌 금지). 세션 메시지는 footer 닫기와 같이 `{"op":"close"}`.
5. **최소화·최대화.** 페이지 프로토콜이 아니라 호스트 로컬(`window.rs` → `shell_provider`). footer의 되돌리기/적용/닫기는 유지한다.
6. **가장자리 크기 조절.** 이번 조각의 필수는 아니다. 제목 줄·undecorate·min/max/close면 로드맵 기준("같은 제목 줄 규약")을 충족한다.

## Consequences

- [settings-ui-design](../sot/settings-ui-design.sot.md) 창 caption 행은 `aw-window-chrome`으로 갱신한다. 이전 ADR의 "OS 프레임" 서술은 당시 상태이며, 후속은 이 ADR이 정본이다.
- M2-V5: 설정 caption이 공유 `aw-window-chrome`/`--aw-surface-chrome`/`--aw-titlebar`를 쓰는지 DOM·CSS 잠금 테스트로 확인. brand/name/captions flex와 context 신축 규칙을 components.css로 올려 플레이어와 동일 계약. 시안 전용 제목줄 hex는 복제하지 않음. 가장자리 크기 조절 띠는 남은 작업이다.
- M2-V5b(실제 X11 창, 창 관리자 있음): 제목 줄 끌기 이동, 두 번 클릭 최대화/복원, 최소화·최대화(복원)·닫기 단추, 본문 입력 포커스를 실화면으로 확인했다. 확인 중 두 결함을 고쳤다. (1) caption 아이콘만 담은 부분 prop 갱신을 Meari가 거절해("exact declared bindings required") 최대화 뒤 아이콘이 바뀌지 않고 이후 caption·닫기 클릭이 사라졌다. 이제 세션이 전체 prop(페이지 상태·마지막 적용 오류·대화상자 닫힘·최대화 상태)을 함께 넘긴다. 두 번 클릭 직후에는 창 관리자가 상태를 늦게 알려 줄 수 있으므로 요청한 상태로 아이콘을 맞춘다. (2) Meari는 `:focus-visible`을 맞추지 않아 입력 포커스가 엔진 기본 파란 테두리로 보였다. 공용 `components.css`의 `input:focus`도 `--aw-focus`를 쓴다(플레이어 입력에도 같이 적용). 시안과 다른 점은 공용 규칙을 따른다: 제품명 `kinekino`, 공용 brand 아이콘(외곽선), caption 아이콘 크기 `--aw-caption-icon`, 둥근 창 모서리 없음. 단추의 키보드 포커스 표시는 엔진이 `:focus-visible`을 지원하기 전까지 남은 작업이다.
