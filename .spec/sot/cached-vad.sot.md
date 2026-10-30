---
id: sot:kinekino/cached-vad
title: Cached VAD 제품 활성화 전 근거 차이
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Cached VAD 제품 활성화 전 근거 차이

## 현재 저비용 후속: 공식 문서와 공급 경계

중앙 회신으로 issuer8의 actual pin 일치 및 정적 검토상 확정 P1/P2 미발견을 회수했다. 이는 타입/실행 통과가 아니다. 현재 우선순위는 아래 확보 가능한 공식/기존 근거와 실제 부족한 제품 입력을 분리하는 것이며 SDK sole-heavy, Media source-only 배정을 유지한다.

2026-10-21 사용자 재개 지시와 중앙 현재 배정을 다시 확인했다. SDK actual parent8 gate와 별개로 아래 문서 읽기와 source-only 후보 정리를 진행한다. Cargo·OS probe·native load·모델 실행·바이너리/모델 다운로드는 수행하지 않는다.

| 사실 | 정확한 출처 | 판정과 후속 |
|---|---|---|
| 공식 Silero MIT 본문 | GitHub connector로 `snakers4/silero-vad`, `master`, `LICENSE`를 읽었다. 반환 blob SHA는 `0bf5e90cac691b999d4a35044f97167d7bbbf0b9`이며 [공식 파일](https://github.com/snakers4/silero-vad/blob/master/LICENSE)에 Copyright (c) 2020-present Silero Team이 있다 | blob 식별자는 Git object SHA이며 SHA256이 아니다. master 읽기 시점의 텍스트와 고정 HF 변환 모델의 재배포 관계는 별도다. 반환 텍스트를 원 파일 bytes로 간주해 SHA256을 발명하지 않는다 |
| 고정 HF 선언 | [고정 revision README](https://huggingface.co/onnx-community/silero-vad/blob/e71cae966052b992a7eca6b17738916ce0eca4ec/README.md), 로컬59B README SHA256 `394AC7912169C60FA95CC6C33C615F78368749F0F305B6CB43058A28233CF00A` | license:mit 선언과 실제 upstream license 본문·변환 provenance 연결을 구분한다. 모델 재다운로드 없이 조사 가능하다 |
| native 공식 고지 위치 | ORT [v1.31.0 LICENSE](https://raw.githubusercontent.com/microsoft/onnxruntime/v1.31.0/LICENSE), [ThirdPartyNotices.txt](https://raw.githubusercontent.com/microsoft/onnxruntime/v1.31.0/ThirdPartyNotices.txt) | 공식 source 문서 위치 확인이다. 기존 dfbin 두 파일의 archive/build source/config 및 배포 고지 일치 인수는 아직 아니다 |
| OS 역사적 관측 | 기존 `runtime/verification/ai-vad-normal-r1/loaded-runtime5.json`과 `ai-vad-build-r1/normal-pe-dependencies-r3.txt` | 선택 module5와 PE import를 구분한다. 제품 worker의 전체 현재 loaded closure·최소 Windows 인수를 주장하지 않으며 새로운 OS 관측은 중앙 배정 뒤다 |

구체 issuer 후보는 `runtime/development/vad-product-issuer-r1/source8.json` SHA256 `D76A939493A893AAC1C38434C17593143CFD41FCFA558BD5403B72649DB5FA2B`, before8 `D9E692FAC69762A36131CE4777687A4F1B7069321C113FEC4736AADC60EF845C`다. release-binding37 기반 proposed44 `16F4391E38479DC126B7E12C14EC4ED137113BE0CDB68650D5EBF563E57BE012`이며 SDK active38에 합성하지 않는다. 직접fmt8/authored5 외 타입·시험·native 실행0이다. `PATH-REVIEW.md`에 normal root module/dependency 미승격을 명시했고 중앙 독립 검토에 인계했다.

다음 공급 경계는 기존 parent application 조립이 선택한 release/profile/manifest/worker pin과 원 immutable file guard를 기존 spawn owner가 유지하는 방식으로 좁힌다. child가 실제 요청 bytes와 대조할 기대값을 caller JSON에서 승인하지 않도록 expected configuration의 생성·보관·전달 주체를 먼저 명시한다. 기존 PID/QPC 결속은 원 parent와 control의 일치만 입증한다. 별도 token·일반 signing registry를 도입하거나 self hash를 worker에 넣지 않는다. 제품 고정 구성의 권한·보호 전제가 아직 없는 부분은 명시하고, 시험용 고정 구성과 제품 활성화를 분리한다. 현재 `PRODUCT_RELEASE=None`은 미완료 경계이며 최종 제품 해법이 아니다.

modoo.today `modootoday/software-qa/code-change-review-verification` 및 읽은 `references/verdict-table.md`의 동일 artifact/관측 분리를 적용했다. 이 문서 갱신은 권리 승인이나 제품 기능 통과가 아니다.

2026-10-21 중앙 배정 source-only 후속. [기존 배포 조사](vad-native-distribution-closure-2026-10-21.md)를 보완한다. 신규 실행·다운로드·빌드·native load·등록0이며 SDK union18 입력을 변경하지 않았다.

| 항목 | 확인한 원 근거 | 남은 최소 조치 |
|---|---|---|
| 역사적 BIN 구분 | runtime/coordination/ai-resume-vad-r1/input-tool-pins.json의 F763A4CF…/41413120B는 준비 단계다. runtime/verification/ai-vad-build-r1/normal-binary-pin.json의 최종 r3는 09F77408…/41708544B다 | 최종 r3 normal-pe-dependencies-r3.txt와 원 BIN을 사용한다. 두 시점 차이를 변조로 판정하거나 옛 pin을 덮어쓰지 않는다 |
| 실제 의존 범위 | r3 PE 원문에 DirectML, VC4, d3d12/dxgi 및 Windows/APIset import가 있다. 기존 loaded-runtime5.json은 정확히 선택한5 모듈의 실제 관측이다 | module5를 OS/전이 의존 전체 목록으로 승격하지 않는다. 설치 prerequisite와 제품 payload를 별도 확정한다 |
| native build 정보 | local ort-2.0.0-rc.14/src/lib.rs:189의 ort::info()는 GetBuildInfoString을 호출한다. frozen r3 model-spike/src에서 ort::info/GetBuildInfoString/GetVersionString 호출은 검색되지 않았다 | 기존 hardcoded wrapper 문자열로 native commit/flags/version을 입증하지 않는다. 향후 중앙 배정된 Rust 진단 후보로만 둔다. 이 함수도 native load/call이므로 현재 실행하지 않는다 |
| archive/build 동일성 | 기존 dist row의 archive 기대 hash와 캐시 폴더명이 대응한다. 이는 개별 lib SHA·upstream commit·build recipe와 별개다 | 원 archive→개별 payload→build source/config와 고지를 연결해야 한다. runtime reported build 정보만으로 전체 공급 근거를 완료하지 않는다 |
| runtime pin | 현재 static lib는359948106B이고 release runtime pin 상한은64MiB다. 최종 private BIN은 여러 실험을 포함한 도구다 | build-only lib를 runtime pin에 넣지 않는다. 제품 worker executable과 runtime pin/worker pin의 동일성 및 DLL/notices 역할을 명시해야 한다. 현재 private tool을 제품 worker로 배포하지 않는다 |
| 최소 버전 | release.minimum_version은 현재 bounded 선언 문자열이다. PE import·API31·DLL FileVersion은 Windows 최소 버전의 실제 검증과 다르다 | 엔진 ABI/버전과 OS prerequisite의 의미·비교 주체를 정하고 별도 검증한다. 현 선언을 실행 호환 인수로 표시하지 않는다 |
| 권리·고지 | wrapper license, 모델 README 선언, native ORT/DirectML/VC 고지는 서로 다른 책임이다 | 기존 조사에 남은 package별 정확 고지와 payload 대응을 PM이 인수하기 전 bundle/제품 활성화하지 않는다 |

최소 진행 순서는 현재 SDK admission gate의 원 결과 인수, 제품 worker wire/runtime pin 계약 확정, 공급·고지 근거 인수, 정확 입력과 자원 배정을 받은 별도 Rust worker/SDK·CLI gate다. 단순 cargo check나 parser 통과를 링크·native load·실제 취소/cleanup·품질 승인으로 대체하지 않는다. 새로운 엔진이나 다운로드로 현재 누락을 우회하지 않는다.

modoo.today `modootoday/software-qa/code-change-review-verification`의 동일 artifact/원 관측 판정을 위 표에 적용했다. 법률적 재배포 승인이나 새 품질 측정은 수행하지 않았다. 전체 목표와 도구 reset 불일치는 [현재 목표](../../../../../plans/player/ai/media-current-goal.md)에 기록했다.

## 실행 pin 접합의 소스 확인

현재 root의 `packages/media-vad-executor/src/application/release_manifest.rs`는 files.runtime을 VadProfile에 넣고, minimum_version은 bounded 문자열로 보존한다. `packages/media-vad/src/application/analyze.rs`는 ProfileAdmission 뒤 runtime 실제 bytes를 reader로 읽는다. 한편 `src/infrastructure/vad_job_attempts.rs`의 assemble_admitted_attempt_executor는 별도 VadAttemptConfiguration을 VadAttemptResources에 넘기고, build는 그 worker와 worker_pin으로 NativeTransportFactory를 준비한다. 이 조립 함수에서 admission의 profile.runtime과 configuration.worker_pin을 대조하는 분기는 없다. 단발 조립 `src/infrastructure/vad_job_files.rs`도 별도 configuration.worker_pin을 사용한다.

이는 현재 비활성 제품 후보의 계약 확정 과제다. worker 실행 전 자체 pin 검증이 없다는 주장이나 공개 operation의 우회 취약점 판정이 아니다. runtime이 worker executable을 뜻할지 별도 엔진 artifact를 뜻할지부터 확정해야 한다. static 배포에서 전자를 선택한다면 같은 trusted release record에서 executable pin을 공급하고 bytes/SHA 및 지정 identity의 일치를 검사하는 조립 계약이 필요하다. 후자를 선택하면 worker pin과 engine pin을 별도 역할로 두고 둘의 approved release 결속을 명시해야 한다. runtime bytes와 임의 별도 worker pin의 개별 검증만으로 결속을 인수하지 않는다.

후속 검증 후보는 (1) 정상 trusted release와 worker 결속, (2) 같은 크기의 다른 worker digest 거절, (3) 다른 release의 유효 worker 교차 결합 거절, (4) 계약에 따른 identity 불일치 거절이다. 모두 worker spawn 전 실패해야 하며 현재 union18의 예상43에 추가하거나 통과로 세지 않는다. 이 source-only 조사에서는 코드를 변경하거나 새 시험을 실행하지 않았다.
