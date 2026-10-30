---
id: sot:kinekino/ui-meari
title: 제품 UI·Meari 통합과 인수 행렬
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 제품 UI·Meari 통합과 인수 행렬

2026-10-26 PM 통합 설계. UI는 제품 선언/props/binding/adapter, SDK는 기능 정책, Meari는 공통 frontend/compiler/runtime/router/plugin을 소유한다. 기존 승인 시안·화면별 요소 ID·공통 토큰을 유지한다. 이 문서는 새 UI 구현·수동 검토·라이브러리 기능 지원을 선언하지 않는다. 각 계약 변경은 원 source pin과 실제 producer/consumer를 같은 후계에 고정해야 한다.

## 공통 화면 계약

UI 선언은 typed SDK 결과/오류/수명 상태의 얇은 투영이다. 동일 Document의 선언 Props·Rust 명시 producer·callback signature·binding key/owner lease·SDK queue와 feedback를 연결한다. shape/schema fingerprint는 권한·전체 source identity·동일 semantic action의 증거가 아니다. 정확한 signature여도 다른 SDK 명령을 dispatch하면 실패다. 선언을 읽어 unknown key를 자동 삭제하거나 strict binder를 느슨하게 하지 않는다.

화면/본문/inspector/queue는 공통 프레임·용어·단위·저장/취소 의미를 공유한다. 각 route는 Empty/Populated/LongLabels/NeedsInput에 더해 loading/pending/working draft/dirty/saving/conflict/SDK error/cancel requested/cancelled/completed/history/stale revision을 해당 기능 의미에 맞게 제공한다. 미설치 모델·원본 없음 때문에 결과 구조를 제거하거나 placeholder를 정상 결과로 표시하지 않는다. 모르는 진행률/ETA/confidence/PTS는 ‘미상’이며 샘플 숫자로 채우지 않는다.

공통 필드 상태 normal/hover/pressed/focus/selected/disabled와 오류 안내·키보드 접근을 유지한다. 화면별 본문/전용 toolbar/rail/inspector/main CTA/feedback는 [screen composition](screen-composition.md)과 요소별 정본을 따른다. 시안의 장식/생성 오류는 기능 정책으로 승격하지 않는다. 아래 API 이름은 [SDK 표면](../../application/api/host-support-boundaries.md)의 실제 주입 조건을 전제로 하며 계획 명령을 지원으로 표시하지 않는다.

## route·task 연결

routes.json의 VIEW3/Studio20 선언에 대응한 23행이다. 이것은 runtime 등록/시각 검수/기능 완료 개수가 아니다. 기존 source null과 UI WIP는 기능 부재 증거가 아니다. route의 stable ID는 표시 이름과 독립으로 유지한다.

| route ID | 데이터/행동의 SDK 소유 | 고유 UI 표시·입력/인수 |
|---|---|---|
| view.start | source/project 선택과 capability 조회 | 임의 demo source 자동실행 금지, 최근항목 source/revision 확인·선택실패/취소 표시 |
| view.browse | catalog.search/episodes.list/provider jobs | category/작품/회차 안정ID·pagination/지원필터, raw provider 기술 단계는 최종 사용자 CTA로 노출하지 않음 |
| view.watch | native/control/status/episode transitions/skip | 원 engine/session·seek/volume/track, pending/관측미확정·회차오류, fullscreen/cursor/focus는 창 capability 분리 |
| studio.segments | manual skip/evaluate/edit/override + 별도 자동 detect operation | OP/ED handle·시간축/근거·자동/수동/미검토 구분, original issuer/ACK/consume를 재생 성공으로 뭉치지 않음 |
| studio.subtitles.speech | ASR/VAD/transcript edit/export | waveform/cue·언어/트랙·원문/교정 분리, 근거 없는 시간/확률 금지 |
| studio.subtitles.speakers | diarize/align/edit speakers | 화자 이름/merge/split·overlap·cue mapping, 모델 없이 수동 편집과 분석 실행 분리 |
| studio.subtitles.ocr | OCR ROI/표본/cue edit | 원본 좌표 ROI·언어·오차/저신뢰, 읽기와 burn-in 제거 구분 |
| studio.subtitles.correction | proposals/reviews/raw/draft/accept/output decision | 원문/후보/diff·known issues·NeedsRepair·decision ID/이력, 시간불명/range null·사용자수정·판정 분리 |
| studio.subtitles.conversion | subtitles.convert/export_range + loss | 원본/출력 문법 비교·실제 loss 동의·경로 picker/저장 receipt·충돌·원본보존 |
| studio.objects | subject/mask/track revisions | 원본/마스크/합성·points/brush/eraser/undo·컷/소실/가림·track와 사용자label, OBJ 기존 WIP 보존 |
| studio.shorts.manual | clip project/render task | 원본+세로crop·multi-track 시간축·audio/subtitles, exact vs keyframe boundary 표시 |
| studio.shorts.recommendations | candidate generation→동일 clip project | 근거/원본구간·수정/제외/선택, 추천 생성과 rendering/품질승인 분리 |
| studio.shorts.batch | batch plan/명시 submit/실패 retry | 회차선택·후보/길이/예산·계획/시작 분리, 개별실패와 성공산출물 보존 |
| studio.frames.original | source/native capture | 실제 frame PTS/정확도·회전/color/subtitle 상태·원본 저장, OSD/문서 캡처와 구분 |
| studio.frames.thumbnail | snapshot/crop/편집 project/export | crop/text selection·layer/filmstrip·미지원 font·원본/편집본 분리, PNG crop 성공만으로 전체 편집 완료 아님 |
| studio.cards | card project/proposal output/export | thumbnail순서·canvas·copy/delete/undo·text overflow·template/alt, 로컬저장≠게시 |
| studio.assets-2d | reviewed mask/LayerAsset/sequence export | checker/layers·pivot/bounds/trim·alpha·원본재합성, unknown source PTS/누락 frame 표시 |
| studio.projection.depth | depth/planes/camera draft | relative/metric·좌표/단위·hatch holes·XYZ/시점·원본복귀, 완전3D 연구와 구분 |
| studio.projection.package | projection manifest/packaging/validation | file tree·역할/manifest·대상 round-trip 결과, 원 모델/data 없으면 NeedsInput |
| studio.video-save | download preparation/owned job | source/quality/track·original/remux/transcode·예상크기/재개/호환성 미상, 명시 큐 추가 |
| studio.models | model inventory/manifest/admission/install/remove | 후보/설치/verified/admitted/loaded/quality 구분, pinned revision/license/precision·in-use 삭제·offline/취소 |
| studio.jobs | jobs.get/list/watch/cancel/retry | 1페이지+상태filter·선택detail/단계/실측·원 expected attempt/revision·cancel requested/ack·입력필요/실패별 안내 |
| studio.exports | output receipt/project/artifact inspect | 원 입력/출력 hash·실제길이/속성·충돌·reopen·선택된 출력, 저장과 채택/품질/게시 독립 |

## Router·plugin·문서 수명

Meari의 공식 React Router 호환은 원 upstream API/mode별 지원행렬과 동일 source trace로만 인수한다. 기존 로컬 문서의 version pin은 역사 기준이며 이 문서에서 최신 공식 API라고 검증하지 않았다. Declarative/Data/Framework 및 pending/error/loader/action/fetcher/abort/HMR·실험 flag backlog를 첫 제품 route subset으로 축소하지 않는다. 부족한 기능은 precise Unsupported/미검증이며 독자 onNavigate 구현을 full compatible로 부르지 않는다.

제품 params/query는 공통 Rust validation→SDK inspect에 연결한다. raw draft/path/signed URL/credential을 URL에 넣지 않는다. dirty 이동은 유지/저장/버리기의 명시 선택, Back/Forward/deeplink/reload는 선택ID·revision·scroll/focus를 복원한다. 역순 loader/취소/stale generation 응답은 새 selection에 적용하지 않는다. navigation 취소와 이미 실행된 SDK mutation의 취소는 별개다.

plugin은 Meari의 기존 explicit Rust registry/manifest·target grant·contract version/opaque instance epoch를 사용한다. 제품 typed payload와 RequestContext를 보존하고 새 bus/SDK store를 만들지 않는다. native path picker의 사용자 취소≠저장성공, Web File/Blob grant≠OS path. browser window minimize/close를 다른 기능 성공으로 대체하지 않는다. register/unregister·중복등록·grant/schema 거절·pending cancel·listener 해제·stale completion·HMR를 인수한다. HMR은 mutation 자동 replay/모델 작업 재실행을 하지 않는다.

native retirement는 원 SDK ticket을 유지해 pending 동안 새 inspection/mount를 막고, native Complete와 retained route 관측을 모두 기다린다. 창 User close veto 후 coalesced completion/actions/timer/lifecycle 처리·원 callback owner/generation·one-shot timer 수명을 보존한다. 새로운 timer/polling 정책으로 원 owner fault를 우회하지 않는다. media engine/child surface/parent window/SDK worker 소유권을 각각 기록하고 사용자나 다른 세션 player를 정리하지 않는다.

## 기능/렌더/사용자 인수 분리

| gate | 필요한 근거 | 대신할 수 없는 것 |
|---|---|---|
| UI01 props/state | 선언+Rust producer+binding+정확 상태 factory의 누락/extra/type/semantic dispatch 회귀 | bootstrap만 렌더·schema fingerprint·같은 signature |
| UI02 공통 SDK 기능 | 같은 source의 CLI/SDK 성공/오류/취소/저장·재조회 | UI 자동클릭·demo·렌더 성공 |
| UI03 Native/Web runtime | 같은 source/action의 state/render/a11y·plugin/router traces, 각 target 실제 실행 | Core/Native helper로 Web/OS 지원 대체 |
| UI04 창/DOM 고유 | original parent/child surface·geometry·focus/IME·cursor/fullscreen·close/cleanup·DOM listener | 비디오 청취/장기안정성·전체 기능 검증 |
| UI05 요소 시각 | 기준시안 version/화면ID/상태·crop1:1/2x/4x·실제 최종빌드·차이판정 | 테스트/Clippy/headless나 전체창 축소 screenshot |
| UI06 접근성/반응형 | 1440/1280/1180/980/760·760x480·keyboard/focus/scroll/설정서랍·명칭/역할/상태 | DOM/AccessKit node 존재만으로 screen-reader/IME 완료 |
| UI07 사용자 AI 품질 | 실제 제품 GUI의 해당 원본/후보·필요 재생·수정·사용자 명시 verdict/이력 | mock/CLI 판정/실추론/기술accept/개발자 품질기록 |

SDK/CLI 기능 인수 뒤 UI 고유 검토를 묶어 수행한다. 수동 GUI/시각/추론/다운로드 보류는 이 설계로 해제하지 않는다. 실제9는 원본불변이며 테스트용 쓰기는 격리된 namespace로 한정한다. 전체190/정상 제품 bins는 아직 미실행이다.

연결: [요소 기록](element-review-contract.md), [제품 routing](web-routing.md), [제품 plugin](meari-product-plugins.md), [작업 통합](../../application/production/task-integration-contract.md), [인수 순서](../../../../plans/player/design-completion/implementation-order.md).
