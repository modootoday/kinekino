---
id: evidence:kinekino/20261029113701
title: 독립 애니메이션 ASR와 음악 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 독립 애니메이션 ASR와 음악 관측

2026-10-18. [고정 Sprite Fright 입력](animation-independent-input.md) 4개에 Qwen ONNX 혼합 INT4 ASR와 YAMNet을 각각 실행했다. 신규 신경망 실행8회이며 모든 worker가 exit0, timeout=false로 종료했다. ASR EOS도4개 모두 도달했다. 실행 성공은 내용 정확도 또는 자동 자막·skip 승인과 다르다.

## ASR 결과

공식 제작사 영문 자막을 proxy reference로 사용했다. Unicode 영숫자·소문자 정규화로 공백·구두점을 제외하되 원문 출력의 프로토콜을 삭제하거나 교정하지 않았다. 사람의 음성 전사 gold는 없으므로 표는 실제 ASR 오류율 확정치가 아니다.

| 클립 | proxy 편집/문자 수 | proxy CER | 추론 ms | 표본 최대 private bytes |
|---|---:|---:|---:|---:|
| latin-17 | 15/60 | 25% | 1749 | 2820710400 |
| dialogue-28 | 4/197 | 2.03% | 4878 | 3443437568 |
| tea-205500 | 12/125 | 9.6% | 3078 | 3029827584 |
| caption-gap-0 | 참조 없음 | 미산출 | 1357 | 2752901120 |

첫 클립의 라틴어 명칭은 출력에서 다른 영어 이름으로 바뀌었다. tea 클립의 마지막 나열도 배포 자막과 크게 달랐다. 마지막 자막 공백 클립은 `The sky is blue.`를 출력했다. 원문을 보존하며 자막이 없다는 사실만으로 환각을 확정하지 않는다. 우선 청취 검토 대상으로 분류한다. 실제 사람 전사·여러 작품·일본어 애니 발화 게이트는 남아 있다.

ASR worker SHA256은 `58598154fb2d389b65c0a702eb81256de6c4958ff51325bea6ed7380d8797f36`, 모델 revision은 `4fc24a1402e74db89c4d2ef256875e71680128c4`다. 정밀도는 FP32 encoder + INT4 decoder + FP16 embedding 저장/F32 캐시이며 전체4bit 모델이 아니다. CPU4 intra/1 inter, English 강제 언어, max128tokens로 실행했다. 원본/decoder 변환 계보와 재배포 조건 미해소 상태는 유지한다.

실제4회 orchestrator는 `b71b7098341261b5d6fb71c3e443fe3406948a221f4335de94f7ce14b9126717`이며 컴파일 입력94파일과 EXE1개, 총95개를 선보존했다. label은 `qwen-onnx-sprite-{latin,dialogue,tea,gap}-int4-r1`이다. 각 json/stdout/stderr/preflight와 `-proxy-evaluation.json`을 보존했다. 평가기는 실제 음원 해시·샘플 수·언어·variant·revision·worker·원본 manifest 연결을 확인한다. gap 평가만 후속 동결 build로 수행했고 새 추론은 발생하지 않았다.

이 코호트는 같은 입력 FP32 대조가 없으므로 양자화에 의한 변화로 해석하지 않는다. 과거 ONNX5회+같은ONNX4쌍8회와 별도의 신규4회이며 17개의 독립 gold 입력이 아니다.

## 음악과 발화 점수

YAMNet 고정 revision `ac2ca3bd45d12ec1f19f1144205ea529b4e9dedf`, 모델 SHA256 `1510041dce24a2e9e84ec546807ac408ae496da6d1ed41bc3ccba649623f8e19` 및 class map SHA256 `cdf24d193e196d9e95912a2667051ae203e92a2ba09449218ccb40ef787c6df2`를 검증한 뒤 실행했다. 신규 adapter는 전체 frame×521 score, 클래스 이름, mean score, top8을 모두 보존하고 비어 있거나 비유한·범위 밖인 점수 행렬을 거절한다.

| 클립 | score 행렬 | Music 평균 점수 | Speech 평균 점수 | 추론 초 |
|---|---|---:|---:|---:|
| latin-17 | 17×521 | 0.58304 | 0.27040 | 0.01413 |
| dialogue-28 | 33×521 | 0.35758 | 0.40228 | 0.02328 |
| tea-205500 | 30×521 | 0.40078 | 0.48270 | 0.02184 |
| caption-gap-0 | 24×521 | 0.42381 | 0.00376 | 0.01829 |

평균 음악 점수와 발화 점수는 상호배타 분류나 보정된 정확도·OP/ED 확률이 아니다. gap의 Speech는 top8 밖이며 전체 mean_scores에 보존했다. 정확한 음악/발화 구간과 오탐·누락은 사람 라벨이 필요하다. 낮은 Speech 점수를 근거로 ASR 출력을 자동 삭제하지 않는다. 단일 작품의 클립4개로 반복 OP/ED 탐지를 검증했다고 표현하지 않는다.

음악 worker/orchestrator SHA256은 `deeeb9ac3f2ebacf07c6b01b9e080fe8dd4f5b045f64c35720f9149a3d4fb105`, 선보존 컴파일 입력95파일과 EXE1개, 총96개다. label은 `yamnet-sprite-{latin,dialogue,tea,gap}-r1`이다. 실제4회 모두 inferred 출력과 음원/모델 해시를 확인했다. 약0.24초 이하 짧은 프로세스에서 100ms 메모리 표본은2개뿐이므로 최대 메모리·속도 최적화 증거로 사용하지 않는다.

## 검사와 잔여 범위

`summarize-animation`을 실제 재생성 실행해 고정 클립·각 stdout/측정 해시·기존 proxy 편집 수를 대조하고 전체 YAMNet frame score에서 클래스 평균521개를 재계산했다. 4개 모두 원시 값과 허용오차1e-6 안에서 일치했다. 집계는 새 추론0회이며 `runtime/ai-spike/results/sprite-asr-music-summary-v1.json`에 보존했다. 집계 EXE `3379bca491dc431730f7da4f6fad3955f67d08bd42685e8e144a5a0946ff4491`의 컴파일 입력96파일과 EXE1개, 총97개도 선보존했다. 실제 실행95/96개 bundle과 집계97개 bundle을 구별한다.

담당 standalone 도구는 fmt check, test37개, clippy all-targets -D warnings, locked offline build를 통과했다. ASR4회와 음악4회는 순차로 실행하고 앱·캐시·서비스·VM·DPI를 변경하지 않았다. 다른 player가 실행 중인 correctness 관측이며 성능 비교 eligible=false다. 제품 SDK의 AI job 및 자동 적용은 아직 지원 상태로 승격하지 않았다.

다음은 사람 청취·전사/음악 라벨, 독립 언어/작품 평가, 필요 시 같은 입력의 정밀도 대조, OP/ED 반복 구간 및 취소 정책과 공통 SDK job 연결이다. [잔여 게이트](../../../plans/player/ai-quality-gates.md)를 따른다.
