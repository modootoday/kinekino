---
id: evidence:kinekino/20261029114227
title: 검토 원본·고정 도구·Job 예약의 공동 소유권
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 검토 원본·고정 도구·Job 예약의 공동 소유권

후속 완료: 234 공통 admission3시험 통과, fmt --all --check 통과, 235 제품 및 owned-process(test-support 포함) all-targets clippy -D warnings terminal=true/exit0. 기존 dependency blitz-paint의 deprecated warning은 담당 WIP로 남겼다. 현재 stage된 경로 없음. 이 결과를 SDK 재생·전체190·정상 bins·실제 품질 성공으로 확대하지 않는다.

232 후속 실제시험1 통과: 봉인 Sprite source102392846바이트·실 복원 mpv SHA·실 빈 Job을 typed admitted owner 하나에 보유했다. factory/reader/store drop 후 source/tool/lock의 writer-open은 sharing violation32, owner drop 후 lock은 재취득 가능함을 확인했다. allocation runtime/build-cache/tmp/review-admitted-30128-1793028301000996100 보존. media process/playback_ready/quality 성공은0이다. Cargo.lock SHA는 기존 B2E3D540… 그대로다. 233 영향시험은 실행 경로 입력 오타로 child_spawn 전 실패했고 보존했으며234로 교정했다.

2026-10-27 중앙 PM. 기존 WIP·소유권·원 증거와 C cache/heavy1/CPU2 정책을 유지한다.

## 구현

공통 application의 admit_review_runtime은 기존 profile/입력 예산·lease 검사와 snapshot 검사를 유지하면서 ReviewInputWithRuntime<L>로 concrete owner를 보존한다. 종전 boxed AdmittedReviewInput은 type alias로 유지한다. GuardedReviewMedia::admit은 같은 file/root/case session과 해당 입력을 한 GuardedAdmittedReviewMedia<L>에 옮긴다. 필드와 생성 경로는 외부에서 위조하지 못하며 wire Default/Deserialize·unsafe Send는 추가하지 않았다.

ReviewRuntimePool은 명시한 단일 private root의 review-runtime.lock, 고정 TrustedMpvExecutable, 실제 PreparedFamilyJob을 함께 보유하는 ReviewJobReservation을 발행한다. tool SHA·committed-memory 수치·affinity CPU 수는 profile과 일치해야 하며 현재 보수 정책상 최대2 CPU다. OS의 affinity 허용 범위와 실제 Job 수치는 기존 PreparedFamilyJob이 재확인한다. pool이 해제돼도 permit의 root/tool/lock/Job은 유지된다.

이 lock은 같은 root를 사용하는 review runtime 호출의 독점이다. 다른 root/다른 producer까지 중앙 heavy 배정을 강제하는 전역 scheduler라는 주장을 하지 않는다. 중앙 PM은 기존 heavy1 배정을 계속 소유한다. 이 owner는 아직 비어 있는 Job의 사전 입장이며 실제 child가 시작된 후에는 별도 running owner/physical exit 정리로 전환해야 한다. IPC·seek·clock·Verified·GUI 또는 AI 품질 완료를 뜻하지 않는다.

## 의존 도구 복원

새 다운로드나 시스템 설치 없이 보존된 runtime/mpv/mpv-x86_64-20261017-git-dde0671072.7z에서 mpv.exe만 runtime/toolchains/mpv-dde0671072-review-r1로 복원했다. archive SHA2821D9EEF879215E99975FAE9616E1777F5E3747168EDD3A571C41E21B8ABBBC, exe SHA F8BF7E8FD8D6696E35441C55091498060E4594F724A132FD6C46579341BC0790은 [역사 provenance](../playback/sot-005-playback-validation.md)와 일치한다. 다른 physical 파일이며 과거 FileID나 실행 성공을 새 호스트로 승계하지 않는다. 원 archive/doc/라이선스 자료는 보존하고 PATH·registry·전역 Cargo/Rustup은 변경하지 않았다. 배포는 범위 밖이며 재배포 closure 승인이 아니다.

## 검증 상태

231 첫 실제 조립 빌드는 일반 lib에서 owned_process를 찾지 못해 실패했다. 기존 root Cargo는 이 crate를 dev-dependencies에만 선언했다. 새 production runtime이 소비하므로 동일 로컬 path를 dependencies로 옮겼고 dev 중복은 제거했다. 외부 crate를 다운로드하거나 버전을 바꾸지 않았다. 첫 실패 receipt/stderr를 보존하며232로 후속 검증한다. 230 unit 성공은 일반 lib 조립 성공으로 계상하지 않는다.

230 actual empty Job/lock + inert tool fixture1 통과. factory drop 후 Job 검사와 tool/lock writer-open 거절, permit drop 후 해제를 확인했다. inert mpv.exe는 실행하거나 PE 지원으로 계상하지 않았다. fixture runtime/build-cache/tmp/review-runtime-pool-35084-1793028060225114200을 보존했다.

231은 봉인 Sprite 실제 source와 복원한 고정 mpv·실 Job의 공동 보유 시험이다. 완료 결과는 별도 receipt로 확인하며 아직 성공 계상하지 않는다. 전체190/정상 bins의 최신 통합 기준은 여전히 과거 SOURCE396이고 이번 변경을 포함하지 않는다. 원 Sprite/actual9의 선택 source 및 판정은 변경하지 않는다.

기존 적용 modoo.today test-fixture-and-mock-build/code-change-review-verification의 실제/합성·factory/원 owner·pin/실행 구분을 적용했다. [이전 Job 근거](review-prepared-job-20261027.md) 및 [경로 custody](review-playback-object-custody-20261027.md)를 함께 따른다. 수동 GUI·추론·다운로드·WARP·push 보류 유지.
