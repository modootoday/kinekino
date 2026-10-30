---
id: sot:kinekino/design-asset-usage
title: 디자인 에셋 사용 현황과 정본 시안 대응
status: draft
created: 2026-10-30T06:45:01Z
updated: 2026-10-30T06:45:01Z
rel:
  governs: [assets/design]
---

# 디자인 에셋 사용 현황과 정본 시안 대응

## Rule

## Why

## Verification
## 범위

`assets/design/`(런타임 SVG)와 비공개 시안(시안 이미지·데모 표본, 저장소 밖 로컬 `.spec.local`에만 보관)의 실제 참조 현황과 화면별 정본 시안 초안이다. 설정 창의 정본은 [설정 창 디자인 정본과 시안 대응](settings-ui-design.sot.md)이 정하며 여기서는 반복하지 않는다. 우선순위는 사용자 지시 → 공통 토큰·아이콘·용어 계약 → 탭별 계약 → 이미지 순이다. 시안 파일명은 아래 표처럼 시안 묶음 안의 상대 경로로만 적고 링크하지 않는다(공개 문서가 로컬 문서를 가리키지 않는다).

조사 방법(2026-10-30 main): 파일명과 확장자 뺀 이름을 `git grep`으로 `crates packages xtask tools`(코드)와 `.spec docs README.md assets`(문서)에서 찾았다. 아이콘은 `crates/kinekino-player/src/presentation/icon_asset.rs`의 `include_bytes!(concat!(".../icons/", $name, ".svg"))` 매크로와 `web_ui/assets/requirements.json`으로 이름만 참조되므로 이름 검색을 함께 썼다.

## 코드에서 참조되지 않는 에셋

| 에셋 | 문서 참조 | 판단 |
|---|---|---|
| `demo/anime/traveler-layer-v1.png`(비공개, 옛 `assets/design/demo`) | 비공개 시안 묶음 | 데모 표본. 런타임 미사용. 삭제하지 않고 로컬로 옮김 |
| `demo/landscape.svg`(비공개) | 비공개 시안 묶음 | 위와 같음 |

2026-10-31: 데모 표본(`demo/`)은 공개 트리에서 빠졌다. 기존 `kinekino-player`는 실행 시 `.spec.local` 또는 `$KINEKINO_DEMO_ASSETS`에서 읽고, 없으면 자리표시로 그린다.

`assets/design/icons/*`, `assets/design/window/*`는 모두 `icon_asset.rs` 또는 `requirements.json`에서 참조된다. 시안 이미지는 코드가 참조하지 않는다. 시안 묶음(index·manifest·화면 색인)은 비공개 로컬로 옮겼다.

## 중복 시안(같은 SHA-256)

| 보관본 | 현행 | 정본 |
|---|---|---|
| `archive/ui-tabs/{convert,ocr,short,subject,subtitles}.png` | `v2/studio/` 같은 이름 | `v2/studio/` |
| `archive/viewing/native-player-{fullscreen,playback,skip,start,studio,web}-v2.png` | `v2/viewing/` 같은 이름 | `v2/viewing/` |
| `archive/viewing/native-player-design-v1.png` | 없음 | 보관 전용(v2가 대체), 정본 아님 |

보관본은 이력 링크 때문에 유지한다. 새 문서는 `v2/` 경로만 가리킨다.

## 화면별 정본 시안(초안)

규칙: 같은 화면에 `-refined`가 있으면 `-refined`가 정본, 기본본은 비교용. `review/*-rNN-*`은 세부 요소 확대 검토본으로 정본이 아니라 검토 근거다.

| 화면·섹션 | 정본 | 비교·이전본 |
|---|---|---|
| 시작 화면 | `v2/viewing/native-player-start-v2.png` | `archive/viewing/native-player-design-v1.png` |
| 재생 | `v2/viewing/native-player-playback-v2.png` | |
| 전체 화면 | `v2/viewing/native-player-fullscreen-v2.png` | |
| 스킵 오버레이 | `v2/viewing/native-player-skip-v2.png` | |
| 스튜디오 진입 | `v2/viewing/native-player-studio-v2.png` | |
| 웹 탐색 | `v2/viewing/native-player-web-v2.png` | |
| 다운로드 | `v2/studio/download-refined.png` | `v2/studio/download.png` |
| 모델 | `v2/studio/models-refined.png` | `v2/studio/models.png` |
| OCR | `v2/studio/ocr-refined.png` | `v2/studio/ocr.png` |
| 3D 투영 | `v2/studio/projection3d-refined.png` | `v2/studio/projection3d.png` |
| 작업 큐 | `v2/studio/queue-refined.png` | `v2/studio/queue.png` |
| 숏폼 추천 | `v2/studio/short-recommend-refined.png` | `v2/studio/short-recommend.png` |
| 스냅샷 | `v2/studio/snapshot-refined.png` | `v2/studio/snapshot.png` |
| 2D 에셋 | `v2/studio/asset2d.png` | |
| 카드 | `v2/studio/cards.png` | |
| 변환 | `v2/studio/convert.png` | |
| 보정 | `v2/studio/correction.png` | |
| 내보내기 | `v2/studio/export.png` | |
| 투영 패키지 | `v2/studio/projection-package.png` | |
| 원본 프레임 | `v2/studio/raw-frame.png` | |
| 숏폼 일괄 | `v2/studio/short-batch.png` | |
| 숏폼 | `v2/studio/short.png` | |
| OP/ED 스킵 설정 | `v2/studio/skip-settings.png` | 앱 설정 창과 다름(settings-ui-design) |
| 대상(subject) | `v2/studio/subject.png` | |
| 자막 | `v2/studio/subtitles.png` | |
| 앱 설정 창 | 시안 없음 → 파생 | [settings-ui-design](settings-ui-design.sot.md) |

미결: 시안끼리 다른 요소(예: 상단 caption, 탭 바 높이, 아이콘 양식)의 요소별 정본은 settings-ui-design의 "충돌과 선택" 방식으로 화면 이관 시 각 SoT에 기록한다. 이 표는 화면 단위 정본만 정한다.

## 플레이어 화면 정본(2026-10-31 확정)

원 님 결정(2026-10-31 00:54): 영상은 libmpv 렌더 API로 Meari 텍스처에 그리고 Linux·Windows 플레이어 모두 Meari UI를 쓴다. 아래는 플레이어 화면의 요소 단위 정본이다. 시안이 있는 요소는 시안대로, 없는 요소는 [디자인 토큰](design-tokens.sot.md)·기존 Meari 컴포넌트 규칙에서 파생하고 "파생"으로 표시한다.

### 화면 단위

| 화면 | 정본 시안 | 1차 구현 범위 |
|---|---|---|
| 시작(미디어 없음) | `v2/viewing/native-player-start-v2.png` | 예 |
| 재생 + 회차 목록 패널 | `v2/viewing/native-player-playback-v2.png` | 예 |
| 전체 화면 | `v2/viewing/native-player-fullscreen-v2.png` | 예(컨트롤 자동 숨김) |
| 스킵 오버레이 | `v2/viewing/native-player-skip-v2.png` | 후속(구간 데이터 연결 후) |
| 스튜디오 진입 | `v2/viewing/native-player-studio-v2.png` | 후속(스튜디오 이관 배치) |
| 웹 탐색 | `v2/viewing/native-player-web-v2.png` | 후속(P3 BrowserProviderPort, UI에 WebView 금지) |

### 요소 대응

| 요소 | 근거 | 규칙 |
|---|---|---|
| 제목 표시줄 | start/playback 시안 | 좌: 제품 마크 + 제품명 + `·` 구분 미디어 제목, 우: `window/{minimize,maximize,restore,close}.svg` |
| 제품명 | 파생 | 시안 속 제품명 문자열은 예시 데이터이며 쓰지 않는다. 표시 문자열은 `kinekino`, 마크는 `icons/brand.svg` |
| 영상 영역 | playback 시안 | libmpv 렌더 결과 텍스처, 종횡비 유지 letterbox 검정 |
| 시작 화면 중앙 | start 시안 | 원형 play 아이콘(`play.svg`, 비활성 중성색), 제품명, `파일 열기`(folder, amber 외곽=기본) · `작품 탐색`(search, 중성 외곽), 힌트 `Ctrl+O로 파일 열기` |
| 탐색 막대 | playback 시안 | design-tokens MediaRail(amber 채움·원형 thumb, track3). 미디어 없음은 비활성 SECONDARY |
| 하단 컨트롤 | playback 시안 | 좌 `previous`·`play/pause`·`next`, 시간 `mm:ss / mm:ss`(현재 강조, 전체 SECONDARY, 1시간 이상 `h:mm:ss` 파생); 우 `volume`/`volume-zero` + MediaRail 볼륨, `list`(회차 패널 토글), `fullscreen`, `more-vertical`, 구분선 뒤 백엔드 표시 |
| 백엔드 표시 | playback 시안 `네이티브` | 파생: 값은 현재 `PlaybackBackend` 이름(`mpv`), 문구는 한국어 라벨 규칙 |
| 회차 목록 패널 | playback 시안 | 우측 고정 폭, 헤더 `회차 목록` + close, 작품명 소제목, 행 `번호 제목`; 현재 행 amber 좌측 막대·amber 글자·`play` 표식 |
| 전체 화면 | fullscreen 시안 | 제목 표시줄·패널 숨김, 하단 컨트롤 오버레이, 입력 없으면 숨김(지연 시간은 파생: 기존 토큰의 motion 값 사용) |
| 아이콘 | 기존 `assets/design/icons`(Phosphor Regular outline) | 플레이어 1차 범위에 신규 SVG 불필요. 추가 시 출처·라이선스·이유를 이 표에 기록 |
| 글꼴 | design-tokens | OFL 글꼴만 |

### 실기기 한계

헤드리스 CI는 소프트웨어 렌더 프레임 대조까지만 증명한다. 실제 GPU 렌더·HDR·화면 재생률·창 관리자 상호작용·시안과의 육안 대조는 실기기(Linux 데스크톱, Windows build-host)가 필요하다. macOS는 GitHub Actions 러너 빌드·헤드리스 시험으로만 대체한다.
