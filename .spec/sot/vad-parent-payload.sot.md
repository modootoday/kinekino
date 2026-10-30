---
id: sot:kinekino/vad-parent-payload
title: VAD parent payload 공급·수명 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# VAD parent payload 공급·수명 계약

2026-10-21 Media 제안과 SDK의 실제 기존 타입/생성자 경계 대조로 확정한 source 계약이다. 구현·컴파일·native 실행 인수는 별개다. r4 worker 입력/취소 관측과 별도 후속이며 현재 UI gate에 합성하지 않는다.

후속 source 상태: runtime/development/vad-parent-payload-r2에 같은 native hook과 trusted producer/고정 sidecar/request guard 및 Media 생성자를 연결했다. 새 constructor는 assemble_static_worker_attempt_executor(configuration,input,admission,inputs)이며 SDK의 같은 원 호출 지점에서 inputs Arc를 추가로 받는다. SDK caller는 별도 소유이며 아직 변경하지 않았다. root source/P0 취소 정책/현재 gate와 분리하고 직접 Rustfmt21만 확인했다. Cargo/type/test/native 실행0이다. 아래 계약의 실제 runtime 인수는 여전히 미완료다.

## 단일 작성자

Media는 media-vad-executor NativeTransportFactory/native hook·payload 준비/guard·구체 release producer와 vad_job_attempts 생성자를 소유한다. SDK는 src/infrastructure/vad_job_executor.rs의 안정된 operation adapter와 prepare_owned 호출 지점·공용 요청/영속 envelope를 소유한다. Media가 SDK 파일을 동시에 수정하지 않는다. 공통 JobService/AttemptScope/PreparedJobInput/RequestEncoder의 새 DTO·scope·map·registry는 만들지 않는다.

## 정확한 연결 위치

VadJobExecutor::prepare_owned의 원 identity input/source 검사 → from_prepared → self.admit(Some 원 scope) → reconstructed 전체 SHA==expected → check_scope 뒤, 현재 assemble_admitted_attempt_executor(configuration,input,Arc::new(admitted)) 지점에서 per-attempt trusted payload를 캡처한다. 그 뒤 check_scope와 같은 scope로 executor.prepare_owned 위임을 유지한다.

admitted_attempt_resources는 inner.prepare가 끝난 뒤 admission을 transport/publication에 attach한다. 따라서 native factory가 사용할 데이터는 attach 이후 탐색하지 않고 원 per-attempt resource를 만들 때 캡처해야 한다. PreparedVadAdmission은 descriptor.raw/profile_manifest pin과 원 leases를 보유하지만 release manifest 실제 bytes는 보관하지 않는다. 그 bytes는 기존 trusted producer의 실제 read/guard로 별도 공급한다. profile나 pin에서 파일 경로를 재추정하지 않는다.

## NativeTransport의 최소 확장

Media의 선택적 with_payload builder는 trusted supplier를 factory/session에 캡처한다. prepare는 기존처럼 inert session을 만들고 supplier를 호출하지 않는다. launch_read는 원 executable의 실제 bytes/SHA와 guard를 확인하고 original evidence root/stdin.raw를 만든 뒤, 원 request/root/RunControl을 빌려 staging을 수행한다. request는 기존 PinnedRequestEncoder의 정확 bytes이며 encoder/public wire는 바꾸지 않는다.

staging 결과는 기존 SnapshotLease를 재사용한 typed owner와 원 승인 release manifest pin을 포함한다. supplier가 돌려준 owner를 **같은 NativeTransport에 spawn 전에 저장**한다. Fallible stage/pin 검사는 child 생성 전에 끝나야 하며, 이후 spawn 오류·panic·Pending에서는 기존 retaining session/원 Supervisor가 owner를 계속 보유한다. 같은 session의 signal_sent와 poll_close를 원 AdmittedTransport가 위임한다. 새 process controller/owner/deadline을 만들지 않는다.

typed bootstrap의 release pin은 parent가 실제 승인한 profile_manifest와 동일해야 한다. native launch argv의 고정 private flags로 pin을 전달하는 최소 후보를 사용할 수 있지만, 이는 아직 main에서 지원된 계약이 아니다. caller JSON의 profile을 자체 승인하거나 expected bytes/SHA 없이 sidecar 내용을 승인으로 쓰지 않는다. main/loader 구현은 다음 단계다.

## 고정 sidecar와 원 identity

원 descriptor.json과 release.manifest.json/model.onnx/model-license.txt/input.wav는 parent가 create_new·bounded read/hash·ordinary file/ancestor guard로 제공한다. request stdin.raw도 원 encoder bytes와 대조한 read guard를 유지해야 한다. 기존 PreparedEvidence는 생성 후 input을 read-only File로 다시 열어 자식에 상속한다. stdin.raw 저장 파일과 실제 pipe stdin EOF는 같지 않으며 raw EOF/가족 종료는 기존 Supervisor에서 관측한다.

opaque public ID/worker JSON 값은 경로·환경·실행 프로그램이 되지 않는다. static release.runtime은 실제 worker executable pin과 전체 일치하며 런타임 bytes를 sidecar에 복제하지 않는다. runtime_dependencies의 파일이 stage됐다는 이유로 loader 승인으로 보지 않는다. 실제 DLL 원 path/identity/SHA·bundle/OS prerequisite와 license/notices/ABI/build closure는 별도 미완료다.

## 실패·반환과 검증

stage 실패 전에도 supplier가 원 admission/source leases를 보유한다. child를 만들기 전 오류는 기존 NotStarted/원 오류이며 취소 ACK가 아니다. 성공 후 owner는 원 session/cleanup 동안 유지하며, terminal/EOF/family/receipt 확인 전 drop·재시작으로 해결하지 않는다. artifact 파일명 충돌·원 request/descriptor/release pin 변경·worker pin mismatch·stage 중 cancel/만료는 spawn 전 거절 fixture가 필요하다. 성공 경로의 실제 child identity/signal/EOF/cleanup과 forced/panic/incomplete도 별도 gate다. 지금 계약 확정은 그 실행 통과가 아니다.

modoo code-change-review-verification을 실제 caller→resource→native launch 경로에 적용했다. SDK가 확인한 핵심은 attach 순서와 PreparedVadAdmission의 release bytes 부재다. 이 근거로 supplier 캡처를 원 resource 생성 지점으로 정했다. source 증거의 exact manifest는 독립 runtime/development/vad-parent-payload-r1 후보에 보존한다.
# 2026-10-21 동결 입력 접합 확인

후속 중앙 source-only 배정은 `runtime/development/vad-parent-worker-integration-r1/HANDOFF.md`로 구현·동결했다. source52 `56BDF6C48B26372D3A062A0FBE031830C5838605E6233853B69CA43F67519A14`, before `E80C7E3FADA6B5B289CE356886354FC48223B8AEFEFF960FF7350B83546B9F55`다. 공통 crate 경로 연결·binding 재노출·parent sidecar→worker/P0→공통 응답 authored3이며 직접 Rustfmt48만 확인했다. 아래 편집 배정 전 상태는 이 후속으로 갱신한다. SDK 별도 source2와 함께 future snapshot에서 소비해야 하며 실제 Cargo/타입/시험/main/native 검증은 아직 없다.

SDK 읽기 재확인: root `1a919dc3b51d57474a48c5b1b3eb30b78a278d53`/Meari `b7ed5ab`, SDK actual35 manifest `90714B052FC7ACD38518B9587E7320F9515599DC87462E62D255ADA8D29B9408`를 기준으로 r2 source21은 capture35와 경로 겹침0이다. SDK 소유 호출부 SHA는 `455E0300A625E4F7F91B7C5CCF7DFB44AE7894684A304061E74DC568C4F2EB41`, tests entry는 `FA6917763691A5D5B2202A47084B81DE5192494A5BBE008744258894C2457170`다. 별도 SDK stage 제안은 `runtime/development/vad-sdk-static-payload-r1`의 두 파일이며 아직 편집 배정·구현은 아니다.

Media 읽기 대조에서 r2의 protocol 9개는 r4 protocol-overlay의 대응 파일과 SHA가 전부 같다. 이 9개는 하나의 공통 crate로 소비할 수 있는 원문 후보이며 중복 구현이나 중복 통과 수로 계산하지 않는다. r4 Cargo는 현재 독립 workspace/local overlay 경로를 가리키므로 실제 제품 공통 crate로의 경로 재결속과 P0 overlay 채택 범위를 별도 stage에서 명시해야 한다. r2 executor worker_binding 채택과 기존 r4 private binding의 단일 정책 연결도 소스 합성 시 확인할 대상이다. 이 읽기 대조는 Cargo graph·타입·실제 입출력 검증이 아니다.

SDK는 원 identity→controlled admission→reconstructed 전체 SHA→check_scope 뒤 기존 생성자 import/call1을 `assemble_static_worker_attempt_executor(configuration,input,Arc::new(admitted),self.inputs.clone())`로 연결하고, 다음 check_scope와 동일 scope의 prepare_owned를 유지하는 경계를 확인했다. 새로운 Scope/Deadline은 만들지 않는다. SDK tests entry의 sibling path2는 SDK 단일 작성이다. Media r4/rebase21과 SDK 두 파일은 분리 manifest·parent pin으로 인계하고 중앙의 해당 source-only 배정 뒤 편집한다. root/S/T/원 r4/현재 UI 입력 변경·Cargo·ORT·모델·child 실행0을 유지한다.

