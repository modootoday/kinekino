---
id: evidence:kinekino/20261029114016
title: OBJ 수동 샘플 마스크 실제 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# OBJ 수동 샘플 마스크 실제 검토

2026-10-18. OBJ-04b 부분 개선 기록이며 실제 추론이나 전체 OBJ 인수 증거가 아니다. [계약](../../player/application/subject-mask-read.md), [시각 정본](../../player/presentation/studio/subjects-visual.md)을 따른다.

## 참조·구현

subject.png의 mask 영역(790,409,249,225)을 기존 Rust 도구로4배 확대해 cyan contour·반투명 채움·white square handles를 관측했다. 파생물 runtime/design-inspection/subject-mask-r34-4x.png, 원본 변경 없음. 시안의 고양이 대신 기존 ImageGen forest 샘플의 인물을 사용한다. 원본 asset의 인간 수동 근사 SVG는 디자인 검토 입력이며 모델 출력이 아니다.

SDK와 앞서 조율한 optional MaskSnapshot을 workspace_read에 추가했다. 원본1672×941·PTS52000000·track_id·polygon·fullcanvas alpha·검토 상태·origin/state를 분리한다. Live와 Empty는 None. mask_overlay 공용 컴포넌트는 SourceTransform을 공유하고 alpha28% 토큰·청록 contour·bounds를 영상에 clip한다. mask 원본asset/PTS가 프레임과 다르면 표시하지 않는다. 영상 아래 수동 예시/미검토/실제 분할 결과 아님을 표시한다.

## 실행 관측

runtime/review-runs/ui-obj-mask-20261018-r34/player.exe, PID652. 기존 가상 모니터 위치3500,100에서 단일 검토 player를 실행했다. VM/DPI/모니터 구성 변경 없음.

- 1440×900 Populated: 이전 단순 사각형 대신 인물 실루엣의 청록 윤곽·반투명 채움·외접 bounds가 함께 표시됐다. 얼굴/옷의 원본 색과 선화는 채움 아래 보인다. 직접 작성한 rail도 유지한다.
- 760×480: 이미지/마스크/경계가 함께 정렬된다. body를270스크롤하면 하단 silhouette와 안내가 보이며 header 밖으로 그려지지 않는다. 상단 도구가 viewport 위로 잘리는 것은 body와 함께 scroll되는 현재 구조다.
- 760 Empty 전환: 이미지·윤곽·채움·bounds·수동 마스크 안내가 제거되고 소스 연결 안내/비활성 도구만 유지한다. user-input 감지로 한 입력이 거절되어 상태를 재관측한 후 다시 진행했다. 이를 제품 실패로 판정하지 않는다.

caption 닫기 입력으로 종료했다. 실제 화면은 도구 반환 이미지로 확인했으며 캡처 파일을 저장했다고 주장하지 않는다.

## 검사·제한

초기 mutable Ui 차용 컴파일 오류를 수정했다. 최종 cargo fmt --check, cargo test -j2(78통과/8ignored), cargo clippy -j2 --all-targets -- -D warnings, cargo build -j2 --bin player 통과. SVG polygon과 typed points 일치·원본asset·PTS·범위·origin·Empty/Live 부재 테스트를 추가했다.

수동 polygon은 머리카락·옷·손의 정밀 경계를 재현하지 않으며 일부 경계에 근사 오차가 남는다. 원본/마스크/합성 보기 전환·선택 track 목록·실패 interval·opacity 편집·실제 추론/저장·GL custom callback은 미구현. alpha asset 로딩 시간/장치 성능·HDR·1180/980 표본·실제 편집 keyboard는 미검증이다. 이번 검토는 마스크 표현의 누락을 줄인 증거이며 전체 시안 일치 완료를 뜻하지 않는다.
