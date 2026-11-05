---
id: adr:kinekino/20261105161614
title: "우클릭 메뉴 골격: 포인터 위치 팝업과 다섯 묶음"
status: active
created: 2026-11-05T16:16:14Z
updated: 2026-11-05T16:16:18Z
rel:
  governs: [crates/kinekino-player-ui]
---

# 우클릭 메뉴 골격: 포인터 위치 팝업과 다섯 묶음

## Context

로드맵 plan:kinekino/20261031153240의 M2-2다. M2 완료 기준 (2)는 우클릭 메뉴에서 열기·재생·자막·오디오·화면 명령을 고를 수 있는 것이다. Meari·libmpv 플레이어 창에는 전송 막대의 ⋮(더 보기) 메뉴만 있고(adr:kinekino/20261031055110), 오른쪽 단추 누름은 문서로 그대로 넘어가 아무 일도 하지 않는다. 이번 조각은 메뉴의 틀(열고 닫는 규칙, 위치, 묶음)만 세우고, M1 명령 전부를 연결하는 일은 M2-3에 둔다.

## Decision

1. **명령과 상태.** `Command::ContextMenu(x, y)`는 문서 좌표(CSS px)의 왼쪽 위 모서리에 메뉴를 연다. `View.context_menu: Option<(f64, f64)>`가 열린 위치다. 다시 우클릭하면 새 위치로 옮긴다. `Command::CloseMenu`는 두 메뉴(⋮, 우클릭)를 모두 닫는다. 우클릭 메뉴를 열면 ⋮ 메뉴는 닫힌다.
2. **닫힘 규칙.** ⋮ 메뉴와 같다. Esc는 열린 메뉴만 닫고 전체화면·열기 취소보다 먼저다. `ContextMenu` 밖의 모든 명령(메뉴 항목 포함)은 우클릭 메뉴를 닫는다. 메뉴 밖을 왼쪽 단추로 누르면 창이 `CloseMenu`를 보내고 그 누름은 문서로 넘기지 않는다(데스크탑 메뉴 관례). 메뉴가 열려 있으면 전체화면 자동 숨김을 하지 않는다.
3. **위치.** 창이 누른 점을 배율로 나눈 문서 좌표와 뷰포트 크기를 `menu::context_anchor`에 넘긴다. 메뉴 상자(`CONTEXT_MENU_SIZE`, 240×360 px 예약)가 오른쪽·아래로 넘치면 누른 점의 왼쪽·위로 펼치고, 그래도 넘치면 0에 붙인다. 위치는 `contextMenuStyle` 문자열 prop(`left: …px; top: …px;`)으로 인라인 `style`에 넣는다. Meari 0.2.0이 prop으로 바뀐 `style` 속성을 다시 배치하는 것은 레이아웃 시험으로 확인한다.
4. **다섯 묶음(골격 항목).** `#contextMenu`(`role="menu"`, `aria-label="메뉴"`) 안에 머리말과 이미 있는 명령 하나 이상씩을 둔다.

| 묶음 | 단추 id | 명령 | 미디어 없을 때 |
| --- | --- | --- | --- |
| 열기 | `ctxOpen` / `ctxBrowse` | `Open` / `Browse` | 사용 가능(탐색은 플러그인 있을 때만 보임) |
| 재생 | `ctxPlayPause` | `PlayPause` | 비활성 |
| 자막 | `ctxSubtitle` | `CycleSubtitle` | 비활성 |
| 오디오 | `ctxAudio` | `CycleAudio` | 비활성 |
| 화면 | `ctxFullscreen` | `ToggleFullscreen` | 사용 가능 |

단축키 표기는 adr:kinekino/20261105151827의 표를 따른다(Ctrl+O, Space, Alt+L, Alt+A, F).

5. **이번 범위 밖(남은 일).** 속도·음량·시크·구간·자동 다음화·이전/다음 항목 연결과 하위 메뉴(M2-3), 키보드로 항목 이동(↑/↓·Enter), 메뉴 키·Shift+F10으로 열기, 실제 화면에서의 모양 확인(시각 미확인).

## Rationale

- ⋮ 메뉴와 같은 닫힘 규칙을 쓰면 Esc 우선순위 SoT를 한 줄만 넓히면 된다.
- 위치를 컨트롤러가 아닌 순수 함수로 계산하면 창 크기 없이도 시험할 수 있고, 컨트롤러는 받은 좌표만 기억한다.
- 메뉴 높이를 고정 예약으로 두면 실제 레이아웃을 기다리지 않고 한 번에 놓을 수 있다. 시험이 실제 상자가 예약 안에 드는지 확인하므로 M2-3에서 항목을 늘리면 예약도 함께 고쳐야 한다.

## Consequences

- `tests/controller/context_menu.rs`가 열기·옮기기·닫기, Esc 순서, 단추→명령 매핑, 미디어 없음 비활성, 넘침 시 위치, 실제 레이아웃 위치·크기를 검증한다.
- `window/context.rs`가 오른쪽 단추 누름과 메뉴 밖 누름을 다룬다. 창 연결은 단위 시험(`inside`)과 실제 화면 확인으로 본다.
- SoT sot:kinekino/player-window-controls의 메뉴·Esc 문장이 우클릭 메뉴를 포함하도록 바뀐다.
