---
id: evidence:kinekino/20261029114051
title: 시스템 화면 컨트롤 후속 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 시스템 화면 컨트롤 후속 검토

2026-10-18. [기준 확대](system-elements-2026-10-18.md), [MODEL](../../player/presentation/system/models-visual.md), [JOB](../../player/presentation/system/jobs-visual.md), [OUT](../../player/presentation/system/export-visual.md)를 적용한다. 전체 요소 시각 일치 완료가 아니다.

## r16 수정 전 실제 관측

가상 모니터 위치 3500,100, 창 1440×900. `runtime/review-runs/ui-all-pages-20261018-r16/player.exe`(PID 68784)를 하나만 실행했다. VM·DPI·모니터 구성을 변경하지 않았다.

- MODEL 검색 입력이 공통32보다 낮고, 모델 상세는 전부 동일 입력 상자이며 선택 모델명이 없었다. 다운로드가 본문에 붙어 있었다.
- JOB 동시 실행 수에 미연결 값1이 표시됐다. 진행률/시간/입출력이 편집 상자처럼 보이고 재시도·취소·폴더 버튼이 세로로 본문에 붙었다.
- OUT 경로/파일명 높이가 낮고 하단3동작이 세로로 나열됐다. 충돌 미검증 안내와 동시에 새 이름 radio가 선택돼 있었다.

r16은 캡션 닫기로 종료했고 이후 player 프로세스0개를 확인했다. 원본 확대 `job-stage-2x.png`를 다시 열어 진행/시간 readout, 단계 안내 callout, 가로 동작 구성을 대조했다.

## 수정 책임

공용 inspector_field를 system_inspector에서 조합한다. MODEL의 정밀도·장치 Select와 메타데이터 Readout, JOB의 진행/시간/입출력 Readout, 공통 Info 안내를 적용했다. 선택 모델명과 설치 문구는 실제 snapshot row의 디자인 값을 사용하며 지원 여부를 새로 만들지 않는다. MODEL/JOB 본문 스크롤과 고정 footer를 분리했다. OUT는 SDK field_id를 유지한 Readout을 두 요약 위치에서 공유하며 저장 버튼은 중앙 확인 영역에만 둔다.

MODEL 검색·OUT 경로/파일명 입력은 공통32 높이, OUT 하단은3열 Flex/Grid로 바꿨다. 미연결 동시 실행 수는 —, 충돌 방식 radio는 미선택과 연결 필요로 표시한다. 다운로드·재시도·취소·저장은 여전히 실제 명령 미연결 상태다.

## 남은 검증

수정 후 빌드와 실제 화면 관측을 완료하기 전 이 수정의 시각 통과를 선언하지 않는다. 모델 revision/라이선스 pill·repo 링크·기능별 모델 목록, job별 실제 단계/capability, OUT 폴더 picker·충돌 후보, 전체 상태·폭·키보드 검증은 남아 있다.

## r17 수정 후 실제 재검증

fmt/test/clippy/build 통과(71 passed, 외부 의존 5 ignored) 후 `runtime/review-runs/ui-all-pages-20261018-r17/player.exe`(PID 120572)를 같은 가상 모니터 위치·1440×900에서 실행했다. 다른 세션의 모델 워커 종료를 확인한 뒤 빌드를 재개했다.

- MODEL: 검색 높이32, metadata readout·정밀도/장치 selector, 고정 다운로드 위치 확인. 두 번째 모델을 클릭해 상세 제목이 `모델 표시 예시 2`로 동기화됨을 확인했다. 실제 revision/라이선스·정밀도·설치는 미확인 상태다.
- JOB: 미연결 동시 실행 수 —, 단계 안내 카드, 진행/시간의 편집 상자 제거, 소스/출력 readout, 고정 가로3동작 확인. 실제 실행/진행/취소는 확인하지 않았다.
- OUT: 경로/이름 입력 높이32, 미선택 충돌 radio, 편집 상자가 아닌 요약값, 가로3개 버튼 확인. 요약의 `새 이름으로 저장`은 SDK DesignSample 문자열이며 실제 충돌 정책 선택/파일 검사 결과가 아니다.

사용자 입력 감지로 일부 자동 입력이 거절됐을 때 최신 화면을 다시 얻고 대상 창 활성화 후 진행했다. 이를 제품 클릭 실패로 판정하지 않았다. 모델 목록 방향키 선택, JOB 필터별 선택/scroll 보존, 세 화면의 좁은 창·긴 문자열·모든 상태는 미검증이다. 검토 후 캡션 닫기로 종료했다. 저장된 실제 캡처 파일은 없으며 화면을 직접 관측했다.

## r18 — 모델 키보드·시스템 좁은 창

모델 행의 입력 ID를 row_id로 격리하고 클릭 시 focus를 지정했다. 기존 공통 keyboard_selection으로 Up/Down/Home/End에 선택·focus·scroll 이동을 연결했다. fmt/test/clippy/build 통과(71 passed, 외부 의존5 ignored).

소유 가상 모니터 위치 3500,100에서 r18(PID112280, runtime/review-runs/ui-all-pages-20261018-r18/player.exe)을 하나만 실행했다. 1440×900에서 첫 모델 클릭→Down은 두 번째 모델, End는 세 번째, Home은 첫 번째로 선택과 상세 제목이 함께 이동했다. 각 대상에 focus 외곽선이 표시됐다. Up·Tab만으로 첫 진입·긴 목록/빈 목록의 실제 조작은 이번 확인에 포함하지 않는다.

창 크기 메뉴로 760×480을 선택했다. DPI·VM·모니터 토폴로지 변경은 없다.

- MODEL: 설정을 열어 metadata부터 정밀도/장치/크기까지 본문 scroll로 접근했다. 다운로드는 고정 위치를 유지했고 Escape는 설정만 닫았다.
- JOB: 탐색 레일을 내려 접근했다. 설정에서 단계·안내 카드→진행/시간→입출력·진단까지 scroll로 접근했고, 재시도/취소/출력 폴더는 가로로 고정됐다. Escape 후 대기열 화면이 유지됐다.
- OUT: 요약과 저장 위치 카드가 위아래로 적층됐다. 본문 끝까지 scroll 후 취소/뒤로/저장 시작 세 버튼과 로컬 저장 안내가 창 안에 표시됐다. 이 버튼은 본문 끝에 있으며 고정 footer라고 주장하지 않는다.

설정 첫 진입에서 request_discard 경고가 일시 관측됐고 후속 scroll 프레임에는 없었다. 실제 추론/저장/다운로드·오류 복구와 다국어 긴 문자열은 미검증이다. r18은 캡션 닫기로 종료했다.
