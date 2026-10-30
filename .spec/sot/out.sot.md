---
id: sot:kinekino/out
title: OUT 요소별 시각 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# OUT 요소별 시각 계약

[r156 속성 행 실제 검토](../../../verification/presentation/export-cycle-review-2026-10-19.md):1440/760의4개 아이콘·라벨·값 정렬과760 하단동작 접근·Empty 이전값 제거를 확인했다. 시안의5개 내용 항목과 현재 read 계약4개는 다르며 없는 값은 생성하지 않았다. 긴 속성값/hover·실제 저장 등은 미검증이다.

OUT-06a r156 후속: export.png의(1012,578,530,218)3배 확대는 확인사항의 glyph→label→value 정렬을 보여준다. 공용 `metadata_row`가 glyph20,고정label144/유동value,minheight32,FIELD8과 TEXT 색을 소유한다. 긴 label/value는 줄바꿈하고 원문 hover를 제공한다. OUT의 현재 inspector 항목만 명시적인 field_id로 Folder/Warning/FileOutput/Info에 대응시키고, 없는 재생 시간·해상도·오디오·자막 값은 생성하지 않는다. 파일 출력 glyph는 공통 stroke1.75/cap/join의 직접 작성 SVG로 관리한다.1440/760/Empty의 실제 검토 뒤 결과를 별도 기록하며 기존 시안 일치 완료로 확대하지 않는다.

[r155 실제 요약 행 검토](../../../verification/presentation/export-cycle-review-2026-10-19.md):1440 출력3행·긴문구와760 긴문구3행/다음저장영역 스크롤·Empty 이미지/파일명 제거를 확인했다. 썸네일은 소스 이미지이며 실출력 결과가 아니다. 원본 구간/설명·속성 아이콘 목록·소스 제목 위치와 기타 미검증은 남아 있다.

OUT-01a r155 후속 설계: export.png의(300,615,666,116)3배 확대를 기준으로 요약 제목 아래에 썸네일·출력명·형식의 복합 행을 배치한다. 공용 `output_summary`는144×81 의미 토큰,16:9 contain/CONTROL radius,16간격의 Taffy 고정 썸네일+minwidth0 텍스트 구조를 소유한다. 이름은 strong·줄바꿈과 원문 hover, 형식은 보조 글자로 표시한다. 현재 read 계약은 출력명/형식과 별도 소스 이미지뿐이므로 원본구간·설명·출력 썸네일을 임의 생성하지 않는다. 이미지에는 ‘소스 미리보기 · 출력 결과 아님’ 안내를 제공한다. DesignSample의 소스 이미지만 사용하고 Live 미연결·Empty에서는 이전 이미지/파일명을 남기지 않는다. 여러 출력은 각각 안정 row_id로 같은 구조를 사용한다.1440/760·LongLabels·Empty 실제 관측 후 결과를 기록한다. 아래 기존 관측을 이 수정본의 검증으로 사용하지 않는다.

[r150 수정 실행 검토](../../../verification/presentation/export-cycle-review-2026-10-19.md#r150-수정-실행-검토)에서1440/760 공용프리뷰·미상시간/비활성transport·영상아래요약·충돌미선택문구일치와760Empty잔상제거를확인했다. 복합요약/속성목록·나머지상태검증은남는다.

OUT-01a 프리뷰 후속 설계: r148에서 검정 전체 폭 면과 하단 transport가 빠졌음을 확인했다. export.png의 프리뷰 하단(299,492,667,65)4배 확대를 기준으로 기존 MediaPreview::unavailable를 OUT 요약 카드의 첫 영역에 조합한다. 원본 디자인 이미지는 contain16:9 표시를 유지하며 출력물의 확정 화면비로 승격하지 않는다. Play/Volume/Fullscreen32·미상시간·비활성seek·CHROME transport와 VIDEO_SURFACE·THIN/BORDER·CONTROL radius를 CLIP과 공유한다. 카드 상단의 요약 제목을 프리뷰 아래로 이동해 영상→요약 순서를 맞춘다. 실제 clip 재생 capability/PTS가 없으므로 시안의00:00/00:30은 복제하지 않는다. 썸네일 복합 요약/속성 아이콘 목록은 별도 미완료다. 확인 사항의 collision도 미선택 radio와 같이 ‘선택 필요’를 표시하도록 UI 데모 read 계약을 정정한다. SDK 세션은 preview_inspector.rs 동시 수정 계획 없음을 확인했다. 실제 파일 충돌 정책은 변경하지 않는다.

[r148 순환 검토](../../../verification/presentation/export-cycle-review-2026-10-19.md)에서1440 두카드/760적층·하단동작접근·Empty잔상제거를확인했다. 시안의transport/썸네일요약/속성목록누락과미선택충돌radio에대해샘플확인사항이새이름저장을표시하는불일치가남는다.

OUT-06a 후속: OUT는 자체 완결된 요약/저장 확인 화면으로 shell의 별도 우측 inspector와 좁은 폭 설정 drawer를 생성하지 않는다. 확인 값은 중앙 저장 card에 한 번만 표시하고 다른 화면의 inspector는 유지한다. 1440 중앙 폭 전체를 두 card가 공유하며980 미만은 한 열로 적층한다. 충돌 처리 두 radio는 공용 described_choice로 제목/설명을 묶되 실제 정책 연결 전 미선택·비활성이다. 새 이름의 실제 후보명이나 충돌 성공을 생성하지 않는다. 실제 실행과 미검증은 별도 기록한다.

OUT 진입 시 이전 화면의 설정 열림 상태를 해제한다. 보이지 않는 drawer 상태가 Escape를 소비하지 않도록 한다. 수정 전/후 실행 근거는 [내보내기 배치 검토](../../../verification/presentation/export-layout-review-2026-10-18.md)에 둔다.

[기능 계약](export.md)과 [공통 구성](../shared/screen-composition.md)을 적용한다. 기준 export.png, [확대 근거](../../../verification/presentation/system-elements-2026-10-18.md). 치수는 논리 설계값이다.

| ID | 배치·형태 | 상태·SDK·컴포넌트 |
|---|---|---|
| OUT-01a | heading/설명 → 왼쪽 영상 contain/transport → thumbnail·출력명·원본구간·설명 → 속성 요약 | ExportSummary, source_revision·선택 개수·형식·손실. 이미지 샘플과 실측 frame 구분 |
| OUT-02a | 오른쪽 저장 위치 label → Folder glyph20/유동 path field32/찾아보기 | PathPicker port, minwidth0/truncate+tooltip, 선택 취소 시 기존값 보존 |
| OUT-03a | 파일 이름 label → 전폭 field32 | 예약이름/확장자/길이 오류를 field 아래, 이름을 조용히 바꾸지 않음 |
| OUT-04a | 동명 충돌 amber icon+banner → 새 이름 권장 radio·실제 후보명 보조 → overwrite radio·설명 | destination revision/개별 파일 범위, 충돌 해결 전 실행 불가. 일괄 skip 옵션은 대상 있을 때 표시 |
| OUT-05 | 크기·여유/임시 공간 summary | 알려진/추정/미상 구분, 가짜0 bytes 금지 |
| OUT-06a | 구분선 → 확인사항: 시각/형식/크기/오디오/자막/alpha/color icon-label-value | 중복 요약은 같은 typed snapshot으로 생성, 화면 간 문구/값 불일치 금지 |
| OUT-07a | 취소/뒤로/저장 시작 row, 주요 amber40, 나머지 중성 | 고정 preflight snapshot 생성, 클릭 중복 방지. 뒤로는 초안 보존, 닫기는 작업취소 아님 |
| OUT-08/09 | 검증 단계·결과 summary/상태 | 파일쓰기→검증→원자적 확정, 시작 화면과 완료 receipt 구별 |
| OUT-10/11 | 실패/취소 해결 동작, 완료 경로/크기/검증·폴더열기 | 소유 임시파일만 정리, 완료파일 보존. 재실행과 폴더열기 별도 |
| OUT-12a | 하단 info glyph+로컬 저장 안내 | 외부 게시 버튼 없음, CARD 게시 대상 프리셋과 실제 업로드 혼동 금지 |

1440에서 중앙 유동 영역을 요약/저장 두 열로 나눈다. 오른쪽은 긴 inspector 폼을 별도 작은 카드에 압축하지 않고 저장 확인 영역으로 유지한다. 980 미만은 요약→저장 세로, 공통 drawer 접근을 적용할 때 두 번째 저장 버튼을 만들지 않는다. footer는 760에서도 wrap하여 세 동작 접근 가능, body13·small11·group16·padding12/radius5를 사용한다.

충돌/쓰기불가/경로없음/디스크부족/형식불가/부분출력/완료를 별도 확인한다. NeedsInput sample은 실제 파일 충돌 검사 결과처럼 표시하지 않는다. 시안에서 CLIP 탐색이 선택돼 있어도 OUT 목적과 돌아갈 작업을 명시한다. 현재 OUT 일반 검색+table은 이 저장 확인 구성을 대신하지 못한다. typed export plan/summary/preflight/collision candidate/receipt가 필요하다.
