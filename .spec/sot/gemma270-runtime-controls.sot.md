---
id: sot:kinekino/gemma270-runtime-controls
title: Gemma270 runtime controls의 설정값·실측 증거
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Gemma270 runtime controls의 설정값·실측 증거

2026-10-19 source-only 조사. 동시 개발 중인 `tools/boundary-observation/src/domain/conversion.rs`의 RunReceipt/WorkerControls와 vendor·LocalTextServer source를 읽었다. root의 FEA352 startup source 시험·CLI build에는 손대지 않았다. 아래 callback은 후속 제안이며 현재 구현된 API나 실측으로 표현하지 않는다. 코드 편집·cargo·추론·다운로드는 하지 않았다.

## 증거 수준과 현재 DTO 제약

- `configured_worker_value`: spawn 직전 Command.get_program/get_args/get_envs에서 관측한 명시적 설정이다.
- `resolved_runtime_observation`: 같은 child가 파싱·loader·scheduler 변환을 마친 뒤 실제 적용한 값이다.
- `unavailable`: 현재 receipt/API로 직접 관측하지 못한 값이다. false/0/default로 채우지 않는다. 고정 source를 근거로 한 추론도 actual resolved와 구분한다.

현재 RunReceipt.resolved의 WorkerControls는 nonoptional이다. 필요한 resolved 값이 unavailable이면 receipt를 완성하기 위해 상수를 넣지 말고 증거 수집 미완료/conversion blocked로 남긴다. 동시 개발 DTO와 verifier의 실제 통합 시점 hash를 새로 고정해야 한다. 현재 라이브러리 코드는 수정하지 않았다.

## 필드별 최소 수집 지점

vendor root는 `runtime/ai-spike/source/mistral-gemma-cap-source-r1`이다.

| 필드 | configured 증거 | resolved 관측 위치 | 현재 한계 |
|---|---|---|---|
| prefix_cache_entries=0 | argv `--prefix-cache-n 0` | core/engine/mod.rs::Engine::new의 최종 prefix_cache_n/no_prefix_cache, scheduler cache-enable, PrefixCacheManagerV2의 accepts_sequence_cache()/accepts_paged_recurrent_prefix() | HTTP에 값 없음. 빈 cache나 hit0은 용량0 증거가 아님. 작은 callback 필요 |
| max_sequences=1 | argv `--max-seqs 1` | Engine::new:580의 최종 SchedulerConfig에서 계산한 max_active_sequences | logger가 `/metrics`의 mistralrs_sequences_capacity에 게시하지만 갱신 지연·engine 귀속 때문에 보조 증거. callback으로 최종 variant/값을 함께 수집 |
| speculative=false | draft/mtp 옵션이 argv에 없다는 사실 | loaded pipeline의 SpeculativePipelineExt::has_speculative_proposer(); multimodal.rs:1756은 model에 위임 | 옵션 부재를 false로 만들지 않음. attach 이후 실제 getter 결과 필요 |
| engine_flavor | worker의 engine profile/EXE SHA | actual get_kind/category/device와 native Gemma3 GGUF text loader 분기 | 기존 HTTP가 정확한 flavor를 제공하지 않음. 실제 branch identity와 typed 값의 receipt 필요 |
| cpu | `--cpu` | pipeline.device().is_cpu 및 실제 raw forward cpu | raw actual 관측 있음. 같은 run/EXE/model join 유지 |
| paged=false | `--paged-attn off` | 최종 scheduler의 kv_cache_manager().is_some 및 raw ctx.is_paged | 실제 raw 관측 있음. argv만으로 모든 경로를 단정하지 않음 |
| native32768/runtime2048 | `--max-model-len 2048`; native는 모델 규격 | 실제 Gemma cfg/model metadata와 raw model_metadata | raw actual 있음. native를 runtime 값으로 덮지 않음 |
| http_n_choices=1 | request n 또는 명시적으로 검증한 기본 정규화 | raw request.controls.sampling_params.n_choices와 HTTP payload | raw actual 있음. max_sequences와 다른 축 |
| cpu_threads=4 | RAYON_NUM_THREADS/OMP_NUM_THREADS/MKL_NUM_THREADS 각4 | Candle get_num_threads, 초기화 후 Rayon current_num_threads 및 실제 backend pool 폭 | 환경은 설정 증거. 전체 프로세스 thread cap4가 아님. 의미를 고정하고 필요 pool을 관측하기 전에는 unavailable |
| offline | HF_HUB_OFFLINE=1/token-source none/local file | pipeline/hf.rs::is_hf_hub_offline()의 child 내 실제 해석 결과 | HF cache-only 제어이며 전체 network 차단/통신0 증거가 아님 |

## 실제 source의 해석 처리

`Engine::new`는 no_kv_cache를 pipeline metadata와 합성하고, no_prefix_cache에 no_kv_cache·pipeline.metadata.no_prefix_cache·prefix_cache_n==0을 OR한다. no_kv_cache에 따라 PagedAttentionMeta를 DefaultScheduler::Fixed로 바꾼 뒤 max_active_sequences를 계산한다. 따라서 원래 argv보다 **이 최종 국소값**이 정확한 관측점이다. scheduler.set_prefix_caching_enabled(!no_prefix_cache)와 PrefixCacheManagerV2::new(n,no_prefix_cache,has_paged_attention)의 입력을 함께 기록하면 scheduler/manager 양쪽 설정을 대조할 수 있다.

`prefix_cacher.rs:178/222`의 PrefixCacheManagerV2는 private n_on_device/no_prefix_cache/has_paged_attention을 보유한다. 공개 accepts_sequence_cache()/accepts_paged_recurrent_prefix()는 캐시 수락 여부를 알려준다. 현재 사용량과 구성 용량은 다르므로 cache0개/hit0만으로 prefix 용량0을 증명하지 않는다.

`pipeline/loaders/mod.rs:485`의 ModelKind는 Normal/GgufQuantized/Adapter/GgufAdapter/AnyMoe 등이다. speculative 여부 enum이 아니다. MultimodalPipeline::get_kind는 self.kind를 반환하며, MultimodalPipeline이라고 Gemma text-only가 vision 입력을 쓴다는 뜻도 아니다. has_speculative_proposer()가 실제 loaded model 상태를 확인하는 별도 getter다.

`server-core/mistralrs_for_server_builder.rs:895/1088`에는 mtp_config에서 attach_speculative_with_runtime을 호출하는 경로가 있다. 따라서 draft 옵션 부재만으로 false를 증명할 수 없다. getter 관측은 모든 attach가 끝난 뒤 Engine::new 또는 실행 직전에서 해야 한다. 이후 tune/reload/attach로 상태가 달라질 수 있다면 cohort 전후에도 비교해야 한다.

`MistralRs::config(model_id)`(lib.rs:2575)는 실제 EngineInstance.config를 반환하며 ModelKind/device/category/modalities/max_seq_len/generation_defaults를 담는다. prefix 용량·maxseq·speculative bool은 없다. `/v1/models`는 ID/status/tool availability 등, model status는 loaded/unloaded/reloading, system info는 OS/CPU/GPU 정보다. 필요한 resolved controls 전체를 HTTP로 얻는 경로는 확인되지 않았다. system doctor 등의 network 진단을 이 목적의 대용으로 실행하지 않는다.

## 최소 read-only callback 제안

현재 FEA352를 수정하지 않고 후속 검토 source에 다음 두 지점을 제안한다.

1. **Engine::new의 최종 scheduler/prefix manager 구성 뒤**: run/EngineSessionID/EXE identity, scheduler variant/max_active_sequences, prefix_cache_n/final_no_prefix_cache/no_kv_cache, manager의 두 accepts 값, has_paged_attention, pipeline.get_kind/category/device, pipeline.has_speculative_proposer(), is_hf_hub_offline 결과를 한 번 복사한다. 기존 pipeline lock 아래에서는 owned DTO 복사만 하고 파일 I/O·다른 lock·model/RNG 동작을 호출하지 않는다.
2. **load_native_gemma3_text의 실제 loader 분기**: architecture/quantization/실제 Config::Text 선택/loader identity를 기록한다. 이 정보를 pipeline 구성 때 metadata로 전달해 첫 callback에 합칠 수도 있다. engine_flavor 문자열은 이 typed 관측과 trusted pin의 정규화 결과여야 한다. 원하는 worker profile 이름을 그대로 resolved에 복사하지 않는다.

예를 들어 새로운 label을 정의할 수는 있지만 현재 API가 이미 그 문자열을 반환한다고 표현하면 안 된다. source branch의 고정 label만으로 실제 사용 모델·EXE·kind를 인증한 것도 아니므로 다른 증거와 join한다. callback 실패는 설정·stop·seed를 바꾸지 않고 증거 미완료로 처리한다.

```rust
// 문서용 제안이며 현재 FEA에 추가한 API가 아니다.
struct RuntimeControlObservation {
    max_active_sequences: usize,
    configured_prefix_capacity: usize,
    final_no_prefix_cache: bool,
    accepts_sequence_cache: bool,
    accepts_recurrent_prefix: bool,
    has_paged_manager: bool,
    has_speculative_proposer: bool,
    hf_hub_offline: bool,
    // typed kind/category/device/loader identity와 별도 pool 관측값도 연결한다.
}
```

이 receipt는 raw model frame과 별도로 둘 수 있지만 같은 run/engine/model/startup lifecycle에 귀속돼야 한다. duplicate·missing·다른 engine·실행 중 변경은 fail-closed다. 최초 snapshot만 보고 후속 상태를 자동 보장하지 않는다. 새 hook이 필요하면 별도 source/EXE·patch·compiled inputs를 선보존하고 기존 FEA/빌드/실측을 덮지 않는다.

## threads4·offline의 표현 범위

core/lib.rs::build_engine_runtime은 Tokio worker_threads(candle_core::utils::get_num_threads())를 쓰고 engine thread 시작에서 Candle global pool을 초기화한다. CPU attention에도 get_num_threads로 만드는 별도 LazyLock Rayon pool이 있다. CLI main runtime, engine 전용 thread, HTTP/logger/I/O/blocking thread, Rayon/OpenMP/MKL pool도 존재할 수 있다.

따라서 env 각4는 **child 환경 설정**이지 프로세스 전체 최대4 thread·모든 pool 실제4·CPU core4개 구속의 증거가 아니다. WorkerControls.cpu_threads를 Candle 계산 pool 폭으로 쓸 것이라면 그 의미/API/시점을 명시하고 실값을 수집한다. Rayon/OMP/MKL/Tokio는 서로 다른 필드로 보존하는 것이 정확하다. OMP/MKL 실제 runtime getter가 확보되지 않았다면 configured4/actual unavailable로 둔다. OS process thread count를 측정하더라도 다른 지표이며 cap과 혼동하지 않는다.

pipeline/hf.rs:29의 is_hf_hub_offline은 1/true/yes/on을 trim·소문자화해 해석한다. child 내 getter bool을 기록하면 설정→해석 증거가 된다. 그러나 HF cache-only 정책이며 전체 네트워크를 OS에서 막았다는 증거는 아니다. 그런 요구가 있다면 별도 격리 증거가 필요하다.

## worker Command·종료 receipt

spawn 직전 get_program/get_args/get_envs의 allowlist 수집 계획은 타당한 configured 증거다. **get_envs는 명시적 추가·삭제만 반환하고 상속된 전체 child 환경을 보여주지 않는다.** 필요한 HF_HUB_OFFLINE/RAYON/OMP/MKL 키와 removed 상태를 구분하고, 전체 env/token/secret을 복사하지 않는다. model·traceplan 경로는 기존 confinement/bytes SHA와 연결한다. config/defaultfile override가 없는지는 resolved 관측으로 대조한다.

JobClose와 child.wait의 실제 ExitStatus, 종료 전/후 raw SHA, outer worker 측정의 exit는 각각 다른 증거다. Job kill-on-close는 정상 exit0이 아니다. owned_termination_after_completed_journal과 실제 exit status를 명시하며 worker_exited가 child인지 outer인지 schema 의미를 통일한다. outer 종료는 worker가 미래값0을 써서 증명할 수 없으므로 supervisor 실제 측정을 최종 binding한다.

raw before/after SHA 일치는 종료시 bytes 안정성 증거다. 마지막 flush 성공까지 자동 증명하지 않으므로 recorder complete/flush_failures0/소유 writer 종료·최종 receipt와 함께 본다. Drop에서 wait 결과를 버리면 ExitStatus 증거가 남지 않으므로 후속 worker는 명시적 owned shutdown→receipt 반환을 검토해야 한다. 이 세션은 구현하지 않았다.

## 회귀·미검증

모델 없는 Rust 회귀는 argv/env allowlist·removed·상속 구분, scheduler 변환 후 값, prefix 용량/사용량/disabled 구분, MTP 존재 시 speculative true, loader flavor 오결합, reload/중간변경/복수 engine, 정상exit/ownedkill/outerfailure, raw 전후 차이·flushfailure 거부를 포함한다. callback이 getter 외 model/seed/stop 함수를 호출하지 않는지도 검증한다.

이 조사 체크포인트에서는 수집 지점만 확인했으며 resolved callback actual은 없다. raw actual이 있는 native/runtime/CPU/paged/n_choices는 원래 source/EXE/run 연결을 유지해 사용할 수 있다. 나머지 unavailable을 DTO의 nonoptional 타입에 맞추려고 합성하면 안 된다. source/compiler provenance·sensor 완결·shared DTO verify·비간섭·품질은 별도 gate다. 이 조사 자체는 vendor/library/SDK/UI/모델을 변경하지 않았다.

## R3 실행 이후 남은 계약

후속 worker가 명시적 launch allowlist와 owned JobClose+wait receipt를 구현했고, PM 단일 실행의 [R3 실제 관측](../../../../verification/ai/generation-gemma270-startup-lifecycle-actual.md)에 원문을 보존했다. startup/cohort ledger·schema3 raw complete·실제 server exit0·outer exit0·raw 종료 전후 안정성을 관측했다. 이 실행도 이 문서의 최종 scheduler/prefix/speculative/loader/offline 해석값 callback과 flush ACK를 추가하지 않았다. frozen FEA352/worker f763을 이 목적으로 수정하지 않는다. 새 hook은 별도 검토 source/EXE pin과 비간섭 검증을 필요로 한다.

공통 DTO strict join은 미수집 필드를 unavailable로 유지하고 거부해야 한다. launch configured 값과 외부 실제 종료값은 서로의 대용이 아니며 raw complete=true도 resolved control이나 writer flush ACK를 보충하지 않는다. SDK 소유 세션은 보존본의 schema3 decode/startup sidecar와 실제 외부 receipt를 검증하고, 부족한 근거의 정확한 실패를 별도로 기록한다. 제품의 제안 수입·사용자 수정/채택 연결과 마지막 모델 품질 검토의 순서를 바꾸지 않는다.
