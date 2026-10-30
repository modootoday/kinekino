---
id: sot:kinekino/clip
title: CLIP 요소별 시각 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# CLIP 요소별 시각 계약

## React/TSX 추천 행 제작 전 대응 — 2026-10-20

PM이 `src/presentation/web_ui/studio/clips/recommendation-item.tsx`, `src/presentation/web_ui/styles/shorts.css`, `assets/design/icons/source-link.svg`의 신규 미연결 source-only 제작을 배정했다. 기존 공통 CSS/TSX/tokens·SDK/Native/generic/manifest·route/import/export 등록은 변경하지 않는다. 기준은 short-recommend-refined.png(1586×992)이고 원본과 x260/y622/w878/h86·4배 crop을 직접 열었다. crop은 `runtime/coordination/ui-clip-recommendation-s1/reference-item-4x.png`이며 개별 bounds/논리 치수 실측은 아니다.

| 요소 | 원본 대응 | source·데이터·공통 책임 | 미검증 |
|---|---|---|---|
| CLIP-05b/include | 첫 열 checkbox | preview 선택과 다른 Rust authoritative included/canInclude/reason·Boolean intent. Shared Checkbox 원본 선언 유지 | controlled checked/Boolean host callback 미지원. Action toggle·생략·항상 비활성 mock으로 대체하지 않음 |
| CLIP-05b/preview | 16:9 thumbnail·후보 제목/번호 | 실제 등록 asset을 Rust가 resolve한 경로/alt·missing 안내. 명시 previewSelected·선택 Action, ID는 표시값이며 권한 아님 | asset delivery·선택/이미지/Fit·missing·long labels 미검증 |
| CLIP-05b/source | 범위/길이·chain SVG 원본 이동 | canonical source 참조의 Rust 표시 문자열·canOpenSource/reason·Action. 공통 source-link.svg 직접 작성, 기존24 viewBox/1.75 stroke/round cap·join/색 상속 | source PTS/원본 이동·경로 해결/paint·정규화 배율 미검증. JS 시간 계산0 |
| CLIP-05b/evidence | 근거 종류·원문 snippet | Rust가 검증한 evidence/source text·미상/품질 표시를 그대로 표시. 임의 발화·confidence·cue/scene/manual 이유 생성0 | 실제 source binding과 명시 수정 provenance·장문·지원 범위 미검증 |
| CLIP-05b/actions | play/preview·scissors/edit·delete/exclude | 기존 Button/play.svg/clip.svg/delete.svg와 독립 canX/reason·Action. 포함/preview/제외/실제 파일 삭제는 별개 | callback/권한·pending/실패·키보드/disabled·실제 효과 미검증 |
| CLIP-05b/states | 중성 card/selected amber선/focus 별도 | scoped aw-clip-recommendation*·기존 토큰. thumbnail132/CandidateActions240은 기존 Rust size 정본의 scoped Web projection, 980 미만 actions 다음 행 | actual spacing/border/radius·선택/focus/hover/pressed·1440/1180/760·전체 목록 Up/Down/Home/End 미검증 |

source 제작은 기존 UI 실행의 인수를 상속하지 않는다. 동일 source를 Rust parser/generator/builder/runtime consumer·typed callback/asset delivery와 실제 확대 시각 gate에서 검증하기 전 완료로 표시하지 않는다. 포함2상태 Checkbox는 SEG와 같은 controlled Boolean 후속이며 indeterminate/tri-state를 임의 추가하지 않는다. 범용 지원은 [Meari callback 계획](../../../../../packages/meari/.spec/plans/native-document-callbacks.md)이 소유한다. UI는 추천/시간 변환/저장·삭제/선택 업무 정책을 복제하지 않는다.

작성한3파일 source-only 후보 inventory는 `runtime/coordination/ui-clip-recommendation-s1/candidate-pins.json` SHA256 `f15e24827194a39cdc3f0bfa327d251d94bc6835ac64bdfe74c0c67c73ec2be0`이며 source/ 사본·ownership.md·preflight.md·기존 UI16/SEG/generic/공통 입력 불변 관측을 같은 디렉토리에 보존했다. 실제 CSS/asset import 등록과 parser/compile/GUI 실행0이다. 중성 근거색과 시안 보조색 차이, wide240 actions의 실제 wrapping, status/사유·장문으로 인한 높이, 좁은 폭/키보드와 glyph·폰트 실측을 미검증으로 유지한다.

[r147 배지·격자 실제 검토](../../../verification/presentation/batch-source-table-review-2026-10-19.md#r147-배지세로선-실행-검토):1440/760 긴제목과760 Empty의표시를확인했다. 행간세로선틈·시안대비배지크기차이는남아있다.

CLIP-06a 상태·격자 후속 설계: short-batch.png의 표 확대본(r143,3배)에서 상태는 중성 면의 둥근 배지이며 열 사이에도 가는 구분선이 있다. 상태 문자열은 그대로 유지하고 기존 공용 badge::show의 FIELD/BORDER/SECONDARY·PILL·SMALL·공통 padding을 재사용한다. ‘검토 전’을 ‘대기 중’으로 바꾸거나 작업이 큐에 들어갔다고 암시하지 않는다. 배지는 가용 상태 열 안에서 말줄임·전체 hover를 제공하며 버튼으로 취급하지 않는다. 세로 선은 열 규격에서 계산한 경계를 기준으로 THIN/BORDER를 사용하고 제목/썸네일·텍스트 padding 및 바깥 radius를 침범하지 않아야 한다. 독립 화면 painter 좌표를 복제하지 않는다. 이 항목은 아직 설계이며 구현·실행 검토 전이다.1440/760·긴 상태·Empty·스크롤에서 프레임과 정렬을 다시 확인한다.

CLIP-06a 세로 정렬 후속: r143에서 썸네일 옆 제목이 위쪽에 붙은 차이를 관측했다. 회차 내부의 공용 Taffy 두 열도 Center 정렬하여36 높이 썸네일과 제목의 중심을 맞춘다. 행 바깥6열의 중앙 정렬과 동일한 기준이며 별도 수동 y보정을 넣지 않는다. 수정 실행본1440/760에서 재검토 전 완료로 표시하지 않는다.

CLIP-06a 표 폭 후속: short-batch.png의 checkbox/회차(thumbnail+제목)/범위/후보수/출력/상태6열을 공용 BatchSourceTable로 조합한다. header/body는 동일한 Taffy 열 계약을 공유한다. checkbox32·시간132·후보/출력48·상태88, 회차는 유동 열이며 고정 합은 TABLE_FIXED_MAX65% 제한을 따른다. thumbnail64×36·행 최소48·padding8·border1/radius5·CHROME header를 사용한다. 후보 미상/출력 미연결은 대시로 표시하며 실제 media ID/포함 command가 없으면 checkbox는 비활성이다. 기존 디자인3행을 실제 회차로 승격하지 않는다.1440/760 LongLabels와 Empty를 실제 확인한다.

[r142 검사기 첫 제목 검토](../../../verification/presentation/inspector-first-heading-review-2026-10-19.md)에서1440 세 하위 화면의 선행 빈 구분선/여백 제거를 확인했다. 후속 그룹 구분과 footer는 유지한다.

[r140/r141 순환·프리뷰 면 검토](../../../verification/presentation/clip-preview-surface-review-2026-10-19.md)에서 수동/추천/일괄 기준 실행과 AI추천 검정 면/transport 수정본1440·760·Empty·후보 End·drawer/Escape를 확인했다. 일괄 표 폭·수동 crop·검사기 상단 여백은 남은 차이다.

CLIP-05 프리뷰 면 후속: r140 실제1440 AI추천은 세로 이미지가 본문 바탕 위에 단독 배치돼 short-recommend-refined.png의 검정 전체 폭 canvas/하단 transport와 다르다. 공용 MediaPreview는 VIDEO_SURFACE 면·THIN border·CONTROL radius 안에 기존9:16 artboard를 가운데 두고 아래 별도 CHROME transport를 조합한다. Play/Volume/Fullscreen SVG32와 알 수 없는 시간/비활성 seek·명시9:16을 표시한다. 실제 재생/clip-local PTS가 없으므로 시간을 만들거나 조작 성공을 표시하지 않는다. 해당 variant는 AI추천에 먼저 적용하고 수동의 두 preview/일괄 표와 섞지 않는다. 실제1440/760·후보 선택·Empty를 재검토한다.

후보 행·일괄 요약의 r76 기준 관측과 r77 수정본1440/760·LongLabels·Empty·목록 키보드·설정/Escape 결과는 [후보 행 검토](../../../verification/presentation/short-candidate-review-2026-10-18.md)를 따른다. 이 기록은 전체 CLIP 시각 인수가 아니다.

CLIP-05 후속: 공용 selectable card로 후보 행의 panel/border1/radius5/padding12와 선택면·좌측 amber선·별도 focus를 공유한다. 선택된 preview 후보와 작업 포함 checkbox는 별개다. 포함 계약이 없으면 checkbox는 비활성·미선택으로 둔다. thumbnail132×74.25·제목·snapshot range/duration·수동 디자인 근거 안내·미연결 원본 링크와 공용 icon+label32 버튼을 조합한다. 980 미만은 동작을 다음 행으로 배치하고 Left/Right가 아닌 목록 Up/Down/Home/End로 preview 선택을 이동한다. 실제 근거·발화·후보수·시간을 새로 생성하지 않는다. CLIP-06b 요약은 공용 card 안의 반응형 grid로 제목/네 값을 묶고 현재 값의 미연결 상태를 보존한다. 적용 후 실제 실행으로 검토한다.

r50에서 트랙 header SVG·라벨을1440/760 및 Empty로 [실제 확인](../../../verification/presentation/timeline-track-header-review-2026-10-18.md)했다. 설정→Escape 후 스크롤 위치와 트랙 구조도 유지했다. 다국어 긴 label·실제 track 조작은 별도 범위다.

CLIP-12 트랙 label rail 보완: 영상은 filmstrip, 자막은 사각 T, 오디오는 음표 SVG를 사용한다. 공용 TimelineTrackHeader가 typed kind별 용어·glyph 매핑과 icon20/간격6/라벨 기준선을 소유한다. 현재 rail80을 유지하고 row의 수직 중앙에 정렬하며 부모 clip을 상속한다. 음소거·선택 같은 동작을 추가하지 않는다. 적용 후 실제1440/760 및 Empty를 재검토한다.

r49에서 공용 세로 grip을 CLIP1440/760·SEG1440·OBJ760/Empty로 [실제 확인](../../../verification/presentation/timeline-range-grips-review-2026-10-18.md)했다. label·부모 clip 회귀를 확인했으며 active range 의미와 편집 상태 및 시안의 투명 선택면은 남아 있다.

CLIP-12 범위 손잡이 후속 설계: 공용 filmstrip 범위 양끝을 amber 세로 grip으로 표시한다. 원형 하단 점을 제거하고 중앙 정렬한12×32 논리 단위, 공통 small radius와 얇은 내부 mark를 사용한다. track와 부모 clip rect를 함께 적용하고 label은 손잡이와 겹치지 않게 제한한다. 표시 snapshot만 사용하는 현재 단계에서 drag 편집 동작을 암시하는 cursor를 추가하지 않는다. 실제 CLIP/SEG/OBJ 검토 전 완료로 표시하지 않는다.

2026-10-18 CLIP-12 후속: short.png의3트랙 영역(272,666,868,240)을2배 확대했다. 공통 ruler/동일 x변환 위에 영상 filmstrip→보라 자막 blocks→녹색 waveform 순서와 좌측 label rail을 둔다. 원본 µs·표시 범위·현재PTS는 같은 snapshot에서 받으며 가짜 seek/편집은 제공하지 않는다. DesignSample의 반복 프레임·구간 label·수동 파형은 안내로 명시하고 live/Empty에는 주입하지 않는다. ruler32·label rail80·영상64·자막32·오디오44·gap4는 token 기반 논리값이다. 실제 실행 후1440/760과 SEG/OBJ 공용 filmstrip 회귀를 별도 기록한다.

[기능 계약](shorts.md), [공통 구성](../shared/screen-composition.md), [확대 관측](../../../verification/presentation/segment-subject-clip-elements-2026-10-18.md)을 적용한다. 기준 short/short-recommend-refined/short-batch. 아래 치수는 논리 설계값이다.

| ID | 배치·형태 | 데이터·상태·컴포넌트 |
|---|---|---|
| CLIP-01 | heading/설명 → 수동/AI추천/일괄 탭32·amber밑줄 | stable subview ID, 동일 ClipDocument revision 연결. 탭별 다른 구도와 read model |
| CLIP-02/03 | 수동: 세로 crop canvas `|` 원본 preview/transport → 범위·길이/zoom → 다중track 시간축 | 원본PTS와 clip local time 명시, 양끝 handle·선택면·독립 playhead. crop view를 generic 가로 이미지로 대체 금지 |
| CLIP-04 | 추천 inspector 후보수/길이/근거 checkbox/중복 제외/model/추천생성 | 실제 능력·근거만, 입력필요/생성중/결과0개 구분 |
| CLIP-05 | 추천 세로 preview → 후보 count/목록: checkbox·thumbnail·번호·시간/길이·원본이동·근거/짧은문구·3동작 | CandidateRow. preview/edit/exclude SVG32, cue/scene/manual 근거 label. 추천은 초안이며 원문없는 대사 생성 금지 |
| CLIP-06a | 일괄: 회차선택 card(header·선택개수/전체선택) → checkbox/thumbnail·회차·범위·후보수·출력·상태 표 | EpisodeBatchTable, stable media_id. 후보 미계산은 `—`, 시안12/8/15를 실제결과로 복제 금지 |
| CLIP-06b | 표 아래 계획요약 card: 비율/크기/모델/경로, inspector 템플릿·회차당설정·자원·재시도 → 계획/시작 | BatchPlanSummary. 선택0개는 시작불가, 계획과 실제 queue 실행 분리 |
| CLIP-07/08 | 9:16 crop frame·3분할 grid·corner/edge handles·외부 어둡게, 원본 비교 | normalized crop, fit(여백)/fill(잘림) radio/설명. output aspect 유지, 양쪽 시간 동기 |
| CLIP-09 | 피사체 selector+자동/수동구도 | 검토된 subject ref, 소실/컷 needs review, 고정crop대안 |
| CLIP-10 | 세로 canvas 자막 안전영역 dashed box, 시간축 subtitle blocks 보라·텍스트 | CueRange/FontStyle, burn-in과 sidecar 구분, Pretendard 기본·출력px 명시 |
| CLIP-11 | audio track 녹색 waveform·glyph/label, inspector 원음volume/mute | 실제 샘플 또는 명시 DesignSample, audio timeline과 clip source time 동기 |
| CLIP-12 | video filmstrip/자막 blocks/audio waveform 3track, label rail·같은 ruler | TrackTimeline. 가로스크롤/zoom은 공통 time transform, selected clip stable_id, undo/순서 명명 동작 |
| CLIP-13/14 | inspector 비율/해상도·format/codec/audio·정확cut/빠른cut | capability 기반, encoder없음/확대경고/키프레임오차 표시 |
| CLIP-15/16 | 출력미리보기 secondary/작업추가 primary40 | 저장과 preview별도, OUT preflight, 일괄실패별재시도·중복방지 |

검사기300·입력32·주요40·body13/small11/padding12/group16. 후보 row는 내용 기반 최소높이64, 썸네일은 16:9 contain/선택한 crop 용도 구분. 980 미만 수동 crop/원본은 세로, 후보행 근거·동작도 다음행으로 wrap, 일괄요약4열은2열/1열. table 폭을 window 밖으로 늘리지 않고 보조 상세를 확장행으로 이동한다. 760에도 탭/설정/큐/주요동작 접근을 유지한다.

Empty/Populated/LongLabels/NeedsInput은 각 하위 화면별로 필요하다. 선택후보와 선택회차를 공유 index로 섞지 않는다. 작업 추가된 상태와 편집중 초안, 후보제외와 원본삭제를 구분한다. keyboard로 범위숫자·후보선택·원본이동·편집에 접근하며 drag만 강제하지 않는다.

필요 컴포넌트 CropComparison/TrackTimeline/CandidateList/BatchEpisodeTable/BatchPlanSummary/ClipInspector. 일반 workspace table/timeline으로 전용구조를 대체하지 않는다. typed subview/crop+safe-area/tracks/waveform/candidate evidence/batch plan/capability 계약을 SDK와 정하고 UI에 편집정책/encoder 명령을 복제하지 않는다.
# 실제 검토 보강

[일괄 작업 공용 표 r143](../../../verification/presentation/batch-source-table-review-2026-10-19.md): 1440/760 긴 제목·빈 상태·drawer/Escape 관측과 남은 정렬·구분선·배지 차이를 구분한다.

