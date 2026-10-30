---
id: evidence:kinekino/20261029114456
title: 원 native operation 분리 — R192/R193
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 native operation 분리 — R192/R193

## R206/R207 원 parent 상태 ledger 연결

`review_source::NativeProcessLedger`는 원 operation이 launch 전에 등록한 동일 Arc 상태를 보관한다. 상태 셀의 등록/갱신은 crate 내부 원 Rust 실행 경로에만 있고 JSON Deserialize·PID 조회·receipt 파싱·사용자 callback은 없다. 공개 읽기 결과는 진단/관측이며 publication permit을 만들지 않는다. launch 직전 Pending, 실제 원 family/receipt/일반 파일 EOF 관측을 갱신한다. run의 callback panic 정리 중에도 같은 상태를 갱신하고 operation 결과 소비/해제 뒤 ledger가 보관한다. 내부 launch panic이면 마지막 Pending을 보수적으로 유지하며 성공으로 가정하지 않는다.

MutationControl의 선택 포트와 source ProbeControl·output verifier Control을 연결했다. 기존 원 Jobs가 같은 ledger를 유지하고 원 렌더·검증 이후 등록 상태의 closure를 확인한 다음 기존 단일 publication 경로를 사용한다. NotStarted는 자식 없음만 뜻하며 exit0 성공과 다르다. regular_files_eof는 [R205 실제 일반 파일 EOF](native-regular-output-drain-20261029.md)이고 pipe_eof_observed:false를 유지한다. Core OwnedAttemptSession/OwnedOutputCandidate 등록과 전체 retained owner 이전은 아직 미완료다.

수정 전 `preimage-r205-process-ledger/`에 원 파일을 보존했다. R2062460파일 inventory SHA256 `E1F85FCA1E3E619803E2F1CFCCBEB367F8D2851E4CCE5811854F042ED4CFD805`, SOURCE SHA256 `F048FF084E6EA277940B115E467B849B96104121F3BBAA8F74B49085AD4D704B`. 로그2835fmt/2836snapshot/2837native strict 종료0이며 실제 시험 입력으로 계상하지 않는다. R207에서 callback unwind 시험도 원 ledger의 actual forced closure/EOF 관측을 확인하도록 보강했다.

R2072460파일 inventory SHA256 `0DFB62E50DC3E673ABF17FE0EE27B837306ADA699BF30A18826386830E85AF6C`, SOURCE SHA256 `681D80AA6D50511B43C0CA973AB456B7FCF2A79F75E0344444085EBE11476C4B`. 로그2838fmt/2839snapshot/2840native strict/2841default strict/2842lib no-run/2848fmt check 종료0. 고정 lib exe/PDB는 mirror fixed-r207/native-core/, exe SHA256 `478C574B3E7F1B13899B8883E5B7F8C94B4D459A3CF0112C5281C0B47A5B8293`.

2843 실제 원 operation1시험: launch 전 동일 ledger의 NotStarted, launch 후 미종료, 원 child0/family Empty/receipt/regular EOF, 결과 소비·원 operation 해제 후 같은 ledger의 종료 관측을 확인했다. 2844 실제 callback unwind1시험: 원 동일 ledger에 forced=true/exit 관측·family Empty/receipt/regular EOF가 남고 원 typed panic payload의 식별도 유지됐다. root `runtime/verification/source-sdk-r207-native-unwind-r1/`.

2845 원 Jobs 자막 없는 렌더/발행/재사용/재열기, 2846 선택 자막, 2847 취소·정상 재기동·수동 attempt2 각1 실제 회귀 통과. root `runtime/verification/source-sdk-r207-jobs-plain-r1/`, `source-sdk-r207-jobs-subtitles-r1/`, `source-sdk-r207-jobs-retry-r1/`. 2849사후 custody 원 실행 종료0, 2460파일 내용·식별 동일성을 확인했다. 전체190/정상 bins의 최신 완료 입력은 R197이다. 기존 QA fixture/변경 검증 skill 적용과 전체 요구·소유권·보류를 유지한다.

## R197 공통 native driver의 unwind 정리

공통 blocking driver는 준비한 원 operation을 catch 경계 밖에 보관하고 launch/poll을 실행한다. callback unwind 시 외부의 미디어·폰트·입력 guard가 아직 unwind되지 않은 이 경계에서 원 operation을 정리한 다음 같은 opaque panic payload를 다시 전달한다. 정리·영수증이 미완료면 원 operation과 payload를 보유한 채 기다리며, 원 receipt의 명시 recovery도 유지한다. recovery callback 자체가 다시 panic하면 그 추가 payload도 보관하고 해당 callback의 반복 호출은 중지한다. receipt 실패를 암묵 재시도하거나 성공 ACK로 바꾸지 않는다.

수정 전 `preimage-r196-native-unwind/`에 원 mod/tests 보존. `input-r197`2456파일 inventory SHA256 `6CF9AA3F67C609803B921DF8AE9A53D2BB3980B22046E52F48723F6C7EBBBF07`, SOURCE.md SHA256 `EA7C8802A093D78BB2C43AD07A3C28D56A345491A90FA9F6BFFF387B99F0F3D0`. `2764`fmt/`2765`native strict/`2766`default strict 종료0, `2767-source-sdk-r197-lib-no-run` 종료0. 고정 lib exe/PDB는 mirror fixed-r197/native-core/, exe SHA256 `A9AF013AB5CB4A09B93999CA5C84367A182C5A2A7292B98E97900BD432C16CB7`. 이 경계는 Jobs owned-attempt 전체 opt-in이나 모든 infrastructure 내부 panic의 정리를 입증하지 않는다.

`2768-source-sdk-r197-native-unwind` 실제1시험 통과. callback panic 뒤 원 family child_exit124/family Empty/forced=true·receipt를 정리한 다음 같은 typed panic payload의 Arc identity가 유지됨을 확인했다. 바깥 작업 입력 소유자는 그때까지 유지됐고 해제 뒤 새 파일이 사라졌다. root `runtime/verification/source-sdk-r197-native-unwind-r1/`에 input-facts/verification과 원 process 증거 경로를 보존한다. 입력 소비 완료·pipe EOF·전체 Jobs owned 전환을 입증한 시험은 아니다.

`2769` 원 receipt 실패 pending·명시 복구/최초 오류, `2770` actual regular stdin 소비 후 취소·정리, `2771` 실제 MP4 검증/발행/입력 해제 각1회귀도 통과했다. root `runtime/verification/source-sdk-r197-stdin-cancel-r1/`, `source-sdk-r197-render-r1/`와 원 process 영수증을 보존한다. `2772-source-sdk-r197-combined-no-run` 종료0.

`2773-source-sdk-r197-normal-bins` 종료0. 정상 exe/PDB는 mirror `fixed-r197/native-bins/`, CLI SHA256 `A9ECCBD81287175C09880E52119AFE0B93E0455151FF09506A0F1C5CBF322DF2`, GUI SHA256 `9DD596DDD8511E03A320A0B8E575B54A10C8BB01D22DE7E871ABC1115F9B07B2`. GUI 수동 실행은 하지 않았다.

`2774` 선택 자막16CLI/typed source/catalog/submit·원 Jobs2완료, `2775` 기존15CLI/typed 여정·원 Jobs2완료, `2776` 취소/조회/submit·retry 재전송과 수동 attempt2 실제8CLI가 각각1시험 통과했다. root `runtime/verification/source-sdk-r197-cli-r1/`, `source-sdk-r197-cli-cancel-r1/`. manual caption 및 in-process 취소 host 범위는 유지하며 실제 전송 ACK 유실·중단 복구나 사용자 품질 승인으로 계상하지 않는다. `2777`Product145/`2778`Core18/`2779`Native26/`2780`OS1의 execution_result·producer_integrity 모두 Ok이고 같은 R197 SOURCE SHA를 참조한다. Meari/OS는 기존 고정 producer의 이번 실제 재실행이며 새 컴파일로 표시하지 않는다. `2781`fmt check/`2782`사후custody 종료0,2456파일 content/identity 동일 확인.

## R194 정리 관측과 강제 종료 후보 거절

원 operation은 실제 Session에서 not_started/child_exit/family_empty/receipt_recorded/forced를 관측한다. 결과 수령 뒤에도 마지막 관측을 보유하고, Finished라도 실제 정리·영수증 조건이 충족되지 않으면 결과를 내주지 않는다. 관측 JSON은 권한이나 worker의 소유권 증명으로 소비하지 않는다.

강제 종료가 관측되면서 종료 코드가0인 경우도 성공 후보를 거절하도록 보강했다. 먼저 기록된 취소/timeout/실패가 있으면 그 최초 오류를 유지하고, nonzero exit 오류도 기존 우선순위를 유지한다. 이는 Jobs owned-attempt 연결이나 강제 취소의 전체 ACK 정책 인수 완료를 뜻하지 않는다.

수정 전 operation/mod 파일은 `preimage-r193-native-progress/`에 복사했다. tests-reconstructed.rs는 신규 시험을 제거해 복원한 preimage이며 R193 inventory의 원 SHA256 `F4A26F5CD3AAC6F4FCA542A50A547209D513FB37BDA62829E40B3A3CBC5AB8FA`와 실제 일치를 확인했다. 원 FileID/mtime 근거는 원 R193 inventory를 유지하며 복사/복원 파일을 원 파일 식별로 표시하지 않는다.

`input-r194`2454파일 inventory SHA256 `3416FC54484E87ABE8A9EFF711352F96269C0FFBE51585E8BBE2414424A586C2`, SOURCE.md SHA256 `08E2B6BC4911CF5BBE330C32F3B8E05389951E35ED5D1C52A4DBFAA31DB6C635`. `2736`fmt/`2737`native strict/`2738`default strict/`2739`lib no-run 종료0. 고정 lib exe/PDB는 mirror `fixed-r194/native-core/`, exe SHA256 `0E0294DA6D1D04EF16CC29B1452654BB360F6098CD83FDCFCE2E816B7775E1EF`.

`2740`은 forced+exit0 거절·최초 오류 보존 정책1시험 통과이며 실제 OS 강제 종료 경합을 재현한 근거는 아니다. `2741` 원 operation의 준비/단일launch/정리 후/결과수령 후 관측 유지, `2742` 원 receipt 실패 pending·명시 재기록, `2743` 실제 stdin 취소, `2744` MP4 검증/발행/해제 각1 실제시험도 통과했다. root `runtime/verification/source-sdk-r194-stdin-cancel-r1/`, `source-sdk-r194-render-r1/`와 원 process 증거를 보존한다. `2745-source-sdk-r194-custody` 종료0,2454파일 content/identity 동일 확인. 새 정상bins/190·SDKCLI 및 Jobs owned-attempt 연결/전체 정리 인수는 미완료이며 최신 통합 검토 빌드는 아래R193이다.

private overlay의 `frame_capture/process/owned/operation.rs`에 준비·명시 launch·poll·결과 수령을 가진 원 operation을 분리했다. 기존 blocking run도 이 객체를 구동한다. 각 poll은 기존 Session의 원 WindowsFamily·deadline·stdout/stderr 한도·영수증과 복구 세대를 사용한다. 다른 PID로 프로세스를 재탐색하거나 종료 소유권을 대체하지 않는다.

준비 poll은 자식을 실행하지 않고, launch 중복은 RevisionConflict로 거절하며 원 실행 상태를 바꾸지 않는다. 결과는 원 Session이 정리를 완료한 뒤 한 번만 수령한다. 부분 launch 실패의 owner와 최초 오류도 기존 처리와 같이 보유한다. 원 operation을 작업 상위 소유자가 유지할 수 있는 준비 단계이며, 아직 Jobs owned-attempt 연결·중단/receipt 복구 전체·panic 정리 인수를 완료한 것은 아니다. 미완료 객체를 drop해도 안전하다는 계약도 추가하지 않는다.

- 수정 전 `preimage-r191-native-operation/` 보존. R1922454파일 inventory SHA256 `2D3882F40CE4CE0D799660D28CD53445009938ED7405A5A51101403D040089EA`.
- 첫 실패 `2716-source-sdk-r192-native-strict`: items_after_test_module. 원 파일은 `preimage-r192-operation-order/`에 보존하고 시험 모듈을 구현 뒤로 옮겼다.
- R1932454파일 inventory SHA256 `E76B5F534DFCE946B9C29B3ED1658343746DEBE29B254635BEF6652BB3C5BA17`, SOURCE.md SHA256 `B5DC52D1FABBCACFF6225C6C68517DE56D1E3CF1BB093B45923B262E69309F2D`.
- 로그 루트 `runtime/verification/source375-datalab-b0-r1-logs/`. `2717-source-sdk-r193-fmt`, `2718-source-sdk-r193-native-strict`, `2719-source-sdk-r193-default-strict`, `2720-source-sdk-r193-combined-no-run` 종료0. 고정 lib/Jobs/unitCLI exe 및 PDB는 mirror `runtime/verification/fixed-r193/native-core/`. lib exe SHA256 `81918CF0AED445E77A0F3C54121AAE94F2567A82AE4D8F254F68F8EE71EF5B1D`.
- 실제 각1시험: `2721` 단일launch/한 번 결과 수령, `2722` 원 receipt 실패의 pending·명시 재기록·최초 오류 보존, `2723` regular stdin 소비 후 취소·family/receipt 후 입력 해제, `2724` MP4 검증/발행/새 입력 부재, `2725` 자막 픽셀 경계/alpha·PNG 해제 모두 통과. operation 원 증거는 `runtime/build-cache/tmp/frame-owned-28916-1793203405153653900-0/`, 나머지 실제 입력·출력은 root `runtime/verification/source-sdk-r193-stdin-cancel-r1/`, `source-sdk-r193-render-r1/`, `source-sdk-r193-subtitle-video-r1/` 및 원 process 증거에 보존한다.
- `2726-source-sdk-r193-normal-bins` 종료0. 고정 정상 exe/PDB는 mirror `runtime/verification/fixed-r193/native-bins/`, CLI SHA256 `A9A8AA401B6407BAC7B52A0C287E710F33D0D0359EE8E922255493D41D14D511`, GUI SHA256 `1235DE220F8A1BC2974867F5445D0D6F8BB850BFA26665AD4AB5F0A22F995987`. GUI 수동 실행은 하지 않았다.
- `2727` 선택 자막16CLI/typed catalog/source/submit, `2728` 기존15CLI/typed 여정, `2729` 취소·조회·submit/retry 재전송 및 수동 attempt2 실제8CLI가 각각1시험 통과했다. 실제 root `runtime/verification/source-sdk-r193-cli-r1/` 및 `source-sdk-r193-cli-cancel-r1/`. 자막은 manual fixture이고 취소 host는 in-process SDK fixture이며 실제 ACK 유실·예기치 않은 중단 복구를 입증하지 않는다.
- `2730`Product145/`2731`Core18/`2732`Native26/`2733`OS1은 각 group-return의 execution_result와 producer_integrity가 모두 Ok이며 동일 R193 SOURCE SHA를 참조한다. Meari/OS는 기존 고정 producer의 이번 실제 재실행이고 새 컴파일로 표시하지 않는다. `2734-source-sdk-r193-fmt-check` 종료0. `2735-source-sdk-r193-custody` 종료0,2454파일 content/identity 동일 확인. R186/R191 등의 이전 성공을 R193 성공으로 계상하지 않는다. Jobs owned-attempt 연결과 전체 정리/중단/복구 인수는 미완료다.
- 기존 test-fixture-and-mock-build/code-change-review-verification의 실제 원 owner 및 근거 구분 적용을 유지한다. 보류·기존 소유권·private overlay 및 전체 goal을 유지한다.
