---
id: plan:kinekino/20261029113549
title: 제품 CLI 기능군·구성·인수 근거 연결
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 CLI 기능군·구성·인수 근거 연결

R111 최신: [명시 CLI host의 production.export_cards Jobs operation](../../sots/verification/application/card-bundle-cli-20261028.md)과 typed submit_card_bundle을 실제 확인했다. 카드2/원자막1·9CLI요청·재조회/거절/EOF·동일입력190/정상bins. 기존103명령 수는 유지한다. 직접 export RPC는 Unsupported이며 long-running 작업의 원 jobs.submit 경로를 사용한다. renderer 없는 기본빌드는 opt-in을 거절한다. 일반CLI 전체율/전체카드/GUI/품질 인수는 미완료다.

R109 경계: [원 Rust Jobs의 카드 묶음4시험](../../sots/verification/application/card-bundle-jobs-20261028.md)은 명시 programmatic composition에서의 실제 기능 검증이다. CLI host에 아직 등록하지 않았으며 production.export_cards의 직접 CLI 지원으로 승격하지 않는다. 기본103/별도host10·전체CLI 미완료 유지. 다음은 명시 host opt-in과 실제 정상 CLI 인수다.

R106 구현 경계: [카드 묶음 준비/공통 텍스트 엔진](../../sots/verification/application/card-bundle-preparation-20261028.md)은 공통 Rust 코드·선택30시험 단계다. 새 CLI 명령/지원 승격은 없고 기본103/host10 유지. production.export_cards의 실제 adapter/Jobs/묶음 CLI 인수는 남는다. 현재 완료 통합gate R104와 새 R106 입력을 구분한다.

## 현재 R104 — 기본103명령·14책임군

기존100/13군에 card_projects.create/inspect/save_revision 3항목을 별도 카드 편집 프로젝트 책임군으로 추가했다. 명시 --card-project-root에서 [실제24호출/typed SDK 동등성](../../sots/verification/application/card-project-sdk-cli-20261028.md)을 확인했고 기본103명령 발견/schema 대조도 통과했다. 카드 저장/조회 지원은 전체 production.export_cards의 지원이 아니다. 별도 execution host는10항목을 유지한다. 전체CLI 완료율의 분모로103을 쓰지 않는다.

## 이하 R98 이전의100명령·13군 연결 기록

R98: [제작 executor 공백/구현 배정](production/remaining-executors-20261028.md)에서 다섯 명령의 현재 schema Unsupported 및 직접 실행 거절을 실제 확인했다(1901~1910). 전체 카드 묶음의 CardDocument→원 renderer→Jobs→SDK/CLI를 다음 구현 단위로 선정했다. 거절 확인을 기능 완료로 계상하지 않는다.

R97: [카드 검토 PNG 실제 CLI](../../sots/verification/application/card-review-current-cli-20261028.md) 1시험 통과. 이는 production.export_card_review_png이며 일반 export_cards의 완료가 아니다. 다음 중앙 배정은 render_short/export_cards/segment_subject/build_assets/package_projection의 승인 계약·동등 구현·dispatch 공백 대조다. [현재 목표 기준표](product-goal-status-20261028.md)의 제품 여정을 우선한다.

2026-10-28 중앙 PM. 현재mirror의공통API(capabilities.rs와capture_mutation의추가등록)에서발견하는기본카탈로그100항목을13개책임군으로연결한다. proposals.edit_capture/accept_capture는capture_mutation이추가하며COMMANDS상수만의항목수와혼동하지않는다. 명시execution host는같은3개발견이름과7개execution RPC를가진별도10항목구성이다. 수를단순합산해완료율로계산하지않는다. 아래기능군단위연결은명령별모든오류/취소/저장인수종료를뜻하지않는다. 각원SOURCE/CLI를해당정본에유지한다.

|기능군/명령수|명령|필요구성|현재근거·남은범위|
|---|---|---|---|
|발견3|version, capabilities, schema|기본및각명시host|R72 기본/저장소/모델100발견, R92 execution10발견·schema. 발견은동작인수아님|
|미디어준비2|reviews.media.prepare, reviews.media.inspect|review roots·고정selection설정|R72 실제9prepare/inspect·명시최초발행/조회/거절. 전체미디어/동시성/fault는별도|
|검토재생4|reviews.playback.open/status/close/control|fresh producer·명시review host|R82 실제MKV1case의제어/close/재열기/EOF/재시작. 전체9/GUI/기타fault별도|
|검토저장9|reviews.inspect/record_decision/decisions/read_text_asset, association.inspect/create/revise, range_draft.read/save|분리review/user roots|기존실제파일/원SDK·R82association. 모든기록별오류/변경의실제귀속정리필요|
|카탈로그/회차5|catalog.search/read_raw, episodes.list/inspect_resolution/resolve|catalog root·provider/browser원owner|보존DOM의새호스트오프라인재조회. 실제현재사이트/provider별권한·HTTP/probe통합미인수; 보류유지|
|일반재생17|playback.open/status/audio_handoff/play/pause/seek/engine/set_volume/set_muted/set_rate/select_track/previous/next/transition.observe/transition.commit/transition.release/close|명시native/playback/combined host|원소유권/전환/재생근거와검토재생근거분리. 최신제품source연결/전체제어·voice/fault매핑미완료|
|구간14|segments.detect/open/inspect/evaluate/skip/skip_export/export_list/export_peek/export_consume/override/close/edit/accept_skip/set_skip_policy|원session·bounds·skip정책host|원수동skip/전달소비근거. 자동detect/모델분류와전체fault미완료|
|자막8|subtitles.transcribe/diarize/convert/export_range/speakers/align_speakers/edit_cues/edit_speakers|files·proposal roots·명시worker/모델adapter|convert/range/jobs·cue/화자수정선택근거. ASR/정렬/화자분리제품executor·실제입출력통합미완료; 추론보류|
|제작9|production.export_card_review_png/render_short/download_media/capture_frame/export_snapshot/export_cards/segment_subject/build_assets/package_projection|각고정입력/출력·native/card/executor|R43~R93 frame/복구·R94 snapshot연결. card review export와일반export_cards구분. render/분할/assets/package등미구현/미지원경로실제설계대조필요|
|모델5|models.list/inspect/install/verify/remove|명시model root/catalog/engine|list/inspect/verify합성파일실제CLI근거. 설치/제거·실제engine/lifecycle별도미완료; 다운로드/추론보류|
|작업6|jobs.get/submit/list/watch/cancel/retry|job roots·명시worker/provider host|R40/R43 durable lifecycle·변환/range발행취소. operation별executor와HTTP/provider·모든fault별도|
|프로젝트5|projects.create/inspect/save_revision/reconnect_source/export|project/proposal roots|저장/CAS/locator-only재연결·실제9historical103요청. export와실제재생rebind미인수|
|제안13|proposals.create/list/import_historical/inspect/inspect_source_context/edit/accept/export_subtitles/inspect_output_review_target/record_output_decision/output_decisions/edit_capture/accept_capture|proposal/project roots·고정verifier|R64/R65/R67/R72 실제9및공통context. card/캡처/자막각task와accepted output의원근거를유지. GUI비교/사용자품질별도|

## 이번 대조에서 수정한 해석

R96 [현재자막CLI선택15](../../sots/verification/application/subtitle-current-cli-20261028.md)에서직접convert/export_range·기존Sintel코퍼스왕복·accepted subtitle export의현재귀속을보완했다. 초기capability전체동등2실패/자원중첩위반과fixture주입수정·최종직렬재실행을구분해보존했다. 전체ASR/모델·GUI인수로확대하지않는다.

기존기본/모델2구성의unsupported표시는구성상미주입일수있으며제품미구현과같지않다. 예를들어검토재생은명시review host에서실제검증됐다. 반대로capabilities목록에있는render_short/projects.export등은dispatch/adapter가없는경우발견만으로구현으로표시하지않는다. SDK와정상CLI가실제PngSnapshotExport를소비하는snapshot도과거성공만있던최신입력귀속공백을[R94](../../sots/verification/application/snapshot-current-cli-20261028.md)에서확인했다.

## 인수 전개 순서

1. 현재권한으로가능한기존기능의미매핑근거를연결한다. snapshot은현재CLI로실행했고다음은자막직접convert/range·accepted subtitle/card export의기존공통SDK/CLI경계를대조한다. 같은코드의시험을맹목적으로모두반복하지않고현재입력/구성귀속이없는실제공백만실행한다.
2. 제작/프로젝트export/모델executor의미구현을승인설계와대조해공통Rust application/domain의다음구현을선정한다. 네트워크·추론을요구하지않는부분은독립진행하되다른owner의범위/WIP를덮지않는다.
3. 실제provider/모델입출력은관련보류를유지하고코드/계약/오류·취소·저장경계를먼저완성한다. 전체CLI선행인수와GUI통합/시각/AI사용자품질0/9는별도gate로남긴다. 배포제외.

근거:[명령별이력](cli-command-evidence-20261027.md),[제품여정](cli-product-journeys-20261028.md),[최신목표표](product-goal-status-20261028.md),[R92통합](../../sots/verification/application/source-sdk-r92-integrated-regression-20261028.md). 기존AGENTS/WIP/소유권/원실패/privateoverlay·C-only/heavy1/보류유지. modootoday의기존QA/코드검증스킬을적용해발견·구성·실제기능·현재귀속을분리했다.
