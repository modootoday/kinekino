---
id: sot:kinekino/protocol
title: 실제 자산 스파이크 프로토콜
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 실제 자산 스파이크 프로토콜

모델 메타데이터 조회, 다운로드/무결성, 그래프 로드, 실제 입력 추론, 출력 의미 검토, 품질 점수, 제품 통합을 별도 상태로 기록한다. 실행 코드0이나 PNG 생성만으로 기능 품질을 통과시키지 않는다.

새 자체 tool 추론 전에는 [컴파일 지문·버전 보존 계약](../../../../verification/ai/model-tool-preservation.md)에 따라 EXE와 해당 source manifest를 먼저 고정한다. 과거 mutable source 사본은 당시 소스의 증명이 아니며 [감사 한계](../../../../verification/ai/provenance-audit.md)를 따른다.

실제 영화는 Blender Sintel 공식 archive의 H2641280×544·AC3·10개 내장 자막을 포함한다. 배포자 한국어/일본어 번역 자막을 별도 확보한다. 원본 archive/movie/hash를 보존하고107~132초25초 dialogue16kmonoPCM,30~50초20초 audio-event 후보,110초 원본 프레임·자막 렌더 frame/ROI를 파생했다. 번역 자막은 원음 ASR 정답이 아니다. 렌더한 OCR fixture와 원래 영상에 입혀진 자막도 구별한다.

한국어 읽기 음성은 [Google FLEURS](https://huggingface.co/datasets/google/fleurs) test의 config ko_kr/row0/id1883·CC-BY4.0를 사용한다. 정답과 raw reference·음원/가공WAV hash를 저장한다. 단일 표본이며 애니메이션 대사/BGM·발화 겹침 품질을 대표하지 않는다. 일본어 FLEURS 조회는 서버의 Parquet scan 한도 오류로 해당 경로를 검증 완료 처리하지 않는다. 공개 dataset asset의 일시 URL은 정본에 영속화하지 않는다.

실측 전 기존 preflight로 CPU/RAM·소유 시험 앱을 확인한다. 사용자 ChatGPT/Terminal은 종료하지 않는다. CPU4/interop1·무거운 inference1개·BelowNormal·오프라인 준비 캐시·소유 프로세스 종료 후 unload를 적용한다. 자산 변환은 스레드2·기존 파일 덮어쓰기 금지다. 초기 변환은 normal priority였고 이후 adapter에서 BelowNormal/windowless를 명시했다. VM/DPI/전원/보안/OS 캐시 설정을 변경하지 않는다.

FP32/양자화 비교는 동일 source hash·전처리·모델 계열·decoder·장치에서3회씩 교차 실행한다. 새 process 로드를 포함한 wall time과100ms 표본의 private/working set peak를 함께 보고한다. OS cache는 비우지 않으며 cold disk라고 부르지 않는다. warmup/초기 정사각 depth 표본과 종횡비 보존 비교를 혼합하지 않는다. raw stdout/stderr와 시작/성공/실패/timeout을 전부 보존한다.100ms 표본 peak는 연속 최고값 보장이 아니다.

모델이 예측한 시간도 실제 입력 duration·단조성·원본 offset과 대조한다. 범위 밖 cue는 자동 저장하지 않는다. 텍스트는 진단을 제거한 ASR cue만 입력하고 반환 JSON 필수 필드·cue ID 참조·문장 수·새 사실 여부를 구별한다. 문법 성공과 의미 품질은 별개다. 텍스트 엔진이 출력한 실제 sampling/TTFT/prompt/decode/cache-hit 값을 사용하며 다른 길이/모델의 tok/s 순위를 일반화하지 않는다.

CER는 Unicode 영숫자·소문자·공백/구두점 제외 규칙과 정답 문자 수·오류 수를 함께 기록한다. 공백/IME 품질은 별도 exact 비교다. DER/JER·mask IoU·depth 정확도는 수작업 정답/기준이 없으면 계산하지 않는다. 양자화가 더 좋다는 결론을 단일 표본에서 내리지 않는다.

적용 스킬: modoo.today inference-benchmark-workload-protocol·quantization-and-vision-input-parity-check 및 rust-pro. 이 스킬은 프로토콜을 제공하며 실제 실행은 로컬 Rust 도구가 수행했다.

후속 회귀 수정: player.exe와 대소문자 감지를 preflight에 추가했다. 예전 kinekino-player*.exe 감지만으로 새 GUI 부재를 보장하지 않는다. 실제 canonical PID 때문에 ready=false와 무실행을 확인했다. label별 preflight를 저장하고 기존 측정 label은 덮어쓰지 않는다. 구조화 비교는 검증 structured-text.md의 시작 실패/대기와 모델 요청 결과를 구분한다.

일본어 row API 실패는 공식 dev archive의 Rust 스트리밍 확보로 우회했다. 서로 다른 transcript ID3개와 bounded ASR 결과는 검증 audio.md에 기록했으며 화자 독립성/애니메이션 영역 품질은 보장하지 않는다.
