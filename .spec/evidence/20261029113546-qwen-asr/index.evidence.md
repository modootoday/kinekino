---
id: evidence:kinekino/20261029113546
title: Qwen ASR Rust CPU 실제 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Qwen ASR Rust CPU 실제 검증

2026-10-18. 고정 공식 모델 취득과 입력 단계 검증 결과다. 실제 인식 품질 통과로 해석하지 않는다.

## 고정 입력과 엔진

공식 `Qwen/Qwen3-ASR-0.6B` revision `5eb144179a02acc5e5ba31e748d22b0cf3e303b0`의 1,876,091,704-byte safetensors를 스트리밍 취득하고 LFS SHA256 `79d6cbd4c98c7bbffe9db2edac07f56cd6637d0d5944b27f6c2b8353840323ea`를 검증했다. 작은 설정 파일 7개도 같은 revision에서 취득했다. 다운로드 예외 이유는 [ADR010](../../../adrs/player/adr-010-measured-ai-candidates.md)에 남겼다.

Rust `qwen3-asr-rs` commit `c5ef09646af6278d2ba8b8ceaf543ffb32d1a5dc` 기반의 CPU 로컬 어댑터를 별도 `tools/qwen-asr-spike`에 두었다. Hub 자동 다운로드·삭제 경로와 기본 Metal을 사용하지 않는다. GGUF·word timestamps·ForcedAligner 구현은 포함하지 않는다. 빌드의 MSVC LNK4098 경고는 남아 있다.

공식 vocab/merges/tokenizer_config에서 파생 tokenizer를 만들고 vocab 151,643개·added token 62개의 ID, added token encode/decode, 한국어·일본어·영어·공백 roundtrip을 검증했다. tokenizer SHA256은 `e90b3d2464f5b00c6043184aebfa0b41c1fe06ce2e75c0508a3610954776968a`다. 이 검사는 공식 Transformers 전체 토큰 시퀀스·mel 동등성을 증명하지 않는다.

## 최초 실제 실행 실패

root가 기존 FLEURS 한국어 PCM16 음성에 대해 언어 Korean·4 threads·128 max tokens로 `qwen-asr-ko-r1`을 계측했다. 프로세스는 약 0.118초 만에 exit2, `source preprocessing sample rate mismatch`를 반환했다. 가중치 로드·음성 인식 이전 실패다. 모델 추론 속도나 메모리 수치로 쓰지 않는다.

공식 preprocessor_config에는 `sampling_rate` 키가 없다. 어댑터의 필수 키 가정이 잘못됐다. 원본 설정을 보존하고 chunk_length30/n_samples480000/feature_size128/hop_length160/n_fft400 등 고정 설정과 16kHz 입력 계약을 검증하도록 수정 담당 세션에 인계했다. 실패를 모델 정확도 저하나 모델 비호환으로 단정하지 않는다.

원시 측정·표준 출력·표준 오류는 각각 `qwen-asr-ko-r1.json`, `.stdout`, `.stderr`로 evidence snapshot의 results에 보존한다. 수정된 실행 파일 SHA 및 새 label을 사용한 재실행, 한국어·일본어·영어 CER/원문 보존·최대 메모리 비교는 남은 작업이다.

수정본은 다섯 고정 설정을 검사하고 `480000/30=16000`을 sample rate 근거로 기록한다. 키가 명시된 경우 16000과 일치해야 한다. root가 release/no-default-features/locked 상태로 실제 tokenizer를 포함한 시험10개 통과를 재확인했다. 고정 설정 누락·변경·명시 sample rate 충돌 회귀 시험을 포함한다. 수정 EXE SHA256은 `fc9246b1a8bbc477e0cccf858fa24ac38e9a6e599be5b6c0119d672800c960fe`다. MSVC LNK4098은 여전히 관측됐고 경고를 숨기지 않았다. 별도 UI 세션의 실제 창 검증 중에는 resource preflight 정책에 따라 모델 추론을 겹치지 않는다.

재시도 label `qwen-asr-ko-r2`와 `qwen-asr-ko-r3`는 소유 UI 데모 player PID60188이 실행 중이어서 ready=false로 차단됐다. 각 preflight JSON만 생성됐고 모델 worker는 시작되지 않았다. 이 두 대기를 모델 실행 실패나 인식 결과로 집계하지 않는다. UI 창 종료가 확인되면 새 label로 실행한다.

## 평가·재현 경로

Rust model-spike의 `qwen-evaluate LABEL`은 프로세스 성공과 JSON 응답 성공을 구분한다. 성공 결과의 입력 파일 SHA를 다시 계산해 어댑터 기록과 대조하고, 정확히 같은 SHA를 가진 한국어/일본어 기준 발화에만 정규화 CER을 계산한다. 번역 자막이나 다른 파일을 정답으로 대체하지 않는다. 정답이 없는 입력에는 CER을 null로 남긴다. 기존 평가 파일을 덮어쓰지 않는다. 최초 실패에도 평가 JSON을 생성해 모델 품질 성공 분모에 섞지 않았다.

`report`는 측정 결과뿐 아니라 독립 Qwen 어댑터 Rust 소스·시험·Cargo manifest/lock·tokenizer 준비 기록을 evidence에 복사하고 해시를 대조한다. 엔진 EXE는 복사하지 않고 별도 SHA·크기를 기록한다. 가중치와 대용량 vocab는 evidence로 중복 복사하지 않는다. 이 소스 snapshot은 제품 런타임 채택을 뜻하지 않는다.

## 실제 추론 5회와 출력 구분자 문제

실행 전체 분모는 [고정 baseline cohort](../../../evidence/ai/model-spikes-2026-10-18/results/qwen-cohort-baseline-v1.json)로 재구성했다. root Rust 명령 `qwen-cohort`는 preflight 대기2회·프로세스 시작6회(준비 실패1회/완료5회)를 분리하고 완료한5회의 동일 EXE SHA 및 각 평가 파일 SHA를 확인했다. 기존 cohort를 덮어쓰지 않는다.

사용자의 UI 테스트 창 종료 확인 후 같은 수정 EXE `fc9246...`·CPU·4 threads·128 max tokens로 다음을 하나씩 실행했다. 모두 exit0, timeout 없음이다. 한국어 첫 준비 실패1회, preflight 대기2회, 실제 추론5회를 구분한다.

| 입력·측정 label | 입력 길이 | 전체 wall 초 | 최대 표본 private bytes | 발화 CER(편집/기준 문자) |
|---|---:|---:|---:|---:|
| 한국어 / qwen-asr-ko-r4 | 12.48초 | 39.6929 | 3,873,812,480 | 5.66% (3/53) |
| 일본어1516 / qwen-asr-ja1516-r1 | 16.74초 | 24.2887 | 3,837,399,040 | 24.59% (15/61) |
| 일본어1634 / qwen-asr-ja1634-r1 | 12.12초 | 24.4842 | 3,867,451,392 | 0% (0/43) |
| 일본어1578 / qwen-asr-ja1578-r1 | 9초 | 16.6720 | 3,887,386,624 | 12% (3/25) |
| Sintel 영어 / qwen-asr-en-r1 | 25초 | 29.0770 | 3,880,382,464 | 정답 발화 미확보 / null |

한국어는 `밑→및` 오인식과 `15미터→15m` 표기 차이가 남았다. 2011/2017년은 이 입력에서 보존했다. 일본어1516은 시작 구절과 고유명사 오인식이 크고 1578은 ラッカ→落下 오인식이다. 1634의 단일 입력 0%를 일본어 전체 정확도로 일반화하지 않는다. 같은 실제 일본어 입력의 Whisper tiny/base 결과와 비교 가능한 관측이지만 다른 엔진·모델·반복 횟수이며 통제된 성능 순위가 아니다. 사전 CPU 부하와 빌드 병행 상태도 달라 단일 wall 값을 보편 속도로 쓰지 않는다.

영어는 `This blade has a dark past`를 출력해 tiny의 dark past 오인식과 달랐다. 전체 출력은 해당 측정의 원시 JSON에 보존했다. 원음의 전사 정답을 수작업 확정하지 않았으므로 번역 자막으로 CER을 만들지 않았고 숫자를 null로 유지했다. 오디오 청취·모든 단어의 의미 검토 완료를 주장하지 않는다.

다섯 출력 모두 upstream의 언어 강제 decode 경로가 선두 `<asr_text>`를 본문에 노출했다. 공식 tokenizer 설정에서 이 토큰은 special=false이므로 source 속성을 임의로 변경하지 않았다. evaluator v2는 정확한 선두 구분자1개만 별도 분리하고 원시 transcript·raw_character_edits·speech_text·character_edits를 함께 남긴다. 본문 중간의 문자열이나 다른 토큰을 지우지 않는다. 표의 CER은 구분자 분리 후 값이다. 한국어 raw10/53과 발화3/53의 차이를 정확도 개선으로 표현하지 않는다. 이전 r4 평가 v1은 보존했고 v2는 별도 파일이다.

독립 스파이크 어댑터0.1.1에 경계 처리와 원시 출력 보존을 반영했다. 이5회는 구분자가 노출되는0.1.0 EXE로 실행한 증거이며 재실행 성능값으로 대체하지 않는다. timestamp·화자·양자화·제품 GUI/CLI ASR 지원은 미구현 상태다.

## 출력 수정본 검증과 빌드 증거

0.1.1 EXE SHA256은 `d5931cc325f4353248865481c3c8721369fd1c4495ff7b0ed49b2ef58a4b258d`다. 정확한 선두 구분자1개만 분리하고 본문·공백은 보존한다. 본문 내부/알 수 없는 내부 토큰과 빈 본문은 requires_review=true, evaluation_text=null로 표시한다. tokenizer 원본 속성을 바꾸지 않는다. root가 단위10개·통합8개, 총18개 시험 통과를 재확인했다. LNK4098은 계속 관측됐다.

root와 SDK 담당이 `normalize-output`을 저장된5개 실제 결과에 적용했다. root의 독립 Rust `qwen-normalized-audit`가 원본 report SHA·raw 문자열/SHA·upstream_text·입력/모델/언어·기존 load/inference 시간의 보존과 발화의 평가기 일치를 모두 확인했다. [감사 결과](../../../evidence/ai/model-spikes-2026-10-18/results/qwen-normalized-audit-v1.json)를 보존한다. 이 재평가는 모델 추론이나 새 실행 속도 측정이 아니다.

[버전별 빌드 증거](../../../evidence/player/qwen-asr-builds/README.md)는 실제 측정0.1.0 EXE와 고정 lock, 현재0.1.1 source/EXE·원본 엔진 archive·파일별 해시를 분리한다. 0.1.0 source는 관측한 변경분에서 복원한 자료이며 당시 source 파일별 해시가 없어 역사적 byte identity를 증명하지 못한다. 이 한계를 명시하고 실제 측정 EXE 자체는 그대로 보존했다. 최신 source snapshot을 이전 실행 당시 source라고 부르지 않는다.

root가 `.spec` 검사 도구에 버전별 build manifest 검증을 추가해2개 manifest의 실제 파일51개에 크기·SHA를 재대조했다. 경로 탈출·손상 파일 거절 시험을 포함한 도구4개 시험·fmt·Clippy와 실제 검사가 통과했다. `files[].path`는 source/ 상대 경로이고 executable.name은 원본 basename 정보이며 저장된 EXE 이름은 고정 `qwen-asr-spike.exe`라는 [저장 계약](../../../evidence/player/qwen-asr-builds/storage-contract-supplement.md)을 적용했다. 기존 manifest를 이 해석에 맞춰 덮어쓰지 않았다. 파일 무결성 검사는0.1.0 복원 source의 역사적 identity를 추가 증명하지 않는다.
