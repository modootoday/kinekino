---
id: sot:kinekino/host
title: 자막 변환 플러그인의 host 조립 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 자막 변환 플러그인의 host 조립 계약

2026-10-19 제품 host 정본이다. 실제 Meari 등록은 [plugin r2](../../../verification/presentation/product-plugin-registry-2026-10-19.md), 실 ApplicationClient/파일 조립은 [bridge r1](../../../verification/presentation/sdk-plugin-bridge-2026-10-19.md)에서 검사했다. 제품 root 등록·UI source binding/Web 실행은 남아 있다. [제품 플러그인 계약](meari-product-plugins.md)을 따른다.

제품 host 한 벌은 `src/assembly/meari/subtitle_conversion.rs`이며 아직 root module/Cargo에는 등록하지 않았다. 독립 Rust probe가 같은 source를 path module로 컴파일하며 sibling root SDK·제품 plugin·Meari를 의존한다. fmt/실파일4시험/strictClippy/build 및 고정 CLI 성공·충돌·손실 오류를 확인했다. 조립 source는 정책을 추가하지 않고 context factory→ApplicationClient.convert_subtitles만 수행한다.

probe는 canonical Request JSON의 subtitle.convert만 호출한다. canonical SDK context validation 후 caller namespace와 전체 ticket에서 bounded request ID를 파생하고 host별 identity/scope 수명을 유지한다. SDK가 지원하지 않는 idempotency/revision/timeout을 삭제하지 않는다. 실제 새 파일/원본 불변/출력 충돌, direct SDK와 loss 오류 전체 비교/명시 동의, 미지원 context 전달, 동시2host와 재생성host의 SDK request context 구별4시험이 통과했다. Web/렌더러·제품 source binding 지원은 이 Native probe로 입증하지 않는다. probe pin에 포함된 당시 정본 문서는 checkpoint의 source-final/host-contract.md에 보존한다.

## 의존성과 타입

제품 plugin→Meari 최소 SDK + media-subtitles/media-contracts, root host→제품 plugin + ApplicationClient의 DAG를 유지한다. 공통 SDK를 plugin으로 옮기거나 plugin→root 역의존을 추가하지 않는다. 현재 codec은 Meari 의존성 없이 먼저 검사했고 공개 API freeze 후 등록 adapter를 덧붙인다.

host callback의 제품 입력/결과는 `media_subtitles::application::convert::{ConvertSubtitles,ConversionResult}`와 `media_contracts::errors::ApiError`다. `ApplicationClientHandle`, `RequestContext`는 root host만 소유한다. Meari generic SDK에 이 제품 타입/파일 정책을 넣지 않는다.

## 요청 흐름

1. Meari registry가 target·등록 epoch·document/instance/source generation·schema·permission/grant를 확인한다. 아직 해당 본체 계약과 구현을 인계받아 검증해야 한다.
2. 제품 등록 adapter가 현재 invocation과 canonical subtitle codec을 결합한다. callback을 invocation별로 묶어 host가 실제 SDK request ID/context를 생성할 수 있어야 한다. 모든 호출에 고정 `cli` ID나 이전 문서 context를 재사용하지 않는다.
3. host는 현재 invocation에 대응하는 SDK request context를 만들고 `ApplicationClient::convert_subtitles(context, request)`를 호출한다. expected revision/idempotency/timeout 등은 기존 command의 지원 계약을 따른다. plugin이 새 정책을 만들거나 UI의 임의 JSON을 context로 통과시키지 않는다.
4. canonical 성공 결과를 reply schema로 encode한다. SDK 오류는 message/details/remediation/request_id/retryable까지 보존하는 error schema로 encode한다. Meari dispatch 오류와 SDK usecase 오류는 구별한다. 큰 오류를 임의 잘라 정상 성공으로 반환하지 않는다.
5. renderer는 결과/오류를 요청을 시작한 document generation에만 투영한다. 화면 교체로 projection을 폐기하더라도 SDK의 실제 저장 결과는 사라진 것으로 표시하지 않는다.

최초 변환은 SDK의 현재 동기 typed 호출을 그대로 소비한다. Immediate 완료를 Pending이나 진행률로 위장하지 않는다. 장시간 실행이 UI thread를 막는 경우 SDK 작업 큐/worker port를 별도 연결한 뒤 Pending/event로 전환한다. 없는 취소 API를 UI에서 성공 처리하지 않는다. CLI의 기능 인수와 UI의 focus·응답성·접근성 검증은 별개다.

## Native와 Web

Native root host는 주입받은 실제 ApplicationClient를 사용한다. Web WASM은 동일 typed input/result/error를 운반하는 host bridge/codec과 권한 resource 계약이 별도로 필요하다. Rust Fn/trait만 작성했다고 browser transport가 생기지 않는다. transport가 없으면 기존 unavailable adapter의 Unsupported를 표시한다.

Web 파일 선택은 OS path를 보장하지 않으므로 HTML File/Blob을 `input_path`로 위장하지 않는다. resource grant→SDK storage mapping 또는 browser 전용 import/export capability를 별도 승인된 계약으로 연결한다. 제품 UI가 browser parser나 직접 다운로드 성공을 SDK의 새 파일 publication으로 대체하지 않는다.

## 다음 source freeze와 검사

Meari 담당에게 필요한 공개 인계는 PluginAdapter 서명/호출 thread, manifest의 request/reply/**error** schema, permission/grant 협상, opaque invocation/ticket 수명, cancel/shutdown receipt, registry의 실제 dispatch 오류 보존 경로다. 최초 초안에 error schema와 permission/grant가 없음을 피드백했다. 이후 실제 source 읽기에서 `CapabilityDescriptor.error_schema`, `PayloadDirection::Error`, `PluginDispatch::Immediate(Result<PluginPayload,PluginError>)`와 registry의 capability별 grants/PermissionDenied가 추가된 것을 확인했다. registry는 요청 후 결과의 error 방향 schema와 adapter validation을 별도로 검사한다. 아직 담당의 공개 freeze/해당 회귀 결과를 인수하지 않았으며 source 존재를 실행 지원으로 세지 않는다. 제품에서 중복 registry를 작성하지 않는다.

최신 source의 adapter는 owning host thread에서 호출되고 Pending을 event로 반환하는 계약이다. 최초 subtitle.convert는 Immediate이므로 plugin 전용 background queue/취소 위장을 새로 만들지 않는다. provider/contract hash의 실제 host pin 주입과 SDK ApiError 전용 details encoding은 담당에게 정확 인계 형식을 요청했다. 이 확인 뒤 새 codec dependency와 실제 등록 시험을 하나의 source freeze로 준비한다.

인계 후 같은 canonical codec을 실제 registry에 등록하고 Native/Web target별 동일 요청·미지원·schema mismatch·SDK 오류·stale/unregister를 CLI로 검사한다. host assembly와 실제 SDK 응답까지 연결된 수직 경로의 source 목록·필수 명령을 중앙 슬롯에 제출한다. 기존 r1의 메모리 시험을 이 후속 gate의 증거로 소급 사용하지 않는다.
