---
id: adr:kinekino/20261105151827
title: "단축키 표: PotPlayer 기본 시크·탐색과 확정된 구간 키"
status: active
created: 2026-11-05T15:18:27Z
updated: 2026-11-05T15:18:27Z
rel:
  governs: [crates/kinekino-player-ui]
---

# 단축키 표: PotPlayer 기본 시크·탐색과 확정된 구간 키

## Context

로드맵 plan:kinekino/20261031153240의 M2-1이다. Meari·libmpv 플레이어 창(`kinekino-player-ui`)에는 음량·속도·트랙·수동 구간 단축키가 이미 있고, 시크·이전/다음 항목용 키는 아직 없다. 수동 구간 키(Ctrl+[ / Ctrl+] / S / Ctrl+\\)는 M1-9b에서 잠정으로 두었고 이 표에서 재확인한다. 목표 감각은 PotPlayer 기본 배치(공식·위키와 한국어 단축키 모음을 교차)이며, physical key라 한국어 IME 레이아웃에서도 같다.

## Decision

1. **시크 단계.** 컨트롤러에 `SeekBy(초)`를 둔다. 부호 있는 상대량이며, 새 위치는 `[0, duration]`으로 클램프한 뒤 기존과 같이 `seek <secs:.3> absolute`로 엔진에 보낸다(상대 seek 모드를 쓰지 않음). `duration ≤ 0`이거나 미디어가 없으면 아무 일도 하지 않는다. 시크 뒤에는 기존 `Seek`와 같이 `resume_rewound()`를 부른다. 시크 토스트·OSD는 내지 않는다(M2-5).
2. **시크·탐색 키(이번 구현).** 누르고 있으면 계속 움직이는 키는 음량·속도와 같은 자동 반복 허용 블록에 둔다.

| 키 | 명령 | 반복 |
| --- | --- | --- |
| ← / → | `SeekBy(±5)` (간격 1) | 허용 |
| Ctrl+← / Ctrl+→ | `SeekBy(±30)` (간격 2) | 허용 |
| Shift+← / Shift+→ | `SeekBy(±60)` (간격 3) | 허용 |
| Ctrl+Alt+← / Ctrl+Alt+→ | `SeekBy(±300)` (간격 4) | 허용 |
| PageUp | `Previous` | 무시 |
| PageDown | `Next` | 무시 |
| Backspace | `Seek(0.0)` (맨 앞) | 무시 |

시크 화살표의 수식키는 **정확히** 맞을 때만이다: Ctrl만 또는 Meta만(간격 2), Shift만(간격 3), Control|Alt만(간격 4). Ctrl+Shift·Meta+Alt 등은 시크가 아니다. 파일 열기·구간 키의 `primary`(Ctrl|Meta, Alt 없음)와는 다르다. Shift+↑ 등 PotPlayer 마스터 볼륨은 구현하지 않는다(↑/↓ 음량은 plain만).

3. **이미 있는 키(문서화만, 동작 유지).**

| 키 | 명령 | 반복 |
| --- | --- | --- |
| ↑ / ↓ | 음량 ±5 | 허용 |
| M | 음소거 | 무시 |
| Space | 재생/일시정지 | 무시 |
| F | 전체화면 전환 | 무시 |
| Esc | 전체화면 종료(메뉴·열기 취소 우선은 SoT) | 무시 |
| Ctrl+O (Meta+O) | 파일 열기 | 무시 |
| X / C / Z | 속도 −0.1 / +0.1 / 1.0 | X·C 허용, Z 무시 |
| Alt+A / Alt+L | 오디오·자막 트랙 순환 | 무시 |
| Ctrl+[ / Ctrl+] / S / Ctrl+\\ | 구간 시작·끝·건너뛰기·지우기 | 무시 |

4. **M1-9b 잠정 키 확정.** Ctrl+[ / Ctrl+] / S / Ctrl+\\는 위 표의 수동 구간 키로 확정한다. keys.rs와 adr:kinekino/20261105111300의 provisional 문구를 이 ADR을 가리키는 확정 표현으로 바꾼다.

5. **이번 범위 밖(남은 일).** Home/End 자막 큐 점프(자막 큐 API 없음), 휠 음량, 시크·상태 OSD(M2-5), 마스터 볼륨 키, 사용자 정의 단축키 설정.

## Rationale

- PotPlayer 기본 간격(5 / 30 / 60 / 300초)과 PageUp·PageDown·Backspace는 국내 사용자에게 익숙하고, 기존 음량·속도 키와도 겹치지 않는다.
- 컨트롤러가 절대 위치를 계산하면 엔진 포트와 이어보기 되돌림(`resume_rewound`)을 한 경로로 유지한다.
- 잠정 구간 키를 같은 표에서 확정하면 M2 메뉴 연결(M2-3) 전에 키 충돌을 다시 열지 않아도 된다.

## Consequences

- `tests/controller/seek_nav.rs`가 키 매핑과 `SeekBy` 클램프·no-op·맨 앞 시크를 검증한다.
- SoT `player-window-controls`에 시크·탐색 규칙 블록을 추가하고, 수동 구간의 "M2-1에서 재확인" 문구를 이 ADR 확정으로 고친다.
- 상수 `SEEK_STEP_1`…`SEEK_STEP_4`(5 / 30 / 60 / 300)는 컨트롤러에 두고 키 매핑이 그대로 쓴다.
- 남은 일: Home/End 자막 점프, 휠 음량, OSD(M2-5), 마스터 볼륨·사용자 정의 키.
