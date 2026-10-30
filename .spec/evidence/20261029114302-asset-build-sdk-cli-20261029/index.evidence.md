---
id: evidence:kinekino/20261029114302
title: 자산 제작 SDK/CLI 통합 — 2026-10-29
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 자산 제작 SDK/CLI 통합 — 2026-10-29

[원 CPU Jobs 계약](../../player/application/production/asset-build-jobs.md)의 후속 private overlay다. 기존 소유권/WIP·원 증거·첫 실패·보류와 source375 canonical 미적용을 유지한다. AGENTS SHA `40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C`. preimage-r230-asset-cli와 preimage-r231-producer-path에 변경 전 파일을 보존했다. Core/Meari 담당 파일·전역 설정·의존성은 변경하지 않았다.

## 연결과 첫 실패

엄격한 asset-build-host/1 administrative 파일과 SHA 쌍을 --serve-jobs에 연결했다. 원 config/ancestor guard와 실제 producer pin을 유지하며 registry/shared project store/Jobs/batch를 같은 host에 연결한다. typed submit_asset_bundle과 jobs.submit schema는 공통 application의 AssetBundleRequest/BUILD_OPERATION을 소비한다. 직접 production.build_assets 명령은 Unsupported 그대로다.

R231 inventory2496개 SHA `F043469CED5CD1454AF84217CDF5140737F012999A3320C81024C8221F59118C`, SOURCE.md SHA `E2C6A8455BDCE580C0F0DBE9E1A4B50A72FA6AA0C3CDC6AFA5221E45B88E20A1`.3022/3023/3024fmt·3025snapshot·3026native strict·3027default strict·3028combined lib/Jobs/unitCLI no-run 종료0.3029host option5시험 통과.3030 첫 실제 CLI는 준비 단계에서 immutable local storage PermissionDenied로 종료1이며 성공으로 계상하지 않는다. 원 response와 observed-terminal host 증거는 `runtime/verification/source-sdk-r231-cli-r1/asset-build`에 보존했다. 원 normal CLI SHA `D193BFD43D09826270DAA4E1CD23F5EC8C852A781949B45E15610393477E79BB`와 GUI SHA `5D40E3903BC65309025AFD674AFBD696BC1BC80831B80C6C7FCBB330463E14D1`는 fixed-r231/native-bins에 exe/PDB로 보존했다.

R232는 current_exe의 local VerbatimDisk 표기만 ordinary disk 표기로 바꾸고 동일 실제 파일의 SHA/핸들을 검사한다. UNC/VerbatimUNC/상대/ParentDir는 거절한다. 저장 루트의 전역 정책은 완화하지 않았다. producer parent 오류에는 단계와 원 producer path/details를 보존한다. 최초 응답은 정확한 실패 경로를 포함하지 않아 원인을 전수 확인했다고 주장하지 않으며, 정규화의 단위/실제 재시험을 구분한다.

R232 inventory2496개 SHA `B7FE19BDF136E58B0F439693043AEB77EBB1DA47A236368EF6480617A25CA907`, SOURCE.md SHA `28B816FDC34CC214E6E4819832FE992C60E8852DE075FEB6853DC2B7052D2F5E`.3031fmt·3032snapshot·3033native strict·3034default strict 종료0.3035는 combined no-run 재실행이며 실제 결과 확보 전 인수하지 않는다. 기존 Meari deprecation 경고는 유지했다.

## 인수 범위

3048 최종 사후 custody 종료0: 실제2496개 source content/identity unchanged. 실제 CLI/Native 회귀·190·정상 bins를 동일 R232 입력 경계로 확정한다.

R232 실제 결과:3035combined no-run·3036producer 경로 정규화1시험·3037host option5시험·3038실제 정상 CLI/typed1시험 종료0. 실제 root `runtime/verification/source-sdk-r232-cli-r1/asset-build`. owned-requests10개를 확인했고 CLI의 capabilities/schema/project create/save/과거 inspect/경로 주입·foreign pin 거절/submit/get/replay와 typed inspect/submit을 실행했다. 원 host ready는 자산+기존 subtitle 두 operation을 표시했고 config file 쓰기가 거절됐다. CLI/typed 두 Jobs가 완료됐으며 stdin EOF 뒤 원 process exit0/not forced/family Empty와 pipe/sdk joined·completed_total2를 확인했다. actual marker SHA `65090f06759cdd9f0300d0f46e3ab280baf1839e8960ca36eccebe942bf77ff9`, config SHA `d1fafead0a63fb882b13730a6b11489e677e3d42dc090f92c317c8c7bfa8e1a2`. 각 작업은 선택 publication1을 유지했고9파일/원 pin·재조회가 검증됐다. 직접 production command는 미지원이다.

현재 원 normal CLI SHA `7DDDA67D3FB99AC57A8A9D5D28E3A79733B5FBB634EA5242D65CE4DBBDD091A0`, GUI SHA `162EC06B6DB0108C7D7D030E4F534E54FF085B44644045A7DA0C0BA01546B10D`는 fixed-r232/native-bins에 exe/PDB로 보존했다. lib SHA `A3DCE38C55CF2F4AECDFB80F28D3292756BDBDDED73B78054257756626BFBE09`, Jobs SHA `6CBAE5315E4778391C39D649D77AF94A7C0ED27FFBEE16508B952C0552E2DAB3`, unitCLI SHA `ABE6D75B73F9A234C35C2F49DAC0B3C46CE9FB3557ED39C8C3AB46A090973132`는 fixed-r232/native-core에 보존했다.

3039정상·3040CAS 전 취소/정상 host 재기동/명시 attempt2·3041발행 후 늦은 취소 실제3시험도 같은 R232 lib에서 종료0이다. 각각 source-sdk-r232-native-normal-r1/native-cancel-r1/native-late-r1 증거이며 기존 R230 성공을 대신 계상하지 않았다. 취소된8파일과 원 오류/recovery metadata, stale retry/자동 재실행 부재·동일 입력/재열기 및 늦은 취소 PublicationInProgress가 확인됐다. 이것은 선택 CPU 경계의 시험이며 비정상 crash/ACK 유실이 아니다.

3042Product145·3043Core18·3044Native26·3045OS1은 모두 execution_result/producer_integrity Ok, 동일 R232 SOURCE.md SHA다. 결과는 `runtime/verification/source-sdk-r232-exact190-r1`이며 Core/Native/OS는 기존622/624 컴파일 producer를 실제 재실행한 것이고 새 컴파일로 계상하지 않는다.3046정상2bins·3047fmt check 종료0, 정상 bin 해시는 위 고정본과 다시 같았다.

R232는 구성된 host의 jobs.submit 경로를 실제 검증한 통합 이정표다. 원 source는 static PNG/수동 mask·PTS null이며 추론/GUI/색/사용자 품질 승인과 다르다. artifact inspector/reconciliation·self-contained source bundle·마스크 편집/undo·실제 video/대규모 sequence·projection·모든 fault/batch/비정상 crash/ACK 유실·전체 CLI/GUI 인수는 여전히 남는다. process-owned CPU도 미인수다. 전체 제작0/5·AI품질0/9·전체 goal을 유지한다.

기존 modoo.today QA fixture/변경 검증/verdict 및 image-postprocess의 원본 불변/출력 재검증을 적용한다. 출처·SHA와 Rust 적용 판단은 [원 입력 검증](asset-input-admission-20261029.md)에 연결한다. 불필요한 모델/미디어 다운로드·GUI·push·배포는 하지 않는다.
