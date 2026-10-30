---
id: evidence:kinekino/20261029113905
title: OCR 결과 영역·선택 검사기 재검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# OCR 결과 영역·선택 검사기 재검토

2026-10-18. SUB-05a/06a 부분 개선이며 OCR 실제 실행과 전체 자막 시각 인수는 미완료다.

## 참조와 변경 전

ocr-refined.png의 결과 영역(272,654,865,245)을 Rust visual-review로2배 확대했다. 파생물 runtime/design-inspection/ocr-results-r38-2x.png. 감지된 텍스트/낮은 신뢰도 탭, 번호·시작·종료·텍스트·후보·검토 표, 검사기 원문/인식 결과 필드와 읽기 안내를 확인했다.

r38: runtime/review-runs/ui-ocr-review-20261018-r38/player.exe, PID101128,1440×900. OCR에 일반 구간 timeline·분할/병합 도구·화자 열이 표시되고 검사기 텍스트 검토가 없었다. ROI 황색선과4개 흰 원형 corner는 유지 대상이다. r38은 caption 닫기로 종료했다.

## 변경 책임

screens/ocr.rs가 기존 typed TableSnapshot을 OCR 전용으로 투영한다. 원본 row_id/start/end/original/correction을 유지하고 화면 순서 번호를 별도로 표시한다. 검토/신뢰도 미연결을 미상·비활성 사유로 표시하며 성공/확률을 만들지 않는다. 일반 timeline/편집 toolbar를 OCR에서 제거한다.

선택 row를 inspector의 원본/후보 읽기전용 필드에 연결하고 공용 notice로 글자 제거 기능이 아님을 안내한다. 편집 command는 미연결로 명시한다. shared data_table은 stable row/cell focus, 선택 amber선과 focus 외곽선,32단위 행, 공용 keyboard_selection을 사용한다. 기존 탭/모델의 동일 키 처리는 공용 함수 재노출로 유지한다.

## 첫 수정 관측과 열 보정

r39: runtime/review-runs/ui-ocr-results-20261018-r39/player.exe, PID44400,1440×900. OCR의 불필요한 timeline/toolbar가 제거되고 결과 표와 텍스트 검사기가 보였다. 일본어2행 클릭→검사기 원문 일본어 표시, Down→3행과 한국어 원문으로 동기화, 첫cell focus와 행 선택 중성면을 관측했다.

동일 폭 열 때문에 번호가 과도하게 넓고 감지문이 잘려 번호48/시간132×2/상태88 토큰과 유동 텍스트2열로 후속 보정했다. 고정열 합이 폭의65%를 넘으면 축소한다. r39 종료 후 후속 빌드를 실제 재검토한다. r39는 최종 열 보정 근거가 아니다.

기존 가상 모니터 위치3500,100만 사용하며 VM/DPI/모니터 구성 변경 없음. 실제 화면은 도구 반환 이미지 관측이며 저장 캡처를 주장하지 않는다.

## 잔여 범위

최종 r40: runtime/review-runs/ui-ocr-columns-20261018-r40/player.exe, PID50896.1440×900 OCR에서 번호/시간/상태의 폭과 유동 감지문/후보 열을 확인했고 세 샘플의 한국어·일본어 감지문이 모두 온전히 보였다.2행 클릭→일본어 검사기, Home→1행 한국어 검사기 갱신을 관측했다. 같은 실행의 SEG에서1행 클릭→End로 마지막 행 이동과 focus/selection을 확인했다.

760×480 OCR은 body scroll로6열 표에 접근하며 시간은 온전히 보이고 긴 텍스트는 말줄임된다.2행 선택→설정 열기→검사기 scroll로 일본어 원문/후보/편집 미연결 안내를 확인했고 고정 검토/시작 footer가 유지됐다. Escape는 설정만 닫고2행/본문 위치를 유지했다. Empty 전환에서 샘플 이미지·ROI·행이 제거되고 결과 연결 안내와 표 header는 유지됐다. r40은 caption 닫기로 종료했다.

검사: cargo fmt --check·cargo test -j2(78통과/8ignored)·cargo clippy -j2 --all-targets -- -D warnings·cargo build -j2 --bin player 통과. 중간 Taffy 숫자 literal의 f32 추론 경고를 명시 type으로 수정했고 최종 clippy/build에서 경고 없음. 이번 전체검사에서 이전 capabilities 비교의 일시 실패는 재현되지 않았다. 공용 표/키 처리의 모든 실제 화면 회귀를 이 두 표본으로 대체하지 않는다.

ROI 원본 좌표/역변환/실제 OCR, burn-in 텍스트 샘플, 다중선택 checkbox, 신뢰도/검토 데이터와 필터, 실제 편집·실행은 미완료다. 모든 글꼴 조합/IME·실제 OCR 결과·caption 상태별 인수도 미검증이다.
