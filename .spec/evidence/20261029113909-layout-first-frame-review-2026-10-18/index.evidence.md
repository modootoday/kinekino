---
id: evidence:kinekino/20261029113909
title: 중첩 레이아웃 첫 표시 안정화
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 중첩 레이아웃 첫 표시 안정화

2026-10-18. r94의 SUB 음성→변환 첫 capture에서 시각 문자열이 한 글자씩 세로로 줄바꿈되고 검사기 폭이 흔들렸으며 다음 capture는 안정됐다. [기존 실제 관측](subtitle-conversion-comparison-review-2026-10-18.md). 이번 작업은 해당 전환 결함의 공용 원인을 조사한다.

## 재현·원인

현재 egui0.36.2 Options 기본 max_passes는2다. egui_taffy0.14.0은 새 leaf를 sizing/invisible로 측정하고 이전 측정값으로 중첩 layout을 재계산하며 request_discard를 요청한다(lib.rs add_container_dyn/recalculate). egui는 패스 상한에 도달하면 재요청이 남아도 해당 프레임을 반환한다. 단순 추가 repaint는 잘못된 중간 프레임을 이미 표시한 뒤에 일어난다.

실제 화면과 같은 StudioShell·Pretendard·theme·Populated,1440×1400 headless 전환을 비교했다. 이는 화면 캡처나 GPU 성능 측정이 아니다.

| 상한 | 실제 패스 | 첫 원본 시각 문자열 줄 수 | 잔여 재계산 |
|---|---|---|---|
|2|2|27|있음|
|4|4|1|있음|
|8|7|1|없음|
|12|7|1|없음|

## 공용 변경

layout::configure는 정상 player 호스트와 offline demo 호스트 생성 시 max_passes 상한8을 설정한다. theme·폰트·색 토큰과 별도 수명 설정이며 캐시의 기존 측정 구조를 유지한다. 무한 retry나 매번8패스가 아니다. 새 중첩 구조가 상한을 넘으면 원인을 재분석한다.

회귀 검사는760/980/1180/1440폭에서 첫 변환 프레임의 실제 text galley가 한 줄이고 잔여 discard가 없는지 검사한다(검사 높이2000은 양쪽 cue를 관측하기 위한 headless 표본이며 실제 창 검증과 구별). 별도 입력 검사는 강제7패스에서 클릭을 한 번만 처리하고 다음 idle이1패스인 것을 확인했다. 실제 GUI 입력·GPU/CPU 프레임 비용 검증은 후속 결과와 분리한다.

## 실제 실행과 최종 검사

최종 cargo fmt --check, cargo clippy --all-targets -- -D warnings, cargo build --bin player, cargo test가 통과했다. 라이브러리69통과/1ignored 및 CLI·통합 통과, 기존 외부 미디어 ignored 유지. 회귀 검사의 유휴1패스는 CPU/GPU 프레임 시간 측정의 대체물이 아니다.

### 데모 r95

`runtime/review-runs/ui-layout-host-20261018-r95/player.exe`(PID27104), 가상 모니터3500,100.1440×900에서 SEG→SUB 음성→변환, FRAME 원본, MODEL, JOB를 순서대로 열었다. 전환 후 관측된 변환 시각 문자열/검사기·스냅샷/썸네일·모델 배지·대기열 열은 정상 폭이었다. 입력 직후 capture가 이전 탭인 경우에는 추가 입력 없이 새 capture로 실제 전환 결과를 확인했다. r94의 세로 문자열/검사기 압축 상태는 관측되지 않았다. 이는 프레임마다 촬영한 영상 증거가 아니므로 모든 중간 표시의 부재를 GUI capture만으로 입증하지 않는다. 첫 출력 프레임의 한 줄 증거는 위 headless 회귀로 구분한다.

760×480으로 바꾼 뒤 SUB 변환 복귀, 음성 탭 클릭→End로 변환 탭 선택, 스크롤 후 원본 문법 목록이 정상 폭인 것을 확인했다. r95는 캡션 닫기로 종료했다.

### 일반 호스트 r96

`runtime/review-runs/ui-layout-host-20261018-r96/player.exe`(PID6628)는 ui-demo 없이 실행했다.1180×760 시작 화면에서 P로 회차 빈 패널 열기→Escape 닫기, 제작 버튼으로1440폭 제작 진입→Escape로1180폭 시청 복귀를 확인했다. 호스트의 공용 패스 설정은 일반 player 생성 시에도 적용된다. 실제 영상/외부 페이지는 열지 않았다. r96도 캡션 닫기로 종료했고 후속 CIM에서 player.exe/mpv.exe가 없었다. 두 실행을 겹치지 않았으며 VM·DPI·모니터 구성은 변경하지 않았다.

## 미검증과 남은 범위

모든 화면·하위 탭·fixture에서 첫 프레임을 측정한 것은 아니다. 실제 GUI980/1180 제작 전환·대형 문서/많은 레이어·실제 미디어·팝업/HMR·장치별 CPU/GPU 비용, 반복 전환의 계측은 남아 있다. 모든 화면 시각 인수와 장치별 안정화 완료를 선언하지 않는다.
