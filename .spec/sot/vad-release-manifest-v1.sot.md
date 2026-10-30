---
id: sot:kinekino/vad-release-manifest-v1
title: VAD release manifest v1
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# VAD release manifest v1

정식 generic [ModelManifest](model-manifest.md)의 모든 작업을 구현한 schema가 아니다. 현재 VAD 전용 `vad.release/1` decoder·고정 파일 매핑은 [r2 actual23](../../../verification/ai/vad-release-profile-r2-cpu-2026-10-21.md) 범위로 채택됐고, 공용 SDK operation/실 worker 실행은 아직 등록하지 않았다. 단일 schema 소유자는 Media이고 SDK가 이를 복제하지 않는다. [원 요청·scope 소비 계약](vad-job-consumer-binding.md)과 연결한다.

## 원 바이트와 필드

public expected_profile_manifest_sha256는 **release 파일 전체 원 bytes**의 SHA256이다. private VadDescriptorV1 profile snapshot이나 worker 요청 descriptor의 hash로 치환하지 않는다. decoder는 기대 bytes/raw64KiB 상한·정확 길이/SHA를 확인한 뒤 strict serde parsing한다. 원 release 파일을 canonical JSON으로 덮어쓰지 않는다. 아래는 현재 decoder의 형식 계약이며 네이티브 배포/품질을 승인하는 필드가 아니다.

| 필드 | 현재 규칙 |
|---|---|
| schema_version / task | `vad.release/1` / `audio.vad` |
| profile_id | 기존 P0 VadProfile의 내부 ID. public product ID와 호스트가 명시 매핑 |
| repo_id / immutable_revision | P0가 지원하는 `onnx-community/silero-vad`, lowercase40hex revision. main/latest는 실행 profile revision이 될 수 없음 |
| format / quantization / license_id | `onnx` / P0 `fp32` / P0 model license `MIT`. INT8·FP16·다른 모델을 지원으로 선언하는 schema가 아님 |
| files.weights / license_document / runtime | 기존 ArtifactPin의 id·sha256·bytes. bytes는 DecimalU64 문자열이며 role 상한은 기존 P0 model4MiB/license64KiB/runtime64MiB |
| files.runtime_dependencies |0..16 pins, 각 기존 runtime 상한. manifest에 경로/URL/argv가 없음 |
| files.runtime_notices |1..16 pins, 각 기존 license 상한. 전체 files id는 유일해야 함 |
| runtime | minimum_version(비어 있지 않은256B 이내 선언), provider=CPUExecutionProvider, intra_threads2/inter_threads1. 실제 설치 버전 검증이나 runtime load 관측이 아님 |
| provenance | source_model/source_revision/conversion_recipe/conversion_version. 각 bounded 선언이며 source_revision은40hex. 실제 upstream/build/conversion 계보의 완전성은 별도 검증 |
| evaluation_id / measured_memory_bytes | 누락/null이면 미제공, 제공 값도 선언이다. evaluation은 bounded 문자열, memory는 lossless DecimalU64. 원 평가나 OS peak를 조회한 것으로 승격하지 않음 |

핀 id·SHA·positive bytes는 기존 ArtifactPin 검증을 재사용한다. 원 primary profile은 VadProfile::new로 만들고 precision/provider/threads/license를 그 지원 값과 비교한다. 알 수 없는 필드·지원하지 않는 literal·잘못된 값은 거절한다. 원 bytes 검증은 선언한 모델 파일을 실제로 읽었다는 증거와 구별한다.

## 고정 파일·신뢰·수명

호스트가 구성한 FixedVadRelease의 원 manifest pin, private snapshot pin, 실제 파일 catalog만 사용한다. public profile/audio/destination ID는 caller가 제공할 수 있지만 경로·프로그램·environment·새 timeout을 선택하지 못한다. public ID80/dot와 내부 OpaqueReference128의 문법을 혼합하지 않고 명시된 고정 매핑만 허용한다. 원 imported audio pin과 파생 WAV/PCM pin도 별개다.

ProfileLookup은 guarded release read→decode→기존 private profile lookup→전체 VadProfile 일치→정확 catalog pin 집합→실제 weights/runtime/license/dependency/notice bytes·SHA를 확인한다. 반환 ProfileSnapshot.manifest는 release pin이고 composite lease는 원 release/private snapshot/asset File과 ancestor guards를 보존한다. audio·destination은 기존 구체 producer 정책을 사용한다. 입력 범위/실제 samples는 기존 VadInput에서 판단한다.

ProfileAdmission은 신뢰된 제품 조립 port이며 caller JSON이 이를 구현하지 않는다. 파일 hash 일치만으로 native build provenance, 고지 전체성, 재배포 권리, 최소 runtime 버전, 모델 품질 또는 OS cleanup을 승인하지 않는다. 현재 cached ORT/DirectML/VC [배포 closure](../../infrastructure/inference/research-followups/vad-native-distribution-closure-2026-10-21.md)는 별도 미완료다. MIT model 필드를 다른 native component 권리로 전이하지 않는다.

## 오류와 원 control

manifest 상한은 ResourceExhausted, 원 길이/SHA 불일치는 SourceExpired, schema/핀/지원 profile 값 오류는 InvalidArgument다. 없는 public ID는 NotFound, 다른 expected release/original-source pin은 SourceExpired다. 실제 일반 파일·reparse·경로·no-overwrite 오류는 기존 저장소 어댑터를 따른다. 오류를 성공 빈 결과나 다른 모델로 대체하지 않는다.

r2의 initial admission read는 bounded uncontrolled다. 다음 source-only r5는 원 P0 AnalysisControl reader를 재사용하고 read 전·각8KiB·실 EOF/hash 성공 뒤에 원 control을 확인한다. 실제 JobService의 원 scope에서 동작했다는 검증은 아직이며 SDK의 전후 phase check도 유지한다. 동기 IO/hash를 즉시 선점한다거나 read 취소가 cleanup ACK라는 뜻은 아니다.

현재 후속은 r5 source57 `3E1B323D149B0A43DC50728C5FFF06F152776524C21BD6BC354619733B1F81BA`/원 root delta9 `220F0F96AB0BACC803E6CEBDCFE3DDBED1FB4DA2D75666477D9E10B8E4C403E9`다. 원r4/실제r2 결과를 보존하고 root Cargo·SDK envelope·실 caller는 SDK 단일 합성으로 검증한다. 소스 작성14·형식/경로 확인을 실행 통과나 전체 AI 지원으로 표시하지 않는다.
