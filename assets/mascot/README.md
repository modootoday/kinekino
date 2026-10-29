# kinekino 마스코트 3D 모델 (`assets/mascot`)

kinekino 심볼(귀·눈·꼬리 고양이)의 3D 리그, 프리셋, 런타임, 그리고 GLB 산출물을 담은 자산 패키지입니다. Rust 크레이트가 아니라 Node(three.js) 도구 체인이며, 앱은 `out/kinekino-mascot.glb`과 `src/presets/*.json`을 자산으로 소비합니다.

## 출처와 관계
- 원본: `/workspace/kinekino-brand/model3d` (git 아님). 2026-10-30 KST에 프리셋·카메라·곡예·꼬리 보정 v8 상태를 **복사**로 흡수했습니다. 링크가 아닙니다.
- 이후의 정본은 이 디렉터리입니다. 원본 폴더는 삭제하지 않고 보존합니다(중간 산출물 `shots/`, `out/video`, `out/variants`, v7 백업 GLB는 원본에만 있습니다).
- 문서는 specfold로 옮겼습니다:
  - `.spec/research/20261029161022-mascot-cat-motion.research.md`
  - `.spec/guide/mascot-part-layers.guide.md`
  - `.spec/guide/mascot-preset-reference.guide.md` (`python3 tools/gen-preset-reference.py`로 생성)

## 구성
| 경로 | 내용 |
|---|---|
| `src/presets` | 파트·조합·카메라·상태 머신 JSON(단일 원천)과 클립 빌더 |
| `src/rig`, `src/geometry`, `src/scene`, `src/animation`, `src/physics` | 골격·스키닝, 형상, 장면 조립, 트랙, 꼬리 스프링 |
| `src/runtime` | 레이어 컴포저, 시선, 카메라, 콤보 스케줄, 상태 머신 |
| `tests` | `npm test` (node --test, 50개) |
| `tools` | GLB 내보내기, 렌더(헤드리스 Chrome 1개, nice), 시트 합성 |
| `viewer` | 브라우저 뷰어 (`python3 tools/serve.py`) |
| `out/kinekino-mascot.glb` | 현재 GLB (약 5MB, LFS 없이 일반 커밋) |
| `sheets/` | 대표 키프레임 시트만 선별 |

## 사용
```bash
npm ci && npm test
node tools/export-glb.js                       # out/kinekino-mascot.glb
bash tools/render-v7.sh "combo:acro_show@0,0.35,0.5,0.65,0.8,0.95,1.15,1.9"
ANGLES=15,40 bash tools/render-v7.sh "combo:acro_show@0,0.5,1.15"   # 정면 + 15°/40° 확인 컷
ANGLES=15,40 python3 tools/make-sheets-v8.py
```
- 렌더와 시트의 기본 카메라는 **정면(yaw 0°, tilt 0°)** 입니다. `YAW`/`TILT`로 바꿀 수 있습니다.
- `ANGLES`를 주면 각 각도마다 첫 시각의 컷 한 장을 더 렌더하고, 시트 오른쪽에 별도 칸으로 덧붙입니다(입체 확인용).
- `sheets/acro_show.png`는 정면 + 15°/40° 칸 형식입니다. 나머지 시트는 v8 당시(15° 기본값)에 렌더한 것입니다.
