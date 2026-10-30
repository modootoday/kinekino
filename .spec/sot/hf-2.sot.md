---
id: sot:kinekino/hf-2
title: HF 모델 저장소 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# HF 모델 저장소 계약

2026-10-27 후속: [read-only 모델 카탈로그](model-catalog-reads.md)의 domain serializer/검증·공통 SDK와 models.list/inspect·명시 --model-root CLI를 구현했다. [검증](../../../verification/application/model-catalog-reads-20261027.md)은 합성 catalog/ordinary stub 파일의 조회·미검증 상태·readonly 보존이다. 실제 HF artifact/runtime admission·설치/해시검증/삭제·GPU/dtype/품질 검증과 구분한다. 아래 generic 전체 registry 미구현 설명은 실행/lifecycle 범위에서 계속 유지한다.

`ModelManifest`: task, repo_id, immutable_revision(commit), files(path,size,sha256), format, quantization, source_model/revision, conversion_recipe/version, tokenizer/config, license/ref, supported_languages, runtime/min_version, measured_memory, evaluation_id. 실행 가능한 release manifest는 미고정 main/latest를 허용하지 않는다. 이는 generic 설계 필드 목록이며 전체 serde/registry 구현이나 실행 admission 승인이 아니다.

2026-10-21 현재 상태: 최초 조사에서 revision·해시가 미확정이었던 문장을 모든 후속 모델에 적용하지 않는다. [고정 cached VAD](../../../verification/ai/cached-vad-resource-adapter-2026-10-21.md)는 Silero revision `e71cae966052b992a7eca6b17738916ce0eca4ec`·weights SHA `a4a068cd6cf1ea8355b84327595838ca748ec29a25bc91fc82e6c299ccdc5808`의 제한 실제 정상/취소를 보존한다. [VAD 전용 release v1](vad-release-manifest-v1.md)은 해당 작업의 별도 schema/고정 파일 계약이다. [기존 실제 텍스트9 검토](../../../verification/ai/text-review-sdk-actual.md)의 원 model/revision·파일 pin도 generic release 배포 승인과 구별한다.

저장 정밀도·요청한 runtime 설정·실행 중 실제 dtype 관측은 서로 다르다. 선언한 precision이나 model pin만으로 런타임 전체 정밀도·메모리/속도·변환 계보·라이선스 closure를 검증했다고 하지 않는다. 미제공 관측은 채워 넣지 않으며 [provenance 감사](../../../verification/ai/provenance-audit.md)의 과거 EXE/source 연결 한계를 유지한다. 전체 작업별 manifest/제품 SDK·CLI 연결·품질 인수는 여전히 남아 있다.

설정의 ‘모델 관리’에서 작업별 설치/용량/예상 자원/라이선스·버전·삭제·오프라인 사용을 표시한다. [HF 다운로드 문서](https://huggingface.co/docs/hub/en/models-downloading)와 [revision/캐시 설명](https://huggingface.co/docs/huggingface_hub/en/guides/download)을 참조해 필요한 파일만 받고, 임시 다운로드→크기/해시 검증→가용 상태로 승격한다. 토크나이저·전처리 설정도 같은 revision으로 묶는다. 여러 작업이 쓰는 모델 삭제는 참조 중 여부를 확인한다.

기본은 공개/비게이트 모델. 게이트 모델의 약관·로그인은 사용자가 처리하고 HF 토큰은 OS 자격 저장소 참조로만 관리한다. 토큰을 URL·명령행·프로젝트·진단에 남기지 않는다. 모델 설명/README/Python 코드가 앱 명령이 되지 않는다. 임의 remote code 실행·pickle 로딩을 기본 경로로 쓰지 않고 검토한 safetensors/GGUF/ONNX/런타임 포맷만 허용한다. 파일 경로 traversal·외부 참조도 검사한다. HF 다운로드 호스트는 실제 공식 CDN redirect를 지원하되 인증 헤더를 임의 외부 호스트로 전달하지 않는다. 기존 광고 필터의 예외 목록과 모델 다운로드 정책은 분리한다.
