---
id: evidence:kinekino/20261029113621
title: 모델 검증 상태 정본
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 모델 검증 상태 정본

2026-10-19 정정: 아래 ‘품질 승인0개’는 사용자 승인 대기나 사용자 미응답을 뜻하지 않는다. 실제 결과 비교·판정 기준·판정 저장을 통합한 검토 패키지 제공이 미완료다. 자동 검증과 검토 준비는 개발 담당의 잔여 작업이며, 사용자가 확인할 절차와 기능별 채택 범위는 [모델 결과 검토 계약](../../player/application/model-quality-review.md)에 정의한다.

후속 [Gemma 경계 수정판 실제 재검증](generation-gemma270-context-boundary-fixed.md)에서 같은2047입력은completion1/total2048,2048·2049입력은HTTP400을 확인했다. 원본의total2049문제는 해당API 재실행에서 해소됐으며 내부trace/full gate·제작품질 승인은 여전히 별도다. 품질 검토 [Pages 저장 및 표시 한계](quality-review-page.md)도 구분한다.

원본 ForcedAligner의 [전체 strict 로드](forced-head-full-load-actual.md)에서 실제708tensor·917728896parameter·BF16 저장→CPU F32 로드·peak private7435804672B를 확인했다. 후속 [동일 mel head 실제 대조](forced-head-logits-actual.md)는 KO1·JP2 원본3회/native3회,780000 logits 허용오차 통과·raw 클래스156개 일치다. [원본 repair·최종 시간 대조](forced-timestamp-functions-actual.md)도78단어 모두 일치했지만 일본어 길이0 단어5개는 원본에도 존재한다. 모델명0.6B를 전체 상주 크기로 간주하지 않는다. 사람 품질은 미승인이고 제품 native backend 메모리와 별도다.

기준일 2026-10-19. 실행 이력과 품질 승인을 분리한다. 아래의 **실행 완료**는 제한된 입력에서 실제 모델 추론과 출력 확인을 끝냈다는 의미다. **제품 품질 승인·자동 적용 승인을 받은 모델은 아직 없다.** 모든 실측은 현재 호스트의 CPU 경로이며 GPU·노트북 전력 조건을 검증한 결과가 아니다. 후보 목록 수, 변형 수, 실행 횟수를 모델 수로 합산하지 않는다.

## 분류 요약

보조 참조 실행: [nagisa·soynlp 실제 분절 대조](forced-segmentation-actual.md)에서 nagisa 기본 모델 tagging2회·soynlp tokenize1회를 실행했다. KO19개 일치, JP2개 경계 불일치이며 실제 추론·출력 확인23묶음에 새 기본 모델로 합산하지 않는다. 원본 processor 전체·품질 승인은 별도 미완료다.

JER 후속: [원본 함수 실제 대조](diarization-jer-functions-actual.md)로 기존 앞4개 출력의 RTTM binary64 경계 오류를 수정했다. 새 평가6개는 원본과 frame counts 및 JER가 일치했고 [교차 행렬78원소](diarization-jer-matrix-actual.md)도 정확히 일치했다. 아래 과거 JER 수치는 당시 동결 보고서 값이며 새 정본 표와 구분한다. 모델 품질 승인0개는 유지하고 전체CLI 완료로 표현하지 않는다.

| 구분 | 현재 상태 |
|---|---|
| 실제 추론·출력 확인 완료 | 아래23개 역할/모델 묶음. OCR는 Rust 후처리·자동crop 인식의 제한 fixture 연결까지이며 전체 품질 승인이 아님. 양자화·실행 경로·반복 횟수는 중복 합산하지 않음 |
| 제품 품질·자동 적용 승인 완료 | 0개. 구조·수치 대조 통과와 기능 품질 승인은 구별 |
| 실제 추론 전 | 별도 후보 표의 조사·접근·변환·라이선스 게이트가 남은 항목 |
| 화자 분리 후속 실험 | 같은 모델의 adapter 개선. AMI60초 DER FP32 77.54→69.25%, INT8 77.89→67.98%; 여전히 품질 미달. [실험 정본](diarization-local-anchor.md) |
| 평가 계산 검증 완료 | 화자6개 exact 출력에서 원본 md-eval DER 대조 통과·components차이0us. JER·모델 품질 승인은 별개. [실제 대조](diarization-der-reference-actual.md) |
| 전처리 후속 실험 | WeSpeaker 원본의PCM32768배율을 별도role에 수정·FP32/INT8 실제2회. 기존local-anchor와DER/JER같음, 개선미관측. [실험 정본](diarization-pcm-scale.md) |
| 시간 분리 검증 | 같은AMI의60~120초 FP32/INT8 실제2회. DER15.04/17.14%지만JER52.10/51.72%, 소수발화자누락. 원본DER대조통과·품질미승인. [정본](diarization-temporal-holdout.md) |
| 누락 단계 진단 | 같은구간 FP32/INT8 원시frame진단2회·이전DER/JER재현. FP32는첫20초1178frame전부NO_SPEAKER, 정답overlap창도단일화자. 군집화만으로설명불가. [진단](diarization-segmentation-diagnostic.md) |

## 실제 추론을 시도하고 출력을 확인한 모델

| 역할 | 모델·실행 변형 | 실행 검증 | 관측 결과와 남은 품질 게이트 | 근거 |
|---|---|---|---|---|
| 교정·제작 | unsloth/gemma-3-270m-it-GGUF Q4_K_M | 실제3요청 완료·품질 미달 | 교정 반복/256token JSON잘림. 구조2/3이나 전체7cue 선택·혼합언어/근거없는 highlight 이유. private약1.23GB. 별도context 수정판은2047→total2048,2048·2049→HTTP400 재검증 완료·내부trace 미완료 | [실제3요청](generation-gemma270-structured-smoke.md)·[수정 경계](generation-gemma270-context-boundary-fixed.md) |
| ASR | openai/whisper-tiny FP32 | 완료 | 한국어 CER 20.75%; 영어 오인식. 자동 자막 품질 미승인 | [음성](audio.md) |
| ASR | lmz/candle-whisper tiny Q8 | 완료 | 한국어 CER 20.75%; 일본어 3개 57.38/11.63/40%. 동일 영어 입력에서 FP32보다 느린 관측 | [음성](audio.md) |
| ASR | openai/whisper-base FP32 | 완료 | 한국어 CER 9.43%, 연도 오인식·timestamp 범위 오류. 일본어 36.07/9.30/20%. 패딩 재처리 버그 수정 후 재실행 | [음성](audio.md) |
| ASR | Qwen/Qwen3-ASR-0.6B 원본F32·community ONNX FP32/혼합INT4 | 완료·품질 미승인 | 원본F32 5회·기존INT4 5회·같은ONNX4쌍8회 및 독립Sprite INT4 4회 완료. Sprite 자막proxy CER25/2.03/9.6%, 자막공백에도 문장출력·사람검토필요. 양자화원인단정불가 | [원본](qwen-asr.md)·[INT4](qwen-onnx-actual.md)·[4쌍](qwen-onnx-paired.md)·[독립애니](animation-asr-music.md) |
| 강제 정렬 | Qwen/Qwen3-ForcedAligner-0.6B F32 | 완료·오류 단계 관측 | 실제17회, 구조3통과/14실패. 새default와과거head·같은단위control동일. 원본nagisa분절A/B에서도zero5개(raw equal4/repair collapse1)유지. 기존118row·공식NumPy mel529152원소 및 encoder114공통stage 고정atol+rtol 대조통과. 사람 경계 미검증 | [정렬](forced-alignment.md)·[분절 실제 A/B](forced-segmentation-ab-actual.md)·[head 관측](forced-head-logits.md)·[NumPy 실제](forced-mel-numpy-actual.md)·[encoder 실제](forced-encoder-stage-actual.md) |
| VAD | onnx-community/silero-vad | 완료 | 실제 PCM·상태 유지·출력 확인. 사람 기준 검출률·오탐률 미검증 | [음성](audio.md) |
| 음악 | zeropointnine/yamnet-onnx | 완료·품질 미승인 | Sintel에 더해 독립Sprite4클립 실제실행. 전체frame×521점수 보존·대화와음악점수동시높음. 여러회차 OP/ED 정확도·사람라벨 미검증 | [음성](audio.md)·[독립애니](animation-asr-music.md) |
| 화자 분할 | onnx-community/pyannote-segmentation-3.0 FP32/INT8 | 완료·prototype 품질 미달 | 기존 Sintel 군집5/4, AMI60초 human reference3명 대비8/6군집. 초기 DER77.54/77.89%; 후속 포함6쌍 원본DER 대조 통과. JER 원본 대조·모델 품질은 미승인 | [실제 정답 대조](diarization-quality.md)·[평가기 대조](diarization-der-reference-actual.md) |
| 화자 임베딩 | onnx-community/wespeaker-voxceleb-resnet34-LM | 완료·전체 pipeline 미승인 | 같은AMI prototype의 JER77.79/82.88%. 개별 embedding만의 오류율은 아니며 전체 전처리 parity·개선 필요 | [화자 대조](diarization-quality.md) |
| 한국어·영어 OCR | monkt/paddleocr-onnx / korean_PP-OCRv5_mobile_rec | 제한 입력 실행 완료 | 렌더 한국어 자동crop 문구일치·원본영어크레딧30crop을한session에서실제인식. 인간gold/corpus/읽기순서·전체품질미승인 | [자동연결](ocr-postprocess.md)·[자연입력](ocr-natural-credits.md) |
| 일본어 OCR | PaddlePaddle/PP-OCRv6_small_rec_onnx | 완료 | BGR 전처리 수정 후 좁은 ROI 인식. 검출기·연속 영상 미검증 | [비전](vision.md) |
| OCR 검출 | PaddlePaddle/PP-OCRv6_small_det_onnx FP32 | map·Rust후처리·제한연결 완료 | 렌더positive1/원본negative0와자연크레딧30후보/crop 실제관측. 자연번인·후처리수치parity·corpus품질미승인 | [검출](ocr-detection.md)·[자연입력](ocr-natural-credits.md) |
| 피사체 탐지 | onnx-community/rfdetr_nano-ONNX | 완료 | 한 프레임 사람 1개, score 0.95561. 추적·인물 신원 검증 아님 | [비전](vision.md) |
| 클릭 마스크 | lmz/candle-sam MobileSAM | 완료·품질 미달 | 얼굴·피부·손 중심 마스크, 의상 누락. 전체 피사체 분리 미승인 | [비전](vision.md) |
| 상대 깊이 | onnx-community/depth-anything-v2-small-ONNX FP32/Q8 | 완료 | 동일 입력 각 3회, depth→2.5D GLB 구조·렌더 확인. Q8 메모리 증가 관측. 절대 깊이·완전 3D 정확도 미검증 | [비전](vision.md) |
| 교정·제작 | unsloth/Qwen3-0.6B-GGUF Q4_K_M | 완료·품질 미달 | 후속 고정profile constrained3건 구조3/3·단어보존 확인. 카드 반복·부자연스러운 문장·전체7cue 하이라이트가 남아 자동 적용 불가 | [텍스트](text.md)·[후속3건](generation-qwen06-structured-smoke.md) |
| 교정·제작 | unsloth/Qwen3.5-0.8B-GGUF Q4_K_M | 완료·품질 미달 | 타입 오류·cue ID 누락·환각. 텍스트 경로만 실행 | [텍스트](text.md) |
| 교정·제작 | unsloth/Qwen3.5-2B-GGUF Q4_K_M | 완료·구조 개선 확인 | 기본 3개 요청과 구조화 A/B 18개 요청. 제한 디코딩 strict JSON 9/9지만 누락·환각은 남음 | [구조화](structured-text.md) |
| 교정·제작 | bartowski/Llama-3.2-1B-Instruct-GGUF Q4_K_M | 실제3요청 완료·품질 미달 | 교정은256tokens/length로 잘린 JSON·반복. 카드/하이라이트 구조2/3통과지만 카드 내용은cue1·4/참조ID6·7로 불일치. private약2.12GB·사람/다른자료 미검증 | [실제3요청](generation-llama1b-structured-smoke.md) |
| 의미 검색 | sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2 ONNX FP32/AVX2·원본 Candle F32 | 완료·품질 미승인 | 실제33입력 ONNX 각1회·원본2회. 임시 MRR 0.55/0.5533, hit@3 0.8. 원본 대조 최대성분차2.38e-7. 사람 정답·no-match 기준 필요 | [ONNX](embedding.md)·[원본 대조](embedding-reference.md) |
| 의미 검색 | intfloat/multilingual-e5-small FP32 ONNX | 완료·품질 미승인 | 같은33원문에 역할별 prefix를 적용해1회 실행. 임시MRR0.825/hit@3 0.8, 고마움은8위로 악화·무관질의top0.789. 사람 기준·no-match 검증 필요 | [E5](retrieval-e5.md) |
| 의미 검색 | onnx-community/embeddinggemma-2-ONNX text FP32/q4 | 완료·품질 미승인 | 같은33입력·동일worker FP32/q4 대조. 임시MRR0.82857/0.825·hit@3둘다0.8·thanks7/8위. 표본private약873/448MB·벡터cosine평균0.983. 사람정답·원본framework·no-match 미검증 | [실제 실행](embeddinggemma2-fp32.md)·[양자화](embeddinggemma2-quantized.md) |

CER는 각 제한된 언어 입력의 문자 오류율이며 서로 다른 언어·자료의 종합 순위가 아니다. 의미 검색의 의역 정답은 assistant 임시 지정으로 인간 gold가 아니다. 표의 실행 완료를 제품 SDK/GUI 기능 통합 완료로 해석하지 않는다. 과거 실행 일부의 실행 파일·소스 대응은 [독립 보존 감사](provenance-audit.md)의 제한이 남아 있다.

## 초기 준비 실패 이력

| 역할 | 모델 | 실제 상태와 다음 게이트 |
|---|---|---|
| 텍스트 생성 | unsloth/gemma-3-270m-it-GGUF Q4_K_M | 기존59ee 엔진의context2048 준비 실패/child exit1·추론요청0을 보존. 후속 e862 엔진에서 실제3요청/경계를 실행했으므로 현재 미실행 모델로 중복 분류하지 않는다. [초기 실패](generation-gemma270-loader-failure.md)·[후속 실제](generation-gemma270-structured-smoke.md) |

## 조사·준비만 진행했으며 실제 추론하지 않은 후보

| 역할 | 모델·경로 | 현재 상태·다음 게이트 |
|---|---|---|
| 의미 검색 대안 | EmbeddingGemma-300m | 원본·변환·pooling·라이선스 조사. 해당 모델 실제 graph 로드·실행 전 |
| 화자 분리 | pyannote/speaker-diarization-community-1 | gated 접근·조건 수락 및 전체 VBx 경로 미검증 |
| 음악 | Xenova/ast-finetuned-audioset-10-10-0.4593 | 대안 조사만 완료 |
| 영상 추적 | facebook/EdgeTAM | Rust 시간 메모리·변환 parity 경로 미검증 |
| 깊이 | depth-anything/DA3-SMALL | 메타데이터·라이선스 조사, 추론 전 |
| 깊이 변환 | lmz/candle-dino-v2 / jeroenvlek/depth-anything-v2-safetensors | 변환 provenance·비상업 라이선스 검토 게이트 |
| 완전 3D | stabilityai/TripoSR | 검증된 Rust 실행 경로 없음, 추론 전 |
| 완전 3D | tencent/Hunyuan3D-2mini | 지역 라이선스 제한으로 취득·실행하지 않음 |
| 텍스트 원본·확대 | Qwen/Qwen3.5-0.8B 원본, Qwen3-1.7B/4B 후보 | GGUF 실측과 별개. 해당 원본·확대 모델 추론 전 |
| 경량 텍스트 생성 대안 | google/gemma-4-E2B-it | 공식 후보 확인. E2B 명칭과 전체 가중치/상주 메모리를 구분하며 양자화·Rust 엔진·modalities·배포 검토 후 실제 비교 예정 |

후보 상세는 [역할 대응표](../../player/infrastructure/inference/spikes/coverage.md), 새 검색 후보 근거는 [retrieval 연구](../../player/infrastructure/inference/research-followups/semantic-retrieval.md)와 [Gemma·Llama·Qwen 후보 검토](../../player/infrastructure/inference/research-followups/lightweight-model-families.md)를 따른다. 모델이 아닌 Chromaprint·자막 변환·프레임 저장·GLB 패키징·수동 카드/숏폼 출력은 이 표의 모델 수에 넣지 않는다.

## 진행 순서

후속 원본 MiniLM Candle 추론2회·독립 대조를 완료했다. 초회 일반 측정의 자원 조건 차이를 보존하고, 별도 관측 게이트를 코드로 강제한 후 재실행했다. 같은33입력의 최대 벡터 성분차 약2.38e-7, 의역 관련순위1/1/3/3/12가 일치했다. 기존 MiniLM 모델 묶음의 별도 실행 경로이며 모델 수를 새로 늘리지 않는다. [자원 게이트 차이·토큰 재현 한계와 상세 결과](embedding-reference.md)를 함께 따른다.

1. 완료된33입력 MiniLM 참조 대조와 강제 관측 게이트를 기준으로 다른 언어·길이·실패 입력을 확장한다.
2. E5 FP32의1회 실제 결과를 기준으로 고마움 질의 악화·무관 질의 점수를 추가 검토하고 held-out 질의로 범위를 확장한다.
3. 사람 검토 기준과 held-out 입력을 확보한 뒤 화자·정렬·검색·제작 기능별 품질 게이트를 통과시킨다. 실패 결과를 자동 적용으로 승격하지 않는다.

후속 구현·평가 작업은 [품질 게이트 계획](../../../plans/player/ai-quality-gates.md)에 둔다.
