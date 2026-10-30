---
id: evidence:kinekino/20261029113821
title: A2D 공용 검사기·서랍 회귀 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# A2D 공용 검사기·서랍 회귀 검토

2026-10-19. A2D 설정·공통 shell 후속. 기준 `v2/studio/asset2d.png` 전체와 (1190,765,383,185)4배 확대를 확인했다. 파생물 `runtime/review-runs/asset-inspector-footer-r125-4x.png`. 시안의 출력 옵션→검증/저장 계층을 유지하며 미연결 실행은 비활성으로 구분한다. 좁은 서랍은 공용 반응형 계약이며 이미지에서 직접 관측한 좁은 배치가 아니다.

## 발견과 수정

r124 PID100460의760×480 A2D 서랍은 우측에 열렸으나 자체 body_height/뒤따르는 footer 계산 때문에 외곽 하단이 약396에서 끝났다. MODEL/SEG 공용 서랍의429보다 짧았다. A2D의 별도 최대 높이 계산을 제거하고 `inspector::panel`로 본문 scroll과112고정 footer를 조합했다. 알파/피벗/경계/출력 필드와 두 버튼·sample 안내는 유지했다. 미사용이 된 INSPECTOR_BODY_MIN 토큰을 제거했다. r124는 caption으로 종료했다.

## 실제 재검토

r125 PID104992,1440×900 A2D에서 프레임 정보·알파·피벗·경계가 보이고 본문 scroll로 출력 세 옵션까지 도달했다. 검증/저장·안내는 고정 하단에 유지됐다.760×480 서랍은 x448..748,y45..429를 사용했다. 출력 마지막 JSON까지 scroll하는 동안 두 버튼·안내는 고정됐다. Escape 후 합성 모드와 선택 인물 layer가 유지됐다.

같은 r125의760 PROJ 깊이 탭에서 우측 서랍을 열어 출력 형식/대상 도구/amber 경고까지 scroll하고 검증/저장 footer가 고정되는 것을 확인했다. Escape 후 같은 깊이 탭과 scene이 유지됐다.760 JOB 전체 탭에서 단계 예시가 서랍에 들어오고 입력/진단까지 scroll하는 동안 재시도/취소/출력 폴더가 유지됐다. Escape 후 선택 작업과 전체 탭이 유지됐다. 단계는 디자인 예시이며 실제 pipeline 실행 증거가 아니다.

r125는 caption 닫기로 종료했다. 기존 가상 모니터만 사용했고 VM·DPI·모니터 구성은 변경하지 않았다. 첫 A2D 전환 capture에 일부 이미지가 미표시였지만 재관측에서 원본/배경/썸네일이 표시됐다. 이를 지속 결함이나 GPU 성능 측정으로 해석하지 않는다.

## 검사와 남은 범위

cargo fmt --check, cargo test, cargo clippy --all-targets -- -D warnings, cargo build --bin player 통과. 라이브러리71통과/1ignored. 테스트 빌드 중 확인한 미사용 토큰 경고는 삭제 후 clippy와 최종 build에서 해소됐고 fmt를 다시 확인했다. 외부 미디어 ignored 검사는 실행하지 않았다.

Empty/LongLabels/NeedsInput의 세 화면 새 서랍,980/1180,PROJ 패키지 탭과 JOB 모든 상태 탭,키보드 focus 순회·pressed·스크린리더는 이번에 확인하지 않았다. 시안의 전체 필드 배치·좌표 입력 표현과 모델 그룹 모서리 확대는 별도 남은 작업이다.
