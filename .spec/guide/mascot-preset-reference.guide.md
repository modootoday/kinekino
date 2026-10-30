---
id: guide:kinekino/mascot-preset-reference
title: 마스코트 프리셋 레퍼런스 (자동 생성)
status: active
created: 2026-10-29
updated: 2026-10-29
rel:
  governs: [assets/mascot]
---

# 마스코트 프리셋 레퍼런스 (자동 생성)

## Overview

자동 생성: `cd assets/mascot && python3 tools/gen-preset-reference.py`. 원본은 JSON입니다. 이 문서를 고치지 말고 JSON을 고친 뒤 다시 생성하세요.

## Steps

### 파트 클립

| 클립 | 레이어 | 길이(s) | 루프 | holdAt | 배치 | 메모 |
|---|---|---|---|---|---|---|
| `ear_L.perk` | ear_L | 1.2 |  |  | v6 |  |
| `ear_L.flutter` | ear_L | 1.2 |  |  | v6 |  |
| `ear_L.back` | ear_L | 1.4 |  |  | v6 |  |
| `ear_R.perk` | ear_R | 1.2 |  |  | v6 |  |
| `ear_R.flutter` | ear_R | 1.2 |  |  | v6 |  |
| `ear_R.back` | ear_R | 1.4 |  |  | v6 |  |
| `tail.wag` | tail | 2.0 | 예 |  | v6 |  |
| `tail.curl` | tail | 1.6 |  |  | v6 |  |
| `tail.stiff` | tail | 1.6 |  |  | v6 |  |
| `eye_L.blink` | eye_L | 0.6 |  |  | v6 |  |
| `eye_L.wink` | eye_L | 1.0 |  |  | v6 |  |
| `eye_L.grow` | eye_L | 1.2 |  |  | v6 |  |
| `eye_L.shrink` | eye_L | 1.2 |  |  | v6 |  |
| `eye_L.squint` | eye_L | 1.2 |  |  | v6 |  |
| `eye_L.surprise` | eye_L | 1.4 |  |  | v6 |  |
| `eye_L.sad` | eye_L | 1.4 |  |  | v6 |  |
| `eye_R.blink` | eye_R | 0.6 |  |  | v6 |  |
| `eye_R.wink` | eye_R | 1.0 |  |  | v6 |  |
| `eye_R.grow` | eye_R | 1.2 |  |  | v6 |  |
| `eye_R.shrink` | eye_R | 1.2 |  |  | v6 |  |
| `eye_R.squint` | eye_R | 1.2 |  |  | v6 |  |
| `eye_R.surprise` | eye_R | 1.4 |  |  | v6 |  |
| `eye_R.sad` | eye_R | 1.4 |  |  | v6 |  |
| `body.breathe` | body | 3.0 | 예 |  | v6 |  |
| `body.wobble` | body | 1.4 |  |  | v6 |  |
| `body.jump_recoil` | body | 1.2 |  |  | v6 |  |
| `body.lean` | body | 2.4 |  |  | v6 |  |
| `reveal.show` | reveal | 0.3 |  |  | v6 |  · restExempt=transition |
| `reveal.hide` | reveal | 0.3 |  |  | v6 |  · restExempt=transition |
| `ear_L.twitch` | ear_L | 0.22 |  |  | 1 | 한쪽 귀 짧은 까딱(소리 탐지) |
| `ear_R.twitch` | ear_R | 0.22 |  |  | 1 | 미러: `ear_L.twitch` |
| `ear_L.swivel` | ear_L | 0.7 |  |  | 2 | 귀를 옆·뒤로 돌림(yaw) |
| `ear_R.swivel` | ear_R | 0.7 |  |  | 2 | 미러: `ear_L.swivel` |
| `ear_L.airplane` | ear_L | 1.6 |  | 0.5 | 2 | 옆으로 납작(본 회전 + 귀 스케일 납작) |
| `ear_R.airplane` | ear_R | 1.6 |  | 0.5 | 2 | 미러: `ear_L.airplane` |
| `ear_L.flat` | ear_L | 1.4 |  | 0.3 | 2 | 머리에 완전히 붙임(ear.back보다 강함) |
| `ear_R.flat` | ear_R | 1.4 |  | 0.3 | 2 | 미러: `ear_L.flat` |
| `ear_L.one_back` | ear_L | 1.4 |  | 0.4 | 1 | 한쪽만 뒤로 젖힘(R은 미러) |
| `ear_R.one_back` | ear_R | 1.4 |  | 0.4 | 1 | 미러: `ear_L.one_back` |
| `ear_L.droop` | ear_L | 2.4 |  | 0.8 | 1 | 축 처짐(이완형) |
| `ear_R.droop` | ear_R | 2.4 |  | 0.8 | 1 | 미러: `ear_L.droop` |
| `ear_L.bounce_settle` | ear_L | 0.8 |  |  | 1 | 감쇠 진동 팔로스루 |
| `ear_R.bounce_settle` | ear_R | 0.8 |  |  | 1 | 미러: `ear_L.bounce_settle` |
| `ear_L.listen_beat` | ear_L | 1.2 | 예 |  | 3 | 100BPM 2박 루프(1박째 까딱). R은 combo에서 0.6s(1박) 오프셋. BPM은 timeScale=bpm/100 |
| `ear_R.listen_beat` | ear_R | 1.2 | 예 |  | 3 | 미러: `ear_L.listen_beat` |
| `tail.tip_flick` | tail | 0.3 |  |  | 1 | 끝 2본만 까딱, 반복 가능 |
| `tail.question` | tail | 2.0 |  | 0.6 | 1 | 세우고 끝을 갈고리로 |
| `tail.quiver` | tail | 0.8 |  |  | 2 | 세운 채 고주파 소진폭 떨림 |
| `tail.puff` | tail | 1.4 |  | 0.25 | 2 | 굵기 확대(본 스케일) |
| `tail.lash` | tail | 1.2 | 예 |  | 2 | 0.3s/휘두름 루프(wag보다 빠르고 큼) |
| `tail.thump` | tail | 0.4 |  |  | 2 | 바닥 내려침. body 미세 squash는 조합에서 body.squash_press(약)로 |
| `tail.tuck` | tail | 1.8 |  | 0.5 | 2 | 아래·뒤로 말아 넣기 |
| `tail.low` | tail | 1.6 |  | 0.5 | 1 | 수평 아래로 처짐 |
| `tail.wrap_body` | tail | 2.6 |  | 0.8 | 3 | 앞쪽(+z)으로 돌려 몸 앞을 감음(관통 검사 통과 필요) |
| `tail.slow_sway` | tail | 2.4 | 예 |  | 1 | 2.4s/왕복 루프 |
| `tail.heart_curve` | tail | 2.4 |  | 1.0 | 3 | 연출용 하트 반쪽 곡선 |
| `eye_L.slow_blink` | eye_L | 1.3 |  |  | 1 | 느린 감았다 뜨기(애정 신호) |
| `eye_R.slow_blink` | eye_R | 1.3 |  |  | 1 | 미러: `eye_L.slow_blink` |
| `eye_L.double_blink` | eye_L | 0.5 |  |  | 2 |  |
| `eye_R.double_blink` | eye_R | 0.5 |  |  | 2 | 미러: `eye_L.double_blink` |
| `eye_L.rapid_blink` | eye_L | 0.8 |  |  | 2 |  |
| `eye_R.rapid_blink` | eye_R | 0.8 |  |  | 2 | 미러: `eye_L.rapid_blink` |
| `eye_L.half_lid` | eye_L | 2.0 |  | 0.3 | 1 | 상단 절단 모프 half_lid |
| `eye_R.half_lid` | eye_R | 2.0 |  | 0.3 | 1 | 미러: `eye_L.half_lid` |
| `eye_L.sleepy_droop` | eye_L | 3.0 | 예 |  | 1 | 꾸벅꾸벅 루프 |
| `eye_R.sleepy_droop` | eye_R | 3.0 | 예 |  | 1 | 미러: `eye_L.sleepy_droop` |
| `eye_L.closed_happy` | eye_L | 1.8 |  | 0.15 | 1 | 아치 모프 ^ ^ |
| `eye_R.closed_happy` | eye_R | 1.8 |  | 0.15 | 1 | 미러: `eye_L.closed_happy` |
| `eye_L.closed_line` | eye_L | 1.8 |  | 0.15 | 2 | blink 끝 상태 유지 |
| `eye_R.closed_line` | eye_R | 1.8 |  | 0.15 | 2 | 미러: `eye_L.closed_line` |
| `eye_L.glare` | eye_L | 1.8 |  | 0.2 | 2 | 비대칭 상단 절단(안쪽 낮음, 눈별 지오메트리) |
| `eye_R.glare` | eye_R | 1.8 |  | 0.2 | 2 | 미러: `eye_L.glare` |
| `eye_L.worried` | eye_L | 1.8 |  | 0.25 | 2 | 안쪽 위가 올라감(sad 회전과 별개) |
| `eye_R.worried` | eye_R | 1.8 |  | 0.25 | 2 | 미러: `eye_L.worried` |
| `eye_L.dilate` | eye_L | 1.5 |  | 0.3 | 2 | 세로만 확대 모프 |
| `eye_R.dilate` | eye_R | 1.5 |  | 0.3 | 2 | 미러: `eye_L.dilate` |
| `eye_L.dizzy` | eye_L | 1.6 |  |  | 3 | 작은 원 궤적(위치 오프셋) |
| `eye_R.dizzy` | eye_R | 1.6 |  |  | 3 | 미러: `eye_L.dizzy` |
| `eye_L.sparkle` | eye_L | 0.6 |  |  | 3 | 하이라이트 다이아몬드 모프(COLOR_0 흰색 정점, 정지 시 점으로 수축) |
| `eye_R.sparkle` | eye_R | 0.6 |  |  | 3 | 미러: `eye_L.sparkle` |
| `eye_L.cross` | eye_L | 0.5 |  |  | 3 |  |
| `eye_R.cross` | eye_R | 0.5 |  |  | 3 | 미러: `eye_L.cross` |
| `body.yawn` | body | 2.0 |  |  | 2 | 위로 늘었다 내려앉음 |
| `body.stretch` | body | 1.8 |  |  | 2 | 앞(가로)으로 길게, 부피 근사 보존 |
| `body.knead` | shape | 1.2 | 예 |  | 2 | 하단 좌/우 교대 눌림(디포머 본, shape 레이어) |
| `body.purr` | body | 1.0 | 예 |  | 2 | 미세 고주파 균일 스케일 |
| `body.groom` | body | 2.6 |  |  | 3 | 기울어 반복 굴곡(lean + rotX/scale 진동) |
| `body.shake_off` | body | 0.7 |  |  | 1 | 빠른 좌우 비틀기 감쇠 |
| `body.squash_press` | body | 0.9 |  | 0.15 | 1 | 눌림 유지형, 부피 근사 보존 |
| `body.stretch_hang` | body | 1.4 |  | 0.3 | 1 | 위쪽 고정(집힘) 아래로 늘어짐 |
| `body.hop` | body | 0.5 |  |  | 1 |  |
| `body.pounce` | body | 1.2 |  |  | 3 | 웅크림+엉덩이 흔들기(rotZ)+앞(+z)으로 튀기 |
| `body.crouch` | shape | 1.6 |  | 0.4 | 2 | 윗부분 내려앉고 아래 퍼짐(shape) |
| `body.arch` | shape | 1.6 |  | 0.3 | 2 | 윗부분 둥글게 솟음(crown) |
| `body.loaf` | shape | 2.6 |  | 1.0 | 2 | 하단 평탄·넓게(shape) |
| `body.melt` | body | 1.5 |  | 0.9 | 3 | 강한 squash |
| `body.headbutt` | body | 0.6 |  |  | 3 |  |
| `body.rub_side` | body | 1.2 |  |  | 3 |  |
| `body.tremble` | body | 1.0 | 예 |  | 2 | 두 주파수 합성(불규칙) |
| `body.nod` | body | 0.5 |  |  | 2 |  |
| `body.shake_no` | body | 0.6 |  |  | 1 |  |
| `body.head_tilt` | body | 1.6 |  | 0.5 | 2 |  |
| `body.spin` | body | 0.8 |  |  | 3 |  |
| `body.bounce_loop` | body | 1.2 | 예 |  | 3 | 100BPM 2박. timeScale=bpm/100 |
| `body.typing_bob` | body | 0.3 |  |  | 2 | 입력 1회당 1회 트리거 |
| `body.think_sway` | body | 3.0 | 예 |  | 1 |  |
| `body.fall_land` | body | 0.6 |  |  | 1 |  |
| `body.wobble_hit` | shape | 0.5 |  |  | 3 | 위쪽 움푹(crown 디포머) |
| `reveal.eyes` | reveal | 0.3 |  |  | v6 | 보조: 눈만 표시(eyes 모드 시작) · restExempt=transition |
| `reveal.hide_eyes` | reveal | 0.3 |  |  | v6 | 보조: eyes 모드 끝 · restExempt=transition |
| `reveal.pulse` | reveal | 0.6 |  |  | 1 | eyes 모드에서 띠·귀 안쪽 왕복 · restExempt=revealed |
| `reveal.blush` | reveal | 2.0 |  | 0.4 | 1 | 볼 홍조 COLOR_0 모프(blush) · restExempt=revealed |
| `fx.pop_in` | body | 0.5 |  |  | 2 |  · restExempt=entry |
| `fx.pop_out` | body | 0.4 |  |  | 2 |  · restExempt=exit |
| `fx.peek` | body | 0.8 |  |  | 2 |  · restExempt=entry |
| `fx.hide` | body | 1.8 |  | 0.6 | 3 | 아래로 쏙(클리핑은 앱) |
| `fx.overshoot` | body | 0.45 |  |  | v6 | 보조: big_success 스케일 오버슈트 |
| `acro.spin_yaw_360` | body | 1.0 |  |  | 8 | 세로축 팽이 회전 360° 제자리 |
| `acro.spin_yaw_360_jump` | body | 1.4 |  |  | 8 | 세로축 팽이 회전 360° 점프(도약-정점-착지 squash&stretch) |
| `acro.spin_yaw_720` | body | 2.0 |  |  | 8 | 세로축 팽이 회전 720° 제자리 |
| `acro.spin_yaw_720_jump` | body | 1.4 |  |  | 8 | 세로축 팽이 회전 720° 점프(도약-정점-착지 squash&stretch) |
| `acro.flip_front_360` | body | 1.0 |  |  | 8 | 앞 공중제비(가로축 pitch) 360° 제자리 |
| `acro.flip_front_360_jump` | body | 1.4 |  |  | 8 | 앞 공중제비(가로축 pitch) 360° 점프(도약-정점-착지 squash&stretch) |
| `acro.flip_back_360` | body | 1.0 |  |  | 8 | 뒤 공중제비 360° 제자리 |
| `acro.flip_back_360_jump` | body | 1.4 |  |  | 8 | 뒤 공중제비 360° 점프(도약-정점-착지 squash&stretch) |
| `acro.cartwheel_360` | body | 1.0 |  |  | 8 | 옆돌기(정면 축 roll) 360° 제자리 |
| `acro.cartwheel_360_jump` | body | 1.4 |  |  | 8 | 옆돌기(정면 축 roll) 360° 점프(도약-정점-착지 squash&stretch) |
| `acro.roll_360` | body | 1.8 |  |  | 8 | 옆으로 굴러 한 바퀴(이동 후 미끄러져 복귀) |
| `tail.acro_drag` | tail | 1.4 |  |  | 8 | 회전 원심력 끌림 + 착지 반동 |
| `ear_L.acro_drag` | ear_L | 1.4 |  |  | 8 | 회전 원심력으로 바깥으로 젖힘 + 착지 반동 |
| `ear_R.acro_drag` | ear_R | 1.4 |  |  | 8 | 미러: `ear_L.acro_drag` |

### 조합 프리셋

| 조합 | 그룹 | 의미 | 길이(s) | 루프 | reveal | blend | 트랙 (레이어 클립 @시작) |
|---|---|---|---|---|---|---|---|
| `startled` | v6 |  | 2.4 |  | full | 0.05 | body `body.jump_recoil` @0<br>ear_L `ear_L.back` @0.05<br>ear_R `ear_R.back` @0.05<br>tail `tail.stiff` @0.08<br>eye_L `eye_L.surprise` @0<br>eye_R `eye_R.surprise` @0 |
| `alert` | v6 |  | 2.4 |  | full | 0.2 | ear_L `ear_L.perk` @0.1<br>ear_R `ear_R.perk` @0.1<br>eye_L `eye_L.grow` @0.1<br>eye_R `eye_R.grow` @0.1 |
| `curious` | v6 |  | 3.0 |  | full | 0.2 | body `body.lean` @0.2<br>ear_R `ear_R.perk` @0.3<br>eye_L `eye_L.grow` @0.3<br>eye_R `eye_R.grow` @0.3 |
| `tickle` | v6 |  | 2.0 |  | full | 0.2 | body `body.wobble` @0.1<br>ear_L `ear_L.flutter` @0.1<br>ear_R `ear_R.flutter` @0.1<br>eye_L `eye_L.squint` @0.1<br>eye_R `eye_R.squint` @0.1 |
| `content` | v6 |  | 3.0 |  | full | 0.2 | body `body.breathe` @0<br>tail `tail.curl` @0.3<br>eye_L `eye_L.squint` @0.4<br>eye_R `eye_R.squint` @0.4 |
| `ear_perk` | v6 |  | 1.6 |  | full | 0.2 | ear_L `ear_L.perk` @0.1<br>ear_R `ear_R.perk` @0.1 |
| `ear_flutter` | v6 |  | 1.6 |  | full | 0.2 | ear_L `ear_L.flutter` @0.1<br>ear_R `ear_R.flutter` @0.1 |
| `idle_breath` | v6 |  | 3.0 |  | full | 0.2 | body `body.breathe` @0<br>eye_L `eye_L.blink` @1.5<br>eye_R `eye_R.blink` @1.5 |
| `wink_wag` | v6 |  | 2.4 |  | full | 0.2 | tail `tail.wag` @0<br>eye_L `eye_L.wink` @0.3<br>ear_R `ear_R.perk` @0.3 |
| `sad` | v6 |  | 2.4 |  | full | 0.2 | ear_L `ear_L.back` @0.1<br>ear_R `ear_R.back` @0.1<br>eye_L `eye_L.sad` @0.1<br>eye_R `eye_R.sad` @0.1<br>tail `tail.curl` @0.2 |
| `happy` | 감정 | 기분 좋음 | 2.6 |  | full | 0.2 | tail `tail.question` @0<br>eye_L `eye_L.closed_happy` @0.1<br>eye_R `eye_R.closed_happy` @0.1<br>body `body.hop` @0.2<br>ear_L `ear_L.bounce_settle` @0.5<br>ear_R `ear_R.bounce_settle` @0.5 |
| `excited` | 감정 | 매우 신남 | 2.4 |  | full | 0.2 | tail `tail.quiver` @0<br>eye_L `eye_L.dilate` @0<br>eye_R `eye_R.dilate` @0<br>body `body.bounce_loop` @0.1 {"loop": true}<br>ear_L `ear_L.perk` @0<br>ear_R `ear_R.perk` @0 |
| `affection` | 감정 | 애정(느린 깜박임) | 3.0 |  | full | 0.2 | eye_L `eye_L.slow_blink` @0<br>eye_R `eye_R.slow_blink` @0<br>ear_L `ear_L.droop` @0 {"weight": 0.5}<br>ear_R `ear_R.droop` @0 {"weight": 0.5}<br>reveal `reveal.blush` @0.3<br>tail `tail.question` @0 |
| `pet_purr` | 감정 | 쓰다듬기 반응 | 4.0 | 예 | full | 0.2 | eye_L `eye_L.half_lid` @0 {"hold": true}<br>eye_R `eye_R.half_lid` @0 {"hold": true}<br>body `body.purr` @0.2 {"loop": true}<br>shape `body.knead` @0.5 {"loop": true}<br>ear_L `ear_L.droop` @0 {"weight": 0.5, "hold": true}<br>ear_R `ear_R.droop` @0 {"weight": 0.5, "hold": true}<br>look `look.track_slow` @0 |
| `sleepy` | 감정 | 졸림 | 5.0 |  | full | 0.2 | eye_L `eye_L.sleepy_droop` @0 {"loop": true}<br>eye_R `eye_R.sleepy_droop` @0 {"loop": true}<br>ear_L `ear_L.droop` @0.3 {"hold": true}<br>ear_R `ear_R.droop` @0.3 {"hold": true}<br>body `body.yawn` @1.0<br>tail `tail.wrap_body` @1.5 {"hold": true} |
| `asleep` | 감정 | 수면 루프 | 6.0 | 예 | full | 0.2 | eye_L `eye_L.closed_line` @0 {"hold": true}<br>eye_R `eye_R.closed_line` @0 {"hold": true}<br>shape `body.loaf` @0 {"hold": true}<br>body `body.breathe` @0 {"loop": true, "timeScale": 0.6}<br>ear_? `ear_?.twitch` @3 {"side": "random", "repeat": {"every": 4, "jitter": 2}} |
| `wake_up` | 감정 | 기상 | 4.4 |  | full | 0.2 | eye_L `eye_L.double_blink` @0<br>eye_R `eye_R.double_blink` @0<br>body `body.stretch` @0.3<br>body `body.yawn` @1.8<br>body `body.shake_off` @3.5 |
| `annoyed` | 감정 | 짜증 | 2.4 |  | full | 0.2 | tail `tail.lash` @0 {"loop": true}<br>ear_L `ear_L.airplane` @0.1<br>ear_R `ear_R.airplane` @0.1<br>eye_L `eye_L.glare` @0.2<br>eye_R `eye_R.glare` @0.2<br>look `look.avert` @0.4 |
| `angry_puff` | 감정 | 화·위협 | 2.0 |  | full | 0.05 | tail `tail.puff` @0<br>shape `body.arch` @0<br>ear_L `ear_L.flat` @0.05<br>ear_R `ear_R.flat` @0.05<br>eye_L `eye_L.dilate` @0<br>eye_R `eye_R.dilate` @0 |
| `scared` | 감정 | 겁먹음 | 2.6 |  | full | 0.2 | ear_L `ear_L.flat` @0<br>ear_R `ear_R.flat` @0<br>tail `tail.tuck` @0.1<br>shape `body.crouch` @0.1<br>body `body.tremble` @0.3 {"loop": true}<br>eye_L `eye_L.rapid_blink` @0.5<br>eye_R `eye_R.rapid_blink` @0.5 |
| `shy` | 감정 | 부끄러움 | 2.6 |  | full | 0.2 | look `look.avert` @0<br>reveal `reveal.blush` @0.1<br>ear_R `ear_R.one_back` @0.2<br>body `body.lean` @0.2 {"weight": 0.5} |
| `proud` | 감정 | 뿌듯함 — body.stretch(세로만) -> body.yawn(세로 늘림) 0.7 가중치로 대체 | 2.6 |  | full | 0.2 | body `body.yawn` @0 {"weight": 0.7}<br>eye_L `eye_L.half_lid` @0.2<br>eye_R `eye_R.half_lid` @0.2<br>tail `tail.question` @0<br>look `look.up_think` @0.2 |
| `confused` | 감정 | 혼란 | 2.6 |  | full | 0.2 | body `body.head_tilt` @0<br>ear_L `ear_L.one_back` @0.1<br>eye_L `eye_L.double_blink` @0.3<br>eye_R `eye_R.double_blink` @0.3<br>look `look.scan` @0.5 |
| `bored` | 감정 | 심심함 | 4.0 |  | full | 0.2 | look `look.scan` @0<br>tail `tail.slow_sway` @0 {"loop": true}<br>body `body.melt` @1.5 {"weight": 0.5}<br>eye_L `eye_L.half_lid` @1.5<br>eye_R `eye_R.half_lid` @1.5 |
| `disappointed` | 감정 | 실망 | 2.6 |  | full | 0.2 | ear_L `ear_L.droop` @0<br>ear_R `ear_R.droop` @0<br>tail `tail.low` @0.1<br>body `body.melt` @0.2 {"weight": 0.5}<br>look `look.avert` @0.3 |
| `dizzy` | 감정 | 어지러움 | 2.0 |  | full | 0.2 | eye_L `eye_L.dizzy` @0<br>eye_R `eye_R.dizzy` @0<br>body `body.wobble` @0<br>ear_L `ear_L.bounce_settle` @0.2<br>ear_R `ear_R.bounce_settle` @0.2 |
| `content_loaf` | 감정 | 편안한 휴식 — half_lid 유지 중 slow_blink 반복이 같은 eye 슬롯을 교대 사용 | 6.0 | 예 | full | 0.2 | shape `body.loaf` @0 {"hold": true}<br>eye_L `eye_L.half_lid` @0.5 {"hold": true}<br>eye_R `eye_R.half_lid` @0.5 {"hold": true}<br>tail `tail.wrap_body` @0.5 {"hold": true}<br>eye_L `eye_L.slow_blink` @3 {"repeat": {"every": 5, "jitter": 2}}<br>eye_R `eye_R.slow_blink` @3 {"repeat": {"every": 5, "jitter": 2}} |
| `hunting_stalk` | 행동 | 사냥 집중 | 4.0 | 예 | full | 0.2 | shape `body.crouch` @0 {"hold": true}<br>look `look.stare_down` @0<br>ear_L `ear_L.perk` @0<br>ear_R `ear_R.perk` @0<br>tail `tail.tip_flick` @0.3 {"repeat": {"every": 1.2, "jitter": 0.4}} |
| `pounce` | 행동 | 덮치기 | 2.2 |  | full | 0.2 | body `body.pounce` @0<br>ear_L `ear_L.bounce_settle` @0.8<br>ear_R `ear_R.bounce_settle` @0.8 |
| `groom` | 행동 | 그루밍 | 3.2 |  | full | 0.2 | body `body.groom` @0<br>eye_L `eye_L.closed_line` @0.2<br>eye_R `eye_R.closed_line` @0.2<br>ear_? `ear_?.swivel` @1 {"side": "random"} |
| `play` | 행동 | 놀이 초대 | 2.0 |  | full | 0.2 | shape `body.crouch` @0<br>eye_L `eye_L.dilate` @0<br>eye_R `eye_R.dilate` @0<br>tail `tail.slow_sway` @0<br>body `body.hop` @0.8 |
| `greet` | 행동 | 인사 — 첫 실행이면 vars.entry=fx.pop_in | 2.6 |  | full | 0.2 | body `$entry` @0<br>tail `tail.question` @0.3<br>eye_L `eye_L.slow_blink` @0.6<br>eye_R `eye_R.slow_blink` @0.6<br>body `body.headbutt` @1.0 |
| `goodbye` | 행동 | 작별 | 1.2 |  | full | 0.2 | body `body.nod` @0<br>eye_L `eye_L.closed_happy` @0.2<br>eye_R `eye_R.closed_happy` @0.2<br>body `fx.pop_out` @0.8 |
| `listen` | 행동 | 듣는 중 — swivel(0.7s) 뒤 perk로 교대(같은 귀 슬롯) | 3.0 | 예 | full | 0.2 | ear_L `ear_L.swivel` @0<br>ear_R `ear_R.swivel` @0<br>ear_L `ear_L.perk` @0.8<br>ear_R `ear_R.perk` @0.8<br>look `look.at_target` @0<br>body `body.nod` @0.4 {"weight": 0.4, "repeat": {"every": 1.5}} |
| `loading` | 앱 상태 | 처리 중 | 3.0 | 예 | full | 0.2 | body `body.think_sway` @0 {"loop": true}<br>look `look.up_think` @0<br>tail `tail.slow_sway` @0 {"loop": true}<br>eye_L `eye_L.blink` @1.0 {"repeat": {"every": 2.5, "jitter": 0.8}}<br>eye_R `eye_R.blink` @1.0 {"repeat": {"every": 2.5, "jitter": 0.8}} |
| `loading_long` | 앱 상태 | 오래 걸림 — 5s bored, 10s groom 삽입. 이후 상태 머신이 loading 유지/재진입 | 12.0 | 예 | full | 0.2 | combo `loading` @0<br>combo `bored` @5<br>combo `groom` @10 |
| `success` | 앱 상태 | 성공 | 2.0 |  | eyes | 0.2 | body `body.hop` @0<br>eye_L `eye_L.closed_happy` @0.1<br>eye_R `eye_R.closed_happy` @0.1<br>tail `tail.quiver` @0.2<br>reveal `reveal.pulse` @0.2 |
| `big_success` | 앱 상태 | 큰 성공 | 3.8 |  | full | 0.2 | body `body.spin` @0<br>body `fx.overshoot` @0.8<br>combo `happy` @1.2 |
| `error` | 앱 상태 | 실패 | 2.4 |  | full | 0.2 | body `body.shake_no` @0<br>ear_L `ear_L.one_back` @0.1<br>eye_L `eye_L.worried` @0.2<br>eye_R `eye_R.worried` @0.2<br>tail `tail.low` @0.3 |
| `warning` | 앱 상태 | 주의 필요 | 2.0 |  | full | 0.2 | ear_? `ear_?.twitch` @0 {"side": "random"}<br>look `look.at_target` @0<br>tail `tail.tip_flick` @0.2 |
| `notification` | 앱 상태 | 새 알림 | 2.0 |  | eyes | 0.2 | ear_L `ear_L.perk` @0<br>ear_R `ear_R.perk` @0<br>reveal `reveal.pulse` @0.1<br>look `look.at_target` @0.2 |
| `typing_buddy` | 앱 상태 | 타이핑 중 — body.typing_bob은 composer.trigger("input")마다 1회 | 2.0 | 예 | full | 0.2 | look `look.at_target` @0<br>body `body.typing_bob` @0 {"trigger": "input"}<br>ear_? `ear_?.twitch` @1.5 {"side": "random", "repeat": {"every": 3, "jitter": 1.5}} |
| `music_groove` | 앱 상태 | 음악 재생 중 — bpmScale: timeScale = bpm/100 * bpmScale (BPM 미상이면 100). ear_R 0.6s=100BPM 1박; 런타임은 60/bpm으로 보정 | 2.4 | 예 | full | 0.2 | body `body.bounce_loop` @0 {"loop": true, "bpmScale": 1}<br>ear_L `ear_L.listen_beat` @0 {"loop": true, "bpmScale": 1}<br>ear_R `ear_R.listen_beat` @0.6 {"loop": true, "bpmScale": 1}<br>tail `tail.slow_sway` @0 {"loop": true, "bpmScale": 2}<br>eye_L `eye_L.half_lid` @0 {"hold": true}<br>eye_R `eye_R.half_lid` @0 {"hold": true} |
| `playback_pause` | 앱 상태 | 재생 일시정지 | 1.6 |  | full | 0.2 | ear_L `ear_L.perk` @0<br>ear_R `ear_R.perk` @0<br>look `look.at_target` @0.1 |
| `offline` | 앱 상태 | 연결 끊김 | 2.6 |  | full | 0.2 | ear_L `ear_L.swivel` @0<br>ear_R `ear_R.swivel` @0<br>look `look.scan` @0.2<br>tail `tail.low` @0.5<br>eye_L `eye_L.worried` @0.5<br>eye_R `eye_R.worried` @0.5 |
| `hover` | 상호작용 | 커서가 몸 위 | 2.0 | 예 | full | 0.2 | eye_L `eye_L.dilate` @0 {"weight": 0.5, "hold": true}<br>eye_R `eye_R.dilate` @0 {"weight": 0.5, "hold": true}<br>ear_L `ear_L.perk` @0<br>ear_R `ear_R.perk` @0<br>look `look.track` @0 {"omega": 24} |
| `click_poke` | 상호작용 | 클릭 | 0.9 |  | full | 0.2 | body `body.squash_press` @0<br>shape `body.wobble_hit` @0.15<br>eye_L `eye_L.blink` @0<br>eye_R `eye_R.blink` @0 |
| `double_click` | 상호작용 | 더블클릭 | 1.2 |  | full | 0.2 | combo `click_poke` @0<br>body `body.hop` @0.2<br>eye_L `eye_L.double_blink` @0.2<br>eye_R `eye_R.double_blink` @0.2 |
| `rapid_click` | 상호작용 | 연타 — 5회 이상이면 상태 머신이 dizzy로 승격 | 2.4 |  | full | 0.2 | combo `annoyed` @0 |
| `drag_hold` | 상호작용 | 끌어 올림 | 2.0 | 예 | full | 0.2 | body `body.stretch_hang` @0 {"hold": true}<br>ear_L `ear_L.bounce_settle` @0.3 {"repeat": {"every": 0.9}}<br>ear_R `ear_R.bounce_settle` @0.3 {"repeat": {"every": 0.9}}<br>tail `tail.low` @0 {"hold": true}<br>eye_L `eye_L.dilate` @0 {"hold": true}<br>eye_R `eye_R.dilate` @0 {"hold": true} |
| `drop_land` | 상호작용 | 놓음 | 1.6 |  | full | 0.2 | body `body.fall_land` @0<br>ear_L `ear_L.bounce_settle` @0.3<br>ear_R `ear_R.bounce_settle` @0.3<br>body `body.shake_off` @0.6 |
| `shake_hard` | 상호작용 | 빠르게 흔듦 | 2.0 |  | full | 0.2 | combo `dizzy` @0 |
| `stroke` | 상호작용 | 쓰다듬기 — 과자극 분기는 상태 머신(overstim) | 4.0 | 예 | full | 0.2 | combo `pet_purr` @0 |
| `file_drag_over` | 상호작용 | 파일 끌어오기 | 2.0 | 예 | full | 0.2 | look `look.follow_drop` @0<br>ear_L `ear_L.perk` @0<br>ear_R `ear_R.perk` @0<br>shape `body.crouch` @0.2 {"weight": 0.5, "hold": true} |
| `idle_twitch` | idle 변주 | 미세 idle | 0.6 |  | full | 0.2 | ear_? `ear_?.twitch` @0 {"side": "random"} |
| `idle_look_around` | idle 변주 | 두리번 | 3.0 |  | full | 0.2 | look `look.scan` @0<br>ear_? `ear_?.swivel` @0.5 {"side": "random"} |
| `idle_tail_flick` | idle 변주 | 꼬리 끝 | 1.0 |  | full | 0.2 | tail `tail.tip_flick` @0<br>tail `tail.tip_flick` @0.45 |
| `idle_yawn` | idle 변주 | 하품 | 2.4 |  | full | 0.2 | body `body.yawn` @0<br>eye_L `eye_L.closed_line` @0.4<br>eye_R `eye_R.closed_line` @0.4 |
| `idle_slow_blink` | idle 변주 | 느린 깜박임 | 1.6 |  | full | 0.2 | eye_L `eye_L.slow_blink` @0<br>eye_R `eye_R.slow_blink` @0 |
| `idle_groom` | idle 변주 | 그루밍 | 3.2 |  | full | 0.2 | combo `groom` @0 |
| `idle_stretch` | idle 변주 | 기지개 | 2.4 |  | full | 0.2 | body `body.stretch` @0<br>ear_L `ear_L.bounce_settle` @1.4<br>ear_R `ear_R.bounce_settle` @1.4 |
| `idle_deep` | idle 변주 | 깊은 idle | 20.0 | 예 | full | 0.2 | combo `content_loaf` @0<br>combo `sleepy` @8<br>combo `asleep` @13 |
| `acro_show` | 곡예 | 팽이 점프 | 2.0 |  | full | 0.15 | look `look.release` @0<br>body `acro.spin_yaw_360_jump` @0<br>tail `tail.acro_drag` @0 {"timeScale": 1.0}<br>ear_L `ear_L.acro_drag` @0 {"timeScale": 1.0}<br>ear_R `ear_R.acro_drag` @0 {"timeScale": 1.0}<br>shape `body.wobble_hit` @1.1<br>eye_L `eye_L.blink` @1.1500000000000001<br>eye_R `eye_R.blink` @1.1500000000000001<br>look `look.track` @1.4500000000000002 |
| `acro_roll` | 곡예 | 옆구르기 | 2.3 |  | full | 0.15 | look `look.release` @0<br>body `acro.roll_360` @0<br>tail `tail.acro_drag` @0 {"timeScale": 0.7777777777777777}<br>ear_L `ear_L.acro_drag` @0 {"timeScale": 0.7777777777777777}<br>ear_R `ear_R.acro_drag` @0 {"timeScale": 0.7777777777777777}<br>shape `body.wobble_hit` @0.9<br>eye_L `eye_L.blink` @0.9500000000000001<br>eye_R `eye_R.blink` @0.9500000000000001<br>look `look.track` @1.25 |
| `acro_flip_front` | 곡예 | 앞 공중제비 | 2.0 |  | full | 0.15 | look `look.release` @0<br>body `acro.flip_front_360_jump` @0<br>tail `tail.acro_drag` @0 {"timeScale": 1.0}<br>ear_L `ear_L.acro_drag` @0 {"timeScale": 1.0}<br>ear_R `ear_R.acro_drag` @0 {"timeScale": 1.0}<br>shape `body.wobble_hit` @1.1<br>eye_L `eye_L.blink` @1.1500000000000001<br>eye_R `eye_R.blink` @1.1500000000000001<br>look `look.track` @1.4500000000000002 |
| `acro_flip_back` | 곡예 | 뒤 공중제비 | 2.0 |  | full | 0.15 | look `look.release` @0<br>body `acro.flip_back_360_jump` @0<br>tail `tail.acro_drag` @0 {"timeScale": 1.0}<br>ear_L `ear_L.acro_drag` @0 {"timeScale": 1.0}<br>ear_R `ear_R.acro_drag` @0 {"timeScale": 1.0}<br>shape `body.wobble_hit` @1.1<br>eye_L `eye_L.blink` @1.1500000000000001<br>eye_R `eye_R.blink` @1.1500000000000001<br>look `look.track` @1.4500000000000002 |
| `acro_cartwheel` | 곡예 | 옆돌기 | 2.0 |  | full | 0.15 | look `look.release` @0<br>body `acro.cartwheel_360_jump` @0<br>tail `tail.acro_drag` @0 {"timeScale": 1.0}<br>ear_L `ear_L.acro_drag` @0 {"timeScale": 1.0}<br>ear_R `ear_R.acro_drag` @0 {"timeScale": 1.0}<br>shape `body.wobble_hit` @1.1<br>eye_L `eye_L.blink` @1.1500000000000001<br>eye_R `eye_R.blink` @1.1500000000000001<br>look `look.track` @1.4500000000000002 |
| `acro_combo` | 곡예 | 연속 기술 | 6.4 |  | full | 0.15 | combo `acro_show` @0<br>combo `acro_flip_back` @2.1<br>combo `acro_cartwheel` @4.2 |

### 시선(look) 프리셋

| 이름 | 모드 | 옵션 |
|---|---|---|
| `look.track` | cursor | {"doc": "기본 커서 추적(보조)"} |
| `look.track_slow` | cursor | {"omega": 4} |
| `look.avert` | cursor | {"invert": true, "scale": 0.8, "omega": 10} |
| `look.scan` | path | {"path": [[-0.9, 0.1, 0.7], [0.9, 0.15, 0.9], [0, 0, 0.7]], "omega": 8} |
| `look.at_target` | target | {"source": "event", "omega": 14} |
| `look.up_think` | fixed | {"target": [0.3, 0.9], "omega": 8} |
| `look.follow_drop` | target | {"source": "drag", "omega": 16} |
| `look.stare_down` | cursor | {"omega": 40, "freeze": ["tail", "ear_L", "ear_R"]} |
| `eye.saccade` | saccade | {"interval": [0.35, 1.1], "jump": 0.35, "omega": 60} |
| `look.release` | fixed | {"target": [0, 0], "omega": 30, "doc": "v8: 회전 중 look-at 해제(정면 고정); 착지 후 look.track 으로 복귀"} |

### 앱 이벤트 → 프리셋 (state-machine.json)

| 이벤트 | 조합 | 단계 | 규칙 |
|---|---|---|---|
| `app_start` | greet | info | {"firstRunVars": {"entry": "fx.pop_in"}, "vars": {"entry": "fx.peek"}} |
| `app_exit` | goodbye | drag | {"note": "<= 1s (eSheep kill)"} |
| `load_start` | loading | info | {"delay": 0.3, "cancelOn": "load_end", "note": "< 300ms loads skip the animation"} |
| `load_long` | loading_long | info | {"after": {"event": "load_start", "seconds": 5}, "internal": true} |
| `load_end` |  |  | {"stop": ["loading", "loading_long"], "helper": true} |
| `success` | success | info | {"streak": {"count": 3, "combo": "big_success", "resetOn": ["error", "warning"], "bigStreak": {"count": 6, "combo": "acro_cartwheel", "note": "v8"}}} |
| `error` | error | alert | {"fatalCombo": "scared"} |
| `warning` | warning | alert | {} |
| `notification` | notification | info | {"lookTarget": "payload.target"} |
| `typing` | typing_buddy | info | {"trigger": "input", "releaseAfter": 2.0} |
| `voice_listen` | listen | info | {"until": "voice_end"} |
| `music_start` | music_groove | info | {"defaultBpm": 100, "until": "music_stop"} |
| `music_pause` | playback_pause | info | {"stop": ["music_groove"]} |
| `offline` | offline | alert | {"onRecover": {"event": "online", "combo": "success", "weight": 0.5}} |
| `hover` | hover | direct | {"until": "hover_end"} |
| `click` | click_poke | direct | {"doubleWithin": 0.35, "double": "double_click", "rapid": {"count": 3, "within": 1.0, "combo": "rapid_click"}, "overstim": {"count": 5, "within": 2.0, "combo": "dizzy"}} |
| `double_click` | double_click | direct | {} |
| `rapid_click` | rapid_click | direct | {} |
| `stroke` | stroke | direct | {"until": "stroke_end", "overstim": {"after": 8.0, "steps": [{"clip": "tail.tip_flick", "layer": "tail"}, {"combo": "annoyed", "at": 1.0}]}, "rapid": {"count": 3, "within": 2, "event": "pet_rapid", "note": "v8"}} |
| `drag_start` | drag_hold | drag | {"until": "drop"} |
| `drop` | drop_land | drag | {} |
| `shake` | shake_hard | drag | {} |
| `file_drag_over` | file_drag_over | direct | {"until": "file_drag_end"} |
| `idle_30s` |  | idle | {"internal": true, "note": "starts the random idle variants (est 30s)"} |
| `idle_3min` | idle_deep | idle | {"internal": true, "note": "est 180s"} |
| `input_resume` | wake_up | direct | {"onlyIf": "asleep"} |
| `cursor_fast` | startled | direct | {"note": "speed threshold must be tuned (est)"} |
| `cursor_slow` | hunting_stalk | info | {"onlyIf": "playMode", "then": {"after": 3.0, "combo": "pounce"}} |
| `achievement` | acro_show | info | {"note": "v8: 업적/마일스톤"} |
| `task_complete` | acro_flip_front | info | {"note": "v8: 긴 작업 완료"} |
| `celebrate` | acro_combo | info | {"note": "v8: 큰 축하(연속 기술)"} |
| `pet_rapid` | acro_roll | direct | {"internal": true, "note": "v8: 쓰다듬기 연타(stroke 3회/2s) -> 옆구르기"} |

idle 가중치: {"idle_twitch": 30, "idle_slow_blink": 20, "idle_look_around": 15, "idle_tail_flick": 15, "idle_yawn": 8, "idle_stretch": 7, "idle_groom": 5} · 쿨다운 {"same": 20, "big": 60, "bigSet": ["idle_yawn", "idle_stretch", "idle_groom"]} · 직전 2개 제외 · 시작 30s · 깊은 idle 180s
