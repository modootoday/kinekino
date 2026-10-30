---
id: sot:kinekino/hf
title: 기능별 HF 모델과 스파이크 범위
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 기능별 HF 모델과 스파이크 범위

기준일2026-10-18. 사용자 UI/UX 승인 후 실제 미디어로 검증한다. 이 목록은 모델 출시 확정표가 아니다. 고정 revision·파일 크기·LFS 해시는 실측 증거의 catalog/download manifest를 기준으로 한다. 모든 후보를 내려받거나 실행했다는 뜻으로 metadata-only 상태를 사용하지 않는다.

| 기능 | HF 후보 | 실행 경로·판단 |
|---|---|---|
| retrieval 비교 | intfloat/multilingual-e5-small FP32 ONNX | 고정 자산 취득·SHA 대조, 실제33원문에 역할별 prefix로1회 실행. 임시MRR0.825/hit@3 0.8, 고마움8위·무관질의topcosine0.7894. 사람 기준/no-match 보정 전 품질 미승인 |
| 음성 자막 | openai/whisper-tiny, openai/whisper-base, lmz/candle-whisper Q8 | Candle Rust 실제25초 영화 음성·한국어 FLEURS. Q8과FP32 동일 조건3회 비교. 한국어 시간/문구 정확도는 별도 평가 |
| ASR 최신 대안 | Qwen/Qwen3-ASR-0.6B | 고정 Candle Rust CPU F32로 KO1/JA3/EN1 실제 실행. 약3.9GB 표본 private. 프로토콜 prefix 수정·기존 출력 독립 replay 감사 완료. 언어 전체 정확도·기본 선정·양자화·GPU는 미승인 |
| 단어 정렬 | Qwen/Qwen3-ForcedAligner-0.6B | 고정 lumos Rust CPU로 기본4/숫자 term A/B2/무음 이동2/raw 진단2회, 총10회 구조3통과/7실패. JA1516/1634 raw 진단은 각 baseline 최종bit 동일·기존0구간5개 중4개raw동일값/1개repair축소를 확인. private LIS·다른 발화 원인·사람 경계 정확도는 미검증이며 ASR timestamp를 정렬 완료로 보지 않음 |
| VAD | onnx-community/silero-vad | Rust ort 어댑터, 실제16kPCM·512sample+64context·상태 유지 |
| 화자 분리 | onnx-community/pyannote-segmentation-3.0 + onnx-community/wespeaker-voxceleb-resnet34-LM | ORT·Rust kaldi-fbank·군집 prototype. unknown/겹침 보존. 현재 군집 품질은 승인하지 않음 |
| 화자 최신 대안 | pyannote/speaker-diarization-community-1 | gated 상태/CC-BY4.0 확인. 사용자 로그인/조건 수락을 대신하거나 임의 mirror로 우회하지 않음. VBx와 prototype을 구별 |
| 음악·OP/ED 후보 | zeropointnine/yamnet-onnx; Xenova/ast-finetuned-audioset-10-10-0.4593 | YAMNet 실제음원 실행. AST는 대안 조사. 음악 점수만으로 OP/ED를 확정하지 않으며 반복 fingerprint·위치·수동 검토 결합 |
| 반복 음악 | 모델 불필요 | 순수 Rust rusty-chromaprint 후보. 실제 동일 소스에서 파생한 양성/음성 대조와 여러 실제 회차 반복 구간 시험은 구별 |
| 한국어 OCR | monkt/paddleocr-onnx의 korean_PP-OCRv5_mobile_rec | 실제영화+배포자 한국어 자막을 렌더한 fixture. 넓은ROI 공백→좁은ROI 인식 재현. 한 장을 자연 burn-in 전체 품질로 일반화하지 않음 |
| 일본어 OCR·검출 | PaddlePaddle/PP-OCRv6_small_rec_onnx, PP-OCRv6_small_det_onnx | 최신 공식 ONNX 후보. 일본어 지원 목록과 한국어 별도 모델을 구별. 원본 inference.yml의 전처리/사전 사용 |
| 교정·요약·카드·숏폼 추천 | unsloth/Qwen3-0.6B-GGUF Q4_K_M | Rust mistral.rs 실제자막/ASR prompt. JSON 문법·필수 필드·근거·문장 품질을 각각 검사. 초기 출력 품질 미달을 통과로 표시하지 않음 |
| 텍스트 최신 대안 | Qwen/Qwen3.5-0.8B, unsloth/Qwen3.5-0.8B-GGUF | VLM 포함 architecture와 GGUF/엔진 지원을 별도 확인. 파라미터 수만으로0.6B보다 가볍다고 가정하지 않음 |
| 텍스트 용량 확대 | unsloth/Qwen3.5-2B-GGUF Q4_K_M | 고정 SHA/LFS hash 실제 검증 후 같은 Rust CPU 엔진·3개 프롬프트 실행. 약2.9GB 표본 메모리, 카드 구조는 통과하나 의미 환각·교정 타입/순서·불연속 범위 오류로 자동 적용 미승인 |
| 의미 후보 검색 | sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2 | 실제26자막+7질의 FP32/AVX2관측2회. 임시MRR .55/.5533·hit@3 .8, 일부관련cue10~12위로품질미승인.33벡터평균cosine .992899/최소.936871, top1일치는정확도아님. 독립Rust reference/사람기준·retrieval대안은후속 |
| 클릭 피사체 분리·2D | lmz/candle-sam MobileSAM | Candle Rust 실제프레임·양/음성 point·마스크PNG/오버레이. 원본 좌표/배율 복원과 사용자 선택 근거 보존 |
| 피사체 탐지 | onnx-community/rfdetr_nano-ONNX | Apache2.0 경량 후보. 일반 클래스와 애니메이션 캐릭터 신원을 구별 |
| 추적 | facebook/EdgeTAM | 공식 가중치 후보. 상태/메모리·장면전환·가림·ID 안정성 시험 전 완료 처리 안 함. 한 프레임 SAM은 추적 검증이 아님 |
| 상대 깊이·2.5D | onnx-community/depth-anything-v2-small-ONNX | ORT FP32/양자화 동일 입력3회. 종횡비 보존·14배수·전처리 일치 후 비교. relative depth는 미터/완전 메시 아님 |
| 깊이 최신 대안 | depth-anything/DA3-SMALL | 공식 모델/라이선스/revision 조사. 현재 ORT V2 결과와 성능을 혼합하지 않음 |
| 깊이 Candle 변환 경로 | lmz/candle-dino-v2, jeroenvlek/depth-anything-v2-safetensors | 조사에서 DINO 변환 저장소 CC-BY-NC4.0와 불완전 변환 provenance 확인. 검토 전 배포 경로에서 제외하고 Apache2.0 V2 ONNX 경로로 우선 시험 |
| 완전3D 연구 | stabilityai/TripoSR, tencent/Hunyuan3D-2mini | 별도 연구. Windows Rust runtime·가중치 라이선스·8GBVRAM 예산 확인 전 실행/번들 확정하지 않음. 기존 설계의 평면/depth projection 패키지와 구별 |
| 저장·다운로드·변환·수동 제작·카드 배치·GLB 패키징 | 모델 불필요 | Rust HTTP/포맷/이미지/geometry SDK와 검증된 미디어 어댑터. 모델 다운로드를 선행조건으로 강제하지 않음 |

공식 근거: [Candle](https://github.com/huggingface/candle), [ORT Rust](https://github.com/pykeio/ort), [PP-OCRv6 공식 발표](https://huggingface.co/blog/paddlepaddle/pp-ocrv6), [HF Whisper](https://huggingface.co/openai/whisper-tiny), [Pyannote](https://huggingface.co/pyannote/speaker-diarization-community-1), [HF depth ONNX](https://huggingface.co/onnx-community/depth-anything-v2-small-ONNX). 후보 이름은 위 표의 HF repo ID로 직접 조회할 수 있다.

남은 게이트와 실제 성공/실패는 [검증 색인](../../../../verification/ai/README.md), 실행 순서는 [계획](../../../../../plans/player/plan-007-real-media-model-spikes.md)을 따른다.
