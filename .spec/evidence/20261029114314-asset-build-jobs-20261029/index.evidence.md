---
id: evidence:kinekino/20261029114314
title: 자산 제작의 원 Jobs 연결 — 2026-10-29
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 자산 제작의 원 Jobs 연결 — 2026-10-29

선택 취소 경계·staged recovery metadata·정상 재기동/명시 attempt2 후속 결과는 [R230 검증](asset-jobs-cancel-retry-20261029.md)을 따른다. 아래 R229 성공과 R228 첫 거절은 각 원 입력의 역사로 유지한다.

[계약](../../player/application/production/asset-build-jobs.md)의 private overlay 구현이다. AGENTS SHA `40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C` 재확인. source375 canonical 적용/commit/push 없음. Core/Meari 담당 소유 파일과 WIP는 변경하지 않았다. 원 Native 제품 파일은 preimage-r226-asset-jobs, preimage-r227-producer-pin, preimage-r228-cpu-route에 보존했다. 의존성 변경·추론·모델/미디어 다운로드·수동 GUI·WARP 변경 없음.

## 첫 실행과 선택 계약

R227은 원 owned candidate prototype과 retained immutable file writer를 추가했고2989fmt/2990snapshot/2991native strict/2992default strict 종료0. R228은 실제 현재 producer SHA/원 executable guard와 recipe를 host pin에 추가했다.2993fmt/2994snapshot/2995native strict/2996default strict/2997lib no-run/2998요청 경계1시험 종료0.

R228 첫 실제 실행2999는 원 Core settlement gate에서 RevisionConflict로 종료1이다. NotStarted CPU observation은 Graceful process settlement가 아니므로 publication permit을 받지 못했다. 원 Core 규칙이 거절한 정상 경계이며 가짜 Exited/exit0/family/EOF 관측으로 바꾸지 않았다. 실제 last-job.json은 needs_input/publication_started=false/output=null, 원 shutdown error=null이다. 해당 `source-sdk-r228-asset-jobs-r1` artifact directory의8개 staged data 파일과 원 diagnostic/producer exe `DABDCDBECAB8EA6B57C699DD42D4B0AD27A7AB326018CC5240ABB9690A9CCDDE`를 보존했다. publication.json은 없었다.

R229 선택은 기존 카드 CPU 제작과 같은 원 Jobs execute/control 계약이다. process-owned opt-in은 None이며 prototype은 미인수 WIP로 보존한다. Core의 소유권을 인수하거나 gate를 완화하지 않았다. 같은 source/raster/output을 사용하고 원 control의 reserve_publication을 한 번 호출한 뒤 marker 발행·begin_verification·실제 검증을 수행한다. 이 CPU 성공을 process-owned 인수로 계상하지 않는다.

## R229 입력과 실제 결과

inventory2493개 SHA `9D640CDC87D13DCFEC1801D88F42C033B73AF3ACEB7BE1AC17D714014911C181`, SOURCE.md SHA `53038386693C00BF9ED3DB616D09CAEA75BBB4BA265F93526CB6B795AF4541C8`.3000fmt/3001snapshot/3002native strict/3003default strict/3004lib no-run/3009fmt check 종료0. 원 exe SHA `4419AEDB29ECAECDD8B4654BEC6B3610A8D698360CCC2CFD376A602A726FECDB`는 fixed-r229/native-core에 보존했다. cache PDB는 유지하고 추가 복사는 하지 않았다. 기존 Meari deprecation 경고는 기존 소유 범위로 유지했다.

3005 strict 요청/host path 주입 거절1통과.3006 기존 PNG를 사용한 원 Jobs 실제1통과(16.64초): root `runtime/verification/source-sdk-r229-asset-jobs-r1`. 고정 R215 request와 R212 static PNG pair를 원 admission/raster/codec으로 소비했다. 캡처 후 head2를 저장해도 선택한 publication1/원 pivot을 유지했다. 잘못된 expected source SHA는 제출 전 거절했다. 원 job Completed/publication_started=true와 actual publication bytes/hash, 원 attempt/input/source/submission binding, 데이터8+marker1파일의 전체 SHA/크기를 확인했다. 원 source/project bytes도 보존됐다.

publication SHA `2bf919e524cdbb8417dee6e033fefc6071d21860e34c78998ffd05f3b67e6072`. 같은 idempotency key 재제출은 같은 job/created=false였고, 원 worker shutdown이 완료됐으며 read-only 저장소 재열기에서 같은 output SHA를 확인했다. 종료 후 source image/mask 쓰기 열기가 허용되어 정상 해제도 확인했다. actual producer pin/host config는 host-config.json에 보존했다. 품질/색 승인·CLI·GUI는 false다. source PTS는 static PNG의 null이며 실제 video 관측을 뜻하지 않는다.

3007 기존 immutable publish를 포함한 project 저장/typedSDK/capture13통과·기존PNG2ignored.3008 codec2통과. 이 수를 전체 제작 인수 항목으로 합산하지 않는다. 기존 immutable publish는 기존 Write+DELETE 권한/동작을 유지하고 새 retained variant만 READ를 더해 원 writer handle을 그대로 보유한다.

## 남은 범위

3010 사후 custody 종료0: R229 실제2493개 source content/identity unchanged. 이 현재 입력의 실제 CPU Jobs 결과와 과거 R228 process-owned 실패를 분리했다.

Native artifact 독립 inspector/reconciliation, SDKCLI host/registry/실제 명령 연결, cancel-before-CAS/중간 취소·CAS 거절의 staged recovery metadata·late error·권한/disk-full/tamper/CloseHandle fault·실제 ACK 유실/비정상 중단/수동 attempt2·다중 프레임/batch/한도는 별도 미완료다. process-owned CPU settlement는 원 Core 담당의 계약 확장 없이는 지원하지 않는다. 외부 프로세스 성공을 가짜로 만들지 않는다. source bundle 재편집·mask editor/undo·projection·GUI·사용자 AI 품질도 남는다. 최신 지정 전체190/정상 bins는 R221이며 이번 lib/native CPU 실행으로 대체하지 않는다. 전체 제작 인수0/5·AI 품질0/9와 전체 goal을 유지한다.

기존 modoo.today QA fixture/변경 검증/verdict의 첫 실패 보존·실제 경계/소유자 증거 구별과 image-postprocess의 원본 불변/출력 재검증을 적용했다. 정확한 출처·SHA와 적용 판단은 [원 입력 검증](asset-input-admission-20261029.md)에 연결한다.
