---
id: sot:kinekino/worker-production-admission
title: Worker production admission의 조건부 접합과 부족 입력
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Worker production admission의 조건부 접합과 부족 입력

2026-10-21 PM source-only 후속으로 기존 VAD root 경로를 읽었다. 실제 심볼은 FixedVadRelease/GuardedReleasedVadInputs·PreparedVadAdmission·외부에서 주입하는 ProfileAdmission·LoadedRuntimeAdmission이며, 조사한 VAD 제품 경로에 별도의 TrustedRelease 타입이나 concrete non-fixture production issuer는 발견하지 못했다. 따라서 이름·해시·creator/control 관측을 이미 성립한 승인으로 취급하지 않는다.

별도 `runtime/development/vad-worker-release-binding-r1` source6 `0B3598D885655E69552D5C81CBE491690BA9778B1BAA1FB272206025454999B9`/before6 `B7D386DB69D6BFABAFEEB72CD40082811539E83804D2351C18C78C038BFC941C`를 동결했다. 기존 union34 위 lib/test/native adapter3 경로를 명시 override하고 세 경로를 추가하는 union37은 `E0458DC529E7D02ED34262A1DB164E38DF76B001FF0DE3BF00DAB75F043ED24D`다. 원 SDK12/child17/fixture6/entry/stderr/parent raw는 변경하지 않았다. 이 후보를 SDK actual8 실행 입력에 묵시 합성하지 않는다.

ReleaseInputs는 기존 승인 full profile/원 manifest pin/ProfileAdmission을 빌릴 뿐 승인 객체나 registry/token을 발급하지 않는다. run_released_request는 원 admission→guarded 실제 manifest를 기존 bounded hash/EOF/release decoder로 확인→독립 expected full profile 및 기존 static worker binding 대조→WorkerArtifactFiles→기존 run_request/P0의 재승인·실제 model/license/executable/WAV hash로 연결한다. serve_released_request는 같은 control/IO 경로를 재사용하고 native-only wrapper는 기존 OrtVadEngine<LoadedRuntimeAdmission>을 조립한다. 현재 main에 이 입력을 공급하는 concrete issuer/native verifier가 없으므로 main의 Unsupported는 유지한다.

native module guard는 session/environment보다 먼저 선언하고 같은 before_inference 시점에 초기화하도록 후보에서 조정했다. 이후 native guard가 실소유 module handle을 보유하더라도 세션 파괴보다 일찍 해제되지 않게 하는 선언 수명 배치다. 현재 concrete guard와 native 실행은 없으므로 관측된 crash 수리나 actual lifetime 검증으로 주장하지 않는다.

새 authored5는 원 승인 거절이 manifest open보다 앞서는지, 원 full release의 unit Partial 연결, 같은 크기 manifest 변조·유효한 다른 release·실제 다른 executable bytes 거절을 다룬다. 직접 Rustfmt6만 통과했으며 Cargo/타입/등록/시험/native/OS/모델/다운로드는0이다.

HANDOFF.md의 실제 call-path 표와 ABI-and-deployment.md는 다음 부족 입력을 분리한다: 원 부모 승인/lease에서 독립 child approval로 이어지는 concrete 구성, 승인 배포의 actual executable/DLL/prerequisite/notice/권리/minOS, 실제 module identity/ABI/version/build, 원 cutoff 중 native IO와 session 수명, 실제 SDK/CLI/model 품질 인수. ORT API31 선언·cache버전·QPF/creator 일치·합성 fixture 통과는 이 입력을 대신하지 않는다. static import는 main 전 loader에 관여하므로 parent 사전 배포 검증과 child 실제 loaded-module 검증을 함께 설계해야 한다. 전역 DLL 검색/PATH/시스템파일은 변경하지 않았다.

modoo code-change-review-verification 본문9F73865F…/참조9D9619A8…와 Rust-pro를 적용했다. 사실/조건부 코드/미검증을 구분하며 전체 AI·미디어 목표를 축소하거나 완료하지 않는다.
