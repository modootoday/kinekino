---
id: evidence:kinekino/20261029114116
title: 원본 Review·실제 Native 문서·SDK 인수
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원본 Review·실제 Native 문서·SDK 인수

2026-10-19 원본 proposal-review-intent.tsx→실제 Native module document→제품 ProposalDocumentSession→단일 registry→실제 SDK/ProposalFiles 회귀가 통과했다. 추가 Native1, 전체23, strict Clippy, 최종 fmt가 종료0이다. 실제 GUI 창·Web·카드/하이라이트 Native 문서·root 제품 등록·전체 시각 인수는 포함하지 않는다.

## 입력과 자원

prepared-native-sdk-r1의24개 source 입력은 실행 직전 모두 일치했다. 이후 E0308 수리와 형식 변경은 별도 보존했다. 원문/lock/upstream pins·receipt·로그는 runtime/quality-review/player-ui-native-document-r1의 gate-r1, retry-refmut, final-format에 둔다. 실제 case는 selected Native1의107828-1792400394054793300과 전체 suite의110564-1792400452208452400이다.

root lock SHA256은 `50323b23fb1bbf496e3ad42be71b4ccbdc3e1f38de1de5a5154a1ba5d78da5f7`로 유지됐고, 독립 lock은 Native dev dependency/local paint patch의 offline 동기화 후 `c4091684061e724f1aee2f79fa7f7bff4f7dcd54f38d473c1af432dfa1d141dc`다. Native의 window/GL default feature를 켜지 않았으며 다운로드·추론·player.exe 실행은 하지 않았다.

검증된 resource-runner에서 affinity3/logical_cpus2/priority16384(BelowNormal)·공유 native cache·-j1·RUST_TEST_THREADS=1을 사용했다. Native retry1은74609ms0, 전체23은66724ms0, strict165554ms0, 최초fmt244ms1 후 최종fmt451ms0이다. 완료 즉시 슬롯과 root/Meari 소비 동결을 반환했다.

## 실제 관측

원문은 include_str로 직접 parse_module_graph/ModuleProgram에 전달했다. callback 권한은 InteractiveDocument::from_module_program_with_callbacks가 제공한 실제 root instance/source generation에서 만들었다. 마운트 전 CallbackReference는 값일 뿐 ProbeScope를 생성/복사하지 않았다. 같은 registry에 SDK proposal adapter와 UI enqueue adapter를 등록하고 RegistryProposalHost/제품 document 수명이 이를 소비했다.

Native set_input_value/Activate로 생성한 typed pending handle을 제품 port로 전달했다. callback dispatch 중 canonical store readback이 불변이며, dispatch 반환 후 drain에서 명시 저장이 일어났다. 잘못된 props 교체 실패 뒤 보관한 Save handle은 유효했고, 성공 교체 뒤 old handle은 stale이었다. editor node ID·Unicode 문구와 실제 accessibility label도 유지됐다. 이는 Node/DOM 기능 인수이며 시안 pixel 일치나 Pretendard 렌더 인수가 아니다. 기능용 CSS/960×720 viewport이며 제품 전체 스타일·자산 전달은 별도다.

전체 suite 실제 case의 sdk-calls.json은 edit→inspect→accept→inspect→inspect다. 첫 post-accept read Timeout은 test store adapter의 명시 주입이며 이후 refresh는 inspect만 실행했다. source.json SHA는 `4c36a3ee24f86ffbbabe633dc04775c28c44f8ec34af091ea7303f5cad35b552`, 원문과 generated raw가 보존됐고 수동 기술 수정은 `명시 수정 日本語 🦀`다. close 뒤 이미 drain된 handle은 stale이었다. 새 SDK/ProposalFiles로 재열어 최종 상태 전체를 비교했고 문서3/receipt draft2/output3·automaticfalse를 직접 대조했다. 기술 fixture이며 모델 품질 승인/실제 미디어 연결/자동 적용은 아니다.

events.json은 onDraftChange/onAccept/onRefresh의 deliver 경로를 기록한다. old lease를 확인하려 따로 보관/전달한 Save 및 close-stale 요청은 이 목록에 없어 완전한 event audit log가 아니다. Save의 실제 Native Activate/handle·revision1→2 및 SDK sequence assert는 통과했다. 불변 관측은 canonical StoredProposal 기준이며 전체 storage 파일 byte hash 관측으로 확대하지 않는다.

## 최초 실패와 수리 경계

첫 Native dependency graph 컴파일은448929ms 후 제품 document_session.rs의 E0308로 종료101이었다. `&mut try_borrow_mut()?` 대신 지역 RefMut를 만들고 dispatch에 전달하는 수리만 했다. 원진단·document_session.before-refmut-fix.rs를 보존하고 캐시로 Native1→전체23→strict를 통과했다. 원본 TSX/본체 API/SDK 정책은 수리하지 않았다.

fmt 실패는 동일 dispatch 호출의 줄바꿈 차이뿐이었다. before-format source와 최초 diff를 보존하고 cargo fmt→fmt --check를 통과했다. 의미 변경이 없으므로 전체 graph 재빌드를 반복하지 않았다. final-format/final-pins.json은 최종 source/manifest/lock 핀이다. 최초 wildcard literal pin 수집은 실패해 고정 디렉토리의 파일 목록을 열거한 뒤 동일 최종 pin 파일을 완성했다. 이는 hash 수집 수리이며 검사 결과 변경이 아니다.

## SDK 동결 경합의 귀속

SDK는 동결 알림 직전 domain/mod.rs·media.rs를 잠시 r2로 바꿨다가 R1 원본으로 복구했다고 보고했다. 이를 숨기거나 전체 root가 실행 내내 불변이었다고 주장하지 않는다. SDK 정본 freeze-race.md, R1/current SHA4개 비교, 파일 시각, 실제 root rustc command·creation time, artifact 시각, 소비 dep-info를 gate-r1에 보존했다.

SDK가 제시한 복구 완료 상한은08:54:17UTC이며 실제 kinekino_player rustc PID103024의 시작은08:56:48.661504UTC(17:56:48 KST)로 뒤였다. 두 파일의 현재/원본 SHA는 각각 `8692586aaffdfed6b290354a2748f6e8f826675346d539c9097e9a3f9a17a840`, `5845017419255c6a22f3b28001692e1f50a0fc0db4c665344569ac0881d2d65b`였다. 소비 dep-info는 두 등록 파일을 포함하고 미등록 request_referer/request_headers는 포함하지 않았다. 초기 slash 고정 검색의 false 관측도 보존하고 실제 Windows separator를 반영한 대조로 구분했다. 이 근거는 해당 두 파일의 R1 복구 뒤 컴파일을 뒷받침하며 다른 root 모든 파일의 전체 이력을 증명하지 않는다.

## 후속 범위

이번 Native case는 cue-a/source revision7의 기술용 CueSourceSnapshot과 typed Correction을 직접 create했다. 기존 actual9 R3의 DocumentSourceSnapshot·NeedsRepair·companion 선택/repair 경로는 이 Native 실행에 포함하지 않는다. 기존 actual9 SDK/CLI 완료 근거는 해당 이전 checkpoint 범위로 유지하며 이 Native 결과로 합치지 않는다.

재열기는 같은 test process에서 기존 객체를 닫고 새 SDK/ProposalFiles 객체로 실제 디스크를 읽은 결과다. 새 CLI 프로세스를 실행한 증거가 아니다. 별도 process 경계의 이전 CLI 근거와 구분한다.

RefreshFailureOnce는 accept publish 직후 inspect Timeout1회를 주입했다. provenance NotFound·corrupt·reference mismatch 실패를 시험한 것으로 확대하지 않는다. generation_provenance_bound=false/quality_decision_known=false를 유지하며 모델 품질/provenance 연결 승인으로 표현하지 않는다.

실제 검증 payload는 Cue Correction 한 종류다. DocumentCorrection·Cards·Highlights의 기존 SDK/field/read model 근거와 구별하며 실제 Native 문서 유형별 인수는 남는다. Product Drop의 borrow-conflict/deferred cleanup 등 모든 실패 조합을 이번 한 case로 입증하지 않는다. [제품 수명](../../player/presentation/shared/proposal-document-session.md), [root 등록](../../../plans/player/presentation/proposal-product-registration.md), [원본 수직 계획](../../../plans/player/presentation/native-review-sdk-gate.md)을 연결한다. 전체13화면군/모든 하위탭·시각·키보드·패널 인수는 계속 미완료다.
