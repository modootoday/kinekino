---
id: sot:kinekino/provenance-licenses
title: 실행 모델의 원본·변환 계보와 출시 전 권리 게이트
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 실행 모델의 원본·변환 계보와 출시 전 권리 게이트

조사일2026-10-18. 앞선 실행 준비 검증과 별도로 등록한 provenance 조사 목표의 산출물이다. 이 세션은 기존 download/catalog manifest·공식 HF 메타데이터·README·라이선스·소스만 읽었다. 모델/미디어 다운로드·추론·설치·빌드·기존 증거 수정은 하지 않았다. 문서 확인 완료와 출시 권리 승인 완료를 구별한다.

## 판정 규칙과 근거 범위

`확인`은 아래 고정 위치에서 해당 표기/문서를 읽었다는 뜻이다. `불명`은 변환자의 원본 SHA·변환 기록·권리 고지 연결이 없거나 인증 때문에 읽지 못했다는 뜻이며, 불법 또는 자유 이용 가능이라는 판정이 아니다. 공식 소프트웨어의 Apache/MIT가 가중치·tokenizer·사전·학습데이터에 자동 적용된다고 추론하지 않는다. HF card의 SPDX 표기도 라이선스 원문 파일 보존과 구분한다.

실행한 파일의 identity는 원래 세션의 `runtime/ai-spike/download-{mask,asr-q8,ocr-ko,yamnet,speaker-seg,speaker-embed}.json`이 정본이다. 아래 SHA는 그 manifest에서 읽었다. GitHub의 이번 조사 SHA는 원본 권리 문서 스냅샷이며 당시 변환자가 사용한 소스 revision이라고 주장하지 않는다.

## 변환물 고정 identity

| 실행 변환물 | HF revision·실제 파일 | 파일 SHA256(기존 manifest) | 변환 저장소의 권리 문서 |
|---|---|---|---|
| [lmz/candle-sam](https://huggingface.co/lmz/candle-sam/tree/8b4cb7c743f3b3cb8afd212a86ae15b1bbfdac97) | `8b4cb7c743f3b3cb8afd212a86ae15b1bbfdac97`; `mobile_sam-tiny-vitt.safetensors` | `5ad43a841b57cc798e2837e251c15aa9f602868a1780d195459383d117784cf7` | 고정 파일목록은 .gitattributes와 safetensors2개뿐. README/LICENSE/NOTICE 없음; cardData=null |
| [lmz/candle-whisper](https://huggingface.co/lmz/candle-whisper/tree/02d8350e5402f18725eadb6101b4963d181b0b5e) | `02d8350e5402f18725eadb6101b4963d181b0b5e`; `model-tiny-q80.gguf` 및 config/tokenizer-tiny.json | GGUF `edcc907db61aef092f1244dc1e53c55056b472be343e9bbb4dc12ebd4740392f` | README/LICENSE/NOTICE 없음; cardData=null. Q8_0 변환자의 권리/원본 SHA 선언 불명 |
| [monkt/paddleocr-onnx](https://huggingface.co/monkt/paddleocr-onnx/tree/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65) | `7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65`; `languages/korean/{rec.onnx,dict.txt,config.json}` | rec `322f140154c820fcb83c3d24cfe42c9ec70dd1a1834163306a7338136e4f1eaa`; dict `a88071c68c01707489baa79ebe0405b7beb5cca229f4fc94cc3ef992328802d7` | README Apache-2.0 선언; 별도 LICENSE/NOTICE 없음 |
| [zeropointnine/yamnet-onnx](https://huggingface.co/zeropointnine/yamnet-onnx/tree/ac2ca3bd45d12ec1f19f1144205ea529b4e9dedf) | `ac2ca3bd45d12ec1f19f1144205ea529b4e9dedf`; yamnet.onnx·yamnet_class_map.csv | ONNX `1510041dce24a2e9e84ec546807ac408ae496da6d1ed41bc3ccba649623f8e19`; map `cdf24d193e196d9e95912a2667051ae203e92a2ba09449218ccb40ef787c6df2` | README Apache-2.0 선언; 별도 LICENSE/NOTICE 없음 |
| [ONNX segmentation](https://huggingface.co/onnx-community/pyannote-segmentation-3.0/tree/733a93b6473d019a773298e08cefa686894b1854) | `733a93b6473d019a773298e08cefa686894b1854`; onnx/model.onnx·model_int8.onnx | FP32 `057ee564753071c0b09b5b611648b50ac188d50846bff5f01e9f7bbf1591ea25`; INT8 `465d0975bf70fbf14fb77c0589a5d346a9c07c2170345f529cf774678446db76` | README MIT 선언·원본 링크·export 예제. 별도 LICENSE/NOTICE 없음 |
| [ONNX WeSpeaker](https://huggingface.co/onnx-community/wespeaker-voxceleb-resnet34-LM/tree/6a61a1833ff2583aabeba044f5c8221f00b67ceb) | `6a61a1833ff2583aabeba044f5c8221f00b67ceb`; onnx/model.onnx·preprocessor_config.json | ONNX `3955447b0499dc9e0a4541a895df08b03c69098eba4e56c02b5603e9f7f4fcbb` | README는 CC-BY-4.0 front matter만 있음. 별도 LICENSE/NOTICE·원본 revision·변환 스크립트 없음 |

별도 LICENSE/NOTICE 부재는 조회한 해당 HF revision 파일목록 기준이다. 원본 저장소 전체나 모든 의존 패키지에 NOTICE가 없다는 뜻은 아니다. 카드 README도 권리 선언 근거로 보존하되 별도 원문 파일을 읽은 것처럼 쓰지 않는다.

## 모델별 원본 연결과 확인/불명

### MobileSAM safetensors

공식 원본 [MobileSAM README](https://github.com/ChaoningZhang/MobileSAM/blob/f706ad9c4eb7f219c00d9050e46328518ffb65d2/README.md)는 `weights/mobile_sam.pt`, TinyViT encoder와 SAM decoder 계보를 설명한다. 같은 SHA의 [LICENSE](https://github.com/ChaoningZhang/MobileSAM/blob/f706ad9c4eb7f219c00d9050e46328518ffb65d2/LICENSE)는 Apache-2.0이다. 루트 tree에는 별도 NOTICE가 없었다. 이 Apache 문서의 존재는 확인했으나 lmz safetensors의 원본 checkpoint hash·변환 버전·가중치별 적용 선언은 확인하지 못했다.

[공식 Candle SAM 예제](https://github.com/huggingface/candle/blob/48fb1719f0f013947e226580e2c63b4946741135/candle-examples/examples/segment-anything/main.rs)는 Rust 실행 경로의 1차 참조 위치다. 실행기가 lmz 파일을 사용할 수 있다는 사실은 원본→변환 hash 대응이나 재배포 허가를 증명하지 않는다. Candle 소프트웨어 라이선스는 이 가중치의 누락 고지를 대체하지 않는다.

남은 출시 게이트: MobileSAM 원본 checkpoint의 고정 ID/hash와 변환 recipe·tensor 명칭/개수/dtype 대응, Apache 적용 범위·저작권 고지·SAM/TinyViT 파생물 고지 확인. 학습에 사용한 SAM 계열 이미지 데이터의 권리를 코드 Apache로 표시하지 않는다. 이번 조사에서 개별 학습데이터 재배포 권한은 확인하지 않았으며 제품에 학습데이터를 번들하지 않는다.

### Whisper tiny Q8_0 GGUF와 tokenizer

공식 [OpenAI README의 License 절](https://github.com/openai/whisper/blob/86098128c0b4f24f0e2aa2994de830614b474227/README.md#license)은 코드와 모델 가중치를 모두 MIT로 명시하며 [고정 LICENSE](https://github.com/openai/whisper/blob/86098128c0b4f24f0e2aa2994de830614b474227/LICENSE)를 확인했다. 루트 별도 NOTICE는 없다. 원본 모델에 대한 명시 근거는 MobileSAM보다 강하다.

그러나 lmz GGUF는 원본 tiny checkpoint SHA와 quantizer revision/옵션을 밝히지 않는다. config/tokenizer 파일도 원본 HF/Whisper/tiktoken 파일의 어느 revision에서 왔는지 불명이다. 원본 MIT의 변형/배포 허용과 해당 변환 파일의 실제 lineage 확인은 별개다. 원본 MIT 저작권·허가 고지를 동봉하고 변환 출처·정밀도·tokenizer derivation을 manifest에 명시하기 전 출시 provenance 완료로 표시하지 않는다. Whisper 학습 음성 데이터 전체가 MIT라는 뜻은 아니다. [공식 model card](https://github.com/openai/whisper/blob/86098128c0b4f24f0e2aa2994de830614b474227/model-card.md)의 학습자료 설명은 데이터 배포 라이선스가 아니다.

### 한국어 PP-OCRv5 ONNX와 사전

고정 monkt README는 PaddlePaddle PP-OCRv5 collection·Paddle2ONNX를 원본/변환 도구로 지목하며 Apache-2.0을 선언한다. config는 `korean_PP-OCRv5_mobile_rec`, 사전 이름 `ppocrv5_korean_dict.txt`를 명시한다. 공식 [한국어 모델 카드](https://huggingface.co/PaddlePaddle/korean_PP-OCRv5_mobile_rec/blob/c02ecaf1f22bfd1c618cce154fd19185b47e663a/README.md)도 Apache-2.0 표기지만 별도 LICENSE/NOTICE 파일은 해당 HF tree에 없다.

공식 [PaddleOCR LICENSE](https://github.com/PaddlePaddle/PaddleOCR/blob/dab3fe35379033fdcb2d0e9572fac0b36c9a9ebf/LICENSE)는 Apache-2.0이다. 같은 SHA에 [ppocrv5_korean_dict.txt](https://github.com/PaddlePaddle/PaddleOCR/blob/dab3fe35379033fdcb2d0e9572fac0b36c9a9ebf/ppocr/utils/dict/ppocrv5_korean_dict.txt)가 존재하며 Git blob ID는 `e2202631098b4e49662ed4d29a380df4cbf6de78`이다. 이것은 SHA256이 아니다. monkt dict와 byte·문자 순서·BOM/개행·공백 토큰까지 동등하다고 이번에 검증한 것은 아니다.

남은 게이트: monkt ONNX의 정확한 원본 inference.pdiparams SHA, Paddle2ONNX 버전·opset/변경 기록, official inference.yml의 character_dict와 monkt dict의 Rust 비교. 사전을 코드 라이선스만으로 승인하지 말고 upstream 파일의 적용 고지·수정·재배포 조건을 함께 보존한다. 루트 NOTICE는 없지만 공식 tree에는 `deploy/ios_demo/NOTICE`가 있어 필요한 컴포넌트를 포함할 경우 그 범위를 별도로 검토한다. 한국어 학습/평가 이미지의 권리 목록은 카드의 정확도 수치만으로 확인되지 않는다. 학습자료를 Apache로 단정하거나 번들하지 않는다.

### YAMNet ONNX와 class map

변환자 [고정 README](https://huggingface.co/zeropointnine/yamnet-onnx/blob/ac2ca3bd45d12ec1f19f1144205ea529b4e9dedf/README.md)는 Google의 [Kaggle YAMNet version1](https://www.kaggle.com/models/google/yamnet/tensorFlow2/yamnet/1)을 직접 링크하고 tf2onnx 명령을 제공한다. 단 dependency 범위만 있고 정확한 버전·SavedModel hash·변환 후 parity는 없다. Kaggle 페이지는 이번 도구에서 내용이 표시되지 않아 가중치 라이선스 원문을 독립 확인하지 못했다. 변환 카드 Apache 표기를 Google 가중치 원문 확인으로 바꾸지 않는다.

공식 [yamnet.py](https://github.com/tensorflow/models/blob/34a21326906b9574fa11c4d6d0a5c534ff039267/research/audioset/yamnet/yamnet.py) 헤더는 Apache-2.0이다. [원본 README](https://github.com/tensorflow/models/blob/34a21326906b9574fa11c4d6d0a5c534ff039267/research/audioset/yamnet/README.md)는 가중치와 class map 위치를 제공한다. 이것은 소프트웨어/자산 연결 근거이며 가중치 또는 CSV의 모든 권리를 한 번에 확정하지 않는다.

[Google AudioSet 배포 페이지](https://research.google.com/audioset/download.html)는 dataset에 CC-BY-4.0, ontology에 CC-BY-SA-4.0을 구분한다. 학습 데이터 label/ontology/원본 YouTube 음성/모델 가중치/class map CSV를 같은 Apache 묶음으로 표시하지 않는다. class map의 ontology 유래·변형 여부·적용 고지와 원본 CSV byte 대응을 확인해야 하며, CC-BY-SA가 가중치 전체에 자동 전파된다고도 결론내리지 않는다. 출시 전 Google version1 모델 원문·변환자 고지·CSV attribution/권리 범위를 확보한다.

### Segmentation FP32/INT8와 WeSpeaker embedding

segmentation 변환 README는 원본 [pyannote/segmentation-3.0](https://huggingface.co/pyannote/segmentation-3.0/tree/e66f3d3b9eb0873085418a7b813d3b369bf160bb)을 직접 지목하고 Torch→ONNX export 코드를 공개한다. 단 원본 revision·torch/pyannote version·INT8 calibration/quantizer는 고정하지 않는다. 원본 HF metadata는 MIT와 gated=auto, LICENSE 파일 존재를 보여준다. [고정 LICENSE 경로](https://huggingface.co/pyannote/segmentation-3.0/blob/e66f3d3b9eb0873085418a7b813d3b369bf160bb/LICENSE)의 raw 조회는401이었다. 따라서 MIT 카드 표기와 LICENSE 원문 독해를 구분한다. 사용자 동의·권한 확인 없이 다른 mirror로 gated 약관을 우회하지 않았다.

WeSpeaker ONNX README는 CC-BY-4.0만 선언하므로 직접 변환 원본은 불명이다. 별도 공식 [pyannote wrapper 카드](https://huggingface.co/pyannote/wespeaker-voxceleb-resnet34-LM/blob/837717ddb9ff5507820346191109dc79c958d614/README.md)는 WeSpeaker pretrained 모델의 wrapper라고 명시하고 CC-BY-4.0을 표시한다. 이는 후보 계보 근거이며 ONNX가 이 wrapper의 특정 checkpoint에서 변환됐다는 증명은 아니다. 원본 WeSpeaker와 pyannote wrapper와 onnx-community의 세 단계를 구분한다.

[WeSpeaker toolkit LICENSE](https://github.com/wenet-e2e/wespeaker/blob/9fecd6cb4f47475d01761d87c826298dff4ef18c/LICENSE)는 Apache-2.0으로, 모델 카드의 CC-BY-4.0과 다르다. [공식 pretrained 목록](https://github.com/wenet-e2e/wespeaker/blob/9fecd6cb4f47475d01761d87c826298dff4ef18c/docs/pretrained.md)의 링크와 각 checkpoint를 고정해 원본을 대조해야 한다. CC-BY 법문은 [4.0 legalcode](https://creativecommons.org/licenses/by/4.0/legalcode)를 참조하되 실제 배포자의 attribution·변형 고지가 필요하다. [VoxCeleb 공식 페이지](https://www.robots.ox.ac.uk/~vgg/data/voxceleb/)는 YouTube 수집 데이터와 privacy notice를 별도 안내한다. 모델 CC-BY 또는 toolkit Apache가 원본 영상/음성 전체의 재배포 허가를 뜻하지 않는다.

남은 게이트: segmentation 원본 MIT 원문·접근조건·정확한 checkpoint SHA, FP32/INT8 export 및 quantization 계보; WeSpeaker ONNX의 실제 원본 SHA/변환 recipe·CC attribution. 두 모델의 학습 corpus별 약관은 별도 inventory 대상으로 남긴다. 단순 모델명 또는 실행 성공으로 출시 권한 확인을 대신하지 않는다.

## 출시 담당의 구체적 다음 작업

1. 기존 실행 증거는 보존한다. 모든 파일을 `권리 불명이라 이미 실행 안 됨`으로 소급 변경하지 않는다. 로컬 연구 실행과 제품 번들/다운로드 제공의 권리 게이트는 별도다.
2. 위 고정 README/LICENSE 링크에서 필요한 원문·저작권/NOTICE 범위를 출시 패키지의 license manifest에 고정한다. 이번 세션은 원문 파일을 새로 다운로드해 저장하거나 다른 증거를 쓰지 않았다. 카드만 있는 항목은 카드 선언/원문 미확인을 그대로 표시한다.
3. 우선 lmz MobileSAM·Whisper의 원본 SHA/변환 record, WeSpeaker ONNX의 직접 원본, OCR 사전 byte 대응을 확보한다. 확인 불가면 출처가 명확한 공식 artifact에서 재변환하는 별도 승인된 Rust 도구 작업을 계획한다. Python 설치나 자동 재변환 명령을 이 문서에서 실행하지 않는다.
4. 배포 manifest는 코드/engine, 모델 원본, 변환 artifact, tokenizer/사전/class map, 학습자료를 별도 항목으로 관리하고 변경·attribution·사용자 조건을 연결한다. 필수 원문·원본 대응이 불명인 artifact는 출시 provenance 게이트를 미통과로 유지한다.

조사 완료 상태: 6개 실행 모델군의 identity·공식 근거·누락 구간과 다음 검토를 기록했다. 모든 모델의 재배포 권리를 최종 승인했다는 뜻이 아니다.
