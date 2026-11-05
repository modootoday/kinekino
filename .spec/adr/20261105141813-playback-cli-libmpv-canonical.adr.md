---
id: adr:kinekino/20261105141813
title: 일반 재생은 libmpv 플레이어 경로가 정본이고 CLI mpv-IPC는 실험으로 둔다
status: active
created: 2026-11-05T14:18:13Z
updated: 2026-11-05T14:18:38Z
rel:
  governs: [crates/kinekino-player-ui/src, crates/kinekino-libmpv/src, crates/kinekino-player/src/application/api/native_session.rs, crates/kinekino-player/src/infrastructure/mpv]
---

# 일반 재생은 libmpv 플레이어 경로가 정본이고 CLI mpv-IPC는 실험으로 둔다

## Context

로드맵 plan:kinekino/20261031153240의 M1-10(PL-5)다. 예전 로드맵 항목 7·P2는 일반 재생 17명령(`playback.open`/`play`/`pause`/`seek`/`set_volume`/`set_muted`/`set_rate`/`select_track`/`previous`/`next`/`close` 등)을 `kinekino-player`의 mpv 프로세스+IPC 어댑터(`infrastructure/mpv`, ADR 20261029113547)로 Linux에 올리려 했다. 그 뒤 플레이어 창은 Meari·libmpv로 바뀌었고(ADR 20261031020000), 볼륨·속도·트랙·이어보기·자막·수동 구간(M1-2~M1-9)은 `kinekino-player-ui` 컨트롤러와 `kinekino-libmpv`로 이미 검증됐다. CLI `playback.*`는 여전히 네이티브 세션·mpv IPC에 묶여 있고, 기본 capabilities에서는 `playback_available`/`control_available`이 꺼져 있어 대부분 unsupported다. 두 경로를 모두 제품 정본으로 키울지, 하나를 정본으로 두고 다른 쪽을 정리할지 재판정이 필요하다.

## Decision

1. **제품 재생 정본.** 일반 재생(열기·재생/일시정지·시크·볼륨·음소거·속도·트랙·이전/다음·닫기)의 제품 경로는 `kinekino watch` → `kinekino-player-ui` → `Engine`/`VideoSurface` → `kinekino-libmpv`다. PL-5·M1 완료 기준의 명령 검증은 이 경로의 controller 테스트로 본다.
2. **CLI mpv-IPC는 실험·레거시.** `kinekino-player`의 `playback.*` CLI(네이티브 세션·`infrastructure/mpv`)는 검토·호스트 조립·증거용 실험 경로로 남긴다. M1에서 libmpv로 이식하거나 Linux 기본 capabilities를 supported로 올리지 않는다. 새 일반 재생 동작은 player-ui에 넣고, 이 CLI 스택에 기능을 얹지 않는다.
3. **이관 범위.** mpv 프로세스 IPC 스택 전체를 libmpv로 바꾸는 작업은 M1 밖이다. 필요하면 이후 마일스톤에서 별도 ADR로 범위를 정한다. 기존 ADR 20261029113547(초기 mpv 프로세스 선택)은 그 실험 경로의 근거로 유지하고, 제품 UI 경로는 ADR 20261031020000이 우선한다.
4. **정리(이번 조각).** 이 ADR과 현황표·큐·로드맵 Record만 갱신한다. 코드 삭제·capabilities 플래그 변경·새 CLI 어댑터는 하지 않는다.

## Rationale

- M1 완료 기준(3)은 controller 테스트로 속도·트랙·볼륨을 요구하며, 이미 libmpv 경로에 있다. CLI를 같은 수준으로 올리는 것은 창·IPC·수명 설계를 다시 짜는 큰 일이고 M1 목표와 겹치지 않는다.
- 두 엔진 경로를 동시에 “정본”으로 두면 단축키·영속·자막·구간 정책을 이중 구현하게 된다. 정본을 하나로 고정하는 편이 안전하다.
- skill: 조회 결과 무관(playback CLI vs embedded engine canonical path).

## Consequences

- PL-5의 “CLI 재생은 옛 mpv 경로” 남은 일은 이 재판정으로 닫는다. 트랙 선택·선호 언어 영속은 후속(M1 밖)으로 남긴다.
- M1 완료 기준 (1)~(5)가 모두 제품 경로에서 충족되면 로드맵 Record에 M1 완료를 적을 수 있다.
- 옛 CLI `playback.*`를 건드리는 변경은 이 ADR을 읽고, 제품 동작을 player-ui에 먼저 넣을지 확인한다.
- 남은 일: 실험 CLI 경로의 폐기·축소 일정(후속), 트랙 선호 언어 영속(후속), M2 메뉴·단축키 표.
