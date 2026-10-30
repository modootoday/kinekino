---
id: evidence:kinekino/20261029113908
title: CLIP 후보 행과 일괄 요약 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# CLIP 후보 행과 일괄 요약 검토

2026-10-18. CLIP-05/06b. 기준 short-recommend-refined.png 전체와 후보 첫 행(260,620,878,89)2×, 동작(835,639,294,48)4×를 확인했다. 후보 frame의 중성 면·얇은 외곽선·곡률, checkbox/thumbnail/시간/근거/원본 링크, 각 icon+label 버튼을 대응 대상으로 삼았다.

## 변경 전 관측

r76(PID113748), `runtime/review-runs/ui-short-review-20261018-r76/player.exe`에서1440 수동·추천·일괄을 순환했다. 수동 세로 crop/grid와 원본·3트랙은 표시됐다. 추천은 행 frame·checkbox·시간이 없고 thumbnail/문자/분리된 아이콘이3등분됐다. 일괄 회차표는 표시됐지만 계획 요약은 card 밖의 일반 텍스트였다. 760 추천 목록까지 스크롤하면 제외 아이콘과 글자가 다른 줄로 갈라졌다. r76은 캡션 닫기로 종료했고 이후 player/mpv 조회는 비어 있었다.

## 변경

공용 clip_candidate가 selectable card·작업 포함 미연결 checkbox·132폭 contain thumbnail·snapshot의 range/duration·수동 디자인 근거 안내·미연결 원본 링크·32높이 icon+label 버튼을 조합한다. 공용 frame/border/radius/padding과 선택선/focus를 유지한다. 980 미만은 동작을 다음 행으로 배치하며 후보 preview 선택에 Up/Down/Home/End를 연결한다. 작업 포함/실제 preview 재생·편집·제외 명령은 아직 연결하지 않는다. 첫 후보의 preview와 행 강조는 같은 fallback index를 사용한다. 일괄 요약은 공용 card와 반응형 grid로 묶는다.

## 실행 재검토

r77(PID8656), `runtime/review-runs/ui-short-candidates-20261018-r77/player.exe`에서 확인했다.

- 1440: 수동 crop/grid·원본·3트랙 표시 유지. 추천은 첫 행에 공용 선택면/좌측선, checkbox·thumbnail·구간/길이·근거 안내와 세 통합 버튼을 표시했다. 두 번째 행 본문 클릭→Down은 세 번째 선택/focus와 자동 스크롤로 이동했다. 표본 이미지가 같으므로 독립 미디어 전환 성공으로 해석하지 않는다.
- 760: 후보 행 상세 다음 줄에 미리보기/편집/제외 세 버튼이 온전히 표시됐다. 아이콘과 글자가 분리되지 않았다. 두 번째 본문 클릭→Home으로 첫 후보 전체가 자동 스크롤됐다. 설정 열기→Escape는 첫 후보 선택선과 목록 위치를 보존했다.
- 760 LongLabels: 한국어·日本語·English 제목과 길어진 구간 문구는 카드 안에서 줄바꿈했다. 근거/링크와 하단 세 버튼이 겹치지 않았다. 일괄 탭 회차 제목은 기존 truncate를 유지했고, 스크롤 아래 요약 card는 네 값을 한 열로 표시했다.
- 760 Empty 추천: 원본 이미지·후보 카드·이전 선택이 제거되고 소스 이미지 연결 필요와 디자인 예시0개를 표시했다.
- cargo fmt --check, 전체 cargo test, cargo clippy --all-targets -- -D warnings와 player 빌드 통과. 초기 float inference 경고는 f32 명시로 수정하고 최종 전체 test를 다시 통과했다. 외부 미디어 조건으로 ignored인 기존 검사는 실행 통과로 취급하지 않는다.

소유 r77은 캡션 닫기로 종료했고 player/mpv 프로세스 조회 결과는 비어 있었다. VM·DPI·모니터 구성을 변경하지 않았다. 980/1180·NeedsInput·pressed/hover 세부·Up/End 실제 입력·1440 일괄 요약2열은 이번 실제 실행 표본 밖이다. 후보 근거의 독립 열과 의미별 glyph·원본 이동 아이콘은 추가 대응이 필요하다.

## 남은 범위

후보별 실제 evidence/source PTS·독립 미디어·포함 선택·회차별 출력/계획 계약·실제 편집은 별도다. 수동 crop의 safe area/handle·transport·입력 및 모든 하위 화면의 완전한 시각 일치는 아직 완료가 아니다. 시안의 대사·시간·추천 성공을 새로 생성하지 않는다.
