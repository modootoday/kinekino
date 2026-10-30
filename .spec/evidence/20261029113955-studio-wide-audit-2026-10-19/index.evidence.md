---
id: evidence:kinekino/20261029113955
title: 제작 전체 순회 r158
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제작 전체 순회 r158

2026-10-19. 특정 화면의 미세 변경만 반복하지 않도록 `.spec/artifacts/design/manifest.json`의 선택 시안과 현재 GUI를 화면군 전체에서 다시 대조했다. 제품 소스를 수정하거나 새 빌드를 실행하지 않았다. 고정 r157 실행본을 새 검토 경로에 복사했고 SHA가 동일함을 확인했다. 이 기록은 새로운 전체 시각 완성 판정이 아니다.

실행본 `runtime/review-runs/ui-studio-audit-20261019-r158/player.exe`, PID96464, SHA256 `de4163dccfe07922294384488601062c5daa984bd9e3813c811bb2b9a34aa515`. 이전 검토 창 없음 확인 뒤 가상 모니터3500,100에서 하나 실행했다. VM·DPI 변경 없음. 마지막 소스 체크는 r157의139pass9ignored/fmt/clippy/build이며 이 문서 작성으로 새 검사를 수행했다고 주장하지 않는다.

## 실제 관측 범위

1440×900, DesignSample Populated. 12개 제작 workspace와 카탈로그상25개 탭을 실제 클릭/관측했다. 별도 탭 버튼이 없는 단일 화면도 한 항목으로 센다. 하단 스크롤 전체·모든 hover/pressed·Empty/LongLabels/NeedsInput·760/980/1180·실제 Live backend·시청 화면은 이번 순회 범위가 아니다. 모든 기능 동작 또는 전 요소 확대1:1 검증을 완료했다는 뜻이 아니다.

| 화면군/진입 수 | 기준 이미지(v2/studio/) | 실제 관측과 남은 큰 차이 |
|---|---|---|
| SEG1 | skip-settings.png | 중앙 contain 프리뷰·filmstrip·구간 표는 존재. 프리뷰 바깥 빈 면과 작은 텍스트 transport가 시안의 전폭 검정 프레임/transport와 다르다. OP/ED별 색·복합행·검사기 그룹도 후속. |
| SUB5 | subtitles.png / ocr-refined.png / correction.png / convert.png; 화자 전용 시안 없음 | 음성/화자/OCR/교정/변환 각각 진입했다. OCR ROI와 결과 탭, 교정 비교, 변환 두 문서는 구분된다. 다섯 화면 모두 기존 작은 transport를 사용한다. 시안의 하단 체크/편집 셀과 검사기 radio·옵션 그룹, 소스 정보 배치는 후속이다. 화자 화면의 전용 시안 미제공 안내를 확인했다. |
| OBJ1 | subject.png | 합성·mask 경계/선택 frame·도구/피사체행 존재. 프리뷰/transport는 기존 경로로 시안과 면의 사용이 다르다. fixture 인물과 시안 고양이의 차이를 실제 탐지 오답으로 취급하지 않는다. |
| CLIP3 | short.png / short-recommend-refined.png / short-batch.png | 수동은 세로 crop/원본/3track 구성이나 원본 영상 transport가 없다. AI는 공용 검정 프리뷰/transport와 후보3행. 일괄은 공용6열 source표/계획 요약이나 시안의 아이콘 요약·선택 개수와 다르다. |
| FRAME2 | raw-frame.png / snapshot-refined.png | 원본은 큰 이미지와 이전/현재/다음 썸네일·동작, 편집은 제목/grid와3썸네일. 텍스트 선택 경계/레이어·소스 미리보기 상세·시안 제목의 광학 크기는 차이. |
| CARD1 | cards.png | 도구/3카드 rail/정사각 canvas/선택 텍스트 inspector 구성. 시안의4개 예시와 fixture3개는 데이터 차이이며 임의 증식하지 않는다. 제목 크기·소스/레이어·설정 밀도는 후속. |
| A2D1 | asset2d.png | 도구/3보기모드/선택경계/pivot·layer/source 참조와 inspector 구성. 현재 합성 모드만 관측했으며 원본/알파 모드와 편집 동작은 이번에 재검사하지 않았다. 생성 시안의 원본 선택+alpha checkerboard처럼 모순된 상태는 기능 규격으로 복제하지 않는다. |
| PROJ2 | projection3d-refined.png / projection-package.png | 깊이 plane/grid/축/source 참조, 패키지 tree/검증표 구성 존재. 깊이 화면의 낮은 viewport/넓은 하단 빈 공간과 카메라 제어·보기 선택기 배치가 시안과 다르다. 패키지는3열 구조이며 실제 파일/검증 성공을 뜻하지 않는다. |
| SAVE1 | download-refined.png | 소스 header/공용 preview/track표/저장방식 구성 유지.2줄 transport·볼륨rail·호환성 Info/Warning 카드·header 치수는 차이. |
| MODEL1 | models-refined.png; 실제 표본 검토 별도 계약 | 기능 그룹/후보/inspector와 새 검토 pane 존재. 저장소 미연결 검토 pane이 기본 펼침으로 카탈로그 위에 큰 공간을 차지한다. 연결 능력에 맞춘 기본 펼침 조정 검토가 필요하다. 카탈로그 분류/설치 상태를 fixture로 승격하지 않는다. |
| JOB6 | queue-refined.png | 전체/입력 필요/대기/실행/실패/완료를 각각 클릭. 선택 밑줄만 바뀌고 status가 없는 같은3행/35·45·55%가 남는다. 필터가 수행된 것처럼 오해할 수 있다. 선택 윤곽은 r157 수정 유지. pill filter/상단 실행 배치·복합셀·목록 바닥면 차이도 남는다. |
| OUT1 | export.png | 공용 preview·복합 출력 요약3행·속성 아이콘/저장 카드 유지. 원본구간/설명/정확한5항목 데이터·소스 제목 위치와 기타 미검증은 남는다. |

## 원인 확인과 다음 작업

1. JOB 상태 필터: `StudioShell`의 탭 인덱스는 바뀌지만 `screens::system::show`에는 전달되지 않는다. WorkspaceSnapshot/TableSnapshot에는 typed JobStatus·상태 필터 capability가 없다. status 열은 UI가 골격으로 추가하고 값을 추정하지 않는다. SDK 세션과 확인했으며 추후 JobStatus/필터 capability/안정 row ID 계약으로 추적한다. 연결 전 non-All 필터를 비활성으로 표시하고 이유를 제공하는 UI 수정이 우선이다. 임의 문자열이나 진행률로 상태를 추정하지 않는다.
2. 영상 프레임 공용화: `screens/media.rs::preview`는 artboard/수동 overlay/작은 toolbar를 따로 조합하며 SEG·SUB5·OBJ에 공유된다. 반면 media_preview는 검정 전체 면/테두리/미상시간·seek/볼륨/전체화면을 소유하고 CLIP AI/SAVE/OUT에서 사용한다. 공용 media_preview에 overlay 경계를 추가하고 ROI/subject 좌표·도구를 보존해 기존 경로를 이관하는 설계가 다음 큰 시각 작업이다. source contain을 깨거나 가짜 재생 상태를 채워 시안의 종횡비 오류를 복제하지 않는다.
3. MODEL 미연결 pane: backend unavailable일 때 검토 영역의 기본 접힘과 이유 접근성을 검토한다. 실제 backend 연결 실행의 명시 SHA/원문/판정 계약은 유지한다.
4. PROJ viewport/camera 및 FRAME/CARD canvas 공통 책임, 나머지 전용 inspector 그룹/시각 밀도는 각 기능 계약을 따라 순환 개선한다. 특정 화면을 고친 사실로13개 화면군의 완성을 대체하지 않는다.

이 순회는 전체 목표의 일부다. 시청·전체화면·캡션/패널의 나머지 상태·모든 크기·모든 시안 요소의 근거가 아직 부족하므로 goal을 완료로 표시하지 않는다.

캡션 닫기 뒤 새 sky 목록에서 r158 경로 필터는 빈 배열이었다. 창 소멸 확인이며 별도 PID 소멸 증명은 아니다.
