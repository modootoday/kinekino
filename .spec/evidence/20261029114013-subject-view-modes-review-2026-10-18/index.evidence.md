---
id: evidence:kinekino/20261029114013
title: OBJ 원본·마스크·합성 보기 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# OBJ 원본·마스크·합성 보기 검토

2026-10-18. OBJ-07, 기준 `.spec/artifacts/design/v2/studio/subject.png`. 전체 이미지와 영역(928,158,242,48)의 2×/4× 파생물을 검토했다. 기준은 어두운 overlay 안의 세 칸, 작은 곡률, 중성 면과 선택 amber 문자/외곽선이다. 생성 이미지의 원본 선택과 cyan 동시 표시는 의미 충돌이므로 복제하지 않는다.

## 구현과 비교

공용 toolbar::overlay_frame에 segmented 세 항목을 조합했다. hit32·최소 폭72·항목 간격8·padding6·영상 안쪽8·border1·radius5를 기존 토큰으로 사용한다. 화면 전체를 기준 이미지와 같은 것으로 판정하지 않는다. 원본은 mask/contour/bounds를 숨기고, 마스크는 동일 SourceTransform의 검정/흰 alpha, 합성은 기존 cyan28%/contour/white bounds를 유지한다. 수동 SVG 표본이며 실제 추론 결과가 아니다. source asset·PTS·DesignSample 일치 검사를 유지하고 UI 보기 상태로 원본/SDK 값을 바꾸지 않는다.

## 실제 관측

r73, PID94392, `runtime/review-runs/ui-subject-modes-20261018-r73/player.exe`, 1440×900에서 기본 합성과 원본 클릭 후 오버레이 제거, 마스크 클릭 후 검정 배경/흰 silhouette를 확인했다. 좌측 도구와 우상단 보기는 겹치지 않았다. Tab/Space 입력만으로 다른 보기 활성화는 확인되지 않아 공용 segmented에 stable ID·클릭 focus와 방향키 이동을 후속 적용했다. r73 창 닫기 후 player/mpv 조회 결과는 비어 있었다.

r74(PID28304)는 stable 항목 ID·방향키 처리 추가 후의 중간 실행이다. 합성→원본 클릭 직후 Right는 원본에 남았지만, 원본을 다시 클릭→Right는 마스크로 이동했다. egui 0.36.2의 Ui::new_child는 id_salt 외에 부모의 자동 순번을 unique ID에 포함한다. mask paint의 자식 생성 유무가 바뀌어 focus가 유실되는 원인과 일치한다. 보기 overlay에 UiBuilder::id로 명시적 ID를 부여하고, 같은 전환 직후 Right로 마스크가 선택되는 Rust 회귀 검사를 추가했다. 이 검사는 통과했다. 첫 Clippy는 test module 뒤의 show 함수 배치를 지적해 test module을 파일 끝으로 옮겼다.

## 최종 실행 재검토

r75(PID100500), `runtime/review-runs/ui-subject-modes-20261018-r75/player.exe`에서 확인했다.

- 1440×900: 기본 합성→원본 첫 클릭→Right가 곧바로 흑백 마스크를 표시했다. End는 청록 mask/윤곽/bounds의 합성으로 복귀했다. r74에서 필요했던 두 번째 클릭이 필요하지 않았다.
- 760×480: 좌측 rail과 세 보기 버튼이 영상 안에서 겹치지 않았다. 원본 클릭→Right→End로 원본/흑백 mask/합성을 확인했다. 영상 하단과 transport는 본문 스크롤 범위이므로 한 화면에 모두 들어간 것으로 판정하지 않는다.
- 760 설정 열기→안정 프레임에서 피사체 목록3개와 고정 하단 동작을 확인했다. Escape는 설정만 닫고 제작 화면과 합성 표시를 유지했다.
- 760 Empty: 이전 영상·mask·선택 강조가 사라지고 소스 이미지 연결 필요와 세 보기의 미선택·비활성이 표시됐다. 좌측 rail·우상단 overlay 골격은 유지됐다.
- 최종 cargo fmt --check, cargo test, cargo clippy --all-targets -- -D warnings, player 빌드가 통과했다. lib65통과/1ignored이며 추가 포커스 회귀 검사를 포함한다. 외부 미디어 조건의 기존 ignored 검사는 이번 실행 성공으로 간주하지 않는다.

소유 r75 창을 캡션 닫기로 종료한 뒤 player/mpv 프로세스 조회 결과는 비어 있었다. VM·DPI·모니터 구성을 변경하지 않았다.

## 미검증

전체 OBJ 피사체 목록 thumbnail/선택 동기화·실제 mask 편집/추론·track 실패 interval·동영상 PTS 연동은 남는다. 실제 마스크가 없는 live 연결과 GPU 합성은 이번 구현 범위가 아니다. 기존 수동 silhouette의 거친 경계와 시안 미디어 차이를 실제 segmentation 품질로 해석하지 않는다.
