---
id: adr:kinekino/20261107231529
title: Player tray icon port and platform adapters
status: active
created: 2026-11-07T23:15:29Z
updated: 2026-11-07T23:20:00Z
rel:
  governs:
    - crates/kinekino-player-ui/src/window
    - assets/design/app-icon
  depends_on: [adr:kinekino/20261031111316]
---

# Player tray icon port and platform adapters

## Context

로드맵 plan:kinekino/20261031153240의 M2-8(트레이 부분, UI-9)이다. ADR 20261031111316은 마스코트에서 파생한 앱 아이콘을 창·트레이에 연결하기로 했고, 2026-11-08에 창 아이콘(X11 `_NET_WM_ICON`)만 연결·실화면 확인했다. 제품에는 아직 트레이 존재가 없다. Meari 0.2에는 트레이 API가 없고, `WindowOptions`에도 아이콘·트레이 필드가 없다. Linux에서는 xfce4-panel 시스템 트레이로 실화면 확인이 가능하다.

권고안(루틴 진행, 2026-11-08): 구현 전에 트레이 **포트·어댑터·플랫폼 범위**만 ADR로 닫고, 같은 파생 자산 연결·실화면 확인은 후속 조각으로 둔다. 새 시안·목업 재생성·아이콘 시트 겹침 수정은 하지 않는다(보류).

코드 현황:

- 창 아이콘: `crates/kinekino-player-ui/src/window/icon.rs`가 `assets/design/app-icon/`의 16–64 PNG를 임베드해 X11에만 설정한다.
- 트레이·StatusNotifier·AppIndicator·NotifyIcon 호출은 저장소에 없다.
- 플러그인이 UI 크롬을 소유하지 않는다. 트레이도 호스트 presentation 영역이다.

skill: 조회했으나 트레이 포트·데스크톱 셸 어댑터에 맞는 도메인 스킬 없음 → 미적용.

## Decision

1. **트레이는 호스트 presentation 포트다.** `kinekino-player-ui`의 `window` 근처(예: `window/tray.rs` + 얇은 포트 trait)에 둔다. 플러그인·사이트 어댑터·Meari 위젯 트리는 트레이를 소유·해석하지 않는다. 사이트별 용어·아이콘·메뉴를 트레이에 넣지 않는다.
2. **첫 경로 계약(설계만, 구현은 후속).** 포트 이름 가칭 `TrayPresence`:
   - **install / uninstall:** 프로세스 수명 동안 트레이 항목 1개. 실패는 한 번 경고하고 플레이어는 계속한다(창 아이콘과 같은 폴백).
   - **icon:** ADR 20261031111316 파생 PNG만. 런타임 재파생·새 시안·시트 수정 금지. 창 아이콘과 같은 임베드 집합에서 트레이에 적합한 크기(우선 32·48, 필요 시 16·24)를 고른다.
   - **tooltip:** 제품명 `kinekino`(일반 문자열). 재생 중인 작품·회차·플러그인 이름을 넣지 않는다(호스트·플러그인 경계, 공개 표면).
   - **activate(왼쪽 클릭 등):** 메인 플레이어 창을 보이기·포커스. 창이 최소화·숨김이면 복원한다.
   - **첫 경로에 넣지 않음:** 트레이 우클릭 메뉴, “닫기 시 트레이로”, 트레이에서 종료, 상태별 마스코트 교체, 알림 풍선. 메뉴·환경설정 연동은 후속 ADR/조각.
3. **플랫폼 범위.**
   - **Linux(첫 구현 대상):** StatusNotifierItem(또는 동등한 패널 트레이 프로토콜) 어댑터. 실화면 확인 호스트는 xfce4-panel 시스템 트레이. Wayland/X11 모두 패널이 SNI를 제공하면 같은 어댑터로 본다. 미가용이면 Unsupported로 한 번 경고.
   - **Windows / macOS:** 이 ADR에서 계약만 남기고 어댑터는 `Unsupported`(또는 no-op). NotifyIcon·NSStatusItem·Meari 트레이 API가 생기기 전에는 ungate·완료 주장 금지.
   - **Meari:** 0.2에 트레이가 없으므로 Meari에 트레이를 우회 구현하지 않는다. 후일에 Meari가 트레이 옵션을 제공하면 어댑터만 갈아끼운다.
4. **이번 조각에서 하지 않는 것.** 트레이 코드·의존 crate 추가, 실패널 배선, 환경설정 키, 메뉴, 창 아이콘 Windows/macOS 경로, 자산 재생성.
5. **후속 조각(가칭 M2-8c).** Linux `TrayPresence` 어댑터 + 파생 아이콘 연결 + xfce4-panel 실화면 확인. 계약 테스트는 포트 목/Unsupported 경로와 아이콘 바이트 계약을 먼저 두고, 실패널은 시각 검증으로 남긴다.

## Rationale

- 로드맵 M2-8은 창·트레이를 마스코트 파생 자산에 연결하고 실제 표시를 확인한다. 창은 끝났고, 트레이는 제품에 존재 자체가 없어 포트부터 정해야 구현 크기가 한 실행에 맞는다.
- Meari에 트레이가 없는 상태에서 UI 트리에 억지 연결하면 호스트·엔진 경계가 흐려진다. 창 아이콘처럼 OS 셸 어댑터로 두는 편이 기존 `window/icon.rs` 패턴과 같다.
- 툴팁·메뉴에 작품명을 넣으면 호스트가 플러그인/카탈로그 문자열을 셸에 노출한다. 첫 경로는 제품명만.
- Windows/macOS를 지금 구현하면 실기 없이 완료를 주장하기 쉽다. Unsupported로 명시해 M6 실기 조각과 구별한다.

## Alternatives rejected

| 대안 | 이유 |
| --- | --- |
| 이 조각에서 Linux 트레이까지 구현 | 한 실행 크기 초과; 큐가 ADR-only로 나눔 |
| Meari/Blitz 셸에 트레이 우회 패치 | Meari 0.2 API 없음; 엔진 포크 범위 밖 |
| 플러그인·사이트별 트레이 아이콘/메뉴 | 호스트·플러그인 경계 위반 |
| 새 시안·아이콘 시트 수정 후 연결 | 보류 항목; ADR 20261031111316 기존 파생만 |
| 트레이 메뉴·닫기-트레이 설정을 첫 경로에 포함 | 범위 비대; 메뉴는 기존 context menu와 별도 설계 필요 |
| Windows/macOS를 Linux와 동시에 완료 처리 | 실기·Meari 옵션 없음; ungate 금지 |

## Consequences

- UI-9의 트레이는 **설계(ADR) 완료**, 코드·실화면은 후속(M2-8c)까지 미구현으로 남는다. M2 닫힘의 “트레이 아이콘 연결·실화면 확인”은 8c 이전에는 충족하지 않는다.
- 창 아이콘 ADR 기록의 “Tray: design first”는 이 ADR로 닫히고, 구현 대기로 바뀐다.
- 후속 구현은 `assets/design/app-icon/` 임베드만 재사용한다. `tools/brand-icons/derive.py` 재실행·목업 재생성은 하지 않는다.
- 트레이 미지원 환경에서도 플레이어 기동은 실패하지 않는다.

## Record

- 2026-11-08: M2-8b — tray port (`TrayPresence`), Linux-first SNI adapter scope, same derived app-icon assets, no implementation; Windows/macOS Unsupported until real adapters; follow-up wires Linux tray and verifies on xfce4-panel.
