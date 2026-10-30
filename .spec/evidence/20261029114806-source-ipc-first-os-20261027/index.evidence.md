---
id: evidence:kinekino/20261029114806
title: Baseline OS self-tests와 첫 B echo — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Baseline OS self-tests와 첫 B echo — 2026-10-27

후속 [OS10 종료](source-ipc-os-completion-20261027.md)에서 보존R4 producer로 나머지9개를 실행했다. 아래 B OS1/9미실행 표기는 첫 echo 당시의 상태다. 중간 alias/표기 오독 정정 기록은 유지한다.

root 제품을 변경하지 않고 [baseline 실행기](source-ipc-baseline-runner-20261027.md)의 잔여 exact self-test7개와 [B 격리 후보](source-ipc-retention-isolated-20261027.md)의 첫 echo OS1을 실행했다. 실제 HTTP/다운로드·원 browser replay·GUI와 구분한다. 원 실패/중간 입력/WIP/소유권 보존, WARP/추론/다운로드/수동GUI/push 보류 유지.

## Baseline의 실제 추가7

고정 실행기7E7AD258/unitAE488649/guard child80888B58/index4289009C와 source F266556C/41을 그대로 사용했다. 원 budget은 top45s/cleanup5s, inner는 각 원 Request 값이며 phase 재시작/원 cutoff 연장0이다. 모든 top 결과는 exact1pass·exit0·physical_closed/selected_one/raw EOF=true, 원 outer familyEmpty/forced=false다. unit pass와 그 안의 의도된 실패를 구별한다.

| 로그 | 정확 case (`tests::` 아래) | 내부 실제 근거 |
|---|---|---|
|836|zero_selected_tests_is_failure_even_with_exit_zero_and_closed_family|원 child exit0/Empty에도 selected_one=false, `libtest did not report exactly the selected passing case`; 원 stdout running0/0pass|
|837|sink_failure_retains_original_incomplete_owner_after_physical_close|원 `original outer sink failure`·original_finish_incomplete/Receipt·receipt_error 유지, process_sink_records0. child0/Empty/rawEOF와 성공 판정은 별개|
|838|nested_family_is_observed_by_same_outer_owner|inner unit27508와 별도 baseline child9996을 원 nested Job으로 관측. baseline child exit124/Empty/forced=true, 별도 nested receipt 보존. 정상 child 성공 주장이 아님|
|839|timeout_stays_failure_after_original_family_is_forced_closed|원1s/cleanup300ms, child32932 exit124/Empty/forced=true, first_error=selected libtest exited nonzero, selected_one=false/rawEOF. 모델 성공이 아니라 실제 실패 판정|
|840|forced_exit_zero_with_one_pass_is_failure|inner child18020은 실제1pass/exit0이나 원 family forced=true. first_error=selected test family required forced termination, 원 TimedOut 및 passed=false를 unit이 검사|
|841|delayed_verdict_keeps_stored_pre_cutoff_record_and_returns_failure|원15s 기한의 실제 sync 지연. 저장 JSON은 remaining14938ms/first_error=null/verdict records0인 저장 전 상태; 반환 후 TimedOut/passed=false는 unit의 명시 단언으로 검사|
|842|delayed_verdict_preserves_original_receipt_failure_and_late_secondary|저장 JSON의 원 receipt 오류와 original_finish_incomplete/기록0 유지. 반환의 원 오류+late secondary·실패는 unit이 검사. 저장 전 JSON을 최종 반환으로 바꾸지 않음|

각 원 로그는 `runtime/verification/source375-datalab-b0-r1-logs/836~842-baseline-runner-*`, 증거 폴더는 `runtime/verification/baseline-runner-{zero,sink,nested,timeout,forced-zero,late-verdict,late-sink}-r1`이다. top outer raw/receipt와 정확한 observed PID의 inner parent를 각각 inner-observed에 복사했다. nested-observed에는 baseline child receipt, forced-zero의 stimulus-observed에는 원 baseline stdout/stderr를 별도 보존했다. 외부 process record와 내부 expected failure/retained owner를 합치지 않는다. late 두 사례의 post-return TimedOut/secondary는 in-process 단언이며 저장 JSON의 독립 관측이라고 과장하지 않는다.

이로써 baseline 실제 범위는 pure11+정상834를 포함한 OS self-test8이다. 등록23 중 자극4를 독립 기능 인수 수에 더하지 않는다. 851은 전체41 input content/identity 동일 exit0. 새 예상 밖 실패는 없으며 원823/833 실패는 그대로다.

## 첫 B OS1

843은 B R4 source32(SOURCE150FF75D)/identity를 다시 확인했다. 844는 고정 baseline 실행기로 `windows_cases::original_duplex_echo_keeps_partial_bytes_and_reclaims_endpoints`만 exact/ignored 실행했다. candidate test1745B48B(1199616B)와 실제 compile-time child alias C6D67C3E(392704B)를 guard했으며 FileID는 outer outcome에 보존했다. operator index는 권한이 아니며 compiled_baseline_proof=false다.

844 원 resource-runner terminal exit0/10939ms, test PID36104/creation134375533305659431, exact1pass/원 child_exit0·Empty·forced=false, selected_one/physical_closed/raw EOF=true였다. 이는 같은 원 data lane으로48KiB를 echo하고 전체 bytes를 비교한 사례이며 actual partial-call count>1/backpressure/peer EOF를 별도 관측했다고 주장하지 않는다.

SOURCE_IPC_EVIDENCE_ROOT는 C tmp 아래 사전 생성한 `source-ipc-echo-os1-r1/inner`였다. 실제 `owned-source-36104-0`의 원 receipt.txt483B·stdout.raw0B·stderr.raw0B가 정상 종료 뒤 남았다. exact 원 경로를 `runtime/verification/source-ipc-echo-os1-r1/inner-observed`에 복사 보존했다. 원 source child35936/creation134375533414265105·exit0/Empty/nonforced, io_reaped=true/local_endpoints_closed=true/had_error=false, 동일 endpoint 재획득 단언 통과다. control_eof/data_eof는 false이므로 명시 peer EOF 인수로 확대하지 않는다.

## PM 배열 표기 오독 정정·producer 선택

PM이 `file.read(&mut [0])`을 길이0 버퍼로 잘못 읽어 결함이라고 설명했다. Rust `[0]`은 **1개 원소 배열**이므로 그 판단은 틀렸고 기존 검사에 그 결함은 없다. 원 실제 raw 파일은0B였고 단언은1바이트 읽기였다.845 R5 snapshot과846strict/847no-run은 동일 의미 `[0u8;1]`의 불필요한 표기 변경 중간 자료로 보존하며 기능 수리·새 OS 인수로 세지 않는다. 이 설명은 사용자에게 즉시 정정했고 원 leaf D5C405B2로 복구했다. 원 후보/제품 코드에는 이 변경을 적용하지 않았다.

복구는 native Copy-Item으로 원 preimage를 사용했다.848 input-r6는 content/identity까지 원 R4와 같아 SOURCE150FF75D/inventoryC6C1C11C/32가 동일하다.850 동일 검증도 통과했다. 이를 변경 이력이 없었다는 주장으로 쓰지 않는다. 중간 R5 leaf는 rebase 증거 폴더의 source-ipc-explicit-one-byte-draft.rs에 있다.

**주의: current debug test alias는 중간 R5 producer3BD29495다.** 원복 copy의 mtime 보존 때문에849 Cargo가 fresh=true로 이를 재사용했다.849의 exit0를 원 R4 producer 증명으로 계상하지 않는다. 원844 실행 전에 보존한 `runtime/build-cache/native/source-ipc-retention-r1/fixed-r4/source_ipc.exe` SHA1745B48B46B1B96D964993B0B24EE4FE75098EC03CD95EB3435FE0746A728B67을 이후 선택한다. 이는813 Cargo producer의 보존 copy이며 root/current debug alias와 다르다. child는 compile-time의 원 debug/owned-process-test-child.exe C6D67C3E4744BAA5F79F2AAB3174A811217C818A0FF0B5E5F3F1B68643CBF134를 계속 guard해야 한다. 원복 소스와 다른 intermediate alias를 조용히 다음 gate로 넘기지 않는다.

B actual은 OS1이며 나머지9는 미실행이다. HTTP source/replay issuer·제품 다운로드·전체CLI/AI 품질0/9/GUI는 미완료다. root 최신190/정상bins는 DFB0에 유지한다. 기존 modoo fixture A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100과 change-review9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B의 실제/예상 실패·주장/원 증거·owner 분리를 적용했다.
