---
id: plan:kinekino/20261029113934
title: 원본 Review·Native 문서·실제 SDK 수직 인수
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원본 Review·Native 문서·실제 SDK 수직 인수

2026-10-19 [실제 Native SDK gate](../../../sots/verification/presentation/native-review-sdk-2026-10-19.md)에서 Native1·전체23·strict Clippy·최종fmt가 통과했다. 기존 단일 registry22를 보존하고 실제 Native 문서 회귀1개를 추가했다. 아래 준비 기록은 당시 이력이며 GUI 창·Web·디자인 시안 일치·root 제품 등록은 이 기능 gate에 포함하지 않는다.

## 입력과 manifest

도구는 tools/player-ui-sdk-bridge다. dev dependency로 meari-native(default-features=false)를 선언하고 본체와 동일한 local blitz-paint patch를 사용한다. window/GL 기능·신규 추론·다운로드를 켜지 않는다. root Cargo는 수정하지 않았다.

새 tests/native_document_sdk.rs는 원본 verification/proposal-review-intent.tsx를 include_str로 직접 읽는다. 원문 단순화·대체 Component·ProbeScope를 사용하지 않는다. 마운트 전 값은 serializable CallbackReference일 뿐이고 실제 권한은 Native constructor의 실제 module root scope factory에서 생성한다. 기능용 CSS/960×720 viewport는 입력/DOM 인수용이며 전체 제품 디자인 구현으로 표시하지 않는다.

초기 cargo fmt/metadata 사전 확인은 meari-native의 workspace.dependencies.meari 상속 오류로 실패했다. 독립 tool에 workspace dependency를 추가해도 같아 그 우회는 제거했고 원진단/수정 전 manifest는 runtime/quality-review/player-ui-native-document-r1/manifest-preflight에 보존했다. 이후 Meari 담당이 package.workspace를 명시해 소유 수리했다. PM은 consumer metadata67ms0/root metadata69ms0을 직접 확인해 인계했다. UI는 Native manifest의 workspace="../.."를 직접 읽었으며 root/core manifest를 수정하지 않았다. metadata 해소는 compile/실행 성공을 뜻하지 않는다.

최종 consumer 준비 대조는 prepared-native-sdk-r1/final-consumer-comparison.json의24개 입력이 모두 일치했고 direct rustfmt/check도 종료0이다. Native dev dependency/default-features=false/local paint patch는 원래 준비값을 유지했다. metadata-fix-observed-manifest-pins.json은 본체 최종 gate 이전 관측이며 실행에 사용할 공개 Core/Native/Cargo pin은 본체 완료 인계 시 다시 고정한다. 새 compile은 시작하지 않았다.

## 실제 수직 source

원본 parse_module_graph→ModuleProgram→InteractiveDocument::from_module_program_with_callbacks의 scope factory에서 registry 하나와 SDK registration을 조립한다. RegistryProposalHost/ReviewSession/ProposalDocumentSession은 동일 scope·registry를 소비한다. 실제 Native set_input_value/Activate와 typed pending callback→제품 opaque handle port→dispatch 반환 후 drain→실제 SDK/store 경로다.

검사는 disabled Save 거부, Unicode 입력, callback dispatch 동안 canonical store 불변, drain 후 명시 save/inspect, 잘못된 props 교체 실패 뒤 old drained handle 생존, 성공 교체 뒤 old handle stale, 명시 accept와 post-accept store read 오류1회 주입/refresh, editor node·본문·a11y label 보존, close 뒤 handle stale, 새 SDK/store 재열기를 확인하도록 작성했다. 오류 주입은 실제 SDK 장애/모델 품질 판정이 아니다.

store adapter는 실제 ProposalFiles로 load/publish를 위임하고 첫 post-accept load의 기술 Timeout만 주입한다. CAS·receipt·source 검증을 다시 구현하지 않는다. SDK 호출은 기존 ticket context factory에서 capability를 관측해 edit1/accept1/inspect3을 검사한다. source SHA/raw·문서3/receipt2·3/automaticfalse 및 원본 byte 보존을 확인한다. 결과 artifact는 runtime/quality-review/player-ui-native-document-r1의 소유 PID/time 디렉토리에 source/events/sdk-calls/restored를 쓰도록 준비했으며 아직 성공 관측은 없다.

## 실행 조율

### 고정 source의 artifact/assert 읽기 대조

SDK root gate 중 제품 source는 변경하지 않고 이전 실제 case와 assert 범위를 대조했다. 이전 case98864의 edit1/accept1/inspect_attempts3/sdk_inspects2·single_registry_headless는 기존22 checkpoint 관측이며 Native 결과로 소급하지 않는다. 새 Native case는 store adapter 내부에서 read 실패를 주입하므로 registry SDK 호출 예상은 edit1/accept1/inspect3이고, 이전 host wrapper의 sdk_inspects2와 같은 측정값으로 취급하지 않는다.

Native events.json은 deliver로 처리한 이벤트를 기록한다. failed reproject old lease 확인을 위해 따로 보관해 dispatch한 Save와 close-stale 요청은 이 목록에 포함하지 않는 현재 source다. 따라서 전체 이벤트의 완전한 audit log로 주장하지 않는다. Save는 실제 Native Activate→held callback→실패 뒤 dispatch→문서revision1에서2 변화 및 SDK capability sequence assert로 인수한다. SDK calls는 등록된 SDK adapter의 ticket factory 관측이며 read-only 비교용 client.inspect 호출은 제외한다.

dispatch 중 불변 비교는 canonical StoredProposal readback 기준이다. source/raw 파일의 bytes는 끝에서 별도 비교한다. 전체 storage 파일 byte hash snapshot은 이전 headless22가 입증한 범위이며 새 Native case에서 다시 측정한다고 표현하지 않는다. 신규 actual Native compile/실행 결과는 아직 없다. 이 대조에서 제품 source나 시험 수를 변경하지 않았다.

Meari의 shared Core queue 이관 source와 Native 공개 서명은 기존 Native25 snapshot과 구분한다. 본체 영향 gate 종료·manifest 소유 수리 이후 현재 checkpoint를 고정한다. 소비 동결 범위는 root SDK/API·Meari Core/native/Cargo/paint·media contracts/plugins 및 제품 adapter/consumer source다. 검증된 resource-runner의 2 logical CPU/BelowNormal·-j1·RUST_TEST_THREADS1·offline 독립 lock 동기화→Native 실제 회귀/기존23/strictClippy/fmt를 중앙 슬롯에서 수행한다. 실패 원문/source를 보존하고 필요한 좁은 수리 후 같은 원본으로 재검증한다.

제품 staged 수명 소스의 현재 검증 상태는 [document 계약](../../../sots/player/presentation/shared/proposal-document-session.md), 본체 listener와 제품 root 등록 경계는 [인계안](proposal-product-registration.md)을 따른다. 전체13화면군·모든 하위탭의 시각/키보드/패널 인수는 별도 남는다.
