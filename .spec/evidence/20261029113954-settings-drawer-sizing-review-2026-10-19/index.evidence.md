---
id: evidence:kinekino/20261029113954
title: 설정 서랍 크기 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 설정 서랍 크기 검토

2026-10-19. 공용 shell/설정 패널 후속. MODEL 연속 그룹 상태 검토 중 발견했다. 기준은 [화면 구성 정본](../../player/presentation/shared/screen-composition.md)의1280미만 설정 서랍·본문 스크롤·하단 실행 영역 계약이다. 큰 화면 모델 검사기의 필드→안내→실행 계층은 `.spec/artifacts/design/v2/studio/models-refined.png`를 따른다. 좁은 화면은 시안에서 직접 측정한 배치가 아닌 공용 반응형 계약이다.

## 발견

r122 PID25852에서1440 Empty 후보0/선택 안내와 음성 그룹 클릭·Space를 확인했다.1440 LongLabels 후보/배지는 영역 안에 표시됐다.980 LongLabels에서 후보2 본문 클릭→End 후보3 선택 후 설정을 열었을 때 좌측 상단 약290×420 floating window로 표시됐다. 창 아래 공간이 남는데도 본문이 짧아 고정 revision 이하가 스크롤에 숨었다. r122는 caption으로 종료했다.

## 수정

공용 `components::inspector::drawer`가 viewport 기준 위치/크기를 소유한다. 오른쪽PANEL12 여백·INSPECTOR300폭, 제목줄32 아래12부터 하단큐40 위12까지의 외곽 rect를 매 프레임 계산한다. egui Window fixed_rect로 기존 기본 크기/임의 위치를 대체하며 열림·닫기와 inspector 본문/하단 실행 책임은 유지한다. 플랫폼 API·모니터·DPI를 변경하지 않는다.

## 수정 후 실행

r123 PID34592에서980×760 MODEL Populated 설정이 우측 x668..968,y45..709에 표시됐다. 필드부터 패키지 크기까지 본문과 하단 다운로드/안내가 보였다. 서랍을 열어 둔 채 LongLabels로 바꿨을 때 상단 배지가 두 줄에 표시되고 아래 필드가 서랍 안에 유지됐다.

열린 채760×480으로 변경하면 우측 x448..748,y45..429로 재배치됐고 제목줄과 하단 큐를 덮지 않았다. 내부 스크롤로 고정 revision→라이선스→정밀도→장치→패키지 크기에 도달하는 동안 다운로드와 안내는 하단에 고정됐다. Escape로 닫은 뒤 모델 화면/선택이 유지됐다.760 SEG에서도 같은 우측 서랍과 본문/두 하단 버튼을 확인하고 서랍 닫기 버튼을 클릭해 닫았다. caption으로 검토 player를 종료했다. VM·DPI·모니터 구성은 변경하지 않았다.

수정 후 cargo fmt --check, cargo test, cargo clippy --all-targets -- -D warnings, cargo build --bin player 통과. 라이브러리71통과/1ignored이며 외부 미디어 ignored 검사는 이번에 실행하지 않았다.

## 남은 범위

모델 그룹 모서리 hover 확대,1180 및 다른 워크스페이스의 모든 서랍 상태·pressed·스크린리더는 별도 검토다. 이번 표본의 LongLabels는 후보/배지에 적용되며 기능별 헤더 자체의 긴 제목 표본을 검증하지 않는다.
