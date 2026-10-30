---
id: evidence:kinekino/20261029113620
title: AMI 동일 녹음의 시간 분리 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# AMI 동일 녹음의 시간 분리 검증

2026-10-18. 첫0~60초에서 관측한 local-anchor/PCM 배율 경로를 원본60~120초에 적용한다. 구간은 해당 정답 점수·화자 수를 열어 보거나 threshold를 조정하기 전에 인접 고정60초로 선택했다. **같은 ES2004a 녹음의 시간 분리 검증이며 독립 회의·언어·애니메이션 일반화 검증이 아니다.** 모델 학습 자료에 대한 완전한 독립성도 주장하지 않는다.

## 입력과 사전 계획

Rust `prepare-diarization-holdout LABEL`은 이미 취득한 원본 SHA256 `3e2560b19bee6952c7c7ce041b0f1ea8a7ea9468044c4eea79d2a2c67e24ab0f`를 확인하고 sample960000부터960000개를 추출한다. mono16kPCM16·정수 sample을 유지하며 리샘플·무음 제거·음량 변경 없이 crop WAV roundtrip와 원본 보존을 검사한다. 새 파일/create_new와 acquisition 기록을 사용한다. 새 download나 VM/DPI 작업은 없다.

가장 최근 명시 실험 role `diarize-pcm16-anchor`를 같은 segmentation FP32/INT8·같은 WeSpeaker 모델·동일cosine0.65·10초창·longest-clean-anchor·unknown overlap 정책으로 각각1회 실행한다. 현재 제품 기본값을 바꾸거나 reference 화자 수를 clustering에 주입하지 않는다. 실제 추론 전 source/EXE를 동결하고 root1worker·CPU<60%/여유12GiB·BelowNormal·4threads/1interop gate를 유지한다.

평가기5202에 원본 source_start_us60000000/duration60000000을 명시해 **reference/UEM만 원본 좌표에서 crop**하고 hypothesis는 입력 로컬0~60초를 유지한다. 원본 RTTM/UEM을 수동 이동하거나 잘못된0 offset의 점수를 쓰지 않는다. DER/JER·MISS/FA/CONF·미확정 구간을 보존하며 첫 구간보다 점수가 낮아도 자동 적용 승인으로 승격하지 않는다. 원본 scorer 대조는 별도 범위로 기록한다.

준비 단계의 crop·합성 검사와 실제 모델 실행/정답 점수는 구별한다. 실제 identity·실행 결과는 아래에 추가한다.

## 실제 입력 준비

acquisition label `ami-holdout-60-120-r1`은 CPU31.804878048780484%/여유38,047,297,536bytes에서 완료했다. EXE `1be7ea3ad1b16c60c5a71efd50a52c596257f2f64a90a5ffd8df924df83f7dcf`의79파일을 준비 실행 전에 보존했으며25tests·fmt·Clippy·locked offline build를 통과했다. 이 단계는 모델 추론0회다.

- crop WAV SHA256: `980ee396b0886623cb237983396979a0eb93c05fb9a2d8018a0b1f108d415006`
- crop raw PCM SHA256: `34b43d7f7483c6dc41bcff4aa10bd7c84c7448355fb5bd28ea7de2a696539e61`
- 정확히960000samples, source[60000000,120000000)us; 원본 SHA 일치·roundtrip true·resampled false.

후속 observation gate는 이 실제 WAV hash 하나를 추가 허용하며 임의 corpus/경로나 동적 manifest를 신뢰하지 않는다. gate를 추가한 새 source/EXE는 추론 전에 다시 동결한다. 평가 offset은 출력의 source_offset_applied_us0과 별개로 원본 reference crop60000000us를 명시해야 한다.

## 실제 추론과 정답 평가

최종 worker/orchestrator EXE SHA256 `8d4e973cdbaea9ad910bd201f7d756b25ab9425eeda6551ea60a2c10395f4ebe`,79파일 manifest SHA256 `5cd0ebc7551e447e8bd942b3adba02e34c3596357cf68e5bb1873c49d020f957`를 두 추론 전에 보존했다. gate 변경 후에도25tests·fmt check·Clippy all-targets-Dwarnings·locked offline build가 통과했다.

| 항목 | FP32 | INT8 |
|---|---:|---:|
| 실제 label | diarize-holdout-fp32-pcm16-r1 | diarize-holdout-int8-pcm16-r1 |
| admission CPU % | 24.90234375 | 11.557296767874636 |
| admission 여유RAM bytes | 36,482,625,536 | 36,702,990,336 |
| Rust DER % | 15.044017424975799 | 17.14106243949661 |
| Rust JER % | 52.10018262457605 | 51.718722523920356 |
| reference speaker-time us | 41,320,000 | 41,320,000 |
| MISS us | 4,600,000 | 4,727,624 |
| FA us | 1,616,188 | 2,355,063 |
| CONF us | 0 | 0 |
| 정답 화자 수 | 2 | 2 |
| known hypothesis 화자 수 | 1 | 2 |

둘 다 root 순차1worker/success=true/exit0/timeoutfalse다. GPU·성능 순위·새 meeting 검증이 아니다. reference/UEM offset60000000us를 사용해 평가했고 UEM은 로컬0~60초 전체를 유지한다. 두 출력 모두 unresolved union/overlap/short가0이므로 이번 MISS를 unknown abstention으로 해석하지 않는다.

FP32는 reference MEO015와 매칭되는 hypothesis가 없고 이 화자의4.6초가 누락됐다. INT8은51개 JER grid frame의 speaker_1이 MEO015의460개 frame에 매칭된다. DER가 첫 구간보다 낮아도 다른 음성 비율·화자 구성의 다른 입력이며 동일 입력에서의 개선량이 아니다. 화자별 JER 약52%와 소수 발화자 누락은 자동 화자 적용 미승인의 근거다. reference 화자 수로 threshold를 맞추지 않았다.

## 원본 DER 교차 검증

INT8은 기존 aed71a64 Rust 어댑터로 `nist-holdout-int8-r1`에 원본17.14%/components차이0us로 통과했다. FP32는 JSON 파싱1ULP 문제로 scorer 실행 전에 거부돼 [정밀도 회귀 수정](diarization-json-precision.md)을 수행한 새136a5773 어댑터로 `nist-holdout-fp32-r2`에서15.04%/components차이0us를 확인했다. 실제 NIST 두 결과 모두 exit0/timeoutfalse/capfalse/stderr empty이며 사전DER0.0051pp/각component1us 기준을 바꾸지 않았다. JER/global parityfalse·품질미승인을 유지한다.

입력은 [보존 fixture](../../../artifacts/spikes/2026-10-18/fixtures/diarization/ami-es2004a-60-120.wav), acquisition은 [기록](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/ami-holdout-60-120-r1/), 실제label.stdout/.json/-evaluation.json은 [보존 결과](../../../evidence/ai/model-spikes-2026-10-18/results/), 원본 scorer는 [별도 실행 evidence](../../../evidence/ai/diarization-der-reference-runs/)에 둔다. 다른 회의·언어·자연 애니메이션·원본 JER 대조·시간 receptive-field 검증은 남아 있다.
