---
id: sot:kinekino/jobs-2-2
title: 원 Jobs의 자산 제작 경로
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 원 Jobs의 자산 제작 경로

[원 입력](asset-projects.md)과 [묶음 출력](asset-bundle-output.md)을 기존 media-jobs CPU 실행 계약에 연결한다. 전역 지원 등록·실제 SDKCLI 렌더·전체 제작 인수와 별도다. process 전용 owned gate는 CPU NotStarted publication을 허용하지 않으므로 기본 owned_executor는 None이다. 첫 거절을 보존하고 기존 카드 CPU 경로와 같은 원 execution control을 사용한다. Core 소유 파일은 변경하지 않는다.

## 고정 입력

AssetBuildHostConfig는 trusted Rust assembly의 project/artifact root와 고정 image/mask catalog 및 producer SHA다. 요청 JSON은 저장된 project revision/SHA·명시 missing policy·review_artifact_only만 선택한다. 현재 native 경로는 static PNG+명시 unreviewed acknowledgement이며 video frame·review record resolver 지원으로 확대하지 않는다.

open은 현재 실행 파일의 실제 SHA를 원 ordinary file handle에서 확인하고 expected producer SHA와 비교한다. 해당 producer/ancestor 핸들을 host 수명 동안 보유한다. host hash에는 실제 producer와 renderer/encoder recipe를 포함한다. producer가 달라지면 같은 경로/값이어도 과거 capture를 그대로 실행하지 않는다. 원 compile/source receipt와 runtime producer pin은 별도의 증거다.

capture는 engine을 시작하지 않고 원 project lease에서 publication/plan/host SHA를 canonical source blob으로 저장한다. request와 source byte 한도는 기존 Jobs 계약을 따른다. execute는 원 Core execution control의 input/source SHA·deadline을 확인하고 source blob 내용과 현재 project/plan/host를 다시 대조한다. 선택한 과거 revision을 새 head로 바꾸지 않는다. source PNG/mask는 실제 admission으로 열어 렌더·encode하고 원 owned output에 보유한다.

## staging과 publication

원 CPU worker가 전달한 execution control에서 identity를 검증하고 candidate를 보유한 뒤 staging을 호출한다. 고정 artifact root의 `asset-JOB-attempt-N`은 create-only이며 기존/불완전 디렉터리를 덮어쓰지 않는다. 각 데이터 파일은 원 write+DELETE handle로 no-replace rename하고 그 핸들을 그대로 유지해 readback·실제 FileID·byte hash를 검사한다. 기존 immutable file publish의 권한/동작은 유지하며 새 retained variant만 read 권한을 추가한다.

application의8개 파일(두 layer 예시)은 아직 완료 publication이 아니다. 기존 CPU Jobs 계약의 원 control.reserve_publication을 정확히 한 번 호출한 뒤 candidate.publish를 호출한다. 마지막 publication.json이 원 attempt/input/source/submission SHA와 묶음 manifest 및 전체 데이터 descriptor를 결합한다. marker도 원 handle의 create-only rename이다. 이후 같은 원 control.begin_verification과 candidate.verify를 호출한다. 별도 CAS나 가짜 process settlement를 만들지 않는다.

Core의 Verifying 이후 candidate.verify는 원 directory/FileID·entry set·모든 데이터와 marker bytes를 원 핸들에서 재확인한다. post-CAS 검사에서는 늦은 취소로 성공을 취소하지 않고 원 deadline을 유지한다. commit 오류에는 may_exist, 확정 commit 후 오류에는 committed와 artifact/publication SHA를 원 오류와 함께 남긴다. 부분 파일 자동 삭제·채택·재시도는 하지 않는다.

## 원 자원과 미완료

R231 CLI 구성은 `--serve-jobs --job-root ABSOLUTE_DIRECTORY --asset-build-config ABSOLUTE_FILE --asset-build-config-sha256 SHA256`이다. administrative 문서는 strict `asset-build-host/1`의 build 객체에 AssetBuildHostConfig를 담는다. 파일/ancestor 원 핸들을 보유하고 byte SHA·schema·catalog 한도와 실제 producer pin을 확인한 뒤 registry에 연결한다. config8MiB·image/mask 각각4096 한도이며 request JSON으로 설정 경로나 실행 권한을 공급하지 않는다. paired pin의 누락/중복/상대 경로/잘못된 SHA는 거절한다. VAD와 아직 미지원인 혼합은 명시적으로 거절하고 adapter를 조용히 누락하지 않는다.

구성된 같은 host는 shared asset project store와 기존 Jobs/batch 서비스를 같은 typed SDK로 제공한다. `jobs.submit` schema는 공통 AssetBundleRequest::schema를 쓰며 ApplicationClient::submit_asset_bundle도 같은 요청을 제출한다. `production.build_assets` 직접 명령은 계속 Unsupported이며 실제 작업은 jobs.submit의 operation으로 실행한다. 저장소 capability의 build_assets_supported=false는 저장 기능의 범위이며 구성된 Jobs operation의 지원과 혼동하지 않는다. 기본 host에 자산 executor를 암묵적으로 추가하지 않는다. 실제 CLI/typed/EOF 결과는 별도 검증 기록에서 확정한다.

CPU/ordinary-file 경로는 native child를 시작하지 않는다. NotStarted를 Exited/Graceful로 바꾸지 않으며 process-owned prototype은 admission하지 않는다. candidate는 원 source/output/staging 핸들을 release까지 유지하고 Rust 소유 값 해제로 해제한다. CloseHandle 오류/각 release fault·프로세스 비정상 종료·실제 ACK 유실·모든 권한/disk-full/tamper·다중 프레임/대규모 한도 및 중간 취소는 별도 인수다. partial_cleanup_supported=false다. CPU 전용 owned settlement를 추가하려면 원 Core 담당의 별도 계약 확장이 필요하며 이번 제품 어댑터가 그 소유권을 인수하지 않는다.

현재 선택 실행은 기존 CPU execute이며 process-owned prototype은 미인수 WIP로 보존한다. publication reconciliation/독립 artifact inspector·SDKCLI host 연결·source bundle 재편집·마스크 편집/undo·projection·GUI·AI 사용자 품질 승인은 남는다. 실제 시험 증거 없이 기본 command/capabilities를 supported로 승격하지 않는다. 기존 소유권/WIP/보류와 private overlay를 유지한다.

기존 QA fixture/변경 검증/verdict 및 image-postprocess의 원본 불변/출력 재검증을 적용한다. 출처·SHA와 Rust 적용 판단은 [원 입력 검증](../../../verification/application/asset-input-admission-20261029.md)에 연결한다.
