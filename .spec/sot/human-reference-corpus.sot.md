---
id: sot:kinekino/human-reference-corpus
title: 화자 분리 실측용 human reference corpus
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 화자 분리 실측용 human reference corpus

조사일: 2026-10-18. **첫 bounded 실측은 영어 AMI `ES2004a.Mix-Headset.wav`의 원본 `[0,60)`초와 BUTSpeechFIT `only_words` RTTM을 권고한다.** 이 구간에는 정답상 세 화자와 겹침이 있다. 한국어 성능 검증으로 확대하지 않는다. 일본어에서는 IngCrowd의 원본 수집·수작업 ELAN 자료와 개별 21.6 MB WAV를 확인했지만 CC BY-NC 조건 때문에 제품 개발 목적의 이용 허용을 이번 조사에서 확정하지 않았다.

읽은 것은 공식 corpus 설명·재배포자 선언·HF metadata·텍스트 annotation·공식 scorer source다. 음성/영상/모델/zip/parquet를 취득하거나 설치·빌드·추론하지 않았다. 인증이나 제한을 우회하지 않았다. 이 문서와 `runtime/coordination/diarization-reference-corpus.md`만 작성한다. retrieval 연구·코드·기존 관측·부모 Git·사용자 앱·Pages·VM·DPI는 변경하지 않는다. HF LFS SHA256은 **metadata 기대값**이며 실제 다운로드 파일의 검증값이 아니다. 작은 text annotation을 원격 열람한 것과 corpus 파일을 로컬 취득한 것을 구별한다.

## 한국어·일본어를 먼저 살핀 결과

| 후보 | 사람 정답의 근거 | 실제 취득 단위·권리/인증 | 첫 실측 판정 |
|---|---|---|---|
| 한국어 AI-Hub 회의 / KconfSpeech | [AI-Hub 원본 소개](https://aihub.or.kr/aihubdata/data/view.do?aihubDataSe=data&currMenu=120&dataSetSn=464)는 다화자 회의 음성인식 자료를 설명한다. 이번 조회에서 개별 clip의 human speaker/time 정답과 원본 byte identity까지 연결하지 못함 | HF `jp1924/KconfSpeech`는 `jp1924-audio-1/KconfSpeech`로 redirect, `gated=auto`, license metadata 없음. 첫 parquet 하나가 **9.56 GB**. 원본 신청/이용 조건은 별도 | 작은 직접 파일·human RTTM·권리까지 검증된 경로가 아니므로 보류. 한국어 정답 corpus 자체가 없다는 뜻은 아님 |
| 일본어 IngCrowd themed dialogue | 원본 제공자인 株式会社イングクラウド가 수집한 Zoom 2인 대화. 카드가 ELAN 수작업 발화 분할/시간 정렬·사람 전사·ASR 미사용을 명시 | 개별 mixed WAV 21.6 MB + EAF 103 KB. 공개·ungated, CC BY-NC 4.0. 영상은 공개되지 않고 별도 요청 대상 | **기술적으로 가장 직접적인 JA 후보**. NC 범위 내 연구 또는 별도 이용 권한이 확인되면 `[0,60)`초 후보. 이번 root용 1순위는 NC 제약 없는 AMI로 구별 |
| 일본어 CALLHOME 변환 `ayousanz/callhome-ja-plus` | [TalkBank 원본 설명](https://talkbank.org/ca/access/CallHome/jpn.html)은 LDC 녹음 동의·사람 audit·전사 구간 선정을 설명. 변환자는 TalkBank → WAV/JSON/RTTM 변환을 선언 | 개별 WAV 약 19.4 MB. 변환 card는 CC BY-NC-SA-4.0, 실제 API ungated. 원본 TalkBank/LDC 조건과 변환 순번→원본 call ID 연결 별도 | 파일이 작아도 원본 권리·시간 offset·원본 call mapping이 완전히 검증되지 않아 첫 추천 제외. 변환자의 SPDX가 원본 권리를 대체하지 않음 |
| 영어 AMI | AMI Consortium human manual annotation v1.6.2 → BUTSpeechFIT의 결정적 RTTM 변환 | 아래 pinned HF mirror WAV 33.6 MB, 독립 text RTTM/UEM. CC BY 4.0 corpus; BUT setup 소프트웨어 Apache-2.0 | **첫 추천**. 재배포본이고 원본 byte 일치와 WAV header는 취득 담당의 검사 대상으로 남김 |
| 영어 VoxConverse | [저자 저장소](https://github.com/joonson/voxconverse)는 RTTM 정답과 corrected v0.3을 제공. 영상에서 구성한 자료이므로 원래 영상 소유권도 별도 명시 | 저자 audio는 split archive, `diarizers-community/voxconverse`의 첫 dev parquet 485 MB. 작은 clip 재생 UI와 취득 단위가 다름 | 이번 개별 수십 MB 우선 조건에서는 제외. HF 변환 정답의 버전도 원본 v0.3과 별도 확인 필요 |

한국어·일본어 우선 탐색은 수행했지만 권리·배포 크기·정답 provenance를 모두 만족한다고 확정한 한국어 경로는 없었다. JA의 NC 자료를 상업 관련 개발에 자유롭게 쓸 수 있다고 단정하지 않는다. [CC BY-NC 원문 설명](https://creativecommons.org/licenses/by-nc/4.0/)은 비상업 조건과 출처·변경 표기를 요구한다. 이 판정 때문에 사용자를 대신해 제공자에게 연락하거나 동의를 제출하지 않았다.

## 첫 추천: AMI ES2004a headset mix, 원본 0~60초

### 원본 → 미디어 재배포 → human 정답 변환 계보

AMI Consortium의 [원본 download 안내](https://groups.inf.ed.ac.uk/ami/download/)는 **manual v1.6.2 약 22 MB**와 **automatic v1.5.1 약 68 MB**를 구분한다. manual annotation은 2014-06-16 내용에서 유지되고 2017-04-10 CC BY 4.0으로 변경됐다고 안내한다. 자동 annotation archive를 human gold로 바꾸어 쓰면 안 된다. 이번 네트워크에서는 원본 사이트 직접 열람/HEAD가 timeout 또는 연결 거부였으나 검색 인덱스의 해당 공식 문서 내용은 확인했다. 원본 파일을 취득했다거나 현재 직접 GET 가능하다고 주장하지 않는다.

[FluidInference 재배포 카드](https://huggingface.co/datasets/FluidInference/ami-corpus-mirror)는 AMI 원본 mirror에서 가져온 오디오를 변경 없이 CC BY 4.0으로 재배포하고 manual annotation zip은 공식 자료의 repack이라고 선언한다. 이는 **재배포자 자신의 계보 선언**이다. 이번 조사에서 원본과 재배포 audio/zip을 byte 비교하지 않았다. zip 이름이 같아도 repack zip hash가 원본 archive hash와 같다고 가정하지 않는다.

[BUTSpeechFIT 원본 setup](https://github.com/BUTSpeechFIT/AMI-diarization-setup/tree/2509d8933721023fab4def2618aabd5c28eb82e9)은 RTTM을 AMI manual v1.6.2에서 직접 도출했다고 명시한다. `only_words`는 words만 speech로 쓰고 같은 화자의 인접 word를 합치며 실제 silence gap은 합치지 않는다. `word_and_vocalsounds`는 별도 reference로, 저자가 vocal sound annotation의 불일치를 설명한다. 이번 첫 실측은 **only_words** 하나로 고정한다. RTTM 변환은 알고리즘이지만 speaker/time 라벨의 원천이 사람 annotation이라는 뜻이며, diarization 모델이 생성한 pseudo-label과 다르다. 원본 word 경계가 모든 acoustic event의 절대 정답이라고 확대하지 않는다.

### 고정 파일과 기대 identity

HF API `/api/datasets/FluidInference/ami-corpus-mirror?blobs=true`의 관측 revision은 **`722d8891643e1e4dc62cfd0d198fa05a1646c3cc`**, gated=false, card license=cc-by-4.0이다. 전체 repo는 약 1 GB지만 **아래 개별 파일만** root 취득 대상으로 한다.

| 용도 | 고정 파일 경로 | bytes | metadata 기대 hash |
|---|---|---:|---|
| 실제 혼합 음성 | `sdm/ES2004a.Mix-Headset.wav` | 33579394 | LFS SHA256 `3e2560b19bee6952c7c7ce041b0f1ea8a7ea9468044c4eea79d2a2c67e24ab0f` |
| 원본 manual 증빙, 필요시 별도 취득 | `annotations/ami_public_manual_1.6.2.zip` | 22887865 | LFS SHA256 `56e29112cd89cfba76fbf8f10215eb245cc89b0d3d27355ba14b406a7ea7776c` |
| mirror 설명 | `README.md` | 1602 | Git blob SHA1 `8ed055f32a7f2886f78360f2030466145e24a7c5`, LFS 아님 |

직접 취득 위치: [고정 WAV resolve](https://huggingface.co/datasets/FluidInference/ami-corpus-mirror/resolve/722d8891643e1e4dc62cfd0d198fa05a1646c3cc/sdm/ES2004a.Mix-Headset.wav), [고정 annotation zip resolve](https://huggingface.co/datasets/FluidInference/ami-corpus-mirror/resolve/722d8891643e1e4dc62cfd0d198fa05a1646c3cc/annotations/ami_public_manual_1.6.2.zip). 이번에는 파일 body를 받지 않았다. root는 HTTP status·redirect 최종 위치·실제 bytes·SHA256을 기록해야 한다.

| 정답 / GitHub revision `2509d8933721023fab4def2618aabd5c28eb82e9` | bytes | Git blob SHA1 (LFS 아님) |
|---|---:|---|
| `only_words/rttms/test/ES2004a.rttm` | 14796 | `a8006e7c9d94cf98eaeb0a4883e910a411d74bee` |
| `uems/test/ES2004a.uem` | 28 | `d8bfa4b6d70fce2e10aa9a18a78c9157a27c5207` |
| `README.md` | 2337 | `0721c68032c7f050c8e6b1dd684f3e46b4a02030` |

직접 text: [RTTM](https://raw.githubusercontent.com/BUTSpeechFIT/AMI-diarization-setup/2509d8933721023fab4def2618aabd5c28eb82e9/only_words/rttms/test/ES2004a.rttm), [UEM](https://raw.githubusercontent.com/BUTSpeechFIT/AMI-diarization-setup/2509d8933721023fab4def2618aabd5c28eb82e9/uems/test/ES2004a.uem). text에는 LFS SHA256이 없고 이번에 로컬 원본 SHA256도 생성하지 않았다. SHA1 blob ID를 SHA256처럼 쓰지 않는다. root 취득 후 raw bytes SHA256을 추가해야 한다.

**채널 주의:** mirror 디렉터리/카드는 `sdm`이라고 부르지만 파일 basename은 `Mix-Headset`이고 AMI signal metadata도 headset mix를 별도 signal로 구분한다. 이 추천은 **헤드셋 혼합 음성**이며 array1 channel1/SDM 원거리 음성 평가가 아니다. 공식 AMI headset 계열의 기대 규격은 16 kHz PCM16 mono이나 이 개별 WAV header는 미검증이다. 취득 담당이 RIFF fmt/data·sample rate·channels·frame 수·시간 길이를 읽은 뒤에만 해당 값을 actual로 기록한다. 불일치하면 임의로 무시하지 말고 원본 및 변환 hash와 변환 방법을 분리한다. 16 kHz mono가 맞으면 재표본화 없이 sample-exact crop이 가능하다.

### 미리 고정할 crop과 정답 화자

원본 UEM text는 `ES2004a 1 0.000 1049.354687`이다. 전체 원본 길이를 60초라고 부르지 않는다. 첫 실험은 **원본 `[0.000,60.000)`초**, local `[0,60)`초, RTTM channel=1이다. 16 kHz라면 sample index `[0,960000)`을 보존한다. ASR/VAD로 무음 제거하거나 word 순서대로 이어 붙이지 않는다.

읽은 RTTM의 이 구간에는 **13개 turn, `MEO015`, `FEE013`, `FEE016` 세 화자**가 있고 다음 turn onset은 63.81초다. 세 ID는 원본 reference의 익명 ID이며 모델의 speaker_1/2/3과 숫자가 같아야 한다는 뜻이 아니다. 전체 회의 참가자 수와 crop 안에 실제 발화가 있는 reference 화자 수를 구별한다.

| 관측 예 | 원본 좌표(초) | 용도 |
|---|---|---|
| MEO015 | 0.37~1.76, 17.88~18.15 | 짧은 화자 turn을 prototype이 누락하는지 확인 |
| FEE013와 FEE016 동시 발화 | 25.18~25.60 | reference상 overlap, 제외하지 않는 기본 DER의 필수 구간 |
| FEE013와 FEE016 동시 발화 | 29.86~30.48, 31.30~31.98 | 반복 overlap과 화자 continuity |
| 마지막 crop 내 turn FEE016 | 49.75~53.18 | 뒤 무음 53.18~60도 UEM에 남겨 false alarm 평가 |

위 수치는 RTTM annotation 관측이며 청취·음성 분석으로 독립 확인한 값이 아니다. 모델 성능을 보며 crop을 바꾸지 않는다. first crop은 correctness/diagnostic이며 영어 한 구간으로 일반 DER·한국어/일본어 성능이나 제품 채택을 승인하지 않는다.

일반 crop `[a,b)` 규칙은 각 원본 turn `[s,e)`를 `[max(s,a),min(e,b))`로 자르고 양의 길이만 보존한 뒤 a를 뺀다. RTTM의 duration은 end가 아니므로 먼저 `e=s+duration`이다. UEM도 원본 scoring region과 crop의 교집합을 구한 뒤 a를 뺀다. 원본 file ID·speaker ID와 crop-local ID 매핑, sample 인덱스·resample delay·source offset을 manifest에 기록한다. round-trip을 위해 원본 annotation은 그대로 보존하고 crop reference는 별도 파생물로 둔다.

## 일본어 직접 원본 후보: IngCrowd dialogue01

HF 원본 제공자 repo [고정 revision](https://huggingface.co/datasets/IngCrowd/japanese-themed-dialogue/tree/e2eed8769400269f20821deb0e47e68b40012e8b)은 **`e2eed8769400269f20821deb0e47e68b40012e8b`**, API gated=false, card CC BY-NC 4.0이다. annotation 생성 주체는 카드상 IngCrowd의 human annotators이며 `AUTHOR` XML 속성은 비어 있어 특정 개인 이름을 추정하지 않는다. 카드의 수작업 선언은 확인했지만 annotator agreement 수치·독립 감사·참가자 동의서 원문을 확보한 것은 아니다.

| `japanese-themed-dialogue01/` 아래 실제 파일 | bytes | metadata 기대 identity |
|---|---:|---|
| `japanese-themed-dialogue01_audio.wav` 혼합 2인 음성 | 21624910 | LFS SHA256 `ea1138f2b05e6b3cc9580ed1196748ed67c5847f222c1cf4a54de6bf4d9d3282` |
| `japanese-themed-dialogue01.eaf` 사람 시간·화자 annotation | 103214 | Git blob SHA1 `fe885dd9299e1aec319550df244c903541447963`, LFS 아님 |
| `japanese-themed-dialogue01_individual_audio1.wav` | 21624910 | LFS SHA256 `9ab7c49391960428920b89b4206da6e9281092651d039d2e1ffbbae001448b54` |
| `japanese-themed-dialogue01_individual_audio2.wav` | 21624910 | LFS SHA256 `686a626dae1572cb32e977a9e60300e9acafacd11a9dfbfaaa373ca21ac80566` |

원본 카드 규격은 WAV 16 kHz / 16-bit / mono이며 개별 header는 미검증이다. mixed WAV 하나와 EAF만 있으면 2인 overlap-aware 평가 입력을 만들 수 있다. isolated tracks를 시스템 입력에 몰래 제공해 speaker 분리를 쉽게 하지 않는다. speaker_1/2의 demographic/individual-track 연결은 카드의 참가자 설명과 구별하고, DER에는 EAF tier ID만 사용한다.

읽은 EAF는 `TIME_UNITS=milliseconds`, media URL은 같은 디렉터리 mixed/individual WAV 3개, 별도 TIME_ORIGIN 선언은 없다. time slot의 ms 값을 두 `ALIGNABLE_ANNOTATION` reference로 연결한다. `speaker_1`의 첫 구간은 ts1=3470~ts2=4440 ms, `speaker_2`는 ts3=4750~ts4=5630 ms다. overlap 예로 speaker_1 18820~19320 ms와 speaker_2 19010~22900 ms가 있다. 따라서 권리가 확인되면 **같은 `[0,60)`초**가 최소 2화자와 overlap을 포함하는 bounded JA 후보가 된다. tiers는 발화 단위라서 word-only AMI와 speech/vocalization 경계 정책이 같다고 가정하지 않는다. laughter/unintelligible annotation 처리 규칙은 EAF annotation value/가이드와 함께 별도 확정해야 한다.

Rust 변환은 XML 외부 entity/DTD/network access를 허용하지 않고 time slot ID와 tier ID를 연결해 `[start_ms,end_ms)`를 보존한다. token/전사 문자열로 forced-aligner를 다시 실행해 human 경계를 대체하지 않는다. 미정 slot이나 `REF_ANNOTATION`을 가짜 0초 구간으로 만들지 않는다. 이 구조를 읽었다는 것이 EAF→RTTM converter를 구현·검증했다는 뜻은 아니다.

## 취득 비용을 숨기지 않는 대안 기록

| HF repo / 관측 revision | 실제 파일 예 | bytes / 기대 LFS SHA256 | 상태 |
|---|---|---|---|
| `jp1924-audio-1/KconfSpeech` `d438aec3d79d422fde2a08873521a36b2d137a5e` | `ASR/train-00000-of-00035.parquet` | 9563131650 / `397196c420a375841cd0f00a67152498078dcb333bebf2cb8443ee93b132dd8e` | gated auto, license null. 작은 clip 직접 취득이라고 표시 금지 |
| `talkbank/sakura` `e7bd5bb421f5247b8d1a8022cd5534bda895ab32` | `data/data-00000-of-00002.parquet` | 458415990 / `6cfea3daef15634e9d4e9b1e99dcd8c48f6e7d8f0957ed19dcdf39f56ce2706f` | ungated이나 API license null; 원본 human/time·권리 연결 미완료 |
| `diarizers-community/ami` `8cdaae2eaf968f3b000b6eb1204ab9b8db006ed0` | `ihm/test-00000-of-00003.parquet` | 294805285 / `d0d57a08d591f0eb22a039e9c7da87715735b6fe13ff87f79268a5d9bd960b1a` | CC BY4 card, 첫 추천의 별도 WAV보다 취득 단위 큼 |
| `diarizers-community/voxconverse` `3acfa1b45ca4b7419aee999d67d94c617f9c9d47` | `data/dev-00000-of-00005.parquet` | 485283393 / `f36c54412f0ac9cfe7ec2682e27f70f3e3df63d8e2b8f288d4f62341a595917d` | CC BY4 card; 원본 영상 저작권과 RTTM v0.3 정합은 별도 |
| `ayousanz/callhome-ja-plus` `3711ecb10f7924b6cdb63a9f95f3fd4dcad0bc7f` | `callhome_jpn_0.wav` | 19409004 / `3f5739030497c66d7616a0e0e72b3144226b5ff4e9f8689a6302caa68699475c` | 변환 card NC-SA. 원본 call mapping/offset 추가 확인 필요 |

CALLHOME 변환의 `callhome_japanese.rttm`은 2367174 bytes, JSON metadata는 5641323 bytes다. README 예시는 606.53초 음성 및 16 kHz를 선언하며 원래 전화 녹음의 시간 origin과 변환 WAV의 origin을 같은 것으로 추정하지 않는다. HF metadata가 null인 것은 무권리 또는 자유 이용의 판정이 아니다. `pyannote/AMI` 등의 API 401은 private/부재/권한을 구별할 수 없어 존재하는 공개 corpus라고 사용하지 않았다.

## 평가 규약: Rust로 구현할 범위를 명시

### 정본 scorer source와 명명

비교 기준은 [dscore](https://github.com/nryant/dscore/tree/e02f949ac6592279300a2c33d03daf9e0c12fd27) **`e02f949ac6592279300a2c33d03daf9e0c12fd27`**의 정의다. 이 조사에서 Python/Perl scorer를 설치하거나 실행하지 않았다.

| 기준 파일 | bytes | Git blob SHA1 |
|---|---:|---|
| `scorelib/metrics.py` | 25080 | `e21183bf6c991f581d214e6708aa5475f154d36b` |
| `scorelib/score.py` | 9018 | `4c6617c5e7bf63677b3e1453865f984a43c223d9` |
| `scorelib/md-eval-22.pl` | 109574 | `27b7bc9da5751eec8fb8de2f7fe570d301f966bd` |

기본 출력은 **`DER: collar=0, overlap=included, UEM=crop-full`**와 **`JER: dscore-style 10ms, min_ref_dur=0, no collar, overlap included`**로 분리한다. Rust에서 수식·좌표·mapping을 구현할 계획이며, 원본 scorer와 수치 대조하기 전에는 'dscore bit-identical' 또는 'NIST certified'라고 부르지 않는다. RTTM/UEM 검증·오디오 crop·평가 계산·테스트는 모두 Rust로 구현한다.

### RTTM/UEM와 DER

RTTM은 SPEAKER/file/channel/onset/**duration**/NA/NA/speaker/NA/NA의 10필드다. 초 좌표의 반열림 구간으로 해석하며 channel 1을 검증한다. NaN·음수 onset·비양수 duration·overflow·다른 file ID는 원인과 함께 거부한다. 같은 화자의 겹치는 turn은 집합 union으로 처리해 자체 중복을 이중 발화로 세지 않는다. 서로 다른 화자의 overlap은 보존한다. source decimal 경계를 임의의 10ms frame으로 반올림해 **DER**을 계산하지 않는다.

UEM은 평가 영역이다. 첫 crop은 `[0,60)` 전체이며 silence도 포함해 false alarm이 계산된다. reference 또는 hypothesis의 speech union만으로 UEM을 자동 추정하면 무음 오탐이 사라질 수 있으므로 금지한다. 원본/크롭 좌표를 섞지 않는다.

continuous-time DER은 reference/hypothesis/UEM의 모든 경계로 원자 구간을 나눈다. 그 구간의 active reference 화자 수 R, hypothesis 수 H, 전역 one-to-one mapping 후 실제 맞은 화자 수 C에 대해 다음 speaker-time을 합산한다.

- MISS: `duration × max(R−H,0)`
- FA: `duration × max(H−R,0)`
- CONFUSION: `duration × (min(R,H)−C)`
- 분모: `duration × R`; DER은 세 오류 합 / 분모 ×100.

mapping은 crop 전체의 reference↔hypothesis 화자별 co-occurrence duration 행렬을 만들고 **총 일치 시간을 최대화하는 Hungarian one-to-one assignment**로 구한다. 구간별/10초 window별로 유리한 mapping을 새로 만들지 않는다. unmatched 화자와 overlap을 남기며 greedy 최대값 순서 선택을 최적 mapping으로 부르지 않는다. DER은 reference speaker-time을 분모로 써 overlap 1초에 2화자가 있으면 분모 2초가 된다. 과도한 FA에서는 100% 초과가 가능하므로 clamp하지 않는다.

기본 collar는 0이다. 보조 결과가 필요하면 **각 reference 경계의 양쪽 250ms 제거**라는 물리 규약을 명시하고 effective UEM·제거 길이를 남긴다. dscore/md-eval `collar=.25`는 ±.25초이지만 [pyannote utils 고정 source](https://github.com/pyannote/pyannote-metrics/blob/2b02d04586bad4dedb7ad05ea01a987c7021bf12/src/pyannote/metrics/utils.py)의 collar는 `t±0.5*collar`다. 같은 제거 구간을 만들려면 해당 pyannote 구현에는 .50이 대응한다. **'collar 0.25' 문자열만으로 비교하지 않는다.** 보조 overlap-excluded 점수는 reference에서 R>1인 시간을 effective UEM에서 제거하는 별도 지표이며 prediction overlap을 제거하는 규칙이 아니다. 기본 overlap-included 결과를 대체하지 않는다.

0-collar와 단순 SPEAKER RTTM 범위만 우선 구현하며 NIST의 lexical no-score·metadata subtype·word-mediated alignment 전체를 재구현했다고 주장하지 않는다. 분모가 0인 입력은 이 첫 평가에서 유효 speech 평가가 아니므로 상태/오류 시간과 함께 별도 보고하고 DER=0으로 성공 처리하지 않는다. 다중 recording 합산 DER은 각 오류 speaker-time과 분모를 합친 후 나누며 파일별 DER의 단순 평균과 구별한다.

### JER은 다른 mapping과 frame 계약

[dscore metrics의 JER](https://github.com/nryant/dscore/blob/e02f949ac6592279300a2c33d03daf9e0c12fd27/scorelib/metrics.py)는 speaker pair의 `1−intersection/union` 비용을 최소화하는 Hungarian assignment를 쓴다. DER의 **최대 overlap mapping을 재사용하지 않는다**. 각 reference 화자에 대해 매칭된 system 화자의 IoU error를 내고 unmatched reference는 1로 둔다. min_ref_dur=0으로 짧은 화자도 남긴다. 평균은 reference 화자별 macro 평균이며 %로 보고한다. matched pair 밖의 추가 system 화자는 JER에서 충분히 벌점이 안 될 수 있으므로 DER·FA·예측 화자 수를 반드시 같이 보고한다.

[dscore score.py의 frame 변환](https://github.com/nryant/dscore/blob/e02f949ac6592279300a2c33d03daf9e0c12fd27/scorelib/score.py)은 `n_frames=int(max_uem_end/.010)`, `times[k]=.010*k`, onset/offset 및 UEM 양 끝에 left `searchsorted`를 적용한다. 그 범위에서 speaker별 multi-hot presence를 만들고 UEM 밖 frame을 제거한다. **frame midpoint sampling이나 nearest rounding이 아니다.** 60초 local UEM이면 6000개 grid point가 기본이다. 겹침은 각 화자의 presence가 동시에 1이다. JER 계산에는 DER collar/overlap-ignore 옵션이 자동 적용되지 않는 것이 dscore 기준이다.

Rust로 이 경로를 재현할 때 step, 좌표 파싱 f64, IEEE754 곱셈 및 lower_bound 경계 비교·반열림 범위·최종 부분 frame 제외를 고정한다. 정수 microsecond로 만든 비슷한 grid가 NumPy float64 searchsorted와 모든 경계에서 같다고 가정하지 않는다. roundoff 경계 fixture의 표준 예상치를 고정한 뒤 parity를 확인한다. continuous interval JER을 추가한다면 이름을 다르게 하고 dscore 10ms JER이라고 표시하지 않는다. pyannote JER과 dscore JER도 mapping/영역 계약의 동일성 확인 없이 혼용하지 않는다.

## 현재 prototype에 적용할 때 숨기면 안 되는 차이

기존 [Rust prototype](../../../../../../tools/model-spike/src/infrastructure/inference/diarization.rs)을 읽었다. pyannote powerset segmentation + WeSpeaker fbank/embedding + greedy cosine clustering이며 공식 community-1 전체 pipeline이 아니다. 10초 window를 사용하고 실제 receptive-field offset 대신 frame을 균등 투영한다. 일부 0.5초 미만 span은 speaker_id=null, overlap은 global_speakers=null이다. **수행 성공을 DER/JER 검증으로 바꾸지 않는다.**

평가용 hypothesis RTTM에는 실제로 식별된 speaker_id 구간만 넣고, null/미해결 overlap을 reference에서 삭제하지 않는다. 해결되지 않은 speech가 정답 시간에 있으면 known-speaker hypothesis의 miss/error로 남는다. unresolved duration·unresolved overlap·모델이 검출한 speech 전체의 VAD 지표는 별도 진단으로 보고한다. 임의의 dummy 화자로 채우거나 같은 local powerset class를 window 간 동일인으로 해석해 정답을 꾸미지 않는다. 이 정책을 바꾸면 다른 평가 variant다.

prototype의 timestamp_note에는 기존 fixture `source offset107s`가 hardcoded돼 있으나 첫 추천 crop은 offset0이다. local 계산값에 107초를 더하거나 메모를 재사용하면 안 된다. root가 새 frozen build/관측 manifest에서 실제 source offset을 명시해야 한다. model frame 좌표 오류를 정답에 맞춰 임의 shift로 최적화하지 않는다. segmentation과 embedding 파일 hash·전처리·threshold·예측 화자 수·raw JSON·frozen 실행파일/source를 같은 label에 연결한다. reference의 3화자를 모델에 강제로 주는 oracle 평가를 기본으로 하지 않는다.

## root 실행 담당의 다음 순서

1. 고정 WAV + RTTM + UEM을 개별 취득하고 실제 bytes/SHA256, WAV fmt/channels/sample rate/frames를 기록한다. CC BY4 attribution(AMI Consortium/Carletta et al.)과 source/mirror 및 crop 변경 표기를 보존한다. BUT Apache 소프트웨어 라이선스를 음성 corpus의 라이선스로 대체하지 않는다. original server와 mirror의 byte 비교가 미완료라는 한계도 유지한다.
2. 원본 0~60초를 Rust로 crop하고 audio/reference/UEM의 시간 축을 확인한다. 지정3화자/overlap을 root가 청취·annotation과 대조한 뒤 독립 검토 결과를 남긴다. short diagnostic 선택을 성능 확인 후 바꾸지 않는다.
3. Rust evaluator는 먼저 label permutation DER/JER=0, 누락 speech=100%, reference A[0,1)/B[1,2)→system X[0,2)에서 DER50%/JER75%, overlap·silence FA·UEM clipping·빈 분모·10ms boundary fixture를 검증한다. 이는 **향후 Rust 테스트 요구**이며 이번 실행 결과가 아니다. 같은 metric 이름을 쓰기 전 고정 scorer 정의/공개 fixture와 수치 대조한다.
4. root 한 worker가 실제 모델 실행을 수행한다. 기본0-collar overlap-included DER과 dscore-style10ms JER, 오류 components·분모·mapping·ref/hyp 화자 수·미해결 시간을 함께 보존한다. English crop 한 개의 결과만 말하고 한국어/일본어 검증 또는 전체 AMI benchmark처럼 보고하지 않는다.

domain은 시간 구간·화자 집합·평가 정책, application은 corpus/crop/evaluation 공통 유스케이스, infrastructure는 HTTP/WAV/RTTM/EAF/ORT adapter를 담당한다. GUI/CLI가 같은 정책을 사용한다. 이번 과제는 문서 조사이며 새 evaluator/다운로더/변환기 구현·Cargo 검증·추론을 수행하지 않았다.
