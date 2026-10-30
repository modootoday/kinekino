---
title: "스튜디오: 타임라인, 클립, 리뷰, 스냅샷"
description: "구간을 표시하고 클립과 스냅샷을 지속 작업으로 만들고, 모든 변경을 리뷰합니다."
ref: studio
nav_order: 11
hero: { c: hero, visual: player-mock, scene: timeline }
sections:
  - { c: features, cols: 2 }
  - { c: split, visual: player-mock, scene: review }
  - { c: prose }
---

프로젝트에는 미디어, 구간, 결과물이 들어 있습니다. 변경마다 리비전이 있어 두 도구가 같은 프로젝트를 고쳐도 서로 덮어쓰지 않습니다.

## 만들기

- **구간 초안**: 보면서 시작과 끝 표시. `status:partial`
- **쇼츠와 클립**: 구간을 쇼츠로 렌더. `status:partial`
- **카드와 에셋**: 카드 내보내기, 에셋 팩 빌드. `status:partial`
- **프레임 캡처**: FFmpeg로 한 프레임을 PNG로. `status:partial`
- **스냅샷**: 잘라서 내보내기, 지금은 Windows, Linux는 P4. `status:partial`
- **피사체 분할과 투영 패키지**: 설계만 됨. `status:planned`

## 바뀌기 전에 리뷰

AI와 스크립트는 프로젝트를 직접 고치지 않습니다. 제안을 남기고, 사람이 리뷰에서 하나씩 수락하거나 거절합니다. 수락된 변경은 출처가 남습니다.

- 미디어와 재생 리뷰 `status:partial`
- 제안 수락·거절 `status:partial`
- 편집 출처 기록 `status:partial`

## 지속 작업

렌더는 작업으로 돕니다. 제출, 지켜보기, 취소, 다시 시도. 같은 멱등 키로 다시 보낸 요청은 두 번 실행되지 않습니다.

```sh
kinekino jobs submit --input render-short.json
kinekino jobs watch JOB_ID
```
