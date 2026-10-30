---
id: evidence:kinekino/20261029114731
title: IPC B 고정 후보 OS10 종료 — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# IPC B 고정 후보 OS10 종료 — 2026-10-27

격리 rebase 입력32 SOURCE150FF75D5BAAAEC3A9B737E1DA723E47AB4C0250FEF5C78865C0AD5AAD9750DA의 비OS5에 이어 실제OS10을 모두 확인했다. 첫 echo844와 이번852~860의 exact9를 합친 **이 후보의15사례**이며, root 제품·원 raw13·전체 다운로드/HTTP/replay 또는 모든 OS 경로 인수가 아니다. private overlay/WIP/소유권·과거 실패·보류 유지.

## 고정 producer와 경로

원813 Cargo JSON에서 나온 test의 보존 copy `runtime/build-cache/native/source-ipc-retention-r1/fixed-r4/source_ipc.exe` SHA1745B48B46B1B96D964993B0B24EE4FE75098EC03CD95EB3435FE0746A728B67/1199616B를 선택했다. 실제 compile-time child는 원 debug/owned-process-test-child.exe SHAC6D67C3E4744BAA5F79F2AAB3174A811217C818A0FF0B5E5F3F1B68643CBF134/392704B다. 중간 R5 alias3BD를 선택하지 않았다. 실행기는 별도 IPC 없는 baseline 고정7E7AD258, sourceF266556C/41이며 자기 candidate Supervisor로 감독하지 않았다.

operator index `runtime/verification/source-ipc-b-os-r4-index.json` SHA14F4BC584A6E65B6DB9943F85CA0E05E344861151DC634E1780AF2A83111AF32는 guarded 데이터이며 권한/컴파일 증명이 아니다. 각 outer outcome에 실제 세 artifact의 path/bytes/SHA/FileID를 보존했다. total45s/cleanup5s와 inner 원5s/cleanup1s는 launch 전 한 번 설정했고 연장하지 않았다. 단일 heavy/conservative2/CPU2/BelowNormal·C tmp·네트워크/추론/다운로드/수동GUI0.

## 실제 exact9 결과

모든 outer는1pass/exit0·ownership complete·first_error=null·원 test child0/familyEmpty/nonforced·raw EOF를 확인했다. 예상 panic/진단 실패가 inner 정상 작업 성공이 된다는 의미는 아니다.

| 로그 / 정확 case 접미사 | outer test PID | 실제 inner 관측·판정 |
|---|---:|---|
|852 control_progress_is_independent_of_a_backpressured_data_write|36720|원 data write가 실제 Pending에 진입한 단언 후 control write 완료. inner33028 exit0/Empty,io_reaped/local endpoints closed/had_error=false|
|853 cancellation_reaps_original_pending_read_without_claiming_family_ack|27280|원 read Pending 뒤 취소/reap. 별도 원 child31996 exit0/Empty와 IPC close를 기록. pipe 취소만으로 family ACK를 대신하지 않음|
|854 peer_eof_is_observed_separately_from_original_family_close|27072|실제 IoPoll::Eof·data_eof=true, inner11608 exit0/Empty. control_eof=false를 덮어쓰지 않음|
|855 partial_launch_failure_retains_same_assigned_owner_and_ipc|16064|원 AssignedSuspended 실패 owner+IPC 보존 후 정리. inner35432 exit124/Empty, receipt forced=false. 정상 source 실행/정상 종료로 가장하지 않음|
|856 action_unwind_reclaims_same_ipc_and_family_before_resuming_primary|27784|동일 IPC/family 정리 뒤 원 panic 재전달. libtest/driver should_panic=true, inner36088 exit0/Empty. 일반 양성과 구분|
|857 diagnostic_create_failure_keeps_original_incomplete_owner_after_physical_close|15168|Rust 진단 생성 오류 주입,원 Incomplete/첫 오류와 물리close·재획득/EOF를 in-process 단언. receipt.txt 없음·두raw0B 보존|
|858 diagnostic_write_failure_keeps_original_incomplete_owner_after_physical_close|19832|Rust write 오류 주입. receipt.txt0B·두raw0B, 원 실패/Incomplete·receipt_recorded=false 단언|
|859 diagnostic_flush_failure_keeps_original_incomplete_owner_after_physical_close|12440|Rust flush 오류 주입. receipt.txt151B는 pipe metadata까지만 있으며 성공 process receipt 아님. 원 실패/Incomplete·receipt_recorded=false 단언|
|860 diagnostic_failure_retains_original_action_panic_and_incomplete_carrier|36656|원 action panic을 잡아 같은 CloseReport/Incomplete와 진단 write 오류 보존. should_panic=false인 내부 catch 검사. receipt0B/두raw0B|

앞의844 echo1은 [첫 OS 정본](source-ipc-first-os-20261027.md)에 원 sourcePID35936/test36104·48KiB·receipt483B와 raw를 보존했다. 이번 원 자료는 `runtime/verification/source375-datalab-b0-r1-logs/852~862-*`와 아래 새 verification roots다.

`source-ipc-{control,cancel,eof,partial,unwind,diag-create,diag-write,diag-flush,diag-panic}-os1-r1/outer`에는 정상 baseline driver의 원 verdict/process receipt/stdout/stderr가 있다. 각 `inner-observed/`는 원 TMP의 `inner/owned-source-{실제testPID}-0`를 정확히 복사한 자료이며 원 폴더도 보존했다. 무관 temp/PID를 scan/reopen/kill하지 않았다.

진단4개는 **실제 OS child/IPC + Rust 진단 오류 injection**이다. 실제 디스크 고장/권한 장애를 측정한 것이 아니다. 원 CloseReport/Incomplete/panic·reacquisition은 원 leaf의 명시 단언으로 확인했으며, 저장 실패로 없는 원 process receipt를 독립 archive에 존재한다고 표시하지 않는다. 검사 후 unit process가 끝나므로 in-memory carrier의 원 보유와 영속화된 raw를 구별한다. 누수 없음·hard realtime·전체 HTTP 정리 증거가 아니다.

## 마감·남은 범위

861 source32와862 baseline41의 content/identity 동일은 exit0다. 선택 test1745/childC6D/driver7E 및 root lockB2E3D540/제품player82DF7DC5/CLI1DF8ADD0 hash는 전후 유지됐다. root index는 stage0이며 root 제품 코드·전역 Cargo/Rustup·원raw13·중간 입력/실패는 유지했다. 이번 exact9의 예상 밖 실패는 없다. 옛 `[0]` 표기 오독 정정/중간alias 제외는 첫 OS 정본에 그대로 보존한다.

다음은 이 private B를 현재 source375 문맥에 어떻게 소비할지 exact preimage/소유권을 고정하고, C HTTP source worker와 D 원 browser replay 발급/소비를 연결하는 것이다. 지금 producer나 metadata만으로 임의 URL replay를 허가하지 않는다. 제품 download operation 등록·source물리ACK·실제 SDK/CLI 출력/취소/저장·HLS/재개 등은 미완료다. root 최신190/정상bins는 DFB0이며 이 OS15를 그 검증의 추가 전체 제품 완료율로 계산하지 않는다. AI 품질0/9·전체CLI/GUI·기존 보류 유지.

기존 읽은 modoo.today test-fixture-and-mock-build A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및 change-review9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B의 원 owner/단언/독립 raw·정상/expected failure 범위를 적용했다.
