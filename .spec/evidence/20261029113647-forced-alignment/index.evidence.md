---
id: evidence:kinekino/20261029113647
title: Rust 강제 정렬 실행·구조 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Rust 강제 정렬 실행·구조 검증

후속 [공식 Torch·Rust encoder 대조](forced-encoder-stage-actual.md)에서 같은 full mel3개·공통38stage씩 총114tensor가 고정 atol+rtol 기준을 통과했다. encoder 전용6회이며 아래 alignment17회에 합산하지 않는다. timestamp head·전체 processor·사람 경계는 별도 미검증이고 기존zero 문제 해결로 표현하지 않는다.

현재는 후속 [고정 분절 A/B5회](forced-segmentation-ab-actual.md)까지 실제 정렬17회, 구조3통과/14실패다. 기존 [전체 classifier logits 관측2회](forced-head-logits.md)의118row 감사와 새 default 동등성은 유지됐고 원본 분절에서도 기존zero가 남았다. 아래 시점별 과거 기록은 보존하며 품질 승인은0이다.

2026-10-18. source 검토·Windows 빌드·실제 추론·구조 검사를 구분한다. 아래 추론 완료는 사람의 단어 시간 정확도 통과 증거가 아니다. 준비 절의 시점별 기록은 이후 완료 기록과 함께 읽는다.

## 선택과 고정

[후보 검토](../../player/infrastructure/inference/research-followups/rust-forced-aligner-runtime.md)와 [사전 준비 조건](../../player/infrastructure/inference/research-followups/lumos-dictionary-readiness.md)에 따라 JP/KO 지원이 있는 lumosimmo/qwen3-asr-rs commit `7984967e8701cc90bb17d06453901036e36ca578`를 root가 별도 source 디렉토리에 취득했다. 원본 Cargo.lock SHA256은 `6ebe7f7b8f12c605bb47cd678567be0ff7615e4110d786cd87fab3510c557e02`다. 원본 소스·전역 Cargo 설정을 변경하지 않는다.

## 실제 빌드 시작

Windows CPU library를 `--locked --release -p qwen3_asr --no-default-features --features forced-aligner,audio-loading --lib -j2`로 빌드하기 시작했다. 전용 target은 `runtime/parallel-build/lumos-aligner`다. 이전 Qwen 실추론5회가 종료된 뒤 시작했으며 정렬 가중치는 아직 취득하지 않았다.

`LINDERA_DICTIONARIES_PATH`는 이 명령의 환경에만 새 전용 경로 `runtime/ai-spike/dictionaries/lumos-7984967`로 지정했다. 사용자 기존 사전 cache나 모델 evidence cache를 지정하지 않는다. build.rs가 취득할 IPADIC archive와 Cargo crate fetch는 구분한다. 사전 archive의 URL/크기·MD5는 사전 조사에 기록했고 실제 파일 SHA·COPYING 보존은 취득 후 검증한다. Cargo --offline만으로 build.rs 네트워크를 차단한다고 가정하지 않는다.

최초 관측 시점에는 고정 dependency fetch와 Rust 컴파일 시작만 확인했다. 이후 종료 결과는 아래에 별도로 기록한다. 기존 Qwen ASR 모델과 구별되는 classify head의 실제 로드·KO/JP 기준 발화 정렬·시간 경계/분절 평가가 남았다.

## 빌드 완료와 사전 실물 검사

위 CPU library 명령은 exit0, release 빌드9분47초로 완료됐다. SDK 담당에게 동일 target/cache 사용을 인계했고 독립 known-text adapter를 준비 중이다. 이 결과는 library build 성공이며 실제 모델 로드·정렬 품질 검증은 아니다. 공식 정렬 모델 revision `c7cbfc2048c462b0d63a45797104fc9db3ad62b7`의 설정/tokenizer 원본 등 작은8개 파일을 root가 취득했다. 가중치 safetensors는 adapter 빌드 확인 후 취득한다.

root Rust `dictionary-evidence`가 실제 IPADIC archive12,907,073 bytes를 읽어 build recipe의 MD5 `a95c409f12f1023fce8ef91f991ef042`와 일치함을 검증했다. SHA256은 `a7ba9f645ffe7094e56ae1c4a81d100df8fbb1e28bbe1792622e9728e162db3d`다. [실물 manifest](../../../evidence/ai/model-spikes-2026-10-18/results/dictionary-evidence-v1.json)를 보존한다.

archive 내부 COPYING3795 bytes를 [원시 바이트 그대로](../../../evidence/ai/model-spikes-2026-10-18/notices/IPADIC-COPYING.txt) 보존했다. SHA256은 `fca02d9adb601d101eccdf3131119abfff2d9c825ef5c759d7a89ebbaa972099`다. 최초 수집기는 UTF8 가정을 적용해 index3792에서 실패했고, 검사 정책을 고쳐 비UTF8 상태를 명시하고 원문을 transcoding 없이 저장했다. 알 수 없는 인코딩을 임의로 확정하지 않는다.

실제로 컴파일한 Windows checkout의 KO dictionary는296843 bytes/SHA256 `9a3d6881b22b976da41359d74dbe4deaae445521997aa05d6287c540f2b4af65`, JP merge는365 bytes/`d6b45ca59691bb69d8f9be33d6355db272279968017fb81a085e9edeb6b55a02`다. 연구에서 읽은 Git blob의 LF 기준 크기275834/354와 다르다. checkout의 autocrlf 영향 가능성을 명시하며 논리적 사전 동등성이나 byte 동등성을 추정하지 않는다. 원본 source는 바꾸지 않았다.

후속 정정: 보존 helper/README 일부의 “Git archive canonical LF” 문구는 실제 바이트와 다르다. root Rust `engine-archive-evidence`가 archive SHA `22ae59e1c784c025200701e2891c9ef0d6bfc248b0facdad3e3f138669904600` 내부3개 member를 fixed Git blob 및 checkout과 직접 비교했다. [독립 바이트 검사](../../../evidence/ai/model-spikes-2026-10-18/results/lumos-archive-byte-identity-v1.json)에서 model.rs archive30740bytes/CRLF866쌍은 checkout과 동일, Git blob29874bytes/CRLF0과 달랐다. KO dictionary296843bytes/CRLF21009쌍, JP merge365bytes/CRLF11쌍도 archive=checkout이며 각 Git blob과 달랐다. 선택3개 파일의 결과만으로 archive 전체를 동일하다고 확대하지 않는다. archive/manifest는 변경하지 않았으며 과거 note의 LF 설명은 이 관측으로 정정한다. Git commit identity·archive byte identity·빌드 checkout identity를 각각 보존한다.

## 실제 입력 준비와 모델 취득

root Rust `alignment-fixtures`가 한국어1883 및 일본어1516/1634/1578의 원음 SHA를 기존 dataset metadata와 대조하고 `raw_reference` 문자열을 수정하지 않은 UTF8 파일4개로 저장했다. [기준 발화 manifest](../../../evidence/ai/model-spikes-2026-10-18/fixtures/alignment/manifest.json)에 원음/텍스트 해시와 파일을 연결한다. 번역 자막이나 ASR 추측 문장을 정렬 정답으로 대체하지 않았다. 이 transcript는 word timestamp 정답이나 서로 다른 화자 증거가 아니다.

별도 known-text adapter의 첫 Windows CPU release 빌드 성공 인계 후 root가 고정 공식 정렬 가중치1,835,544,544 bytes를 취득했고 LFS SHA256 `47831d0e82f96b20e9034dba01a075ee06436654719f6a68289e49f1b65ce0e7` 일치를 확인했다. small8files와 함께 [다운로드 manifest](../../../evidence/ai/model-spikes-2026-10-18/download-aligner.json)에 기록한다. adapter 최종 시험·EXE 해시 인계 후 root 단일 워커로 실제 정렬을 계측한다. 현재 가중치 취득과 입력 준비는 정렬 품질 통과가 아니다.

## 실제 한국어·일본어 정렬 4회

최종 adapter EXE SHA256 `a2e4568bf2a4b1acceaa46b91db46ffd57adb92730bd9dfa31ac3a149744de1b`, 고정 official model·CPU F32·threads4·batch1로 실제 원음과 알려진 원문을 하나씩 정렬했다. SDK 담당의 adapter 시험7개/fmt/Clippy/release 인계 뒤 실행했으며 source/model 원본은 수정하지 않았다. 최대 private는100ms 표본이며 working set과 구분한다.

| 입력·label | 분절 수 | wall 초 | 최대 private bytes | 최대 working set bytes | 실행 결과 |
|---|---:|---:|---:|---:|---|
| KO1883 / forced-align-ko-r1 | 19 | 12.2265 | 3,954,569,216 | 5,495,820,288 | exit0 / 구조 통과·품질 미승인 |
| JA1516 / forced-align-ja1516-r1 | 28 | 14.8416 | 4,083,232,768 | 5,419,315,200 | exit3 / 길이0 분절2개 |
| JA1634 / forced-align-ja1634-r1 | 31 | 12.6751 | 4,034,220,032 | 5,610,815,488 | exit3 / 길이0 분절3개 |
| JA1578 / forced-align-ja1578-r1 | 12 | 12.3189 | 3,975,356,416 | 5,603,676,160 | exit0 / 구조 통과·품질 미승인 |

모델 추론은4회 모두 완료했지만 프로세스 성공은2회다. exit3인2회는 native crash나 모델 로드 실패가 아니라 반환된 정렬의 구조 검사 실패다. timeout은 없었다. 모델 실행·프로세스 성공·구조 통과·사람의 단어 시간 정확도를 서로 다른 게이트로 해석한다. GPU·양자화·재생 경쟁·idle-host 성능 순위는 미검증이다.

JA1516의 index9 `を`는7.84→7.84초, index10 `もっ`는8→8초다. JA1634의 `は`는2.24→2.24초, 숫자 `0` 두 개는2.8→2.8/3.04→3.04초다. 원래 API 값과 IEEE754 비트·정수 us 파생을 보존했고 epsilon 추가·clamp·삭제로 검사 통과를 만들지 않았다. 제품의 자동 word cue 내보내기/정렬 승인으로 사용하지 않는다.

root Rust `forced-evaluate LABEL`이 원음/원문 hash·문자열 보존·public seconds와 기록된 IEEE754 비트·분절 순서·시간 범위를 독립 재검사했다.4개 입력 모두 원본/비트가 일치했고2개 입력의 길이0 문제를 다시 검출했다. public API는 수리된 seconds만 제공하므로 raw class/tick/pre-repair/fixed tick 필드는 null이다.80ms는 config의 선언값이며 공개되지 않은 raw tick을 역산해 관측값이라고 기록하지 않는다. 원시 출력과 `*-evaluation.json`은 [측정 evidence](../../../evidence/ai/model-spikes-2026-10-18/snapshot-manifest.json)에 보존한다.

## VAD 보조 대조와 남은 품질 게이트

JA1516 첫 단어 시작4.96초가 앞선 무음/비발화 구간과 맞는지 별도 Silero VAD를 동일 원음에 실제 실행했다. native ORT CPU의0.5 threshold에서 최초 활성 프레임은 index156, nominal window4.992–5.024초였고 score0.7605였다. 최초 정렬 시작은 그 window 시작보다32ms 앞섰다. [보조 대조](../../../evidence/ai/model-spikes-2026-10-18/results/forced-vad-ja1516-comparison.json)는 두 모델 출력의 비교이며 사람의 발화/단어 경계 정답이 아니다. VAD 모델의 감지 지연과 프레임 window를 구분하고 이 관측으로 길이0 문제를 승인하지 않는다.

남은 작업은 숫자 이외 분절·공식 nagisa 차이 분석, 알려진 오디오 이동량 대조, 사람이 확인한 단어 경계 정답과의 오차 평가 및 자막 cue 저장/use-case 통합이다. 현재 엔진은 독립 실험 도구이며 GUI/공통 SDK 정렬 기능 구현으로 표시하지 않는다.

## 숫자 사용자 term 단일 변수 실험

JA1634 원음·원문·모델·CPU F32·threads4와 실행 파일을 고정하고 기본 merge6개에 `7000`만 추가했다. [독립 비교 결과](../../../evidence/ai/model-spikes-2026-10-18/results/forced-align-ja1634-7000-comparison.json)는 source hash·기본 term 보존·추가 term 한 개·숫자를 다시 펼친 단어열 동일성을 확인했다. 새 기본 실행의 public API 정렬값은 기존0.1.0 기본 실행과도 동일했다.

| 실행 label | 분절 | 길이0 | wall 초 | 최대 private bytes | 결과 |
|---|---:|---:|---:|---:|---|
| forced-align-ja1634-a-v11-r1 |31|3|13.1514|4,020,707,328|exit3 / needs_review|
| forced-align-ja1634-b7000-v11-r1 |28|1|13.0521|4,033,155,072|exit3 / needs_review|

숫자 네 분절이 하나로 합쳐지면서 숫자 `0`의 길이0 두 건이 없어졌지만 `は`의2.24→2.24초는 남았다. 단일 대조쌍의 구조 효과이며 속도 우열이나 정확도 승인이 아니다. 두 실행 각각의 `*-evaluation.json`에서도 원음/원문 hash·원시 seconds/IEEE754 비트 보존과 길이0 문제를 독립 확인했다. raw classifier와 repair trace는 공개되지 않아 단독 원인이라고 단정하지 않는다.

두 실제 추론은 동일한 사전 최종 빌드 SHA256 `dca7db3164c40b27380d35435bd4750653dd43a1583623b5eb397f12c706be13`을 사용했다. 이후 SDK의 Unicode term 검증을 upstream L/N 범주에 맞춘 최종 빌드는 `d88a6e5673af582975b76c24f1b9efbb0d0d39d8f9bdd9d9f510e42410ffa7ff`이며 별도 보존한다. 최종 빌드12개 시험과 실제 tokenizer/processor의 비추론 회귀로 기본6개·ASCII7000·prompt 동일성을 확인했으나 최종 빌드의 새 신경망 추론을 수행했다고 표시하지 않는다. 이전 실행 파일을 교체하거나 결과를 최종 빌드로 소급하지 않는다.

root의 Rust spec-maintenance 검사에 정렬 빌드 증거 검사를 추가했다. 초기0.1.0은 executable/archive·선택된 source4개·Cargo.lock의7개 항목만 검증하며 전체 소스 manifest라고 주장하지 않는다. 사전 최종0.1.1은34개, 최종0.1.1은36개 manifest 항목의 SHA와 선언된 크기를 확인했다. 크기가 없는 초기 archive/선택 hash 형식은 SHA만 검사하며 크기 필드가 선언되면 반드시 일치해야 한다. 경로 이탈·손상·필수 크기 누락 회귀와4개 시험/fmt/Clippy가 통과했다. [검사 결과](../../../evidence/player/forced-aligner-builds/integrity-check-2026-10-18.json)는 파일 무결성 범위이며 정렬 정확도나 빌드 재현성의 증명이 아니다.

## 한국어 1초 무음 이동 대조: 실패 관측

root Rust 도구가 KO1883 PCM 앞에0샘플16000개를 추가했다. 원음199680개 샘플이 suffix로 동일함을 WAV 저장 후 재읽기로 확인했다. 새 오디오 SHA256은 `7dc94c2e4522c2dbf0cafc7a19c2adb5ccb3cd16fc070925791b8c0d67665583`다. 원문과 baseline0.1.0 실행 파일·모델·CPU/threads4는 유지했다. `forced-align-ko-shift1000-r1` 실제 추론은30.3371초/최대 private3,968,933,888 bytes, timeout 없이 종료했지만 exit3이다. 원래 구조 통과였던19분절 입력이 이동 후 `의`8.48→8.48초(index14)의 길이0 문제를 반환했다. 첫 단어 시작은1.2→2.24초로 이동했다. 알려진1초 이동이 전체 단어 경계에 정확히 적용된다고 가정하지 않는다. root 독립 감사에서 source/text hash·원시 IEEE754 비트·19개 단어열 동일성을 확인했다. [이동량 비교](../../../evidence/ai/model-spikes-2026-10-18/results/forced-align-ko-shift1000-comparison.json)의38개 경계는 모두0.96초 또는1.04초 이동해 기대1초 대비 절대 오차가 각각40ms였다(평균/최대40ms). 1초가 선언된80ms 간격의 정수배가 아닌 점은 해상도 가설과 맞지만 raw tick/repair 원인 증명은 아니다. 가공 WAV·manifest·평가·비교 출력은527개 snapshot에 편입했다. 새 실패를 clamp/삭제해 성공으로 만들지 않는다.

## 1.04초 이동 후속 대조

선언80ms 간격의13배인1040ms(16640개 zero sample)를 붙이고 원음 suffix 동일성을 재읽기 검증했다. fixture SHA256 `c03057c8cc29605f927c47a416f2afc75115b278131a7ffffc90a0fd8185a518`, 같은 baseline0.1.0 실행 파일·모델·원문·CPU/threads4로 `forced-align-ko-shift1040-r1`을 실행했다.13.4184초/최대 private3,968,917,504 bytes, exit0·19분절 구조 통과였으며 시간 정확도 승인은 아니다.

[독립 이동 비교](../../../evidence/ai/model-spikes-2026-10-18/results/forced-align-ko-shift1040-comparison.json)에서38경계 중36개는1040ms 이동, `다리` 끝과 `의` 시작 두 경계는960ms 이동했다. 기대 대비 평균 절대 오차4.211ms/최대80ms다. 원음·원문 hash와 raw IEEE754 비트 감사도 통과했다. `의`는8.40→8.56초로 길이가160ms이며 baseline80ms와 다르다. 이는 격자 정수배 이동에서도 경계 차이가 남음을 보여준다. 1초 실험의0구간을 시간 격자 위상만으로 설명하거나 후처리 원인을 확정하지 않는다. 두 대조쌍만으로 전체 언어/발화 정확도나 성능 순위를 일반화하지 않는다.536개 snapshot에 새 음성·manifest·실행·비교·감사 결과를 보존했다. 다음 게이트는 공개 head/repair 추적 가능성 검토와 사람의 경계 기준 평가이며 무의미한 epsilon 보정은 적용하지 않는다.

## 공개 보정 함수 비추론 진단

별도 Rust 도구가 고정 upstream의 `fix_timestamp`와 `ForcedAlignProcessor::parse_timestamp`를 직접 호출한다. root도 독립 실행해 [보존 보고서](../../../evidence/player/forced-aligner-repair-root-v010-r1/diagnostic-report.json)를 생성했다. 합성 equal `[80,80]`은 변경 없이 `[80,80]`/길이0, short inversion `[160,80]`은 `[160,160]`/변경 index1/길이0, monotonic `[0,80,160,240]`은 그대로 양의 구간을 반환했다. 입력 f32 비트·결과 ms·소스 hash·실행 파일·source snapshot을 보존했다. 신경망 추론/가중치 로드는0회다. 이 관측은 raw zero 보존과 repair collapse가 모두 가능한 동작임을 보여주며 실제 일본어/한국어 실패의 원인을 귀속하지 않는다. 실제 raw head→repair 관측용 별도 진단 fork가 다음 단계다.

## 실제 raw head 관측과 기존 최종값 동등성

별도 진단 EXE719dc70921c568afd283413966665508e2e7cfb0d80c47143bd98548182c8b89/패치f20b06e5fe49c14020b16240c7d9cf8e8304323a97ddab1f147b9d8be40e6e9c를 사용해 JA1634를 실제 재추론했다. baseline 수치 body는 유지하고 결과 구성 후 실제 local timestamp positions·선택 class·raw f32 ms를 복사했다. fixed ms는 그 raw에 대한 동일 public fix_timestamp의 결정적 replay이며 private LIS flags는 아니다. 실관측 전까지 null이던 기존 보고서는 그대로 두고 새 `forced-observed-ja1634-r1`에만 관측값을 기록한다.

[관측 전용 자원 정책](../../../adrs/player/adr-011-correctness-observation-resource-gate.md)에 따라 실행 중 UI를 보존했다. 원 preflight.ready=false를 숨기지 않았으며 CPU58.1055%/여유35,372,134,400bytes에서4thread CPU/BelowNormal 단일 작업을 실행했다.15.9019초·표본 private4,033,269,760bytes는 실행 이력일 뿐 idle/baseline 성능 비교가 아니다. exit3/needs_review를 유지한다.

root Rust [독립 raw 감사](../../../evidence/ai/model-spikes-2026-10-18/results/forced-observed-ja1634-r1-raw-evaluation.json)가 원음·원문·모델·언어·dtype·threads 동일성과 기존0.1.0 JA1634의 모든 public item/text/시간 IEEE754 bit 동일성을 확인했다. actual selected class→80ms 곱·raw f32 bit·fixed ms→최종 seconds bit도 대조했다.

| word index | 원문 | actual raw ms | fixed ms | 이 입력에서 확인된 경로 |
|---|---|---|---|---|
|2|は|2240→2240|2240→2240|원시 예측부터 같은 값이며0이 보존됨|
|5|0|2720→2800|2800→2800|원시 양의 구간이 보정 후0으로 축소됨|
|7|0|3040→3040|3040→3040|원시 예측부터 같은 값이며0이 보존됨|

수치가 변경된 timestamp index는10과12다. index12(다음 숫자 시작)도2720→2800으로 바뀌었지만 해당 word는0구간이 아니다. 따라서 이 JA1634 표본의3개0구간을 같은 원인으로 묶지 않는다. raw 선택 class 전체 logits/top2·private LIS 분기·사람의 경계 정답은 아직 관측하지 않았다. 이 결과는 다른 발화의 실패 원인이나 해결책을 일반화하지 않으며 시간값을 늘리거나 삭제해 통과시키지 않는다. 총 실제 정렬은 기존8회와 신규 raw 진단1회이며 구조 통과3/실패6, 품질 승인은0이다.

## JA1516 실제 raw 후속 대조

동일한 동결 진단 EXE와 원음/원문으로 `forced-observed-ja1516-r1`을 실행했다. CPU26.0742%·여유33,862,815,744bytes에서 열린player3개를 보존했다.15.1598초·private4,091,006,976bytes는 관측 전용 실행 이력이며 성능 비교에서 제외한다. exit3/timeout없음/품질false다.

[독립 감사](../../../evidence/ai/model-spikes-2026-10-18/results/forced-observed-ja1516-r1-raw-evaluation.json)에서 baseline JA1516과 모든 public item/f64 bit가 같았고 actual class/ms/f32·fixed/final bit 매핑이 통과했다. timestamp 수치 변경 index는 없었다. index9 `を`는raw/fixed7840→7840ms, index10 `もっ`는raw/fixed8000→8000ms다. 두0구간은 이 실행에서는 repair가 만든 것이 아니라 원시 예측의 동일값이 그대로 남았다. JA1634와 JA1516의 기존0구간5개 중4개는raw equal,1개는positive raw가repair로collapse했다. 이 표본 범위의 분류이며 언어 전체·사람 경계 정확도나 해결책을 일반화하지 않는다. 기존8회와raw2회 총10회 구조3통과/7실패이며 source/hash·560개snapshot을 보존했다.

## 실행·배포 조건

연구 세션의 다운로드/빌드 금지는 담당 분리 범위였고 root의 승인된 실행 작업 전체 금지가 아니다. root 한 곳이 실제 실행을 맡는다. 로컬 비배포 스파이크와 최종 제품 배포의 고지 패키징 게이트를 구분한다. Lindera/IPADIC COPYING, KO dictionary의 원본 연결, 사용자 정의 JP merge와 공식 nagisa의 분절 차이를 보존한다. 공개 source만으로 모든 부속 자산의 최종 배포 조건이 해소됐다고 주장하지 않는다.
