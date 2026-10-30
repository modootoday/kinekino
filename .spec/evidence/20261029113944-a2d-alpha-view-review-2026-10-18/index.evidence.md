---
id: evidence:kinekino/20261029113944
title: A2D 알파 보기·공용 모드 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# A2D 알파 보기·공용 모드 검토

2026-10-18, A2D-02. asset2d.png 전체와 상단 mode 영역(298,123,224,46)4×를 검토했다. 이미지의 합쳐진 알파 합성 문구는 승인된 원본/알파/합성3모드 계약으로 정규화한다.

## 기준 실행

r84(PID48376), `runtime/review-runs/ui-a2d-alpha-20261018-r84/player.exe`,1440×900에서 A2D 합성 및 알파를 확인했다. 알파 선택 후 배경은 checker지만 선택 인물은 컬러 이미지로 표시됐다. 모드 버튼은 작은 selectable label이며 OBJ의 공용 segmented와 달랐다. 기준 창은 캡션 닫기로 종료했다.

## 수정

세 모드는 공용 segmented32·선택/포커스·Left/Right/Home/End를 사용한다. 원본 없는 Empty에서는 미선택·비활성, 확대/맞춤/이동 보기도 비활성이다. mode·view zoom은 원본/레이어/출력 설정을 변경하지 않는 UI 상태다.

알파 보기는 선택된 내장 PNG의 저장된 alpha 채널을 opaque grayscale로 만든다. black0/gray/white255이며 원본 canvas의 검정 배경 위 같은 layer bounds에 표시한다. 원본 크기·SourceTransform·clip·bounds/pivot은 기존 구조를 공유한다. allowlist URI별 context texture 캐시를 사용하며 외부 path를 열거나 저장하지 않는다. 디코드 실패는 알파 미리보기 연결 필요로 표시한다. 이 표시를 모델 분할·threshold/feather 적용·검증 성공으로 표현하지 않는다.

## 수정본 실행

r85(PID37316), `runtime/review-runs/ui-a2d-alpha-20261018-r85/player.exe`에서 확인했다.

- 1440×900: 공용32높이 세 모드가 상단 왼쪽, 기존 줌/맞춤/hand가 오른쪽에 표시됐다. 알파 클릭은 검정 배경 위 흰 인물 실루엣·경계 gray를 표시하며 blue bounds/pivot은 같은 위치였다. Home은 원본 frame과 오버레이 없음, End는 기존 두 레이어 합성/bounds로 복귀했다.
- 760×480: 모드 세 칸과 줌 도구가 한 줄에서 겹치지 않았다. 인물 알파는 검정/흰 표본으로 표시됐다. 레이어 본문 클릭→End 후 안정 캡처에서 배경이 선택됐고, 위로 스크롤하면 불투명 배경의 전체 흰 alpha와 전체 canvas bounds/pivot이 표시됐다. 인물/배경마다 원본 저장 alpha를 사용하며 일괄 흰 채움으로 대체하지 않았다.
- 760 설정 열기→Escape 후 알파 모드와 배경 선택·viewport가 유지됐다.
- 760 Empty: 이전 alpha/선택 경계/레이어가 제거됐다. 세 모드는 미선택·비활성, 줌/fit/hand와 도구 rail도 비활성이고 원본 선택 안내를 표시했다.
- 최종 fmt 검사·전체 cargo test·Clippy all-targets warnings deny·player 빌드 통과. Empty 확대 비활성 보완 후 전체 test를 다시 확인했다. headless 기본 렌더 통과와 실제 알파 보기 관측은 별개다. 외부 조건의 ignored 검사는 실행 성공으로 취급하지 않는다.

소유 r85는 캡션 닫기로 종료했고 player/mpv 조회는 비어 있었다. VM·DPI·모니터 구성을 변경하지 않았다. cache hit/upload 횟수·VRAM/성능·아주 큰 PNG·디코드 실패·zoom/pan 후 alpha·980/1180·LongLabels/NeedsInput은 이번 실제 표본 밖이다.

## 미검증

실제 추론/레이어 편집·시퀀스·HDR/색/출력 알파 계약과 GPU shader 합성은 후속이다. 기존 imagegen cutout의 경계를 실제 모델 정확도로 평가하지 않는다. 전체 A2D 시각 일치는 아직 완료가 아니다.
