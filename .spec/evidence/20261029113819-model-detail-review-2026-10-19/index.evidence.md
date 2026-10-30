---
id: evidence:kinekino/20261029113819
title: MODEL 선택 상세 시각 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# MODEL 선택 상세 시각 검토

2026-10-19. MODEL-02/07. models-refined.png 전체와 (1120,78,445,124)4배 확대를 비교했다. 파생물 `runtime/review-runs/model-detail-r117-4x.png`. 시안은 정보 heading→이름/설치 badge→후보 설명→저장소/revision 필드 순서다.

## 변경

기존 단순 label/weak로 이어진 이름과 설치 원문을 공용 Badge로 구분하고 후보 예시/실제 설치 구분 설명을 추가했다. 저장소/revision은32높이 read-only 필드의 ‘연결 필요’ 골격을 유지한다. 실제 값·입력 가능성을 만들지 않는다. 선택 행이 없으면 정보 선택 안내를 표시하고 임의 후보의 필드 목록을 생성하지 않는다.

다운로드는 공용 Button40에 기존 아래 화살표 SVG를 결합하고 아래에 미가용 이유를 표시한다. disabled 상태와 실제 catalog/command 연결 필요 사유는 유지한다. 기본 모델 설정·설치 허가·실제 모델 상태는 변경하지 않는다.

## 실제 실행

r117 PID119632,최종 빌드.1440 demo Populated에서 이름/설치 배지·예시 설명·32높이 저장소/revision 필드·아래 화살표 다운로드 버튼과 사유를 확인했다. 두 번째 카드 본문 클릭 후 모델2 배지,End 후 모델3 배지로 상세가 동기화됐다. 후보 기본 설정이나 다운로드를 실행한 것은 아니다.

LongLabels1440에서는 상세 이름/설치 배지가 각각 한 줄 말줄임으로 두 줄 배치됐다.980 설정 drawer에서도 폭을 넘기지 않았고 내부 스크롤로 저장소/revision/정밀도/장치/패키지 크기에 도달했다. 다운로드 버튼과 사유는 고정됐다. Escape 후 목록과 선택이 유지됐다.980 Empty로 바꾸고 drawer를 다시 열었을 때 모델 선택 정보 안내만 표시되고 후보 상세 필드는 생성되지 않았다. 다운로드는 계속 비활성이었다.

caption으로 종료 후 player.exe/mpv.exe 조회 결과가 비어 있었다. 기존 가상 모니터 위치3500,100을 사용했고 VM·DPI·모니터 구성은 변경하지 않았다. 최종 cargo fmt --check, cargo test(lib71 passed/1 ignored 및 CLI/통합), cargo clippy --all-targets -- -D warnings, cargo build --bin player가 통과했다. 외부 미디어 의존 ignored 검사는 실행하지 않았다.

## 미검증

typed catalog/details·실제 저장소/revision/license/정밀도/device·다운로드 진행·실제 설치/제거와 모든 화면 크기/상태의 시각 인수는 별도다. 배지를 실제 설치 성공 상태로 해석하지 않는다.
