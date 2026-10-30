---
id: evidence:kinekino/20261029114204
title: Windows DWORD 종료 코드의 손실 없는 정산 — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Windows DWORD 종료 코드의 손실 없는 정산 — 2026-10-27

## 실제 결함과 수정

[독립 source 정리 R11](source-sdk-independent-close-20261027.md)의 종료 관측을 검증했다. 기존 ProcessSourceOpener는 원 receipt.child_exit:u32를 i32::try_from으로 변환했다. 상위 비트가 설정되면 실제 child/Job/IPC/raw 정리가 끝나도 Pending을 반환했다.

오프라인 시험 worker가 고정16byte/명시 BodyEnd를 전송한 뒤 `std::process::exit(i32::MIN+1)`로 종료하도록 명시 fixture 경로를 추가했다. 실제 OS에서 DWORD2147483649(0x80000001)를 관측한다. 실제 메모리 fault나 NTSTATUS 원인 판별 시험이 아니며 HTTP/DNS/모델/GUI를 사용하지 않는다.

R12/1080은 원 source child36600/creation134375681826803157의 receipt가 child_exit=2147483649/Empty/nonforced이고 physical/raw/transport 정리가 완료된 뒤에도 Pending임을 재현했다. exact0pass/1fail/0.09s,외부 test37128 exit101/Empty/nonforced/raw EOF였다. 원 실패/producer/snapshot/receipt를 보존했다.

R13은 `i32::from_ne_bytes(exit.to_ne_bytes())`로 같은32bit를 전달한다. 원 u32 receipt는 바꾸지 않고 core에는 -2147483647로 관측한다. 역변환한 u32가 원2147483649와 같음을 실제 시험에서 확인했다. 숫자 범위를 이유로 Pending에 머물거나 nonzero를0으로 바꾸지 않는다. exit 부재·물리 종료 부재·unassigned 관측 부재의 Pending 정책은 유지한다. publication gate/원 취소 권한은 완화하지 않았다.

## 선택 입력/실제 게이트

변경 전 adapter·worker·시험 helper는 mirror runtime/verification/preimage-r11-high-exit 및 preimage-r12-status에 보존했다. 선택 input-r13 SOURCE.md SHA256 `6FD04606212B3298DE3AD7D563DC89DDF9500AA4EF4531534F922F62B881992A`,inventory `B526492B3AE67E03ACEE3ADE68B476885B5F54E78DFB168B00555C0864A19756`,2285files.

|로그|실제 결과|
|---|---|
|1077/1078/1079 R12|시험 추가 all-target strict/정상 fixture bin/lib no-run 통과|
|1080 R12|위 high-bit Pending 결함 재현 실패;고정 test07510FE1/108969472B·원 입력2B8382A9 보존|
|1081/1082 R13|선택 제품 all-target/시험 feature strict 및 lib no-run 통과|
|1083|정확한 원 DWORD 보존/종료 관측 OS1,1pass/910filtered/0.04s|
|1084|정확한 원 JobsCore high-bit 실패 정산 OS1,1pass/910filtered/0.15s|
|1085|정상 원 JobsCore Completed 회귀 OS1,1pass/910filtered/0.17s|
|1086/1087|전체 workspace fmt 통과/선택2285 content·identity 동일|

선택 `native/source-sdk-integration-r1/fixed-r13/sdk-source-tests.exe`: SHA256 `1E41C1B0E6EC72A58FA17D153F948AAE4AF32162E8E6E7C6675ECF206DC5ACEE`,108969472B. child는 원 R12/1078의 `fixed-r12/source-sdk-fixture-worker.exe`822E9B6E8D26AF735DEFDDE017621407E362371B65322A12A34EFE0337E3FF88/1000960B이며 R13 재빌드로 계상하지 않는다. R13 operator index39C004A7DDA43188C76D29852EF79B2279474469E973CA09DC73A15776172923/388B는 proof=false다. 원 C2 fixed driver7E7AD258이 두 artifact를 guard해45s/cleanup5s로 exact case를 실행했다.

## 성공의 범위

직접 source case `infrastructure::download::process_source::tests::live::high_bit_windows_exit_preserves_original_status_after_physical_close`는 원32bit 역변환·family/raw 종료·nonforced 관측을 확인했다. 별도 jobs case `...::live::jobs::high_bit_source_exit_fails_original_job_without_publication`는 실제 sealed coordinator binding/원 source/기존 settlement gate를 사용했다.

실제 job은 Failed/EngineFailed `Original execution did not settle successfully without force`,publication_started=false/output=null이었다. 원 diagnostic9A6D55B43080E148ED0C8AB94B66E189D3BE023FC49525124C628BE7710B4E30/801B와 원 u32 receipt를 보존했다. source permission/body/BytesProbe는 synthetic이며 실제 browser 권한·media decoder 검증이 아니다. 미발행 후보/partial은 정리됐고 원 retaining host 종료/guard1회 해제를 확인했다. source.check_origin의 보존 출처 검사를 작업 성공 승인으로 계상하지 않았다.

실제 jobs source child15756/creation134375684578946717,child_exit2147483649/Empty/nonforced. 원 final receipt/raw EOF/IPC reap/local close를 확인하고 parent control/data EOF=false를 유지했다. 외부 직접 test26544 및 jobs test35076은 exit0/Empty/nonforced/raw EOF/selected_one=true/ownership complete/first_error=null이었다. 정상 회귀는 Completed 경로를 유지했다.

각 `runtime/verification/source-sdk-high-exit-os1-r12`, `source-sdk-high-exit-os1-r13`, `source-sdk-jobs-high-exit-os1-r13`, `source-sdk-jobs-publish-os1-r13`에 원 outcome/receipt/raw와 inner-observed copy를 보존했고 원 C tmp도 남겼다. 실행 후 고정 artifact hash·root lock B2E3D540·root/mirror AGENTS40C7F5A6 동일을 확인했다. C-only/locked/offline/j1/conservative2/heavy1을 유지했고 live handle terminal 전 다음 heavy 작업을 시작하지 않았다.

## 남은 범위

원 source unassigned rollback·중간/부분 EOF/Abort/timeout·실제 OS probe 및 동시 오류/panic 정리·CLI consumer/browser issuer/실제 HTTP는 미완료다. R11 취소와 순수4는 해당 원 입력의 실행이며 R13 재실행으로 계상하지 않는다. root 제품/canonical/source375 private overlay·기존 WIP/소유권·DFB0 190/bins·AI 사용자 품질0/9·GUI 시각 미인수와 기존 추론/다운로드/수동GUI/WARP/push 보류를 유지했다.

modoo fixture/mock·postcondition 지침을 재사용해 실제 DWORD/원 owner 관측과 synthetic 종료 원인/permission/probe를 분리하고 red→green과 미발행 실패를 보존했다.
