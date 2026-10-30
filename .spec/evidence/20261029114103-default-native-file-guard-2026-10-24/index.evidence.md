---
id: evidence:kinekino/20261029114103
title: 기본 Native fileguard 검증 — 2026-10-24
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 기본 Native fileguard 검증 — 2026-10-24

## 입력과 실행 범위

root `c9c48d20abddb110d37a810c08c3520c10e955fc`, Meari `f85a68df7a6a8fee239d9655895ee15d5eaa5510`, lock `8F2C1D56C8A2C22D36D9E57A285D727C7EE9C7D2D1160902B66CC6630B042C67`. source8 `E3DF2660BB762063BFC155FAD7CF3265981225D44A7CB46931CCB30206D2D94F`, expected3 `8B841C43`, impact5 `45404119`, HANDOFF `B464EDC8` 원 핀을 유지했다. 코드 수리·기대/guard/owner/deadline 변경0이다.

PM phases1~3 배정 뒤 단계4~6 filesystem3를 별도 승인받았다. 원 S/T/resource-runner EEA9DEE6를 재사용하고 매 실행 RAM16GiB/GPU3072MiB/C12GiB/writer0, CPU2/mask3/BelowNormal/descendant Job·offline/locked/j1/thread1을 유지했다. 원 UI 반환FEEB0942의 S6081/T589/carry1374A929E69에 fresh1회 대조 차이0. 승인 canonical UI3는 별도 shared preimage를 보존해 sync baseline6081로 고정했고 fileguard overlay는 기존1/신규1, 나머지6은 readonly hash 확인으로 분리했다.

## compiler·순수 영향 검증

원 04 fmt·05 default all-target strict·06 native-observer-test-support all-target strict·07 native_product_launch no-run JSON은 모두 terminal0이다. native strict46131ms, no-run87516ms였다. 실제 Cargo 호출은 증거의 번호별 resource/receipt와 원 stdout/stderr에 연결된다. producer 명령은 `cargo test --offline --locked -j1 -p kinekino-player --test native_product_launch --features native-proposal-window --no-run --message-format=json`이다.

원 compiler-artifact의 target/profile/features/fresh와 producer/copy SHA·별도 FileID를 고정했다. fixed SHA `0AC6B446E405BFACD7309A7829695A2296D97BDA63DEBE93140E99CCFF585B65`. 실제 list28/ignored11과 기존 ordinary17·expected ignored3가 일치했다. impact5 정확 명단에서 각1pass/0failed/0ignored·terminal0을 확인했고 나머지12는 반복하지 않았다. 원 source의 pure17 전체 결과를 새 입력17 재검증으로 세지 않았다. default 등록0은 별도 producer/list 실제 근거가 없다.

## 실제 filesystem3

fixed BIN에 명시 process 환경의 pinned config `04617E53CD4FDDD701DAC8E319CA55B7C05580EFB1FED59C029D278E928E09F4`를 전달해 각 `--exact --ignored --test-threads=1`로 한 번 실행했다. config는 새 입력 경로/readonly·독립 backup/hash/bytes/mtime/FileID, parent는 새 절대 ordinary path/identity로 고정했다. 고정 image8D5433는 readonly fixture 입력으로만 승인됐고 실행하지 않았다. fixture setup의 actual root/A/B/image volume은 모두 `4061598778`이었다. 드라이브 문자만으로 volume 같음을 판정하지 않았다.

접두사는 `default_launch::file_guard::`다.

| 시험 | 원 terminal/관측 |
| --- | --- |
| ordinary_fixture_identity_matches_handle_and_final_path | 각1pass/exit0/879ms. A FileID1125899906994819·B1125899906994820, 동일3bytes/abc SHA이지만 실제 다른 FileID. 원 handle/final-path reopen은 A와 일치하고 B는 원 binding 비교로 거절 |
| file_and_parent_replacement_stays_denied_until_pin_release | 각1pass/exit0/1057ms. 파일 rename·write-open·부모 rename 모두 원 OS error32(io_kind=Uncategorized)를 보존하고 각 시도 뒤 A binding/bytes/mtime 불변. 원 Pin 명시 해제 뒤 파일/부모 왕복 rename·write 성공, A FileID1125899906994832와 abc SHA·이름 유지. 소유 write의 mtime 변화는 그대로 관측 |
| ancestor_reparse_fixture_is_rejected_before_spawn | 각1pass/exit0/770ms. 실제 directory reparse attrs·link FileID1125899906994847·정상 lookup=A 확인, Stage::Pin/원 ancestor-reparse 거절. 동일 link identity 재확인 후 정확 link만 제거, target A binding/hash 보존 |

원 journal은 각7/20/10 event이고 case-result의 body_succeeded=true/first_error=null/probe_pin_released=true/image_pin_released=true/link_created_and_remaining=false/programs_spawned=0이다. 권한 부족·skip·retry·권한/Developer Mode 변경0이었다. 실제 admission/denial/release/recovery와 journal을 직접 읽었고 pass 문자열만으로 판정하지 않았다. 작은 workspace6 abc파일·raw/config/backup은 보존한다. 재귀 정리0이다.

## 이미지·관측 한계

fixture image는 원 `runtime/verification/default-native-window-r2/produced-aliases/player.exe`, SHA `8D5433A7125DEE7E517ACAE1456B2A13F3B4C2266C6E0785037CA08D11FE872E`, 153160192bytes다. 원 producer root76287/Mearif85/lock8F/native-proposal-window와 연결되며 currentc9 생산물이라고 다시 표기하지 않는다. cfg(test)2+내부 가시성1의 runtime 정책 적합성은 소스상 추론이고 N/C image 인수가 아니다. 이번 no-run이 함께 생산한 현재 root의 player SHA `4224605AF55D6E8AD0C1B4CA9C4FB6156DA53AD4D4623E20F51F7EDF9A8AAF90`와 구분해 각각 보존했다.

이3개는 실제 namespace/file handle/ancestor guard·해제 뒤 복구의 해당 volume 증거다. process memory/section FileID attestation은0이다. source에 작성된 spawn-admission.json은 아직 실제 N/C로 생성·검증하지 않았고 cached image를 fresh query로 표시하지 않는다. N/C·pipe ACL/host/controller 생존·정상 WM_CLOSE·실제 제품 image 실행·GUI/전체 CLI/시각 인수는0이다.

## 복원·보호·반환

원 source existing1 bytes/mtime와 new1 부재를 복원했다. Cargo의 새 player/CLI exe·PDB4를 독립 사본으로 먼저 보존하고 정확 alias를 원 bytes/mtime의 새 파일로 unlink/replace했다. 직접 overwrite0, 새 생산자4 bytes/mtime/FileID는 복원 전후 유지됐고 alias4 FileID 변경은 별도 예외다. 과거 known2/alias 예외를 유지하며 all_cache_protected=true로 표시하지 않는다.

config/backup은 readonly/hash/mtime/FileID/attrs 불변, parent FileID 불변(새 case 생성에 따른 directory mtime 변화 허용), 원8D image bytes/mtime/FileID/attrs 불변을 확인했다. alias/producer는 실제 복원 때 확인한 기록을 최종 감사에서도 재사용해 같은 파일을 중복 해시하지 않았다.

최종 root1383+supplement9/library497/index·manifest·lock9, 비수정6·원 fixed137의 bytes/mtime/FileID 보호가 일치했다. 신규 fixed1도 확인해 carry138 SHA `ACBA73E07F633A800450431A6C120AEF8240F1A8A56EF66945343E713A106ABC`로 인계한다. 승인 sync baseline S6081→최종6081, 비입력 변경·신규runtime0. T589→589이며 변경3(player/CLI deps·native_product_launch)을 원 Cargo JSON/producer SHA/FileID로 귀속했고 신규 exe0이다. 새 생산자 PDB2도 exact4 기록으로 별도 보존했다.

receipt14 전부 terminal/exit0, 실제 고유8pass(ordinary5+filesystem ignored3), 코드 수리/테스트 실패0이다. 기록 PID28 중27은 부재, 60280은 svchost에 재사용됐다. 원 fmt terminal receipt00:30:30.7157139Z 뒤 새 process 생성00:32:14.2317320Z를 대조해 원 identity 생존0/writer0/pending0으로 구분했다. 해당 process 조작0이다. 최초 Get-Process의 null StartTime 관측과 이미 decode된 DateTime을 local로 재parse한 비교 거절은 메타데이터 메모로 보존하고 UTC kind를 유지한 CIM 생성시각으로 정정했다. 이를 시험 실패·product retry와 섞지 않았다. S/T 재감사를 반복하지 않았다.

명시 반환 receipt는 `runtime/verification/default-native-file-guard-r1/slot-return.json`이다. 이 checkpoint는 해당 filesystem 보호 검증이며 정상 N/C의 실제 spawn witness·host·창 인수로 확대하지 않는다.

## 근거와 스킬

원 증거는 `runtime/verification/default-native-file-guard-r1/`의 번호별 resource/stdout/stderr/receipt, 08-fixed-bin, actual-impact5, 16-filesystem-observations3 및 각 case의 records/events.jsonl이다. canonical stage/commit/push0, 새 validation 코드0이다.

modoo.today `test-fixture-and-mock-build` 본문 SHA a393f15f의 acquire/release/recovery·실패에도 원 자원 해제·원 오류/한계 기록을 실제 적용했다. `spec-authoring/plan-authoring` SKILL.md SHA31459a74와 plan.md 참조091d2f8d의 측정/미검증·권한·rollback 분리를 적용했다. 정확 출처는 private HANDOFF와 [GATE 계획](../../../plans/player/presentation/2026-10-24-default-native-file-guard-gate.md)에 있다.
