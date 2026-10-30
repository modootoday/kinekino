---
id: evidence:kinekino/20261029113745
title: JOB 동작 버튼 시각 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# JOB 동작 버튼 시각 검토

2026-10-18 JOB-03/07/08/09. 기준 `v2/studio/queue-refined.png` 전체와 rows(297,245,805,270)2×, actions(1140,698,418,54)4×를 검토했다. 파생물은 design/review/job-rows-r90-2x.png와 job-actions-r90-4x.png다.

## 변경 전 실제 관측

r90(PID104196), `runtime/review-runs/ui-job-list-20261018-r90/player.exe`,1440×900: 재시도/취소/출력 폴더가 아이콘 없는 작은 텍스트 버튼이고 전체 시작도 glyph가 없었다. 표는 시안보다 낮은 일반 행이며 작업별 glyph·입력/상태의 분리가 부족하다. 다섯 단계는 실제 작업에서 공급되지 않는 미연결 상태이며 단계 계약은 후속이다. r90은 캡션 닫기로 종료했다.

## 공용 구현

Button::icon으로 기존 직접 작성 SVG를 label과 같은 hit 영역에 넣는다. Play/Undo/Close/Folder20, 버튼40·border1/radius5를 공유한다. JOB 검사기는 Taffy FlexWrap으로 각 버튼을 온전히 다음 줄로 보낸다. 기존 텍스트 버튼의 색상 처리는 유지하고 egui가 disabled를 처리한다. 작업 명령은 비활성과 연결 필요 사유를 유지한다.

## 실행·검증

r91(PID56316), `runtime/review-runs/ui-job-actions-20261018-r91/player.exe`를 기존 가상 모니터3500,100에서 실행했다.1440×900에서 전체 시작 Play 및 하단 Undo/Close/Folder glyph와 label이 같은 버튼에 표시됐고 세 하단 버튼이 한 줄에 들어갔다.760×480 설정 패널에서는 재시도/취소가 첫 줄, 출력 폴더가 둘째 줄에 온전히 표시됐다. 버튼과 글자가 분리되거나 잘리지 않았다. 긴 작업명은 목록에서 말줄임, 설정 제목에서는 여러 줄로 표시됐고 하단 동작은 계속 접근 가능했다.

760 첫 작업 클릭→Down은 두 번째 프레임 저장 작업 선택과 설정 제목에 반영됐다. Escape로 설정을 닫은 뒤 두 번째 행의 선택 표시를 확인했다. LongLabels 전환은 새 fixture를 구성해 첫 행 상세로 돌아갔으며 이를 이전 선택 보존으로 주장하지 않는다. 긴 제목 상태에서도 설정/Escape 복귀를 확인했다. 실제 작업 동작은 모두 비활성이며 실행하지 않았다.

최종 cargo test 통과(라이브러리65통과/1ignored, CLI·통합 테스트 통과, 기존 외부 미디어 ignored 유지). 중간 색상 처리 정리 후 fmt --check에서 한 줄 포맷 차이가 나와 cargo fmt로 수정했고 최종 fmt --check와 clippy --all-targets -- -D warnings가 통과했다. 최종 의미의 코드로 cargo build --bin player가 성공한 바이너리를 사용했으며 이후 변경은 포맷뿐이다. r91은 캡션 닫기로 종료했고 CIM에서 player.exe/mpv.exe가 없었다. VM·DPI·모니터 구성은 변경하지 않았다.

미검증/미완료:980/1180, Empty/NeedsInput의 이번 버튼 재검토, 활성/hover/pressed/키보드 focus 버튼 상태, 실제 작업 capability, 목록의48높이·작업별 glyph·입력/상태 분리, typed 단계·진단·출력 연결. 이번 변경으로 전체 JOB 시각 인수나 실행 기능 완료를 판정하지 않는다.
