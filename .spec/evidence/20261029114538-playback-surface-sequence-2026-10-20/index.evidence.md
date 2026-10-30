---
id: evidence:kinekino/20261029114538
title: "순차 surface owner 소비 gate: 거부와 명시 재배정"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 순차 surface owner 소비 gate: 거부와 명시 재배정

## 후속 명시 conservative2 배정 완료

PM의 새로운 저부하2CPU 명시 배정으로 같은 snapshot/source4를 새 prefix `conservative2-*`에서 실행했다. 원 build4 admission 거부는 아래 그대로 보존하며 자동 retry/fallback 또는 Cargo4 성공이 아니다. 고정 EEA9 runner를 직접 --profile conservative2/affinity3/BelowNormal/Job, offline locked -j1/testthreads1로 사용했다.

| 입력/검사 | 실제 결과 | receipt |
| --- | --- | --- |
| source4 r1 surface policy | 6 passed/0 fail/0 ignored | 0,124866ms |
| 기존 NativeChild | 7 passed | 0,1304ms |
| 기존 router | 12 passed | 0,1781ms |
| r1 native all-target strict | collapsible_if1 | 101,11862ms 보존 |
| source4 r2 native all-target strict | -D warnings 통과 | 0,32825ms |
| source4 r2 fmt | 통과 | 0,2726ms |

실제 행동25pass는 r1 입력이다. 유일한 수리는 cleanup.rs의 중첩 Some(attempt)→조건 if를 같은 let-chain으로 합친 것이다. None이면 조건을 평가하지 않고, Some일 때 기존 `(revocation<=old || attempt<=old || terminal이Err아님)`의 OR 괄호/단락 순서를 그대로 유지한다. 정책/단언/새 allow 변경0이므로 행동을 반복하지 않고 r2의 compile/static 증거와 구분한다. r1 frozen4660/원101을 유지하고 r2 exact4 pin **9AAF1E25354AC7B6933A94FBA37FD9FAC69E31024F2DBDFBD4B3AB971EACD4DD**와 repair.diff를 별도 보존한다.

최종 independent/frozen/snapshot r2 source4 일치·보호596 delta0, build4 거부125+strict101을 포함한 receipt7 모두 terminal, 기록 PID13 모두 부재/final-owned=[]이다. root live/index/gitlink 변경0. snapshot에는 최종 r2 source4를 그대로 보존하고 heavy/snapshot을 PM에 다시 명시 반환한다. 실행 binary는 native/debug/deps/kinekino_player-c38450de6d906d68.exe, SHA1C8C01707CEA38DA151A82F26486A386070B2C4988BA6F7DE6355A1959F5AB07/152607744bytes이며 r1 실제6+7+12 identity다. 절대 경로는 test-binary.json에 기록했다. r2 시험 재실행/새 player.exe 빌드를 주장하지 않는다.

actual lease/OS/Destroy/mpv·미디어 한계는 유지한다. 현재 producer a77 새14/PNG를 이 snapshot에 섞지 않았다. 다음 절은 첫 배정의 과거 관측이다.

## 최초 build4 거부 관측

2026-10-20. [계약/구현 준비](../../../plans/player/native-video-surface-contract.md). frozen exact4는 `runtime/coordination/sdk-playback-surface4-r1/pins.json` SHA4660A7E2EDBD75117B0BCE52F27F66697B9A07A695AB74D862B16558A1E1C293. 직접 rustfmt/diff 및 독립 source 검토와 실제 Cargo 시험은 구별한다.

검증 기초는 public-p3-parent-r1의 SDK58r2+자막13+parityrepair1이다. before4 원문/신규3 absent·보호596을 보존하고 exact4만 overlay했으며 내용 해시를 유지한 snapshot mtime 갱신을 수행했다. 새 actual slot producer/PNG는 넣지 않았다. 원 gate 계획은 source 동결 디렉터리 gate-plan.md, 실행 증거는 `runtime/quality-review/sdk-playback-surface-r1/`이다.

실행은 새 고정 runner SHAEEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D에 직접 `--profile build4`를 전달했다. 기존2 wrapper를 중첩하지 않았다. 최초 명령은 native-feature lib `assembly::meari::playback_surface::tests::` 예상6이었다.

원 surface.receipt.json은 launcher111708/terminal=true/exit125, admission.accepted=false, effective_logical_cpus=0, child_pid 없음이다. 사유 cpu_not_below_50_percent: 첫1초 busy118750000/total160000000=74.21875%, minimum_free_bytes23349850112. 도구 실행 exit1과 정책 receipt125를 구분한다. Cargo child를 생성하지 않았고 컴파일/시험은0이다. 이 관측을 시험6 실패 또는 통과로 세지 않는다. 자동 재시도·conservative2 fallback·후속 child/router/strict/fmt 실행은0이다.

최종 independent/frozen/snapshot4 일치, 보호596 delta0, launcher PID 부재/final-owned=[]이다. 실행 binary는 새로 생성되지 않았다. snapshot에는 검증 전 exact4 overlay가 남으며 before 원문을 보존한다. root live/index/gitlink·모델/download/GUI 변경0. 단일 heavy와 snapshot 소유권은 PM에 명시 반환한다.

공개 coordinator 회귀6은 작성·미실행 상태다. actual NativeParentLease/slot/OS Destroy/mpv·미디어 성공 증거는 없다. pending/unknown 최종 outer/lifecycle Drop·재진입·무신호 상태 변화 및 전 stage 모든 오류 Arc 동일성을 이 시험 범위로 넓히지 않는다. 재실행은 새로운 중앙 배정 이후에만 가능하다.
