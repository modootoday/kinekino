---
id: adr:kinekino/20261105101125
title: 스타일·이미지 자막은 엔진이 그린다
status: active
created: 2026-11-05T10:11:25Z
updated: 2026-11-16T09:45:00Z
rel:
  governs: [crates/kinekino-player-ui/src/controller, crates/kinekino-libmpv/src]
---

# 스타일·이미지 자막은 엔진이 그린다

## Context

로드맵 plan:kinekino/20261031153240의 M1-8(PL-4, SB-2 첫 단계)이다. 플레이어는 libmpv를 `sub-visibility=no`로 열고 `sub-text`를 받아 자체 오버레이로 자막을 그린다. 이 경로는 일반 텍스트 자막에는 맞지만 ASS/SSA의 위치·스타일·효과는 잃고, PGS·DVD·DVB 같은 이미지 자막은 `sub-text`가 비어 아예 보이지 않는다.

## Decision

1. **판정 기준.** 매 폴링에서 선택된 자막 트랙의 코덱(`current-tracks/sub/codec`)을 읽는다. `ass`, `ssa`, `hdmv_pgs_subtitle`, `dvd_subtitle`, `dvb_subtitle`, `xsub`이면 엔진이 그린다. 그 밖의 코덱과 자막 없음은 기존 오버레이가 그린다.
2. **전환.** 판정이 바뀔 때만 `set sub-visibility yes|no`를 보낸다. 엔진이 그리는 동안 오버레이 텍스트는 비워 같은 줄이 두 번 보이지 않게 한다. 명령이 거절되면 다음 폴링에서 다시 시도한다.
3. **렌더 경로.** libmpv 소프트웨어 렌더는 `sub-visibility`가 켜지면 자막을 프레임에 합성한다. 별도 비트맵 경로(overlay API)는 쓰지 않는다.
4. **위치.** 판정과 전환은 플레이어 컨트롤러(`controller/subtitles.rs`)에 둔다. 엔진 포트는 그대로이고, 사이트별 용어나 플러그인 식별자는 없다.

## Rationale

- 코덱 목록 판정은 엔진 속성 하나로 끝나고, 텍스트 자막의 기존 오버레이 모양(브랜드 글꼴·위치)을 바꾸지 않는다.
- 모든 자막을 엔진에 넘기면 텍스트 자막도 mpv 기본 스타일로 바뀌어 시안과 어긋난다.
- 소프트웨어 렌더 합성은 추가 코드 없이 libass·이미지 자막 렌더를 그대로 쓴다. 실제 libmpv 테스트로 ASS 벡터 상자가 켤 때만 프레임에 그려지는 것을 확인했다.

## Consequences

- ASS·이미지 자막이 원래 모양으로 보인다. 텍스트 자막은 지금과 같다.
- 남은 일: 엔진 자막의 크기·여백을 오버레이 설정과 맞추는 옵션, 플레이어 컨트롤 막대와 겹침 확인(GUI 실기), SB-2의 ASS 입력 변환, 이미지 자막 실제 파일 확인.

## Record

- 2026-11-16: The overlap with the control bar is handled in code. While the controls show, the controller lifts engine-drawn cues with `sub-pos` (percent of the frame height) by the transport height. `sub-margin-y` was not used because mpv documents it as not applying to ASS without a forced style override, nor to picture subtitles. Not yet checked: that libmpv's `osd-height` equals the video box's CSS height under the render API, and the real window itself (GUI).
