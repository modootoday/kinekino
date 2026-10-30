---
id: evidence:kinekino/20261029114234
title: 제품 historical provenance 연결 실행
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 historical provenance 연결 실행

[구현 계획](../../../plans/player/historical-provenance-product-checkpoint.md)과 [API/저장 계약](../../player/application/api/historical-provenance.md)의 실행 정본이다. [parent 등록 인수](sdk-product-parent-2026-10-20.md) 이후 별도 checkpoint이며 저장 version 판단과 증거를 섞지 않는다. 현재 구현·검사 중이며 전체 제품 완료가 아니다.

## 입력과 자원

증거 루트는 `runtime/quality-review/historical-provenance-product-r1`이다. 기존 actual9와 pinned Rust verifier를 재사용하고 추론·다운로드는0이다. root C/native·consumer D 전용 cache를 유지하며 resource-runner의2CPU/BelowNormal/descendants·offline/locked/j1·RUST_TEST_THREADS=1을 적용한다. Meari 임시 슬롯 동안 제품 Cargo/실행을 멈추고 source 작업만 진행했다. PM이 Meari의 동결/정리를 인수한 뒤 SDK에 슬롯을 재배정했다.

root/consumer manifest·lock과 Meari core manifest는 이전 인수 pin을 유지한다. 승인된 consumer2개·studio1개 fixture의 변경은 StoredProposal literal의 historical None뿐이며 `fixture-*.diff`로 보존했다. UI bootstrap 미등록 source와 Meari native는 별도 소유로 구별한다.

## reader 입력과 첫 오류

고정 verifier EXE SHA는 `e00c1726603ef7d0167c6d58deb79e694071044c20a2723de8abe8a504f45e41`이다. 실제 inspect-import 명령450ms/exit0·stdout3517bytes를 확인했다. 이후 application 부모 등록 첫 시험62006ms/exit0·3pass는 준비 MemoryStore/reader fixture의 최초 컴파일 결과다. module fmt 첫 check는 정렬 차이로 실패했고 수정했다.

실제 process reader 첫 all-target check는 ApiError Clone 부재 E0277로 실패했다. 원 source/원문을 보존하고 오류 코드만 전달한 r2는30224ms/exit0이었다. 그 source의 focused50569ms/exit0은 root3pass·1ignored·290filtered이며 별도 media-jobs의 기존 historical-list1개도 통과했다. actual9 성공이 아니다. `cargo fmt --all`은 작성 중인 제외 Meari native의 coordinator.rs 미존재로 실패했고 외부 source를 수정하지 않았다. 이후 제품7package의 기본 fmt 범위를 사용한다.

reader owner 보강 후 focused47205ms/exit0은4pass·2ignored다. stdout/stderr 초과·timeout·nonzero의 실제 자식, pipe read 실패와 kill/wait 실패 주입 후 owner 보존·중복 실행 없는 정리를 검사했다. 주입 실패를 실제 Windows kill 권한 실패 관측으로 부르지 않는다. timeout과 cleanup_complete를 구별하고 Drop은 best effort다. 현재 pinned verifier source의 자식 비생성 범위이며 임의 descendant tree 관리가 아니다.

actual-nine-first4069ms/exit0은 정확1시험 통과·본문2.88s다. 내부9개 item의 pinned 원본 재구성·원 source/raw 결합·다른 companion SHA 거부를 확인했다. reader strict37364ms/exit0도 통과했다. 이 결과는 다음 binding source 이전이며 새 영속/CLI 구현에 소급하지 않는다. 동기 process이므로 무차단 UI/typed Pending·worker completion은 별도 잔여다.

## binding 검사 입력

최초 source-only snapshot inventory는 `c672eb011015e2b79b47e12b6da73eb0669ef00a059a44c497e8fd8da8f96f07`이다. 이후 단일 revision의 binding 전체 제거와 legacy 구별 공백을 source 검토로 찾아 historical document version3/historical binding version1을 도메인에 정의했다. 일반 document·source snapshot1/2는 유지한다. version/전체 이력/checksum을 모두 다시 쓰는 trust 한계는 계약대로 남는다.

binding-fmt-first2481ms/exit0과 binding-check-first29934ms/exit0을 확인했다. 동일 session의 terminal exit0을 받은 뒤 stored.validate 선행과 task별 CLI harness 보완을 적용했다. 실행 중 source를 바꾸지 않았다. 새 r2 snapshot inventory는 `8068a7c7147411ea3a24aa361f9256cbacb1eb906c50984ce3348065bba537f6`이며 제품 source와 CLI harness의 첫 focused 인수 중이다.

새 CLI harness는9import/fresh inspect와 correction/cards/highlights 각각 최소1개의 원문 ref→typed technical edit→foreign/stale 거부→명시 accept/retry→raw/binding 불변·자동 적용false→project capture/reopen을 준비했다. 같은 scratch에서 missing/동일크기 변조·binding 전체 누락·다른 유효 item 교환·부분 publication을 구성한다. 아직 실제 CLI 영속 성공이나9개의 사람 품질 승인·모델 실측으로 기록하지 않는다.

binding-focused-first60510ms/exit0은 root8pass·2ignored·290filtered·본문0.81s다. 별도 media-jobs의 기존 historical-list1시험도 같은 필터로 통과했다. legacy 직렬화 bytes/capture SHA 보존, schema3 binding 필수, 실제 ProposalFiles의 한 revision/한 파일·편집/채택/재시작·증거 누락 시 accept 재요청 거부를 확인했다. 해당 저장 시험의 Reader는 fixture이며 실제 외부 verifier 성공으로 대신하지 않는다. 실제 fresh CLI gate는 같은 source를 동결하고 `cli-first` 증거 root에서 진행 중이다.

## 첫 실제 CLI gate

cli-first107497ms/exit0, 정확1시험 pass·0ignored·본문31.11s로 첫 실행이 통과했다. `cli-first` 하위에 고정 CLI·복사 workspace·실제 proposal/project publication과73개의 요청별 stdout/stderr/PID/exit/reaped receipt를 보존했다. 고정 CLI SHA는 `8065a23237dbc519b1c9e65f0164d54dc4e70e269da5fbc00d55f95c2e3c6266`이며 input-pins.json은 원 AGENTS/3 source manifest/companion/package/objects의 전후 불변을 확인한 입력을 기록한다.

9개 item의 원 generated raw를 그대로 import하고 매번 별도 CLI PID에서 조회했다. 모두 NeedsRepair이고 document3/historical1이다. Gemma correction/cards/highlights 각각 canonical document/cue ref로 기술 편집하고 stale 거부·명시 accept/retry·raw/source/binding 불변·automatic_application_enabled=false와 project capture/fresh reopen을 확인했다. correction은 saved capture·project save CAS·working draft 분리도 포함한다. 이는9개의 수정/사람 승인·새 모델 실행/품질 실측이 아니며3 task의 기술 유스케이스 검증이다.

복사본의 companion/source/candidate 누락·동일크기 변조에서 proposal inspect/accept 재요청/project inspect를 거부하고 추가 publication이 없었다. latest binding 전체 제거·다른 item 교환과 첫 revision binding 제거를 새 checksum을 가진 JSON으로 구성해 거부했다. 단일 revision의 다른 유효 item 교환도 실제 reader의 source/raw 결합 검증에서 InvalidArgument였다. 빈 부분 revision은 기존 recovery 오류를 유지했으며 정상 복구본 조회와 원본 evidence pin 불변을 확인했다. 전체 document version/이력을 legacy로 재작성하는 외부 trust root는 검증한 범위가 아니다.

같은 입력의 binding-strict-first35531ms/exit0과 제품 fmt-final1411ms/exit0을 확인했다. root 전체 회귀를 진행 중이며 consumer 새 입력 회귀와 최종 source/bin pins·소유 정리는 남아 있다.

binding-full-first255986ms/exit0은 root lib296pass·4ignored를 포함해 workspace/integration 합계396pass·0fail·22ignored다. 기본 회귀의 ignored actual gate를 통과로 합산하지 않으며 위 실제 CLI1시험은 별도다. 제품 source 수리 없이 통과했고 독립 D target의 consumer 전체 회귀를 진행한다. 이후 consumer strict/fmt·최종 pins·소유 정리와 PM 반환이 남는다.

## 최종 인수와 반환

consumer-full-first406266ms/exit0은 실제47pass·0fail·0ignored다. 같은 새 source와 고정 Meari native 입력으로 기존 factory/view/bridge/headless 회귀를 통과했다. consumer strict45445ms/exit0·fmt437ms/exit0, root bins3122ms/exit0도 통과했다. binding source의 check/focused/실제CLI/strict/full 및 consumer 회귀는 첫 실행에서 제품 source 수리 없이 통과했다. 앞서 reader 단계의 E0277/fmt 실패와 그 원문은 그대로 보존했다.

최종 GUI SHA는 `b7cb99ee146b3122b59858b84cd08e01240ea4186b7fa26fb089a2a74728d9f0`, CLI는 `8065a23237dbc519b1c9e65f0164d54dc4e70e269da5fbc00d55f95c2e3c6266`이다. 최종 CLI는 실제 gate에 복사·실행한 CLI와 같은 pin이다. final-bin에 보존했으며 GUI 실행/시각 인수를 뜻하지 않는다. SDK 소유 source·승인 fixture·manifest/lock snapshot의 final-source-inventory SHA는 `b2c92d62852d8c423d2acad9de2496891210ae482ae25744896b0a70026cdd35`다.

root Cargo/lock·consumer Cargo/lock·product plugin/Meari core manifest는 parent 인수 pin과 같다. 새 dependency/lock resolve는 없다. 새 SDK publication version3/historical1과 일반 legacy1/2의 bytes/capture SHA 보존을 구별하고 기존 데이터의 일괄 migration은 하지 않았다. 이전 실행 파일이 새 bound field/version을 읽는 forward compatibility는 보장하지 않는다. 이는 parent 등록 추출의 migration 판단과 별도 변경이다.

source-final은 SDK 소유 API/application/domain/infra/CLI/필요 main 조립·새 Rust harness와 승인3fixture를 보존한다. UI bootstrap의 미등록2파일과 Meari native 소스는 별도 담당 작업이며 SDK 변경으로 합산하지 않는다. fixture-*.diff는 None 추가만이고 기존 assert/예산/데이터는 바꾸지 않았다. source-r2 및 최초 source-only snapshot도 덮어쓰지 않았다.

final-process-inventory.json은 cargo/rustc/resource-runner/player/player-cli/고정 verifier의 빈 배열이다. 최종 C 여유50579677184bytes·D265676689408bytes를 기록했다. cache/registry/원 실패 증거를 삭제하지 않았다. 각 CLI child의 종료/reaped receipt는 실제 gate 하위에 따로 있다.

이번 historical provenance checkpoint의 SDK 소유 source 변경과 single-heavy 실행을 종료하고 PM에 명시 반환한다. 남은 범위는 verifier 배포 설치 인수·동기 reader의 GUI worker/Pending/completion/wake 통합·실제 NativeWindow/시각·현재 job/attempt/output provenance 및 실제 자막/카드/미디어 적용·사람 품질 승인이다. 이번73요청/9import/3task 기술 채택을 그 전체 완료로 부르지 않고 원래 제품 목표를 활성 상태로 유지한다.
