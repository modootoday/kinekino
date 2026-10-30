---
id: sot:kinekino/native-release
title: Native release 잔여 근거
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Native release 잔여 근거

2026-10-23 PM 후속 source-only 배정. root `89c0fa0da2ea63aa82cd36b79ad7994e128ac06a`에 notice10 r3가 채택됐다. canonical10의 SHA는 동결 source10과 모두 일치한다. library7fb/lockE469 유지, 새 source 후보·S/T 변경·빌드·native load·모델·추론·다운로드0이다. 아래는 기존 근거의 최신 판정이며 추가 자료 탐색 결과가 아니다.

| 책임 | 확보·검증된 범위 | 남은 정확 공백과 다음 산출물 |
|---|---|---|
| 모델 출처·license | HF e71 revision의 model2243022B/SHA A4A068CD…5808과 같은 revision LICENSE1075B/SHA 2E63E9A3…520B 확보. 원 spike normal/cancel은 역사적 기술 실측이다. | uploader/변환 recipe·version의 실제 provenance와 고정 weights의 대응을 보존해야 한다. manifest provenance 문자열을 임의 작성하지 않는다. model/license bytes를 다시 다운로드할 근거는 없다. |
| 정밀도·실제 runtime | 공통 profile FP32/CPU 명명과 과거 spike 관측이 존재한다. 기존 Rust native 어댑터는 f32 input/state/output 및 길이·finite state를 검사한다. | 새 제품 worker의 native compile/ABI, 실제 graph IO/type/shape·전체 initializer 정밀도 검증은 미완료다. profile 설정을 actual 측정으로 전용하지 않는다. protocol encoder의 actual_runtime=null은 별도 typed 측정 생산 경로가 필요하다. |
| ORT 공급 chain | ort rc.14 SDK 소스와 dist.tsv의 archive 기대값, 현존 build-only onnxruntime.lib pin을 보존했다. 공식 ORT source402979a의 LICENSE1073B/TPN338088B 원 bytes도 확보했다. | 공식 source 고지가 현존 native archive/build의 정확 구성에 대응한다는 chain은 미완료다. archive 부재를 다른 cache 버전 합성으로 채우지 않는다. 359MB static lib는64MiB runtime artifact가 아니다. |
| DirectML·VC 모듈 | 과거 spike의 DirectML.dll 및 선택 System32 VC4 path/SHA 관측과 공식 버전 문서를 보존했다. | DirectML1.15.4 package→현존 DLL hash chain 및 정확 raw license/TPN, 새 worker의 실제 전체 module closure/startup phase/minimum Windows 값은 미인수다. 웹 약관 HTML을 raw notice로 넣지 않는다. 관측5개를 전체 closure로, 임의 현 프로세스 목록을 system whitelist로 채택하지 않는다. |
| Notice 소비 | 채택된 공통 per-file1MiB/aggregate1MiB/max16 정책과 parent/child/actual-file guard의 default actual31이 통과했다. 모델license64KiB/manifest64KiB/config8192B는 유지한다. | 기존338088B TPN 제한 충돌은 해소됐다. 이는 notice의 해당 native build 대응·배포 인수나 ORT load 성공을 입증하지 않는다. TPN을 분할·축약할 필요는 없다. |
| 독립 제품 release record | root의 ConfiguredVadRelease/guarded manifest/ParentSelectedRelease와 child 공급 계약은 있다. PRODUCT_RELEASE=None이다. | 전용 native worker test=false SHA/bytes, 정확 manifest SHA, module/notice pins·phase/system names/minOS의 독립 선택이 필요하다. 이전 private spike/default worker/fixture BIN을 record 빈칸에 넣지 않는다. 현재 record 수정 후보는 만들지 않는다. |

## 후속 경계

다음 실행 제안은 [제품 체크포인트](../../../../../plans/player/inference/vad-native-product-checkpoints-2026-10-23.md)의 별도 native worker 기술 gate다. source 검증→전용 worker BIN/hash→정확 bundle/manifest→독립 root record→normal SDK·CLI audio2 순서를 유지한다. worker에 자신의 SHA를 내장하지 않는다. 기존 사용자 기술 실측 허용은 유지하되 중앙 자원 배정과 실제 입력 공급 전 새 실행을 시작하지 않는다. 별도 법률 검토나 새 사용자 승인 절차를 임의 gate로 추가하지 않는다.

현재 source-only 결과에는 새 개발 코드가 없으므로 구현 preimage manifest도 없다. 다음 exact 구현 범위를 배정받으면 당시 root와 책임 파일의 before/new 여부를 다시 동결한다. UI/SDK/Meari 후보를 자동 합성하지 않는다. notice gate의 캐시4709 producer 미귀속2/all-cache=false도 그대로 남긴다.

정확 전체 pin·원 파일 위치는 [입력 준비표](vad-positive-input-readiness-2026-10-23.md), [문서 bytes](vad-release-document-bytes-2026-10-23.md), [한정 cache 조사](vad-native-cache-inventory-2026-10-23.md)를 정본으로 재사용한다. 이 문서는 중복 pin 정본을 만들지 않는다.

이미 본문과 필수 참조를 읽은 modoo.today `modootoday/software-qa/code-change-review-verification`(본문9F73865F…D64B, verdict-table9D9619A8…2A9)의 실제 artifact/선언/지원 판정 분리와 `modootoday/operations-cs/version-pinned-procedure-from-vendor-docs`(본문eb3f771f…bb23)의 version/document/payload 대응 원칙을 적용했다. 전체 SHA와 출처는 연결한 문서 bytes·notice 검증 정본에 보존돼 있다.
