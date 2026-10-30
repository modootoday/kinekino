---
id: sot:kinekino/seg
title: SEG 요소별 시각 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# SEG 요소별 시각 계약

[r114/r115 프리뷰 크기 검토](../../../verification/presentation/media-preview-sizing-review-2026-10-19.md): SEG 영상이 viewport에 맞춰 확장/축소되며1440·최대화·760 표시를 확인했다. OBJ 도구의 최소 높이 제약과 다른 공유 소비 화면의 검증 범위는 기록에 구분했다.

[r111/r112 공용 도구 모음 검토](../../../verification/presentation/inline-toolbar-review-2026-10-19.md): 아이콘/이름 결합·Delete SVG·1440/760 표시와 설정/Escape를 확인했다. 동일 Taffy ID로 발생한 재계산 회귀를 호출자별 ID와 검사로 수정했다. 행별 편집/삭제 배치와 실제 명령은 여전히 별도다.

[기능 계약](segments.md), [공통 구성](../shared/screen-composition.md), [확대 관측](../../../verification/presentation/segment-subject-clip-elements-2026-10-18.md)을 적용한다. 기준 skip-settings.png. 아래 수치는 논리 설계값이며 raster 측정값이 아니다.

| ID | 배치·형태 | 데이터·상태·책임 |
|---|---|---|
| SEG-01a | 제목/설명 → 영상 contain/transport → filmstrip/ruler → 감지 구간 표 | media_id/fingerprint·PTS, SourcePreview/SegmentTimeline. 미상 길이는 생성된 전체범위로 대체 금지 |
| SEG-02/03/04 | 검사기 감지 방식·모델/revision/준비·분석 시작 | 실제 capability로 옵션 제한. 모델/입력 필요 이유, 무거운 분석은 공통 queue |
| SEG-05a | 썸네일 시간축 위 OP amber/ED 보라 범위 채움·양끝 handle·아래 종류/시간 pill | 종류와 확인 상태는 문자로도 표시. 독립 playhead, source µs→같은 ruler 변환. thumbnail은 실제 frame 또는 표시된 sample |
| SEG-05b | 표 heading/개수 → 종류/시작/끝/길이/상태/작업, header28/row32 이상 | stable annotation_id, 후보/사용자확인 badge, 수정 버튼+삭제 SVG hit32. 임의 confidence 금지 |
| SEG-06/07 | 선택 구간 numeric in/out(ms), 현재시각 지정·±1초·미리보기 | drag와 숫자 동일 draft, 경계 invalid는 필드 오류. 전후3초 clamp, 복귀 위치 보존 |
| SEG-08/09 | 확인/해제·삭제·undo 명명 동작 | 확인 revision과 candidate 분리, 미확인은 자동skip 불가 |
| SEG-10/11 | 검사기 OP/ED mode 별도 selector32·설명/구분선 | 끔/수동/자동 개별 저장. 보라색은 ED 보조 구분이며 focus/오류선으로 혼용 금지 |
| SEG-12/13 | 예고 시간·건너뛰기 후 동작 → 최근 작업 카드·되돌리기 | preview 영상 우하단 countdown/이번에는보기, 재생시간 기반·pause중 정지. 최근 작업은 실제 receipt/evidence |
| SEG-14 | 다음화 자동재생 switch+설명 | ED mode와 별도, 마지막화·극장판 불가 이유 |
| SEG-15/16 | 하단 구간수정 secondary/현재회차저장 primary40, 작품예외/초기화 적용범위 | media/revision 연결, 다른 회차 묵시 복제 금지 |

입력32·주요40·body13·small11·group16/padding12. 1280 검사기drawer, 1180 탐색48, 980 미만 표의 시간/상태 핵심 유지·보조행 확장. filmstrip은 수평 time viewport를 가지며 창을 확장하지 않는다. Empty/NeedsInput에서도 source·시간축·표 골격 유지, populated는 OP/ED 확인 차이, long labels는 tooltip. 경계선/handle/playhead의 hit 영역과 광학 크기를 분리한다.

필요 컴포넌트는 MediaPreview, FilmstripRangeEditor, SegmentRows, SkipPolicyInspector, RecentActionCard다. 공통 시간축 기하만 CLIP/OBJ와 공유하고 데이터·검증 정책은 application 소유다. 현재 generic timeline/table은 썸네일·범위 handle·확인 badge·최근 동작을 대체하지 못한다. 해당 typed read model과 actual command 가용성이 준비되기 전 표식을 편집 가능한 것처럼 활성화하지 않는다.

## React/TSX SEG 검사기 제작 전 대응 — 2026-10-20

PM이 신규 미연결 `src/presentation/web_ui/studio/segments/inspector.tsx` 한 파일 source-only 제작을 배정했다. 기준은 manifest.selected_studio.segments의 v2/studio/skip-settings.png(1586×992 raster)다. 원본과 검사기 crop x1190/y50/w396/h818·2배(`runtime/coordination/ui-seg-inspector-s1/reference-inspector-2x.png`)를 직접 열었다. crop 좌표는 검토 영역이며 개별 요소 bounds/논리 치수 실측은 아니다. 기존 Rust 실행의 검증을 새 TSX에 상속하지 않는다.

| 요소 | 시안에서 확인한 대응 | source 구조·상태·자산 | 남은 검증 |
|---|---|---|---|
| SEG-10/11 | 상단 제목 뒤 OP/ED label+selector, OP amber/ED 보라 가는 선, 아래 확인 안내 | 독립 controlled mode ID/option availability/reason·Text callback. 기존 Input 계열 필드 클래스/공용 info SVG 안내. 선택 기본값·자동 스킵 판정은 Rust 제공 | 실제 select key/focus/arrow·두 보조색 selector border·spacing/glyph 1:1 미검증. 기존 공통 CSS 변경0 |
| SEG-14/08 | 재생 옵션의 다음화/회차별 확인 label 왼쪽·pill switch 오른쪽 | shared Switch checked/onCheckedChange의 Rust 권위 controlled 선언. 마지막화/미디어 종류·확인 가능 여부/reason은 Rust props | Native controlled Switch/Boolean host callback은 현재 미지원. defaultChecked/텍스트 대체/항상 비활성 mock으로 축소하지 않음 |
| SEG-12 | 추가 설정의 건너뛰기 후 동작·미리 알림 selector | application이 제공한 선택지 ID/label/가용성·현재 값만 전달, countdown/paused 시간 계산0 | 옵션 변경/지원 거부/late props 및 시안 치수 미검증 |
| SEG-13 | 최근 작업의 info glyph·실제 이동 문구/시간·되돌리기 | InformationNotice와 기존 공용 info.svg 재사용. 실제 receipt 문구·undo capability/reason·명시 Action callback | 최근 작업 없음/undo 만료/실패·선택 receipt 상관·실제 배치/줄바꿈 미검증 |
| SEG-15/16 | 하단 secondary 구간 수정·amber 현재 회차 저장, 적용 범위 | 기존 inspector scroll/footer·Button token과 controlled canEdit/canSave/busy/결과·범위 표시 | footer 실제 크기·키보드 접근·변경 전후 dirty/저장 충돌·다른 회차 불변 미검증 |
| SEG-02~04 | 상세 분석 방식·모델 상태·시작은 기능 계약의 필수 영역 | Rust가 제공한 방식·모델/revision/정밀도·canAnalyze/reason 표시와 명시 분석 Action. 임의 confidence/모델 준비·progress 생성0 | 시안에 없는 상세 정보의 공통 구성 의도, 실제 queue/SDK bridge 및 모델 가용성 미검증 |

모든 치수·색·border/radius·폰트는 기존 aw-inspector/aw-text-input/aw-metadata/aw-notice/aw-button 및 공통 토큰/Pretendard를 재사용한다. 1280 drawer와 좁은 폭 접근은 기존 InspectorDock 소유이며 이 신규 파일은 새 breakpoint·CSS·raw 수치를 만들지 않는다. 별도 glyph/폰트·새 raster/데모 값도 넣지 않는다. source-only이므로 실제 overflow·hover/pressed/selected/focus/disabled·1440/1180/760·모든 언어 및 시각 일치는 미검증이다.

Native `application/toggles.rs`는 controlled checked를 명시 거부하고 Core host callback signature는 Action/Text뿐이다. Bool state/InputKind::Checked 자체의 기존 지원과 Boolean host callback 부재를 구별한다. 예상 경계는 callback signature의 `unsupported`/`text callback requires one required plain string parameter`와 Native의 `controlled checked requires a state binding; use defaultChecked` 오류이며 이 source의 실제 진단/첫 오류 순서를 실행한 것은 아니다. 필요한 보완은 shared controlled Switch의 Boolean props/Boolean callback·부모 prop 갱신·disabled/keyboard·state 재투영의 의미 보존이다. [Meari 독립 후속 계약](../../../../../packages/meari/.spec/plans/native-document-callbacks.md)을 따르며 Core/Native 보완은 Meari 소유, 제품 정책은 Rust application 소유다. 지원 뒤 같은 원본 source의 Rust parse/consumer와 실제 시각을 재검증해야 한다.

신규 source 후보 SHA256은 `1d4b0da7b0aaba755cfdf29ecc59aa2a56f223596fc67eaccc576038783cdf49`다. graph 조사·원본/crop SHA·기존 입력 불변·정확 Switch2 계약·미검증 범위와 사본은 `runtime/coordination/ui-seg-inspector-s1/`에 보존했다. 새 route/export/import 등록은 없고, 기존 공통 소스·스타일·SDK/Native/manifest 변경0이다. 세부 파싱/실제 실행 성공·시안 일치·전체13화면 완료로 승격하지 않는다.
