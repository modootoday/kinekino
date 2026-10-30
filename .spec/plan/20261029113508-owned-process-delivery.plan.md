---
id: plan:kinekino/20261029113508
title: owned-process 공통 supervisor 대기열
status: active
created: 2026-10-29
updated: 2026-10-29
---
# owned-process 공통 supervisor 대기열

최신 실행 상태: [생산자 검증](../../sots/verification/runtime/owned-process-producer-2026-10-21.md)은 고유16과 r4 strict/fmt·root 등록 strict를 인수했고 source/보호 차이0으로 heavy/shared를 반환했다. 최종 source19는 r4 `7BE399D706B9C0D36FE3CB9C5336808375DDBE11B556DE181AE76012586CC285`다. root 채택/ignore 책임 경로 추가는 PM 소유이고 소비 helper 이관은 별도 source-only 대기열이다. 아래 작성·실행0은 최초 동결 이력이다.

최신 source-only 상태: 독립 staging에 신규 crate19파일과 pure7/명시 Windows7 회귀를 작성했다. 실제 compile/실행은0이며 source 동결 후 독립 감사를 요청한다. 아래 순서는 실제 gate와 소비자 배선 대기열로 유지한다. 원 제안 보관 위치는 Meari checkout이고 현재 이 root 문서와 root sots가 정본이다.

동결 pin은 `5D327CB95A39D289B76DCCC50AF7EB27B80F719199EB8A9DEFC26B74B17BBF14`다. [source 인계](../../../runtime/coordination/owned-process-r1/handoff.md)와 source19/before19/pins/미적용 root registration diff를 함께 보존했다. 직접 fmt/check 및 whitespace 점검만 수행했으며 독립 감사 인수를 기다린다.

감사 중의 구체적인 다음 배선은 [소비자 호환 계획](owned-process-consumer-wiring.md)에 분리했다. public 성공 반환값, fallible owner 반환의 한계, 원 raw 이름과 offset, file stdin/host EOF, 절대 budget 및 환경 cap을 명시했다. source19와 lease helper는 수정하지 않았다.

현재 감사 수리 후보는 r2 source19 `3F31B613725CA797B722FAC54F9F68C4F603AD7F0C5C47C841A112873AE511A8`다. 원 r1은 불변 보존했고 늦은 sink/ResumeThread 후 deadline 재검사만4파일에 반영했다. 작성 회귀16/실행0이며 독립 재감사 이후 최초 compile/자체 Job gate를 배정받는다. 기존 미적용 Cargo registration과 소비 호환 계획은 바꾸지 않았다.

최종 독립 읽기 감사는 r2를 인수했다. [생산자 gate 준비](owned-process-producer-gate.md)에 최신 root35d257c 대조, 미적용 Cargo before/patch, 최소 offline lock 갱신, pure integration8·Windows ignored8의 정확 명령과 nested Job/sentinel 소유를 고정했다. 현재 SDK가 heavy를 소유하며 새 실행·등록은0이다.

[제안 정본](../../sots/runtime/owned-process-supervisor.md)의 소스·Cargo 범위다. 현재 설계만 작성했고 제품/helper 편집·Cargo·실제 child/Job·미디어/AI/브라우저 실행은0이다. UI13/C A11 및 현재 자막8 lease와 독립이다.

## 소비자 배선 순서와 lease

1. packages/owned-process의 core 상태 전이와 Windows adapter를 별도 후보에 작성한다. resource-runner launch/receipt 오류 분리와 snapshot_cli_evidence raw 계약을 추출할 때 기존 semantic 정책은 이동하지 않는다.
2. Rust test child는 같은 패키지의 기본 비활성 test-support bin으로 둔다. ready 미출력/무개행/초과출력/잘못된 record/EOF 무시/자손 spawn/부모먼저exit/자손 stdout상속/명시 실패를 지원한다. 외부 브라우저·mpv·미디어 대신 자체 Rust 자손만 사용한다.
3. 순수 fake clock/family adapter 시험: 절대 deadline 비리셋, overflow, 잔여 budget 전달, original+cleanup+receipt 오류 보존, Complete/Incomplete 구분, Drop nonblocking을 확인한다. 이 시험을 실제 Windows containment 증거로 세지 않는다.
4. 중앙 실행 배정 후 외부 runner 아래 실제 nested owned Job 회귀를 수행한다. suspended assignment/readback/resume, 명시 allowlist, ready timeout, inherited stdout 자손, 부모 exit 후 active family, own-job-only forced termination, ActiveProcesses0/원 child exit를 별도로 확인한다. 다른 sentinel child는 같은 부모에만 두고 own Job 밖에서 살아 있음을 확인하되 생성·정리도 별도 소유 handle로 수행한다. 전역 PID 탐색/kill0이다. assign/resume/query failure fault는 실제 singleton/global 상태를 오염시키지 않는 adapter injection으로 작성한다.
5. tests/support/snapshot_cli_evidence.rs를 우선 얇은 protocol/evidence wrapper로 전환한다. evidence root/call numbering/request bytes/hash/prefix complete/errors/기존 반환값을 유지하고 bounded process/I/O/family 책임만 위임한다. 기존 PNG/capture 의미 검증은 그대로 소비한다.
6. 자막8 lease 반환 및 PM exact scope 승인 이후에만 tests/jobs/fixture.rs와 playback_owner.rs, 필요 시 tests/jobs.rs의 import/wiring을 변경한다. Fixture::call_root는 request 파일 stdin과 finish를 사용한다. Host/NativeOwner::start는 ready record 반환, endpoint/hash 검증은 기존 helper에 남긴다. stop/stop_chain은 EOF+deadline+terminal record+family evidence를 소비하고 joined/native/provider 정책은 원 소비 코드에 유지한다. Drop의 wait/read_to_string은 제거하되 원 실패 evidence 보존을 확인한다.
7. tools/resource-runner는 검증된 EEA9를 불변 보존한다. 별도 후보에서 child_execution의 blocking ownership 부분만 공통 package로 옮길지 검토하고, budget/profile/admission/launch/receipt compatibility를 먼저 통과시킨다. runner 자신을 own-family Job에 넣거나 외부 예산 Job 종료를 추가하지 않는다. 실제 gate가 resource-runner 변경에 선행 의존하지 않도록 우선 fixture 소비만으로 containment를 제공한다.

## 호환성과 실제 gate 순서

새 패키지 pure tests → Windows 자체 Rust child/family → snapshot CLI 기존 bounded fixture 회귀 → jobs helper의 기존 fake/no-media 회귀 → default/native all-target compile·fmt·strict 순서다. source/manifest/lock/runner/보호 범위를 중앙 snapshot에서 고정하며 실제 선택 수0은 통과로 세지 않는다. raw receipt schema는 기존 필드를 삭제하지 않고 family 관측을 additive version으로 분리한다. 현재 resource-runner의 terminal을 새 family empty 필드로 재해석하지 않는다.

위 gate와 자막8 인수 이후 별도 PM 배정에서만 같은 endpoint의 실제3category CLI/mpv 검증을 수행한다. 같은 host owner/endpoint, 원 executable hash, 원 request와 terminal wire, category별 semantic 결과 및 EOF joined를 확인한다. timeout/forced cleanup은 family 격리 성공일 수 있으나 제품의 graceful 성공으로 세지 않는다. 현재는 실제 endpoint/mpv/Job 실행0이며 이 작업이 완료됐다고 주장하지 않는다.

마감은 원 source pin·Cargo 입력·raw stdout/stderr·ready/terminal prefix·process handle creation identity·nested membership/readback·childexit/familyactive0·미완료 owner·모든 원 handle terminal을 분리 기록한다. root/shared/index lease는 PM 배정 전 변경하지 않는다.

## r1 감사와 최초 Cargo 입력 고정

실제 root Cargo.toml/lock 및 helper는 아직 수정하지 않는다. 신규 crate manifest만 staging에 존재한다. registration 제안은 workspace members/default-members에 owned-process 추가와 root dev-dependency path 하나이며, root lock 생성은 중앙 offline 배정에서 실제 Cargo 결과로만 수행한다. standalone source가 현재 root workspace의 미등록 하위 경로이므로 --manifest-path만으로 독립 workspace 검증이 된다고 가정하지 않는다. 첫 gate 전에 PM이 격리 snapshot의 등록 입력/lock을 별도 동결해야 한다. 검증된 runner EEA9는 그대로 사용한다.

감사 중점은 원 deadline clamp, unassigned suspended 실패/Drop, 실제 membership/limit readback, handle 상속 allowlist와 attribute 수명, raw 파일 identity/독립 offset, Unknown 및 original/secondary error 보존이다. synchronous FS/OS 지연은 wall-clock 보장 밖이며 hard deadline을 위한 새 runtime/overlapped I/O 도입은 이번 범위가 아니다. Rust child 안의 의도적인 hang/wait는 시험 자극이며 supervisor Drop에 wait/read/join/sleep가 없는 것과 구분한다.
