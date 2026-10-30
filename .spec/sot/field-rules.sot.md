---
id: sot:kinekino/field-rules
title: 편집 필드와 공통 기본값
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 편집 필드와 공통 기본값

검토용 제품 기본값이다. 플랫폼의 공식 상한이나 실제 모델/encoder 가용성으로 해석하지 않는다. 지원 능력과 교집합이 없으면 작업 시작을 차단하고 이유를 표시한다.

| 필드 | 기본 | 유효 범위와 적용 |
|---|---|---|
| 시간 | 원본 전체 또는 사용자 선택 | 정수 us, 0≤in<out≤duration. 화면 ms, VFR 프레임 경계 별도 |
| OP/ED | 수동/수동 | 끔/수동/자동. 확인된 annotation 없으면 버튼/자동 이동 없음 |
| 스킵 예고/되돌리기 | 3초/5초 | 재생 시간에 연결, 일시정지·탐색 재진입 정책 적용 |
| auto next | 끔 | 다음 회차 확인과 명시적 사용 설정 모두 필요 |
| volume | 기존 재생 값 | 0~100 정수, mute 별도 |
| mask opacity | 50% | 0~100 표시용, 원본 mask 값 변경 아님 |
| mask threshold | 0.5 | finite 0~1, binary 출력에만 적용 |
| mask feather | 0px | 0~64 원본 px, 원본 크기보다 큰 값 금지 |
| brush | 16px | 1~512 원본 px, 입력 크기보다 크면 clamp 후 표시 |
| pivot | 중앙(0.5,0.5) | 정규화 0~1, px 표현과 양방향 변환 |
| crop | 원본 전체 | 정규화 x/y/w/h, w/h>0, 원본 밖 금지, fit/fill 분리 |
| OCR 표본 간격 | 0.5초 제안 | 0.1~5초, 경계 오차와 비용 표시, 모델 요구사항 우선 |
| 자막 오프셋 | 0ms·파일 적용 끔 | -600000~600000ms, 적용 후 음수/끝 초과는 검토 필요 |
| 화자 분석 | 선택 실행·인원 auto | ASR와 별도 모델/작업. fixed/min-max는 선택한 모델 지원 범위 내 1≤min≤max |
| 화자 라벨 | 화자1·화자2 등 | 공통 라벨1~200자, 미지정/동시발화 보존. 음성 신원/캐릭터 자동 식별 아님 |
| cue 텍스트 | 빈 초안 | cue당 Unicode scalar 10000자, NUL 금지, 형식 escape 처리 |
| 한 줄 편집 라벨 | 자동 예시/사용자 | 1~200자, 파일명으로 직접 사용하지 않음 |
| 문서 전체 텍스트 | 사용자 입력 | UTF-8 10MiB 편집 상한, 초과는 구간 분리 안내 |
| 글꼴 크기 | 32 출력 px | 8~256, 캔버스 좌표 기준, 줄바꿈/넘침 재검사 |
| 외곽선/패딩 | 0/8 출력 px | 외곽선0~32·패딩0~128, canvas clipping 검사 |
| 출력 크기 | 원본/프리셋 | 16~8192 정수 px 각 축, encoder 배수 제약 별도 검사 |
| 이미지 배율 | 1x | 0.1~4x, 목표 픽셀과 확대 경고, 상한8192 적용 |
| JPG 품질 | 90 | 1~100, PNG에는 숨김 |
| 숏폼 길이 | 30초 제안 | 사용자 범위와 원본/프리셋 허용 범위 교집합 |
| 추천 후보 개수 | 3 | 1~20, 후보가 부족하면 실제 수만 표시 |
| 일괄 작업 선택 | 없음 | 1~100개 명시 선택, 처리 동시성1 기본 |
| 카드 수 | 1 | 앱 편집 상한200. 플랫폼 업로드 한도로 표시하지 않음 |
| 카메라 FOV | 45도 | 1~170 finite, perspective 모드에만 |
| depth | 모델 원본 상대값 | 단위 없음, near/far=원본 유효 최소/최대, 실제 거리로 변환 금지 |
| 카메라 clip | near0.01/far100 | 0<near<far, 프로젝트 선언 단위, 상대 depth와 분리 |
| parallax preview | 0도 | -10~10도 제안, 홀이 생기면 hatch, 자동 생성 채움 없음 |
| job retry | 자동 끔 | 사용자 재시도 기본. 네트워크 임시 오류만 최대3회 1/2/4초 제안 |
| resource profile | 절약 | 무거운 AI1개, 사용자 앱 종료 없음, 실제 장치 예산 선검사 |

텍스트·문서 상한은 앱의 초기 보호 기준이며 필요하면 버전 있는 정책으로 변경한다. 모델 tokenizer/context 한도는 별도 검사하고 입력을 몰래 잘라내지 않는다. 숫자 spinner는 휠 오작동을 피하도록 focus 상태에서만 변경한다. Ctrl로 정밀 조정하는 입력은 툴팁에 실제 step을 표시한다.

## 컴포넌트 크기

제목줄32, 일반 toolbar32, 입력 높이32, 주요 실행40, 목록 행 최소32·회차40, 표 헤더28, 큐 bar40, inspector label12/body13. 사용자가 OS 큰 텍스트를 적용하면 고정 높이로 잘라내지 않고 최소 높이로 해석한다. 글자 높이와 padding을 측정해 늘린다. 좌우 기준12·그룹 간16·필드 간8.

## 아이콘과 이름

Phosphor Regular 한 굵기와 18/20px가 후보이며 현재 UI 스택은 egui0.36.2와 egui_taffy0.14다. 사용자 지정에 따라 의미별 SVG를 직접 작성하며, 특정 아이콘 라이브러리의 실제 자산을 사용한 것으로 표기하지 않는다. navigation은 icon+label, 48 레일에서는 tooltip·accessible name. Play/Pause, SkipBack/Forward, FolderOpen, MagnifyingGlass, SpeakerHigh/Slash, ListNumbers, CornersOut/In, DotsThree, Scissors, Subtitles, Image, Stack, Cube, Download, Cpu, Queue, X, Minus, Square를 의미별 IconId로 매핑한다. 특정 라이브러리 상수 철자는 구현에서 확인한다.
