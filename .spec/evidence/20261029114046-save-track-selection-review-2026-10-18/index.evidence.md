---
id: evidence:kinekino/20261029114046
title: SAVE 트랙 선택 컴포넌트 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SAVE 트랙 선택 컴포넌트 검토

2026-10-18. 기준은 `artifacts/design/v2/studio/download-refined.png`의 SAVE-03/04다. 확대 관측은 `runtime/design-inspection/save-tracks-r56-3x.png`를 사용했다.

## 구현 범위

공통 미확인 트랙 표, 직접 작성한 refresh·미디어 자막 SVG, inspector와 공유하는 비활성 selector를 추가했다. 영상/오디오/자막의 미확인은 트랙 없음이나 선택 해제로 변환하지 않는다. 실제 트랙 조회·선택·새로고침 application 계약은 연결 전이다.

## 실제 실행 관측

기존 가상 모니터의 r56, PID44376,1440×900 SAVE 화면에서 세 유형의 glyph·미확인 selector·새로고침·unknown 안내를 확인했다. selector가 정보 열을 밀어내는 문제를 발견했다. 이에 헤더와 행을 동일한 grid 열 규격으로 수정했다. 수정본 실행 검토는 아직 남아 있다. 760·Empty 회귀도 미검증이다. 전체 시각 일치 완료로 판정하지 않는다.

r56은 소유 창의 닫기 버튼으로 종료했고 이후 player.exe/mpv.exe 프로세스 조회 결과가 비어 있음을 확인했다. VM/DPI/모니터 구성은 변경하지 않았다.

수정본 r57(PID116600)은 1440×900에서 selector가 선택 열 안에 머물고 정보/품질·언어가 각각 헤더 아래에 표시됨을 확인했다. 760×480에서는 본문을 스크롤해 세 유형 모두 다음 줄의 정보·품질/언어와 마지막 unknown 안내가 보임을 확인했다. Empty 및 설정 drawer 회귀는 이번 표본에서 실행하지 않았다. 최종 fmt 검사·전체 test·Clippy(all-targets, warnings deny)·player 빌드는 통과했다. 중간 빌드의 float literal 경고는 명시적 f32로 수정한 뒤 최종 Clippy/빌드에서 해소됐다.

## 남은 범위

후속 [r59 검토](save-mode-description-review-2026-10-18.md)에서760 Empty의 세 트랙 unknown 유지와 설정/Escape 본문 위치 보존을 실제 확인했다. 저장 방식 설명도 같은 후속에서 수정·검토했다. 소스 카드 상단, 호환성 안내와 실제 SDK 연결은 남아 있다. 테스트 통과를 시각 일치의 근거로 대체하지 않는다.
