---
id: sot:kinekino/model-candidates
title: 후보와 미확정 항목
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 후보와 미확정 항목

조회일 2026-10-18. 아래는 모델 배포자가 게시한 HF 저장소 또는 공식 프로젝트 근거다. 기본 모델 확정은 평가 후 별도 manifest commit으로 한다.

HF API 읽기 전용 조회로 tiny/base·Qwen0.6B·한국어 mobile OCR·EdgeTAM·depth small의 관측 revision/라이선스/파일 크기·LFS SHA 정보를 [조사 스냅샷](../../../../evidence/ai/hf-model-research-2026-10-18.json)에 저장했다. 가중치를 내려받거나 실행하지 않았다. 이 revision 관측은 검증된 release manifest 고정과 다르다.

위 문장은 API 조사 시점의 상태다. 이후 사용자 ‘현 장치를 기준으로 실측’ 지시로 고정 ASR/text/depth 데이터를 작업 폴더에 받았다. 모든 코드 Rust 지시 이후 자체 Python 벤치 코드를 제거하고 [Rust 도구](../../../../../tools/validation/README.md)로 전환했다. Python 추론은 실행하지 않았다. 추론은 재생 단독 60분 완료 뒤 분리 실행한다. 실제 결과가 생기기 전 후보를 실측 완료로 표시하지 않는다. 조사 API의 가중치 파일은 tiny 151,061,672B, base 290,403,936B, Qwen0.6B safetensors 1,503,300,328B, 한국어 rec 13,342,671B, EdgeTAM 56,116,523B, depth small 99,173,660B다. 설정/토크나이저/런타임 및 실행 메모리를 포함한 크기가 아니다.

| 역할 | 후보/근거 | 검토 조건 |
|---|---|---|
| 다국어 ASR | [openai/whisper-small](https://huggingface.co/openai/whisper-small), [Qwen/Qwen3-ASR-0.6B](https://huggingface.co/Qwen/Qwen3-ASR-0.6B) | 한국어/일본어 음성·배경음·무음·노래 평가. ASR 아키텍처별 런타임/시간 정렬 지원 별도 확인 |
| sLM 교정/카드 요약/후보 설명 | [Qwen/Qwen3-0.6B](https://huggingface.co/Qwen/Qwen3-0.6B), [microsoft/Phi-4-mini-instruct](https://huggingface.co/microsoft/Phi-4-mini-instruct) | 0.6B의 속도만으로 품질 선정 금지. 원문 보존/문자열 구조 출력·환각·다국어 평가 후 크기 등급 선택 |
| OCR | [PaddlePaddle/PP-OCRv5_mobile_rec](https://huggingface.co/PaddlePaddle/PP-OCRv5_mobile_rec), [공식 다국어 설명](https://github.com/PaddlePaddle/PaddleOCR/blob/main/docs/version3.x/algorithm/PP-OCRv5/PP-OCRv5_multi_languages.en.md) | 검출·인식·언어별 가중치가 별도일 수 있음. 하나의 rec 파일로 한/일 모두 지원한다고 가정하지 않음 |
| 클릭/박스 기반 분리·추적 | [facebook/sam2.1-hiera-tiny](https://huggingface.co/facebook/sam2.1-hiera-tiny), [공식 SAM2](https://github.com/facebookresearch/sam2) | tiny도 경량 Windows 배포 검증 필요. 공식 구현은 PyTorch/Windows WSL 권장이라 그대로 기본 번들로 선정하지 않음 |
| 상대 깊이 | [depth-anything/Depth-Anything-V2-Small-hf](https://huggingface.co/depth-anything/Depth-Anything-V2-Small-hf), [공식 저장소](https://github.com/DepthAnything/Depth-Anything-V2) | Small은 Apache-2.0, Base/Large/Giant는 CC-BY-NC-4.0로 서로 다름. 선택 모델별 확인. 상대 깊이는 실제 미터 단위나 완전 3D 복원 아님 |

노트북 우선 후보는 [Whisper tiny](https://huggingface.co/openai/whisper-tiny)/[base](https://huggingface.co/openai/whisper-base) 다국어 모델로 좁힌다. small/Qwen ASR는 품질 향상 선택지다. [한국어 PP-OCRv5 mobile](https://huggingface.co/PaddlePaddle/korean_PP-OCRv5_mobile_rec)는 일본어 포함 일반 mobile rec와 별도로 평가한다. PP-OCRv5_mobile_rec 카드에는 한글 지원이 명시되지 않아 한글 기본으로 대체하지 않는다. [facebook/EdgeTAM](https://huggingface.co/facebook/EdgeTAM)은 엣지 추적 대안의 공식 HF 후보로 추가하되 Windows 포맷·메모리·성능 시험 전 SAM2보다 빠르다고 확정하지 않는다. 대형 모델은 상시 기본이 아니다.

공식 엔진 후보: [whisper.cpp](https://github.com/ggml-org/whisper.cpp)는 Windows/CPU·GPU·양자화/VAD 지원을 문서화한다. [llama.cpp](https://github.com/ggml-org/llama.cpp)는 텍스트/VLM 추론·여러 양자화/백엔드를 제공한다. HF 원본을 해당 런타임에 바로 넣을 수 있는지는 모델 아키텍처·변환 포맷·버전마다 검증해야 한다. HF에 등록된 변환본은 원본 모델·변환자·도구·정밀도·revision·라이선스를 함께 추적한다.

[ONNX Runtime DirectML 문서](https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html)는 DirectML이 유지보수 단계이며 신규 Windows 기능은 WinML로 이동했다고 명시한다. 따라서 새 ONNX 경로는 CPU 기준선과 WinML을 우선 비교하고 DirectML은 호환 후보로 둔다. 지원 opset/동적 shape/연산자와 Windows 최소 버전은 모델별 검증한다. Candle/ORT/ggml 등의 Rust 바인딩 선정은 별도 spike 후 결정하며 UI에 Python/WSL 설치를 필수로 요구하지 않는 것을 목표로 한다.
