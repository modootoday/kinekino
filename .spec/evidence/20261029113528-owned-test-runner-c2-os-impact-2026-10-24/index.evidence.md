---
id: evidence:kinekino/20261029113528
title: C2 자체 OS 영향4 실제 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# C2 자체 OS 영향4 실제 검증

2026-10-24 중앙 heavy 배정으로 OS-inputs4 SHA527F5730의 정확4만 실행했다. **결과4 모두 예상과 일치**하며 direct exit1은 의도된 실패 반환이다. 원 tools actual4와 별도 C2 실행 분모다. raw root는 `runtime/verification/owned-test-runner-c2-os-r1/`, 입력은 source18 5696BCB8에서 만든 C2 lib88192259989126A65168F90FF4074847061AD302D33B7C5D33113096A68B268E/1856512B 및 normalB014C8BCE406468143A82051C37759F5337913E972823D9AC46DCDD5B8A55F92/926208B다. baseline8440/293376B·indexCF4D/3365B와 함께 fresh SHA/bytes/FI 입장 확인 및 전후 불변을 기록했다.

| 정확 tests:: case | runner exit / elapsed | 실제 결과 |
| --- | --- | --- |
| forced_exit_zero_with_one_pass_is_failure |0 /23343ms|selftest1pass. 내부 fixture prior1pass/exit0, Complete/forced/Empty/selection1/EOF2, cutoff-specific TimedOut·정확 first_error/passed=false 단언 통과 |
| fixture_parent_exit_with_live_descendant (직접CLI) |1 /12086ms|최종 stdout passed=false, Complete/forced/exit0/Empty/selection1/EOF2, first_error selected test family required forced termination |
| normal_selected_case_has_original_family_receipt_and_raw_eof |0 /11521ms|selftest1pass, 내부 fixture_pass exit0/Empty/nonforced/selection1/EOF2/passed=true 단언 통과 |
| sink_failure_retains_original_incomplete_owner_after_physical_close |0 /11448ms|selftest1pass, 원 FinishIncomplete/Receipt cause와 first_error original outer sink failure 보존, physical_closed/EOF2, owns_original=true/passed=false/receipt_recorded=false 단언 통과 |

selftest3은 outer45s/cleanup5s→inner15s/3s, direct는15s/3s를 유지했다. fresh case별 evidence/TEMP·같은 original SDK Job/Deadline을 사용했고 retry/kill/Abort/기한 확대0이다. 원 runnerEEA9 CPU2/mask3/BelowNormal/descendantJob, 각 resource.json에서 RAM16GiB/GPU3072MiB/C12GiB/writer0 조건을 확인했다. Cargo/overlay/새 코드/다운로드/새 target0이다.

01 A PID51376/creation134372854624271519, B32248/134372854735654255; 02 직접B35136/134372855076285627; 03 A14220/134372855530180866,B14484/134372855641171919; 04 A66460/134372855883881280,B52276/134372855994878250다. 모두 같은 고정 C2 lib의 원 SDK receipt identity로 연결된다. A는 selftest unit, B는 fixture unit이며 outer 성공으로 내부 실패를 바꾸지 않았다. 원 runner normal PID는71256/61408/28684/91544이며 정상 종료 관측과 SDK child-family 종료 근거는 구분했다.

forced 두 건은 fixed baseline parent-exit→hang descendant 자극을 사용했다. 같은 Job 상속/원 Empty receipt·force·raw EOF를 인수했으며 C original Child.wait exit0는 fixture 단언+prior1pass 근거다. C/D 개별 creation·D exit/개별 Job readback은 출력되지 않아 주장하지 않는다. normal/retained 두 건은 fixture_pass이며 baseline child는 guard 대상일 뿐 실행하지 않았다.

retained B의 저장 snapshot은 ownership original_finish_incomplete, stage Receipt, first_error/receipt_error가 같은 원 sink 실패, termination_error=null, process_sink_records0, child exit0/Empty/nonforced/physical_closed/EOF2였다. **내부 outer-receipt.jsonl은 부재**이며 가짜 성공 receipt를 만들지 않았다. injected FailingWriter는 실제 디스크 장애가 아니며 JsonSink의 원 failure latch와 carrier 의미를 검증한 것이다. source/test의 Box::leak로 같은 원 실패 carrier를 테스트 프로세스 teardown까지 보유했다. 이것은 CLI foreground `retain_until_closed` 루프 전체를 직접 실행한 근거나 일반 caller Drop 안전 보장은 아니다. 각 outer A는 Complete라 해당 CLI failure-owner 분기를 별도로 인수하지 않는다.

저장 outcome의 verdict_sink_records0은 final append 전 snapshot이고 direct 최종 stdout records1과 구별한다. B의 EOF는 SDK의 raw 읽기 관측이며 stdout marker만으로 대체하지 않았다. stdin EOF는 요청이지 protocol ACK가 아니다. 실제 파일54개를 evidence-files.json SHA `DE1C18C2F33BD2A61173ACC53120E6BE5F9C272AD8614F094AAA0990D780A986`로 고정했다. FileID는 post-terminal 관측으로 생성 순간 custody를 새로 입증하지 않는다.

원 입력4 fresh SHA/stat/FI 불변, canonical13/index9/SDK23 fresh SHA/stat 및 carry144 retained SHA+fresh stat/기록FI 감사에서 변동0이다. 최신 restored alias FI를 적용한 보호6 및 인수 root producer3 metadata도 변동0, 좁은 owned namespace4는 기존 C2 producer를 유지했다. 전수 S/T 재검색·cache 재해시/중복 백업0, historical alias/known2/all_cache_protected=false를 유지한다. S의 package는 이전 type gate가 복구한 부재 상태 그대로다.

실행 summary4·원 raw/receipt 보존 후 기록 PID 숫자 조회 부재/writer0을 확인했다. 이 조회를 C/D 개별 custody나 SDK Job 종료의 대체 근거로 쓰지 않는다. carry144 B760BC6E19C2E4DB8A80FBAF02B56C381770B677FBC12FECEA0E24BAE7FA284C는 새 BIN 없이 그대로 반환한다. canonical/.gitignore/stage/commit/push/GUI/AI/worker0이다. 기존 modoo QA review-verification/fixture 원칙의 원 cause·owner/물리·성공/주입·실제 분리를 적용했다. 전체 ignored12·원 timeout/late·모든 OS 지원을 인수한 결과는 아니다.
