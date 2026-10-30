---
id: sot:kinekino/settings-ui-design
title: 설정 창 디자인 정본과 시안 대응
status: active
created: 2026-10-30T04:02:16Z
updated: 2026-10-30T04:32:45Z
verify: [cargo test -p kinekino-settings-ui]
---

# 설정 창 디자인 정본과 시안 대응

## Rule

설정 창(`crates/kinekino-settings-ui`)의 색·글꼴·간격·아이콘·구성요소는 아래 정본에서만 온다. 임의 값은 쓰지 않는다. 시안에 있는 화면·구성요소는 시안과 같은 구조·토큰·자산·간격·상태로 만들고, 시안이 없는 화면은 시안의 토큰·구성요소에서 파생한다.

### 우선순위

디자인 일관성 순서를 따른다: 사용자 지시 → 공통 프레임·토큰·아이콘·용어·상태 계약 → 탭별 요소 계약 → 이미지. 이미지는 정본 값을 바꾸지 않는다.

### 정본 표

| 요소 | 정본 파일 | 이유 |
|---|---|---|
| 색·간격·크기·글꼴 크기·반경 | `crates/kinekino-player/src/presentation/web_ui/styles/tokens.css` (`design_tokens.rs`의 웹 투영) | 상위 문서(공통 토큰 계약). 시안 이미지보다 앞선다 |
| 버튼·입력·아이콘 마스크 | `.../styles/components.css` (`aw-button`, `aw-button-compact`, `aw-button-outline-accent`, `aw-text-input`, `aw-icon`) | 공통 구성요소 계약, 플레이어와 같은 파일 |
| 글꼴 | `assets/fonts/pretendard` (Regular·SemiBold·JP, OFL-1.1) | tokens.css `@font-face`가 가리키는 자산 |
| 아이콘 | `assets/design/icons`, `assets/design/window` | 이미 도입한 아이콘 세트(아래) |
| 문구 | 한국어, `kinekino-app` 페이지 모델의 `label` | 용어 계약; 화면이 문구를 새로 짓지 않음 |

### 아이콘 라이브러리

도입된 세트는 외부 라이브러리가 아니라 저장소가 직접 그린 SVG다(`assets/design/README.md`: "Directly authored project SVGs… not copied library artwork", 24×24, stroke 1.75, round cap/join). `screens.md`·`ui-ai`·`potplayer-ui` SoT의 "Phosphor Regular"는 시안 생성 프롬프트의 **양식** 지시이며 의존성이 아니다(저장소·docs/_includes에 lucide·Phosphor 패키지 없음). 설정 창은 `undo`, `warning`, `info`, `brand`만 쓴다. 새 SVG는 추가하지 않았다.

### 시안 대응

| 시안 | 구현 | 비고 |
|---|---|---|
| 앱 설정 화면 | 없음 → 파생 | `v2/studio/skip-settings.png`는 OP/ED 스킵 설정이며 앱 설정이 아니다 |
| 패널 배경·머리줄 | `aw-settings-root`(`--aw-surface-panel`), `aw-settings-header`(`--aw-surface-chrome`, 높이 `--aw-primary`) | 파생: 인스펙터 패널·caption 토큰 |
| 필드 행 | `aw-settings-row`: 라벨 열 `--aw-navigation`, 행 높이 `--aw-primary`, 구분선 `--aw-border` | 파생. `--aw-field-label-width`(72px)는 한국어 라벨(최대 10자)에 좁아 쓰지 않음 |
| 텍스트·숫자·목록 입력 | `aw-text-input` | 공통 구성요소 그대로 |
| 선택(choice) | `aw-button aw-button-compact` 묶음 + 현재 값 `--aw-accent` | 파생: 앰버 선택 표시 |
| 토글 | checkbox | 시안에 스위치 구성요소 없음 |
| 오류·한도 대화상자 | `--aw-warning-surface`/`--aw-warning-border` + `warning` 아이콘 | 경고 상태 토큰 |
| 하단 동작 | `되돌리기`(`undo`), `적용`(`aw-button-outline-accent`), `닫기` | 주 동작 강조는 공통 규칙 |
| 창 caption | OS 프레임 | `aw-window-chrome` 미사용(편차, ADR 참조) |

### 충돌과 선택

| 충돌 | 후보 | 선택(근거) |
|---|---|---|
| 아이콘 출처: "Phosphor Regular"(screens.md·SoT 프롬프트) vs 자체 SVG(assets README) | Phosphor 패키지 도입 / 자체 SVG | 자체 SVG — 실제 자산이며 index 우선순위상 아이콘 계약이 이미지 프롬프트보다 앞섬 |
| 라벨 열 폭: `--aw-field-label-width` 72px vs 한국어 라벨 길이 | 72px(잘림) / `--aw-navigation` 180px | `--aw-navigation` — 증거로 확정할 수 없는 파생값. 권고안으로 진행, 설정 시안이 생기면 대체 |
| 창 프레임: 제품 caption vs OS 프레임 | `aw-window-chrome` / OS | OS 프레임 — 설정 창은 모달 보조 창이고 사용자 정의 caption은 Meari 창 드래그·hit-test 연동이 필요. 권고: 플레이어 셸과 함께 제품 caption으로 통일 |

## Why

이전 설정 창(`gui.html`)은 토큰·아이콘·글꼴을 하나도 쓰지 않았다. 정본을 한 표로 두면 시안이 없는 화면도 같은 값에서 파생되고, 시안이 생기면 대응 표만 갱신하면 된다.

## Verification

- `crates/kinekino-settings-ui/tests`: settings.css의 모든 `var(--aw-…)`가 tokens.css에 있고 색 리터럴(`#`, `rgb(`)이 없다.
- 헤드리스 렌더(PNG)와 xvfb 창 스모크, Windows 실제 창 스크린숏.
