---
id: evidence:kinekino/20261029113613
title: 공식 Torch와 Rust encoder 전체 단계 실제 대조
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공식 Torch와 Rust encoder 전체 단계 실제 대조

2026-10-18. 기존 실제 PCM3개에서 만든 동일 full NumPy mel 바이트를 공식 Qwen encoder와 격리 Rust encoder에 직접 넣었다. **입력별 공통38단계·총114tensor의187592448원소가 사전 고정한 abs(actual-ref)<=1e-4+1e-4*abs(ref)를 통과했다.** 비트 일치나 전체 forced alignment·timestamp head·사람 경계 품질 승인은 아니다. 기존 정렬17회·구조3통과/14실패·품질0은 유지한다.

## 고정 실행 계약

[사설 환경과 원본 source](forced-encoder-dependency-runtime.md)·[실제 import 감사](forced-encoder-original-import.md)를 사용했다. 공식 Qwen commit7c6daf77a2421100f5fb066495372c00129d39ff, 모델 revisionc7cbfc2048c462b0d63a45797104fc9db3ad62b7, weights SHA47831d0e82f96b20e9034dba01a075ee06436654719f6a68289e49f1b65ce0e7이다. config SHAd616c65d46c4b90bdc651b0a0963ea932732241140f337f9bb6b0335a9c8ef09를 확인했다.

공식 Torch2.9.1+cpu·Transformers4.57.6 원본 Qwen3ASRAudioEncoder를 구성하고 thinker.audio_tower. prefix397개 tensor를 strict=True로 로드했다. missing/unexpected key0, parameter316427904개 모두CPU F32·finite를 확인했다. eager·eval·no_grad, 수치thread4·interop1을 적용했다. 원본 source를 수정하거나 Python wrapper/eval/stub을 추가하지 않았다. PyO30.27.2 native callback이 원본 module의 pre/post hook으로 tensor를 읽고 None을 반환해 입력·출력을 교체하지 않았다.

Rust는 [SDK 세션의 준비 정본](../application/forced-encoder-observation-preparation-2026-10-18.md)의 r2-final EXE SHA836dcef4e0a81f08c7fb53c74d328ae19f6d412de48a565f4fe61c72b7f0f4ef와 build manifest SHA7474d7b6be5de75568722e1461e65478d60729b7e47c4268471e2d232d1a1ce1을 사용했다. CPU F32·Rayon4·flashfalse이며 기존 수치 경로에 passive observer만 추가한 버전이다. 전체 weights SHA와 고정 mel/config를 실행 전에 재검증했다.

root의 observe-encoder는 정확한 worker SHA/manifest·3개 case·명령/인자·새 label과 같은 출력 directory만 허용한다. 최초 ko1883-r1은 보존 EXE를 실행 영역 밖 경로로 지정해 **모델 로드 전** 거부됐다. [거부 기록](../../../evidence/ai/model-spikes-2026-10-18/results/encoder-rust-ko1883-r1.orchestrator.stderr)을 보존하고 같은 bytes를 사설 bin/encoder-r2에 복사해 ko1883-r2부터 실행했다. 전체 경로 허용 범위를 넓히지 않았다.

## 실제 수치

| 입력 | 최종 형상 | 공통 단계 | 공통 전체 원소 | 전체 단계 최대 절대차 | 최종 encoder 최대 절대차 | 최종 relative RMS |
|---|---|---:|---:|---:|---:|---:|
| ko1883 | 162×1024 | 38 | 56693760 | 1.14440918e-4 | 5.30481339e-6 | 9.63684314e-7 |
| ja1516 | 218×1024 | 38 | 74332416 | 9.91821289e-5 | 1.38282776e-5 | 9.93661707e-7 |
| ja1634 | 158×1024 | 38 | 56566272 | 3.73840332e-4 | 1.03712082e-5 | 1.15031858e-6 |

허용오차 밖 원소는 모두0이다. 전체 최대 절대차가1e-4보다 큰 지점도 reference 크기에 비례하는 고정 항을 포함한 기준으로 판정했다. 실패 후 기준을 넓히지 않았다. input_mel과 conv_input은 비트까지 같았다. 나머지 단계의 미세한 부동소수 차이를 숨기지 않으며 최종 tensor는 비트가 다르다.

Rust41개와 Torch39개 관측 중 conv 입력/출력·일부 GELU 출력·conv_out·ragged gather·24layer·ln_post/proj1/act/proj2/final 등38개를 각각 짝지었다. input의 Rust1×128×T와 Torch128×T는 선행 unit batch 축만 제거해 같은 bytes를 비교했다. Rust만 관측한 conv3_act·positional_embedding·positional_add3개와 Torch만 관측한 conv_out_input1개는 미대응으로 명시했다. 공통 stage 통과를 모든 내부 attention/QKV/FFN·연산별 kernel 일치로 확대하지 않는다.

실제 Rust cu_seqlens는 입력별[0,162]/[0,218]/[0,158]이었다. 원본 Torch eager의 실제 cu_seqlens 자체는 이번 hook에서 덤프하지 않았다. 양쪽 eager가 mask로 사용하지 않는다는 source 조사와 해당 수치 대조를 구별한다. 더 세밀한 head 경로에서 작은 수치 차이가 argmax 경계에 영향을 줄 가능성을 배제하지 않는다.

## 자원과 종료

| 실행 | wall초 | peak private B | peak working B | 종료 |
|---|---:|---:|---:|---|
| Rust ko1883-r2 | 17.6662973 | 1605705728 | 2238091264 | exit0·timeoutfalse |
| Rust ja1516-r1 | 22.9922066 | 1705201664 | 2333863936 | exit0·timeoutfalse |
| Rust ja1634-r1 | 19.2960012 | 1605484544 | 2234171392 | exit0·timeoutfalse |
| 공식 Torch3개 배치 | 53.064055 | 4728942592 | 2435059712 | exit0·timeoutfalse |

공식 배치는 환경/source SHA 재검증·import·모델 초기화·strict load·3회 forward·전체 stage 파일 출력까지 포함한다. load/parameter 검사는3.6951496초, 입력별 forward+capture는4.8310602/4.876984/3.7059474초였다. Rust는 매번 새 process이며100ms 소유 process tree, Torch는20ms root-child 표본으로 측정 범위도 다르다. I/O·상주 범위·반복 구성이 달라 속도/메모리 순위로 비교하지 않는다. 모델 실행과 다운로드는 한 worker씩 수행했고 사용자 ChatGPT/Terminal·다른 앱을 종료하지 않았다. Torch worker는 사설 kill-on-close Job·300초 deadline, Rust worker는 기존600초 root 측정 감독을 사용했다.

## 재현 증거와 검증

- [공식 전체 결과](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/encoder-official-forward-r1/encoder-forward-result.json), SHA79faf4fe4efeee0fe8c5cb5cef17266d3aec55e49366bc741e429fd67d6811ab
- [공식 관측](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/encoder-official-forward-r1/observation.json), [선보존 build62입력+EXE1](../../../evidence/player/jer-function-reference-builds/56a13fd050ffd2e6e8c4195f9d805c06b61e6ac42dff98091b6f8db036474484/build-manifest.json)
- [Rust KO](../../../evidence/ai/model-spikes-2026-10-18/results/encoder-rust-ko1883-r2/encoder-result.json)·[JA1516](../../../evidence/ai/model-spikes-2026-10-18/results/encoder-rust-ja1516-r1/encoder-result.json)·[JA1634](../../../evidence/ai/model-spikes-2026-10-18/results/encoder-rust-ja1634-r1/encoder-result.json)
- [공통114tensor 독립 비교](../../../evidence/ai/model-spikes-2026-10-18/results/encoder-stage-comparison-r1-encoder-comparison.json), SHA799b119a285b66a8a34d0db03f865dc3164859ab7159a7fdde3fdab8b46d5bfc
- [Rust 입력·stage raw 예시](../../../artifacts/spikes/2026-10-18/results/encoder-rust-ko1883-r2/input-mel.f32le.bin)·[공식 최종 tensor 예시](../../../artifacts/spikes/2026-10-18/acquisitions/encoder-official-forward-r1/ko1883/encoder_final-000.f32le.bin): 전체 case의 raw F32·sidecar를 같은 책임 경로에 보존했다.
- [root 실제 실행 감독 build188입력+EXE1](../../../evidence/ai/model-spike-builds/b90f03a8c2845da5f1dd7b2374aa5c779eb7f4b9c892cc2d78db6a3ab38ec9ed/build-manifest.json)·[독립 비교 build190입력+EXE1](../../../evidence/ai/model-spike-builds/385eaf9dabe09dfa5ea823b1776ddbfdda64d688781a6ead1b9201300d06ab5b/build-manifest.json)

root58tests·fmt·clippy·locked offline build, 참조도구4tests·fmt·clippy·locked offline build가 통과했다. PyO3 native hook용 macros0.27.2와 lockfile 고정 crate를 취득했고 private 환경 재검증 비용을 줄이기 위해 sha2 crate만 dev opt3으로 컴파일했다. 모델 수치 kernel/정밀도나 파일 SHA 확인을 생략하지 않았다. 같은 최종 EXE의 [기본 JER6개 회귀](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/encoder-forward-jer-regression-r1/observation.json)는 exit0·3.440571초, stdout SHA8b516b1ce73187162497b011789a0ee9e4b05b814b06485e7dc766e541b80fba로 이전과 완전히 같다.

이6회는 encoder 전용 실행이며 기존17회 alignment 횟수나 새 모델 수에 합산하지 않는다. 다음은 동일 prompt/expanded token IDs·audio 길이·position/mask로 공식 text 결합과 timestamp 전체 logits를 비교하는 것이다. 원본 whole processor·전체 head·사람 word 경계·다른 언어/음원·GPU·제품 SDK 통합은 미완료다. zero 시간 연장·runner-up 강제 선택·허용오차 완화로 품질을 통과시키지 않는다.

## 독립 보존 감사

[감사 보고서](../../../evidence/verification/encoder-stage-integrity-2026-10-18-r1/build-evidence.json), SHA00483891dbc1999e9093811833739832573892cc76663ac0d48f38908482afe0에 SDK r2 compiler 입력109개·patch4개·full mel3개·config·EXE1개 검증을 추가했다. 원본 참조 build11개·보존 source585개·EXE11개와 기존immutable105개도 검사했다. [링크 감사](../../../evidence/verification/encoder-stage-integrity-2026-10-18-r1/links.json)는336문서 정본 오류0·원본 참조3건 구분이다. checker8tests·fmt·clippy·release build 통과, 동봉 EXE SHA는f82d4764c5aebe9134ddae25f965eab5890f881bd96e4c989ae97a3cf27c98e2다. 동봉 checker-source는 감사 당시 소스 사본이며 compiler-input attestation은 아니다.

다음 head 비교 준비는 [원본 참조 연구](../../player/infrastructure/inference/research-followups/forced-head-pytorch-readiness.md)를 따른다. KO raw IDs 누락·과거 JP mel 경로 차이를 먼저 해소하고 새 same-mel Rust full-model 관측을 마련한다. 현재 encoder 결과를 과거 timestamp logits와 같은 입력의 관측으로 가장하지 않는다.
