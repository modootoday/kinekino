---
id: evidence:kinekino/20261029113534
title: 원 forced-success red/green 타입·기본8 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 forced-success red/green 타입·기본8 검증

2026-10-24 PM execution-leases 최상단 단독 heavy 배정으로 실행했다. 근거 root는 `runtime/verification/owned-test-runner-forced-success-type-r1/`다. 원 GATE-PLAN983403FB/expected20 A1980BB3, red effective8 `0F7A1DAF35D28F92A0D5577A818E8906907AE7B912237A404975C7F6F61141C1`→green effective8 `310535659BF454959B9306BB5437E65A66E6EA6DB63D4DCBEE16E5805C451DF7`를 그대로 사용했다. source repair0, timeout 신규3/C2 혼합0이다.

## 입장·실행 범위

rootabb63c8ed1778d185603b2ca62ada9eb016973dd/Mearif85a68df7a6a8fee239d9655895ee15d5eaa5510/rootlock8F2C1D56, tool standalone lock0B2562/8041B, 기존 S public-p3-parent-r1/T native를 사용했다. 최신 SDK N 반환7559366D와 S6081/T589/carry138 ACBA73E0, Meari 별도 fixed3를 구분해 인수했다. 원 runnerEEA9/conservative2 CPU2/mask3/BelowNormal/descendant Job, offline locked j1/thread1이었다. 입장 RAM30,703,702,016B/GPU6055MiB/C103,458,230,272B/writer0이며 각 fmt/strict/build 전 fresh resource 기록이 있다.

승인 표현의 lib no-run은 bin-only 도구에 해당하지 않아 PM이 `cargo test --bin owned-test-runner --no-run --message-format=json`으로 정정했다. 실패 --lib 실행을 만들지 않았다. C2 --lib producer와 구별한다.

입장 metadata 대조 최초1회는 Meari fixed-after3-stat의 `expectedBytes/expectedId` 형식을 일반 `mtime_ticks/file_id`로 읽어 중지했다. 원 STOP-stat-admission을 보존하고 정확3만 맞는 필드로 읽었다. bytes/FI 모두 일치, 이 참조에는 mtime가 없어 mtime 불변을 주장하지 않는다. 당시 overlay/Cargo0이며 파일 변조나 시험 실패로 세지 않는다. 전수 cache SHA 재계산·백업·새 target/clean/download0이다.

## 실제 결과와 producer

red와 green 각각 fmt check, all-target strict Clippy(-D warnings), offline locked bin unit no-run JSON, normal bin build JSON이 통과했다. 각 stage의 compiler-artifact package/manifest/features/profile.test/fresh/실행경로와 새 fixed 복사를 보존했다. 각 list20/default8/ignored12의 정확 이름이 expected와 일치한다. red는 목록까지만, 순수/OS 실행0이다.

| stage | 고정 producer SHA256 | bytes |
| --- | --- | --- |
| red unit | 5BE0A1230026BBBC85E3C5F460ABB551FA9379E8DE8F6D151199C05E72FD0708 |1772544|
| red normal | 0F8B9F1537CAE41FB74C0B29944CE291A04AF4E811B5C132B1CA0E272D5F4D16 |915456|
| green unit | 8BA73689B014D1450EB6AF48813A6AF0A0898EC806C5C832C6FD9D02E01C0BFD |1772544|
| green normal | 02DD28517C07C49687E8FAF7EBE6EB0C2C95D65AD59F6EA6138E25F9FC9CA422 |916480|

각 fixed2.json에 원 Cargo artifact와 FileID/mtime가 있다. 원normal978/unitFC6900/timeout221F는 기존 control로 보존하며 이번 producer로 이름을 바꾸지 않았다. 새 normal은 빌드/pin만 했고 직접 CLI로 실행하지 않았다.

green은 `tests::forced_exit_zero_is_not_selected_test_success` 신규1부터 기존7(typed_input, unknown_flag, exact_output, empty_selection, sink_failure_latches, verdict_cutoff, late_verdict 순서)의 정확 case를 각각 실행해 **8pass**다. green/actual8.json에 전체 정확 이름·stdout·receipt를 연결했다. 각1pass/0fail/0ignored, 전체 runner receipt20 모두 terminal=true/exit0이다. 예상 외 source/type/test 실패나 수리·재시도0이다.

이것은 공개 ProcessReceipt 모델의 forced+zero 거절 정책과 기존 순수 회귀의 근거다. 실제 forced+exit0 반례, 원 retained OS family/EOF/selection, OS selftest red/green 및 직접 CLI의 exit0/1은 아직0이다. red 타입 통과는 permissive 경로의 실제 재현이 아니다. baseline child/GUI/AI/worker/모델 실행0이고 C2 type/list/actual0도 유지한다.

## 복원·보호·귀속

S exact2는 원r2 before SHA/bytes/mtime와 일치하는 것을 확인한 뒤 red→green 두 입력만 교체했다. 완료 후 이번 preimage2 bytes/mtime로 원복했고 restored2 matches2=true다. 다른6은 stage manifest와 일치했고 SDK owned-process baseline23도 fresh SHA/bytes/mtime를 보존·재확인해 delta0이다. index/lock 명시9도 fresh SHA/bytes/mtime delta0이다. root canonical 변경/stage/commit/push0이며 타 담당 WIP를 덮지 않았다.

S6081/carry138/protected6/기존 producer4는 원 manifest SHA를 **retained**하고 fresh bytes/mtime/기록 FileID를 대조한 범위에서 변동0이다. fresh full source/cache SHA나 새 파일 전수탐색 결과라고 표시하지 않는다. Meari fixed3는 bytes/FI 변동0, mtime 근거는 없다. historical alias4 FileID 예외·known2·all_cache_protected=false를 유지한다.

T 기존exe589 metadata 중 변경3은 native/debug/owned-test-runner.exe 및 deps/owned_test_runner.exe의 green normal02DD2851 두 경로와 deps/owned_test_runner-810aae3316a0ab46.exe의 green unit8BA73689로 원 Cargo JSON에 귀속했다. 변경3만 fresh SHA를 계산하고 나머지는 retained SHA+stat를 사용했다. owned-tool 정확 namespace의 exe3만 열거해 기존 경로와 일치했으며 전체 cache의 신규 파일 부재를 주장하지 않는다. after-T589-retained-SHA.json은 이 혼합 provenance를 유지한 다음 반환 기준이다. 복원 S의 r2와 T의 green producer는 서로 다르다.

기존 fixed138에 새 고정4를 더한 carry142 SHA256 `DB258A103A486A6A71BCA4D4EC3BD7BD7E60E0EFABD7EE4FBC4D7192532B4674`를 반환한다. 새 fixed4는 fresh SHA/FI로 재확인했다. 원 receipt20의 종료 뒤 PID 조회 중25660은 현재 Codex의 node.exe/새 시작시각으로 재사용됐고 원unit/runner/cargo가 아니다. 원 terminal receipts와 현재 image를 구분해 pid-reuse-note에 기록했으며 무관 PID kill0이다. writer0, 본 gate의 원 process 잔류 관측0이며 SDK 원 Job physical cleanup의 새 OS 인수 근거로 확대하지 않는다.

다음 단계는 원 고정 red/green을 사용하는 **별도 OS selftest·직접 CLI gate**의 중앙 배정이다. C2r3 source5696BCB8/expected23은 독립 후계로 보존한다. 기존 modoo.today QA review-verification/fixture 본문의 supplied claim·caller·원 cause·모델/actual 구분 원칙을 적용했다. 원source/GATE/expected의 작성시점 actual0은 이 타입/순수 범위만 후속 갱신하며 실제 반례 해결로 표시하지 않는다.
