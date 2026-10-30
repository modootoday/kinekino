---
id: guide:kinekino/mascot-part-layers
title: 마스코트 파트 레이어 애니메이션 구조
status: active
created: 2026-10-29
updated: 2026-10-29
rel:
  governs: [assets/mascot]
---

# 마스코트 파트 레이어 애니메이션 구조

## Overview
파트 레이어(채널 소유) 규칙. 구현: `assets/mascot/src/runtime/composer.js`, `src/presets/layers.json`. 원본: `/workspace/kinekino-brand/model3d/docs/part-layers.md`.

## Steps

### 채널 소유 규칙 (한 채널 = 한 레이어, 겹침 금지)
| 레이어 | 소유 채널 (glTF node.path) | 비고 |
|---|---|---|
| reveal | body.weights[reveal], eyes.scale | 등장/퇴장 envelope 전담. 파트 클립은 이 채널을 쓰지 않음 |
| body | root.scale, root.translation, root.rotation | 숨쉬기·출렁·점프 반동 |
| ear_L / ear_R | ear_L(.tip).rotation / ear_R(.tip).rotation | 좌우 독립 레이어. "양쪽" 클립은 두 레이어에 같은 클립을 미러로 재생 |
| tail | tail_0..3.rotation | 스프링 물리는 클립 뒤 후처리(기존) |
| eye_L / eye_R | eye_X.scale, eye_X.rotation, eye_X 모프 weights(squint, round, wide) | 크기·기울기·모양. 윙크 = 한쪽 레이어만 |
| look | eye_L/eye_R.translation, head.rotation | 시선 전담. 클립 아님, 순수 함수 출력 |
| (후처리) look→ear | ear_* rotation에 가산 오프셋 (±3° 이내) | 클립 평가 후 더함. 유일하게 허용된 가산 |

### 파트 클립 (각각 별도 AnimationClip, 이름 = 레이어/동작)
- ear: perk, flutter, back(rotX 젖힘), perk_L, perk_R (좌우 따로 = 레이어 지정)
- tail: wag, curl, stiff
- eye: blink, wink(L/R), grow, shrink, squint, surprise(wide 모프+scale)
- body: breathe, wobble(출렁), jump_recoil(점프 반동: translation.y + squash/stretch)

### 조합 (src/presets/layers.json, Rust/Three.js 공용)
```json
{ "layers": { "reveal": {"priority": 0}, "body": {"priority": 1}, "ear_L": {"priority": 2}, "ear_R": {"priority": 2},
              "tail": {"priority": 2}, "eye_L": {"priority": 3}, "eye_R": {"priority": 3}, "look": {"priority": 4} },
  "defaults": { "fadeIn": 0.15, "fadeOut": 0.2 },
  "combos": { "startled": [ {"layer":"body","clip":"body/jump_recoil","at":0},
                            {"layer":"ear_L","clip":"ear/back","at":0.05}, {"layer":"ear_R","clip":"ear/back","at":0.05,"mirror":true},
                            {"layer":"tail","clip":"tail/stiff","at":0.1}, {"layer":"eye_L","clip":"eye/surprise"}, {"layer":"eye_R","clip":"eye/surprise"} ] } }
```
규칙: 같은 레이어에 새 클립이 오면 이전 것을 fadeOut으로 크로스페이드(가중치 합 1). 레이어 간에는 채널이 겹치지 않으므로 합성 순서 무관. 높은 priority는 같은 레이어 내 선점 판단에만 사용. 기존 통합 프리셋(startled, tickle, curious, alert, content, ear_perk, ear_flutter, idle_breath)은 combos로 재정의하고 glb에도 베이크된 통합 클립으로 계속 내보냄(호환).

### look-at 순수 함수 (Rust 이식 대상)
```
lookPose(g: [gx, gy] ∈ [-1,1]², p) -> {
  eye_t:  [clamp(gx)*p.eyeRangeX, clamp(gy)*p.eyeRangeY]   // 타원 안에 머무는 범위 제한(원형 클램프)
  head_rot: [gy*p.headPitch, gx*p.headYaw]                // 수 도 이내 보조
  ear_add: [gx*p.earLead, -gx*p.earLead]                  // 반대쪽 귀가 살짝 따라감
}
smooth(state, target, dt, ω) -> 임계감쇠 스프링 1스텝 (결정적, dt 고정 가능)
```
뷰어: pointermove → 캔버스 정규화 좌표 → g → smooth → lookPose → 노드 값 적용.

### glTF 범위
노드 TRS + 모프 weights만 사용(표준). 눈 모프는 eye 메시의 POSITION 타깃 3개. 정지 상태는 모든 weights 0, eyes.scale 0 → 단색 유지.

---
### v7 (2026-10-30): 조사 페이지 전체 구현
원본은 JSON입니다(`src/presets/parts.json`, `layers.json`, `state-machine.json`). 표 형태는 `docs/preset-reference.md`(`python3 tools/gen-preset-reference.py`로 재생성).

#### 개수
- 파트 클립 72종 = glb 클립 64종(배치1 21 · 배치2 28 · 배치3 15) + 런타임 시선 프리셋 8종(`layers.json` looks; glb 클립 없음). 좌우 클립은 L 정의 + `{"mirror": "…_L…"}` 21개. 보조 클립 3개(reveal.eyes, reveal.hide_eyes, fx.overshoot). parts.json 합계 117.
- 조합 52종(기존 10종 포함 62), 앱 이벤트 27종(+보조 load_end) → `state-machine.json`.
- 제외: fx.split_merge(분열·합체, 메시 분리 필요), 하트·별 눈 및 페이지의 제외 8종.

#### 레이어 (채널 소유자는 정확히 1개, `tests/clips.test.mjs`가 검사)
| 레이어 | 채널 |
|---|---|
| reveal | body.weights(reveal, blush), eyes.scale |
| body | root T/R/S (전신) |
| **shape (v7 신규)** | crown, base_L, base_R 디포머 본 T/R/S |
| ear_L / ear_R | ear_X, ear_X_tip 회전 + ear_X.scale |
| tail | tail_0..3 회전 + scale |
| eye_L / eye_R | eye_X T/R/S + 모프 weights |
| look | eye_X_look 위치, head 회전 (+ear 회전 가산) |

**페이지와 다른 점:** 페이지는 knead·crouch·arch·loaf·wobble_hit를 body 레이어로 적었지만, root를 쓰는 breathe·purr와 같이 쓰려면 슬롯이 달라야 해서 `shape` 레이어로 분리했습니다(클립 이름은 `body.*` 유지, 레이어는 parts.json `layer`가 결정. 뷰어·컴포저는 더 이상 이름 앞부분으로 레이어를 추측하지 않음).

#### 몸 디포머
crown(c=[60,34], r=22), base_L(c=[54,88], r=20), base_R(c=[86,72], r=20). smootherstep 감쇠, 꼬리·귀 영향만큼 줄임, 꼭짓점당 최대 3개·합 0.9. 정지 시 항등이라 정적 실루엣 불변(IoU 0.9995 유지). 접힘 검사에서 crouch·loaf 값을 낮춤: crouch crown −1.5 + scale y 0.9, loaf base y 0.94. wobble_hit은 위쪽 움푹 1종만(좌우 변형 없음).

#### 눈
캡슐 외곽 N=64 사용자 메시, 세로 띠 삼각분할. 모프 9종: squint, round, wide, half_lid, closed_happy, glare, worried, dilate, sparkle. glare/worried는 좌우별 기울기(안쪽 +1=L). sparkle은 흰색 COLOR_0 하이라이트 다이아몬드(다른 타깃에서는 한 점으로 수렴). 하트·별 없음. 확인 시트: `shots/v7/eye_capsule_check.png`.

#### 볼 홍조
body 두 번째 COLOR_0 모프 `blush`(#ff7f8f, 볼 2곳 r=5.5). glb에서는 zero POSITION 공용 accessor(bufferView 없음 = 0 초기화) + 희소(sparse) COLOR_0로 저장.

#### 조합 형식 (v7)
`[layer, clip, at, opts?]`. opts: weight, timeScale, loop, hold(holdAt에서 정지, `release(layer)`로 재개), repeat {every, jitter}, side "random"(레이어·클립의 `?`를 L/R로), bpmScale(timeScale = bpm/100 × bpmScale), trigger(예: typing_bob은 `composer.trigger('input')`마다). `["combo", 이름, at]`은 중첩(시작 시 이전 레이어 정지), `["look", 이름, at]`은 시선 프리셋. 조합 필드: reveal "full"|"eyes", loop, vars(`$entry`), blend(기본 0.2, startled·angry_puff 0.05). 전개는 순수 함수 `src/runtime/combo-schedule.js`(Rust 이식 대상).

#### 상태 머신 (`src/runtime/state-machine.js`)
순수 JS, 시간·난수(`rng.js` mulberry32 시드) 주입. 단계 drag 5 > direct 4 > alert 3 > info 2 > idle 1. 높은 단계가 끊고 같은 단계는 새 것이 이김. **레이어 단위**: 새 조합은 자기가 쓰는 레이어만 가져가고(예: notification은 ear·reveal·look만, music_groove의 body 유지), 낮은 단계는 더 높은 단계가 아직 쥔 레이어가 하나라도 있으면 거절. 규칙: 로딩 300ms 미만 생략·5s 후 loading_long, 성공 3연속 big_success(오류·경고로 초기화), 치명 오류 scared, 클릭 0.35s 더블·1s 3회 연타·2s 5회 dizzy, 쓰다듬기 8s 후 tail.tip_flick → annoyed, 타이핑 2s 후 해제, 오프라인 복귀 시 약한 success(가중치 0.5), 놀이 모드에서만 hunting_stalk → 3s 후 pounce, 무입력 30s idle 변주(가중치 30/20/15/15/8/7/5, 같은 변주 20s·큰 동작 60s 쿨다운, 직전 2개 제외), 3분 idle_deep, 잠든 뒤 첫 입력은 wake_up만.

#### 검사 (`npm test`, 30개)
클립 시작·끝 정지 자세(restExempt: entry/exit/transition/revealed), 레이어 소유, 채널 중복 소유 없음, wrap_body·groom·tuck·heart_curve 꼬리 관통 없음(꼬리 우세 꼭짓점이 몸 두께 안쪽 15%보다 깊이 들어가면 실패하는 근사 검사), 디포머 삼각형 뒤집힘 없음, 조합 전개, 상태 머신(우선순위·레이어 인터럽트·쿨다운·시드 재현).

#### 크기
glb 4.90MB(v6) → 5.17MB(v7). 처음 6.30MB였으나 몸 모프의 0 POSITION 델타를 bufferView 없는 accessor로, COLOR_0를 희소로 바꾸고 쓰지 않는 bufferView를 제거, 파트 클립 키를 선형 보간 허용오차 2e-4로 줄임(`reduceKeys`).

#### 위험·한계
- 관통 검사는 격자 근사(셀 0.02, 여유 15%)라 스치는 수준의 겹침은 잡지 않음.
- 클릭 연타·쓰다듬기 시간, idle 시간, 가중치, BPM 100은 페이지의 추정치.
- three r170 셰이더는 POSITION 모프가 있어야 색 모프를 켜므로 0 POSITION accessor를 유지.

### v8 (2026-10-30)
#### 꼬리 파고듦 수정
- 원인: tail.stiff/question/quiver/thump/heart_curve 의 위쪽 rotZ 가 꼬리를 몸 평면 안에서 위로 들어 올려 몸 볼륨 속으로 넣었고, 뿌리 웨이트 감쇠가 좁고 급했음.
- 수정: `src/rig/skinning.js` TAIL_FALLOFF_Y 0.4→0.7, 램프 시작을 몸 안쪽 8 SVG 단위(TAIL_ROOT_LEAD)로, X 램프 폭 0.6→0.8, smootherstep. 위 5개 클립에 tail_0 rotY 들어올림(min(40°, 2.5×위쪽 rotZ)) 추가.
- 검사: `tools/tail-penetration.mjs` (로고 다각형 × pillowZ 볼륨 안의 꼬리 정점 비율), `tests/clips.test.mjs`가 모든 꼬리 클립 0% 요구.
#### acro.* (body 레이어, root 회전) — 모두 360°의 정수배, 정지 자세로 시작·끝
spin_yaw_360/720, flip_front_360, flip_back_360, cartwheel_360 (+ `_jump`), roll_360. 꼬리·귀 끌림 `tail.acro_drag`, `ear_L/R.acro_drag`; 착지 `body.wobble_hit`; `look.release` 후 착지+0.35s `look.track`.
조합: acro_show, acro_roll, acro_flip_front, acro_flip_back, acro_cartwheel, acro_combo. 이벤트: achievement, task_complete, celebrate, stroke.rapid→pet_rapid, success.streak.bigStreak(6연속).
#### camera 레이어 (런타임 전용, `src/presets/camera.json` + `src/runtime/camera.js`)
glb 아님: 코어 glTF 는 KHR_animation_pointer 없이는 카메라 yfov 애니메이션 불가(three r170 로더 미지원), 돌리 줌은 FOV+거리 동시 제어 필요, 설정 페이지 값으로 바로 조정 가능해야 함. 고정 줌(holdEnd): zoom_in_slow/fast, slow_push → `cam.reset`으로 해제.
#### 카메라 ↔ 상태 머신 (v8.1)
`new StateMachine(cfg, layers, { camera: camera.json })` 이면 dispatch/end 결과에 `{type:'camera', name, at, tier, source}` 가 붙음 (`camStep`). 규칙(`camera.json` rules): 곡예 조합은 자기 카메라 트랙을 내보내고 조합 길이 동안 카메라를 소유(그 사이 같은·낮은 단계 이벤트 카메라는 버림), 그 외엔 단계 ≥ 현재일 때 교체, 이벤트별 쿨다운, 본체가 거절한 이벤트는 카메라도 없음, holdEnd 클립은 cam.reset(hover_end)까지 유지, 곡예 중 reset 무시.
