---
id: research:kinekino/20261029161022
title: 마스코트 고양이 모션 리서치와 심볼 매핑
status: active
created: 2026-10-29
updated: 2026-10-29
rel:
  governs: [assets/mascot]
---

# 마스코트 고양이 모션 리서치와 심볼 매핑

## Question
귀·눈·꼬리만 가진 kinekino 심볼이 실제 고양이 모션 근거를 어떻게 따를 것인가. 원본: `/workspace/kinekino-brand/model3d/docs/cat-motion-research.md`(2026-10-30 KST 흡수).

## Method
공개 문헌·영상 관찰을 출처 ID(R*)로 정리하고 `assets/mascot/src/presets/presets.json`의 `evidence` 필드로 연결.

## Findings
범위: 귀·눈·꼬리만 사용합니다(실사 고양이가 아님). 출처가 없는 값은 **추정**으로 표시했습니다. 증거 ID는 presets.json의 `evidence` 필드와 같습니다.

### 출처 (R) 
| ID | 내용 | 수치 | URL |
|---|---|---|---|
| R1 | Gruart et al. 1995, 고양이 눈깜박임 운동학 | 자발 깜박임 0.2–0.5회/분, 감김이 뜨기보다 약 10배 빠름, 에어퍼프 감김 25–30ms | https://doi.org/10.1152/jn.1995.74.1.226 |
| R2 | 고양이 깜박임 빈도 연구 | 2.4회/분(약 50% 불완전) · 5.0±2.3회/분 · 4.1±2.7회/분 | https://doi.org/10.1111/j.1748-5827.1987.tb01347.x · https://doi.org/10.1111/vop.13051 · https://pmc.ncbi.nlm.nih.gov/articles/PMC8962689/ |
| R3 | Humphrey et al. 2020, slow blink | 반깜박임이 이어진 뒤 눈을 오래 가늘게 뜨거나 감음 = 긍정 신호 | https://www.nature.com/articles/s41598-020-73426-0 |
| R4 | Heads and Tails 2021 | 귀를 세우면 긍정, 납작/뒤로 젖히면 부정; 사람에게 다가갈 때 꼬리를 세움 | https://pmc.ncbi.nlm.nih.gov/articles/PMC8469685/ |
| R5 | Fear Free / ASPCA 자세 가이드 | 관심 있을 때 귀가 약간 앞으로, 불안할 때 옆으로 회전, 공포 시 납작; 근육 32개, 귀 회전 약 180° | https://www.fearfreehappyhomes.com/ears-to-you-listen-to-what-your-cats-ears-say/ · https://www.aspcapro.org/sites/default/files/aspca_felineality_cat_body_postures_0_0.pdf |
| R6 | J Neurosci 1998, 귓바퀴 반사 | 지연 약 25ms, 비대칭(같은 쪽이 더 크게 움직임) | https://www.jneurosci.org/content/18/11/4233 |
| R7 | 전정-귓바퀴 반사 | 머리 기울임에 귀가 반응 | https://pmc.ncbi.nlm.nih.gov/articles/PMC2666401/ |
| R8 | 안정 시 호흡수 | 16–40/분(Merck), 15–30/분(VCA), 수면 중앙값 19–21/분 | https://www.merckvetmanual.com/reference-values-and-conversion-tables/reference-guides/resting-respiratory-rates · https://vcahospitals.com/know-your-pet/home-breathing-rate-evaluation · https://pmc.ncbi.nlm.nih.gov/articles/PMC11383104/ |

### 추정 (E)
| ID | 값 | 비고 |
|---|---|---|
| E1 | 귀 털기(flutter) 약 11–12Hz, 감쇠 진동 | 영상 관찰 기반 **추정**, 1차 출처 없음 |
| E2 | 모든 회전각(귀 ±8–25°, 꼬리 ±9°, 머리 기울임 9°), 꼬리 흔들기 약 0.4Hz | 심볼 가독성 기준 **추정** |
| E3 | 호흡 스케일 진폭 2%, startled 꼬리 세움 정도 | **추정** |

### 심볼 매핑
| 프리셋 | 귀 | 눈 | 꼬리 | 근거 |
|---|---|---|---|---|
| idle_breath | – | 뜬 상태 유지 | – (전체 스케일 2%, 약 0.35Hz ≈ 21회/분) | R8, E3 |
| alert | 빠르게 세운 뒤 안정 | 뜬 상태 | 정지 | R4, R5, R6, E2 |
| curious | 한쪽이 먼저 앞으로(비대칭) | 뜬 상태 | 끝만 천천히 | R5, R6, R7, E2 |
| tickle | 끝부분 flutter(좌우 위상차) | 빠른 깜박임 | – | R6, E1 |
| content | 약간 이완 | slow blink(반깜박임 뒤 길게 감음) | 느린 sway | R3, R1, E2 |
| startled | 순간 위로 | 크게 뜸, 감김 약 30ms 감각 | 위로 세움 | R1, R4, E3 |
| blink (공통) | – | 감김:뜨기 ≈ 1:3 (R1의 약 10:1은 가독성을 위해 완화) | – | R1, E2 |
