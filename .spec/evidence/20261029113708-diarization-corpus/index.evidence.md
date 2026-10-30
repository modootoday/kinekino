---
id: evidence:kinekino/20261029113708
title: AMI 화자 분리 평가 자료의 실제 취득·가공
status: active
created: 2026-10-29
updated: 2026-10-29
---
# AMI 화자 분리 평가 자료의 실제 취득·가공

2026-10-18. 공개 human-derived reference와 실제 음성 입력을 취득했다. 이 문서는 자료 identity·시간 축·가공 검증을 담당하며 모델 품질을 판정하지 않는다. [독립 corpus 조사](../../player/infrastructure/inference/research-followups/diarization-reference-corpus.md)와 [관측 결정](../../../adrs/player/adr-016-ami-diarization-observation.md)을 따른다.

## 고정 원본과 실제 identity

uncropped 음성은 HF `FluidInference/ami-corpus-mirror` revision `722d8891643e1e4dc62cfd0d198fa05a1646c3cc`, 파일 `sdm/ES2004a.Mix-Headset.wav`다. 실제33,579,394bytes/SHA256 `3e2560b19bee6952c7c7ce041b0f1ea8a7ea9468044c4eea79d2a2c67e24ab0f`가 LFS 기대값과 같았다. mirror가 제공한 byte identity를 확인했으며 공식 AMI 원격 서버와 독립 byte 대조를 한 것은 아니다. 디렉토리명sdm과 달리 파일은 Mix-Headset이며 원거리 SDM 평가로 부르지 않는다.

reference는 `BUTSpeechFIT/AMI-diarization-setup` revision `2509d8933721023fab4def2618aabd5c28eb82e9`의 only_words RTTM과 UEM이다. manual v1.6.2의 사람 주석에서 결정적으로 변환된 reference이며 모델 pseudo-label이 아니다. only_words와 vocal sounds 포함 경계는 서로 다르고 모든 acoustic event의 절대 정답으로 일반화하지 않는다.

| 실제 파일 | bytes | 게시 Git blob SHA1 검증 | 실제 로컬 SHA256 |
|---|---:|---|---|
| ES2004a.rttm |14,796 |a8006e7c9d94cf98eaeb0a4883e910a411d74bee |9869c6146c2fd9595403edb36c2caeda65c12ffa2c0af4ce48d6814b673fd5a9 |
| ES2004a.uem |28 |d8bfa4b6d70fce2e10aa9a18a78c9157a27c5207 |514373eae2841ddf15fb07938a7d74bbecc2acad8afc2a28a93e0f7b24bfb446 |
| setup README |2,337 |0721c68032c7f050c8e6b1dd684f3e46b4a02030 |6601323c8bb48e2dd2003371ab808fc13643688d0c8d233ec1e5f715e84c57dd |
| mirror README |1,602 |8ed055f32a7f2886f78360f2030466145e24a7c5 |36a93fe224039211c85f218093a2e41f07d1255e5f165f3a8164603023b8df7e |

Rust sha1=0.10.7은 Git의 blob 길이/header를 포함한 SHA1을 검사하며, 이를 raw SHA1이나 LFS SHA256이라고 부르지 않는다. corpus의 CC BY4 표기와 setup software 라이선스를 구별하고 제공자 설명 원문도 보존한다. 일본어 NC 자료·gated 한국어·큰 archive/parquet는 이번 취득에 포함하지 않았다.

## 실제 오디오와 시간 축

Rust hound로 실제 header를 확인했다:16,000Hz·mono·PCM16 integer·16,789,675samples·1,049.3546875초다. 원본 UEM의 끝1,049.354687초와 음성 길이의 미세한 표기 차이를 숨기지 않는다. 첫 crop은 원본/local 모두 `[0,60)`초·sample `[0,960000)`이며 재표본화·무음 제거·발화 이어 붙이기를 하지 않았다.

crop `ami-es2004a-0-60.wav`의 SHA256은 `54688b088c567a34d18901af07dba4606519e2edbec26d1f10d58e00ca0477a5`, 원시 PCM16 little-endian SHA256은 `b59a26dc91495d1457367aa04dfbddc01d4793404196c80c5e70afeda3b94f3a`다.960,000samples를 저장 후 재읽어 원본 prefix와 같음을 확인했고, 전체 source WAV hash도 전후 유지됐다. 새로운 파일만 생성하며 기존 원본/fixture를 덮어쓰지 않는다.

reference상 이 구간에는13turn과 MEO015/FEE013/FEE016 세 화자, overlap과 긴 무음이 있다. 이는 연구에서 읽은 reference 정보이며 model cluster 수나 평가기 독립 parsing 통과와 구별한다. 별도 평가기는 UEM 전체를 crop과 교차하고 unknown/overlap을 제거하지 않아야 한다. reference 수를 모델에 oracle로 전달하지 않는다.

## 실행·보존과 남은 검증

`ami-corpus-r1`의 실제 preflight CPU33.30049261083744%·여유31,538,053,120bytes에서 Rust가5파일을 순차 취득했고 모두 HTTP200·기대 byte/hash 검사를 통과했다. redirect 최종 위치는 임시 query를 제거하고 기록했다. 사용자 앱·OS·메모리 캐시·VM/DPI·Pages는 변경하지 않았으며 다운로드/가공은 모델 추론이 아니다.

취득 실행 도구 SHA256 `3f6e2479d8b37e1fdcbc58e61ca286ce3ab61649f50a3562f520249c1dab3378`의77파일을 선보존했고19tests/fmt/Clippy-Dwarnings/locked offline build가 통과했다. Git blob의 known-answer 검사를 포함한다. [취득 결과](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/ami-corpus-r1/result.json), [자료 manifest](../../../evidence/ai/model-spikes-2026-10-18/results/ami-es2004a-corpus-v1.json), [RTTM](../../../evidence/ai/model-spikes-2026-10-18/corpora/ami-es2004a-v1/ES2004a.rttm), [UEM](../../../evidence/ai/model-spikes-2026-10-18/corpora/ami-es2004a-v1/ES2004a.uem), [60초 fixture](../../../artifacts/spikes/2026-10-18/fixtures/diarization/ami-es2004a-0-60.wav)를 보존한다. 전체33.6MB WAV는 runtime source에 유지하고 증거 폴더에 중복 복사하지 않는다.

기존 prototype의 timestamp_note에 들어 있던 source offset107s를 수정했다. 새 출력은 입력 파일 로컬 시간/source_offset_applied_us0이며, 이전 Sintel 입력의107초 복원 정보는 그 fixture manifest에서 찾는다. speaker_cluster_count·speaker_count_approvedfalse를 명시하고 과거의 legacy speakers 필드를 실제 화자 수로 해석하지 않는다. 수치 모델/경계 projection을 이 metadata 수정으로 개선했다고 주장하지 않는다.

다음은 고정 관측 게이트의 실제 segmentation/embedding 실행과 별도 Rust DER/JER 평가다. 평가기 준비의 합성 테스트를 실제 corpus 점수나 원본 scorer 동등성으로 대체하지 않는다. 한국어/일본어·애니메이션·원거리 마이크·회차 간 화자 identity는 이 한 영어 meeting 입력의 범위 밖이다.
