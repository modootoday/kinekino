---
id: evidence:kinekino/20261029113838
title: SUB 교정 두 행 비교 구현·검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SUB 교정 두 행 비교 구현·검토

2026-10-19. 기준 correction.png 전체와(1158,139,410,115)4배 확대(`runtime/review-runs/correction-compare-r138-4x.png`)를 대조했다. 기존 미연결 단일 Readout을 공용 TextComparison과 선택 row_id 투영으로 대체했다.

## 구현

두 행 원문/교정안, FIELD 면·SELECTED label 면·THIN border·CONTROL radius·label72/유동 본문·padding12/body13을 사용한다. 문자열은 삭제·말줄임 없이 줄바꿈한다. 공통 Unicode scalar prefix/suffix 사이의 하나의 변경 span을 amber로 표시한다. 여러 변경점 사이의 동일 문자도 포함될 수 있으며 최소 편집 diff가 아니다. 동일 문자열과 삭제만 있는 변경은 문자 안내로 구분한다. 한국어·일본어·이모지/삽입/교체/삭제/동일 문자열 검사에서 UTF-8 경계와 문자열 보존을 확인했다.

correction_inspector는 stable selected row_id의 original Text를 표시하고 DesignSample일 때만 명시 correction Text를 예시 후보로 표시한다. 후보 부재를 원문 복제로 채우지 않는다. Live 제안·수락/기각·변경 이유는 생성하지 않았다.

## 실제 관측

r138 `runtime/review-runs/ui-correction-compare-20261019-r138/player.exe`, PID96268, 기존 가상 모니터3500,100에서 실행했다. VM·DPI·모니터 구성 변경 없음.

- 1440×900 Populated: 교정 첫 진입은 두 행 연결 필요와 행 선택 안내다. 첫 table 행 클릭은 한국어 원문/예시 후보를 채웠고 Down은 일본어 두 번째 원문으로 바뀌었다. 후보는 전달된 예시 문자열 그대로이며 실제 모델 교정 결과가 아니다.
- 1440 LongLabels: 시나리오 변경이 선택을 초기화했다. 두 번째 행을 다시 선택하면 일본어·한국어·영어의 긴 원문/후보가 두 행 안에서 줄바꿈됐고 공통 긴 suffix는 기본색, 다른 앞부분은 amber였다.
- 760×480: drawer 내부에 두 행과 고정 수락/기각 footer가 나타났다. 내부 scroll로 긴 후보 끝과 샘플 안내까지 접근했다. Escape 후 같은 교정 탭/본문 위치로 돌아왔다. 선택 유지의 재개방 확인은 하지 않았다.
- Empty→drawer: 기존 내부 scroll 위치가 유지돼 처음에는 후보 행부터 보였다. 위로 scroll해 원문/교정안 양쪽 연결 필요와 행 선택 안내를 확인했다. 이전 긴 후보는 남지 않았다.
- caption으로 닫은 뒤 CIM에 player.exe/mpv.exe가 없었다.

## 검사·남은 차이

fmt--check·전체 cargo test(lib86pass/1ignore+기본 통합/doc)·clippy all-targets-Dwarnings·player 빌드가 통과했다. 별도 UI target runtime/parallel-build/ui,-j1을 사용했다. 기존 실제 endpoint 시험은 SDK 전용 증거이며 이 UI 수정의 실제 교정 기능 인수로 확대하지 않는다.

실제 화면에서 label 열의 배경면이 글자 폭/기본 높이에만 그려져 두 행의 면 폭과 긴 본문 행 높이가 일정하지 않다. Taffy 예약 폭72는 유지되지만 background stretch는 후속 수정 대상이다. 또한 수락/기각/undo의 시안 배치·실제 제안 근거·일치/삭제 안내의 실제 실행·980/1180·Live/NeedsInput·스크린리더는 남는다. 전체 SUB 시각 일치 완료가 아니다.
