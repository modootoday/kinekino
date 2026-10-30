---
id: evidence:kinekino/20261029114036
title: CARD 문구·CLIP 제안 검토 source checkpoint
status: active
created: 2026-10-29
updated: 2026-10-29
---
# CARD 문구·CLIP 제안 검토 source checkpoint

상태: 패널 TSX/CSS source 작성, 관련 Rust adapter는 [controller gate](proposal-controller-2026-10-19.md)의16검사/strictClippy 종료0. 패널 parser·generator·builder·runtime·실제 화면·시각 일치 검증은 미실행이다.

## 기준과 범위

기준은 승인된 `v2/studio/cards.png`와 `v2/studio/short-recommend-refined.png`다. 두 원본 이미지를 직접 열어 CARD 우측 텍스트/요약 초안 영역과 CLIP 추천 후보의 근거/문구·우측 설정 구분을 재확인했다. 이번 단계는 [CARD-08/10](../../player/presentation/studio/cards-visual.md) 및 [CLIP-05](../../player/presentation/studio/shorts-visual.md)에 연결되는 검토 패널 source다. 실제 화면의 확대1:1 비교·정렬·상태·키보드 인수는 하지 않았다.

CARD 전체 toolbar/thumbnail rail/canvas/출력 설정과 CLIP 전체 세로 preview/후보행/생성 설정을 이 패널로 대체하지 않는다. 모델 문구 제안과 CardDocument·ClipDocument의 저장/영상 편집 정책을 분리한다. 생성 이미지의 예제 문구·카드 수·PTS·품질 승인을 복사하지 않는다.

## 공통 데이터와 입력

`ReviewSession.review_items()`는 canonical Cards/Highlights payload를 소비해 source_refs와 각 TextField를 묶는다. Card는 앞면·뒷면, Highlight는 제목을 보존하며 TextInput은 기존 session owner/draft generation에 결합한다. position은 해당 snapshot 내부 위치이며 영구 card_id/clip_id가 아니다. SDK에 없는 안정 ID나 영상 시간을 생성하지 않는다.

`studio/cards/proposal-panel.tsx`는 두 문구 필드를 독립 callback으로 전달한다. `studio/clips/highlight-proposal-panel.tsx`는 제목과 host의 원본 시간 연결 상태를 표시한다. 공용 ProposalSource는 identity/revision/source_refs/원문/generated raw/quality label을 분리한다. generation provenance와 quality가 현재 DTO에 미연결인 상태를 host가 정확히 표시해야 한다. 참조가 있다고 품질 승인이나 실제 player 구간 연결로 간주하지 않는다.

두 패널은 공용 ProposalActions를 소비하고 controller.session_mut().change_text에 연결할 TextInput을 host에서 보유한다. save/accept/refresh는 같은 controller로 전달한다. source callback은 아직 Meari 미지원이므로 이 연결은 host 소스/계약이며 실제 TSX 이벤트 dispatch 완료가 아니다.

## 토큰과 소비 그래프

입력·근거·포커스 규칙은 subtitles.css에서 components.css로 이동해 자막 전용 stylesheet 의존을 제거했다. 색상/여백/border/radius/글꼴/높이는 기존 --aw 토큰을 사용한다. 신규 장식 에셋은 필요하지 않았다. 카드·클립·교정이 같은 스타일 책임을 소비하며 화면별 복제하지 않는다.

최소 그래프는 `verification/card-proposal.source-graph.json`, `verification/highlight-proposal.source-graph.json`에 보존한다. callback props, typed string[]와 keyed map, useId, local composition, details/summary와 hidden 동작은 실제 consumer에서 인수할 항목이다. 미지원 구문을 지워 단일 고정 참조나 비활성 목업으로 축소하지 않는다. 새 graph는 아직 고정 consumer나 최신 parser로 실행하지 않았다.

## 다음 gate

SDK frame gate 뒤 같은 독립 bridge의16검사와 strict Clippy를 중앙 자원 실행기로 통과했다. 기존 필드 검사에 ReviewItem의 카드 앞/뒤·원문 참조와 하이라이트 제목·참조 보존 assert를 보강했으며 test 개수는 늘리지 않았다. 실제 SDK controller 사례와 synthetic 실패주입의 범위를 구분한다.

Meari callback 인계 후 같은 원본·최소 graph를 parser/runtime으로 재검증한다. 전체13화면군·모든 탭의 시각/입력/키보드/패널 인수는 남으며 이 checkpoint로 완료 처리하지 않는다.
