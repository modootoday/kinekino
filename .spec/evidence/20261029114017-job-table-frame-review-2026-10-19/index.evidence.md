---
id: evidence:kinekino/20261029114017
title: 대기열 표 프레임 수정과 실행 검토 r149
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 대기열 표 프레임 수정과 실행 검토 r149

2026-10-19. r146 실제 표와 queue-refined.png(296,244,807,276)3배 확대에서 둥근 외곽 프레임·헤더 열 구분·행 가로선 차이를 확인했다. 공용 DataTable::show_framed는 기존 열 구성/선택/키보드 구현을 공유하고 THIN/BORDER 외곽선·CONTROL radius와 외곽 셀의 개별 corner를 조합한다. header 열 구분은 각 실제 셀 rect에서 FIELD만큼 세로 inset, 전체 행 모드 가로선은 실제 row rect 상단이다. JOB만 이 변형을 사용하고 기본 표 외형은 유지한다.

## 검사와 실제 실행

session75245 최종exit0. fmt--check·cargo test(lib86pass/1ignored와 기본 통합/doc)·clippy all-targets-Dwarnings·player build 통과. test compile12m23s,clippy15.45s,마지막build31.70s. 동일 UI target-j1 파이프라인을 유지했다.

`runtime/review-runs/ui-job-frame-20261019-r149/player.exe`, PID104372. 새 sky 목록에서 기존 검토 창이 없음을 확인한 뒤 기존 가상 모니터3500,100에 한 개 실행했다. VM·DPI·모니터 구성 변경 없음.

- 1440×900 데이터 채움: 둥근 외곽선·header 면/짧은 열 구분·3행 가로선과 마지막 행 하단 모서리가 표시됐다. 좌측 선택 표시를 유지했다.
- 둘째 행 출력 셀 클릭으로 전체 행 포커스/선택과 상세 소스2가 바뀌었다. Down은 셋째, Home은 첫째, End는 셋째 행으로 이동하고 상세 제목/소스도 대응했다.
- 1440/760 LongLabels: 작업·소스·출력은 열 안에서 말줄임하며 열/프레임 폭을 밀어내지 않았다. 시나리오 교체 뒤 첫 행 선택 초기화는 새 fixture 상태이며 이전 job 선택 보존으로 주장하지 않는다.
- 760 LongLabels: 설정 drawer의 제목·단계 예시와 고정 footer 표시, Escape 뒤 같은 표/첫 선택 유지 확인. 셋째 행 하단은 스크롤 아래에 있어 이 크기의 마지막 모서리 관측으로 확대하지 않는다.
- 760 Empty: 이전3행이 제거되고 둥근 header 프레임·5개heading·등록된 작업 없음 안내가 남았다. 실제 Live 조회의0건 보장으로 해석하지 않는다.

## 남은 차이

caption 닫기 뒤 새 sky 목록에서 r149 창이 반환되지 않았다. 창 소멸 확인이며 별도 PID 소멸 증명으로 확대하지 않는다.

작업별 typed glyph/입력 두 줄 요약·시안의 선택 행 면·상단 필터/실행 영역 배치가 남아 있다. 프레임은 현재 표 콘텐츠 높이를 따르므로 시안의 큰 빈 목록 바닥 면 전체와 다르다.980/1180·hover/pressed·긴 상태·screenreader와 다른 framed 소비자는 미검증이다. 실제 작업 실행/단계/진단 연결을 새로 구현한 것이 아니다. caption 닫기를 요청했으며 새 창 목록으로 종료를 별도 확인한다.
