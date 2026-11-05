---
id: adr:kinekino/20261105111300
title: 수동 구간 건너뛰기는 항목마다 컨트롤러가 기억한다
status: active
created: 2026-11-05T11:13:00Z
updated: 2026-11-05T13:17:22Z
rel:
  governs: [crates/kinekino-player-ui/src/controller, crates/kinekino-core/src/playback]
---

# 수동 구간 건너뛰기는 항목마다 컨트롤러가 기억한다

## Context

로드맵 plan:kinekino/20261031153240의 M1-9(PL-8 첫 단계)다. 수동 구간 정책(`kinekino-player`의 `domain::skip_segments`)은 예전 호스트 세션과 CLI 경로에 묶여 있고, Meari·libmpv 플레이어 창(`kinekino-player-ui`)은 그 크레이트에 의존하지 않는다. 플레이어에서 사용자가 오프닝 같은 구간을 직접 표시하고 건너뛸 방법이 없다. 자동 감지는 M4 뒤로 미룬다.

## Decision

1. **값.** 구간은 `kinekino-core::playback::SkipRange { start, end }`(초)다. 유한하고 `0 ≤ start < end`일 때만 만들어지고, `start ≤ 위치 < end`를 "구간 안"으로 본다.
2. **명령.** 컨트롤러에 네 명령을 둔다. `MarkSkipStart`는 현재 위치를 시작으로 기억하고, `MarkSkipEnd`는 현재 위치를 끝으로 구간을 확정한다(시작이 없거나 끝이 시작보다 앞이면 안내만 하고 바꾸지 않는다). `SkipSegment`는 재생 위치가 구간 안일 때만 끝으로 시크한다. `ClearSkip`은 구간과 표시 중인 시작점을 지운다(없을 때도 안내). 결과는 모두 짧은 토스트로 보인다.
3. **표시 상태.** `View.skip_offer`는 위치가 구간 안일 때 참이다. 폴링과 표시·건너뛰기 때마다 다시 계산한다(일반 시크 뒤에는 다음 폴링에서 맞춰진다). 참일 때만 영상 위 오버레이 단추(`skipSegment`, `clearSkip`)가 보이고, 단축키 S·Ctrl+\\가 같은 명령을 부른다.
4. **수명.** 구간과 표시 중인 시작점은 지금 항목에만 속하고, 다른 항목을 열거나 새 파일·목록으로 대기열이 바뀌면 지운다. 파일에 남기지 않는다.
5. **위치.** 판단은 `controller/skip.rs`에 둔다. 엔진 포트는 그대로이고 사이트별 용어나 플러그인 식별자는 없다.

## Rationale

- 플레이어 창이 `kinekino-player` 전체(예전 호스트·CLI 의존)를 끌어오지 않도록 작은 값 하나만 core에 둔다. 예전 정책의 검증·겹침·자동 모드는 이 단계에 필요 없다.
- 회차마다 오프닝 위치가 다르므로 항목 단위가 안전한 기본값이다. 회차 간 재사용과 저장은 저장 형식을 정할 때 함께 정한다.

## Consequences

- 컨트롤러 테스트로 표시·확정·건너뛰기·지우기·항목 전환이 검증된다.
- 구간 안(`View.skip_offer`)일 때만 영상 위 오버레이에 "건너뛰기"와 보조 "구간 지우기" 단추가 보인다(`#skipSegment`, `#clearSkip`).
- 단축키(잠정, PotPlayer 감각; **M2-1 단축키 표 ADR에서 재확인**): Ctrl+[ 시작, Ctrl+] 끝, S 건너뛰기, Ctrl+\\ 지우기. physical key, 한국어 IME 레이아웃에서도 동일. 자동 반복은 음량·속도만.
- `ClearSkip`은 구간·시작점을 지우고 토스트로 알린다. 없을 때는 "지울 구간이 없습니다".
- 구간 저장·같은 키 재오픈 복원(M1-9c)은 adr:kinekino/20261105131646로 넘긴다. 회차 간 복사는 하지 않는다.
- 남은 일: 메뉴 연결(M2-3), GUI 시안 대조는 실기 확인, 단축키는 M2-1에서 재확인.
