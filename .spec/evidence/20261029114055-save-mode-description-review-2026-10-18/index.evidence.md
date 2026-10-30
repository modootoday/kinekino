---
id: evidence:kinekino/20261029114055
title: SAVE 저장 방식과 예상 정보 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SAVE 저장 방식과 예상 정보 검토

2026-10-18. 대상 SAVE-05/08/10. 실제 실행 결과는 아래에서 설계·소스와 구분한다.

## 기준과 차이

기준 `.spec/artifacts/design/v2/studio/download-refined.png`, 저장 방식 영역(1125,147,440,266)을 Rust visual-review로 3배 확대했다. 파생물은 `runtime/design-inspection/save-mode-r58-3x.png`다.

| 요소 | 시안 관측 | 기존 구현 | 수정 책임 |
|---|---|---|---|
| SAVE-05 제목/설명 | radio 제목 아래에 들여쓴 설명, 각 옵션 간 구분된 여백 | 설명 없이 세 radio만 나열 | 공용 described_choice, 제목 hit32·보조11·gap8·theme icon 기준 들여쓰기 |
| SAVE-05 상태 | 원본 선택/권장 표시 | 실제 계약 미연결 | 세 항목 미선택·비활성·이유 유지. 시안 선택을 실제 정책으로 복제하지 않음 |
| SAVE-08/10 그룹 | 출력 필드 아래 예상 정보 별도 제목 | 같은 필드 목록 안에 섞임 | SAVE 전용 save_inspector가 출력 설정/예상 정보 그룹을 조합 |

새 컴포넌트는 stable ID·제목·설명 외형만 소유한다. 저장 방식의 의미는 SAVE 화면이 소유하고 실제 변환·저장 정책은 application/domain 연결 전이다. 원본 저장·컨테이너 변경·재인코딩의 설명을 구분했다.

## 실제 실행 결과

r58(PID78928,1440×900)을 기존 가상 모니터에서 실행했다. 설명과 예상 정보 그룹은 표시됐지만, 전체 폭 add_sized가 radio 제목을 중앙으로 정렬해 설명과 다른 기준선에 놓이는 차이를 발견했다. 따라서 공용 컴포넌트에서 add_sized를 제거하고 theme의 interact_size.y=32로 hit 높이를 유지하는 방식으로 수정했다. r58은 소유 창의 닫기 버튼으로 종료했고 프로세스 조회 결과 player/mpv가 없음을 확인했다. 이 수정의 실제 재검토는 아래에 별도로 기록한다.

## 미검증과 남은 차이

수정본 r59(PID119380, `runtime/review-runs/ui-save-mode-20261018-r59/player.exe`)에서 다음을 실제 관측했다.

- 1440×900: radio 제목과 각 설명이 같은 왼쪽 텍스트 기준선에 놓이며 세 설명·출력 설정·예상 정보·호환성 및 하단 버튼이 표시된다.
- 760×480: 설정 창의 컨테이너 변경 설명이 두 줄로 자연스럽게 줄바꿈한다. 본문 스크롤 후 예상 정보/호환성까지 도달하며 소스 확인·대기열 추가 버튼은 같은 위치에 남는다. 명령은 미연결이므로 실행 성공을 검증한 것이 아니다.
- Escape는 설정만 닫고 SAVE 본문을 유지한다. Empty로 전환해 영상이 placeholder로 바뀜을 확인했고, 본문 끝의 영상/오디오/자막 세 행은 모두 트랙 확인 필요·정보/품질/언어 미확인을 유지했다. 자막 없음이나 0개로 바뀌지 않았다.
- Empty의 트랙 표 위치에서 설정을 다시 열고 Escape로 닫아도 본문 위치는 같은 트랙 표에 남는다. 설정 내부 스크롤 위치도 별도로 보존됐다.
- 최종 `cargo fmt --check`, `cargo test -j2`, `cargo clippy -j2 --all-targets -- -D warnings`, `cargo build -j2 --bin player`가 통과했다. 전체 탭/fixture headless 검사는 통과했으나 수동 전 화면 검토의 대체 근거로 쓰지 않는다.

검토 후 r59 소유 창의 닫기 버튼으로 종료했다. 이후 Win32_Process의 player.exe/mpv.exe 조회 결과는 비어 있었다. VM·DPI·모니터 구성을 변경하지 않았다.

호환성 info/warning 카드·소스 카드·전용 transport·실제 track/format/save 연결은 이번 수정 범위 밖이며 남아 있다. radio hover/pressed/selected/focus는 실제 계약 미연결 상태이므로 검증하지 않았다. LongLabels 및 980/1180 SAVE 검사기 회귀도 별도다. 모든 화면의 일치 완료를 의미하지 않는다.
