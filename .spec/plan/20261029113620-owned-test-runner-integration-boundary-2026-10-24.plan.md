---
id: plan:kinekino/20261029113620
title: owned-test-runner 통합·공개 경계 계획
status: active
created: 2026-10-29
updated: 2026-10-29
---
# owned-test-runner 통합·공개 경계 계획

Status: active (proposal, PM 미채택). 2026-10-24 중앙 source-only 배정. SDK r2 sole-heavy 중 작성하며 구현 이동/기존 source 수정/Cargo/S/T/OS/redgreen/stagecommit0이다. [패키지 경계 제안 ADR](../../../adrs/player/owned-test-runner-package-boundary-2026-10-24.md)과 연결한다. 이 계획은 기존 forced-success actual GATE-PLAN이나 timeout 검증 정본을 대체하지 않는다.

최신 source-only 입장 계획은 `runtime/development/owned-test-runner-c2-r3/HANDOFF.md`의 Latest standalone C2 gate binding에 좁게 갱신했다. r3 source18 5696BCB8은 r1 raw16+r2 outcome_record1+r3 tests1의 entry별 참조다. 기존 S6081/T589/carry142·원 sibling SDK23·lock0B2562 기준, 부재 package의 정확18 placement/preimage·원복 책임과 C2 --lib authored23/default11 검증을 명시했다. 원 forced actual4 SOT00923855는 다른 producer의 근거이며 C2 actual은 계속0이다. 다음 heavy Meari 진단type 우선, 현재 Cargo/overlay/OS0·C1생략을 유지한다.

후속 PM 검토로 단일 packages lib+bin 경계를 추천안으로 인수했다. **중간 tools C1은 생략을 기본**으로 하고 forced actual gate 뒤 직접 C2 책임분리 후보로 진행한다. 아래 C1은 선택 대안의 이력이며 신규 중간도입 실행 지시가 아니다. [C0 정확8+lock 읽기 감사](../../../sots/verification/production/owned-test-runner-c0-public-source-audit-2026-10-24.md)에 cache registry33/원문66·archive checksum 일치와 프로젝트 권리/전수검사 공백을 기록했다. 프로젝트 license/공개 승인·최종 source 채택은 여전히 미결정이다.

후속 중앙 로컬 구현 배정으로 `runtime/development/owned-test-runner-c2-r1/`에 private lib+bin 책임분리 후보 source18 `AEA49D12C219CD858D393C91A0A5505518AF6A836391F7B1AE1216D026CD041E`를 실제 작성했다. before10/원lock 복사·delta/expected23/HANDOFF/별도GATE-PLAN을 보존한다. Rust leaf15 fmt/check만 통과했고 authored23/default11/ignored12의 타입/등록/actual은0이다. 아래 합성0·구현0 문장은 제안 당시 이력이며 이 private 작성 범위만 갱신한다. canonical/S/T/.gitignore/원frozen8·red/actualforced gate는 변경0이다. 원owner 유지 loop를 application으로 옮기고 CLI가 코드 반환 전 complete guard도 process lifetime까지 retain하여 기존 process::exit의 무destructor lifetime을 보존했다. SDK typed 요청도 domain의 기존 CLI admission을 공유하며 이 연결 차이는 C2 gate의 검증 대상이다.

## Outcome

채택·검증된 감독 동작의 한 Rust 라이브러리와 얇은 CLI를 책임 있는 공개 소스 패키지로 관리한다. 제품 player/player-cli 기능, 공통 프로세스 소유권 SDK, CPU 예산 launcher, 외부 실행 provenance를 각각 분리한다. 새 패키지 경로가 생겨도 기존 private 고정 BIN과 실제 기록은 그 원 producer 경로/해시를 보존한다. 아직 실측하지 않은 forced-success를 공개 디렉터리 이동으로 지원 완료로 바꾸지 않는다.

추천 목적지는 `packages/owned-test-runner/` 한 패키지의 lib+bin이다. 처음에는 `publish=false`/자체 `[workspace]`를 유지하는 독립 개발지원 패키지로 두고 product root 기본 멤버나 release 의존성에 자동 편입하지 않는다. 실제 공통 Rust 호출자가 생기기 전 두 crate로 나누거나 SDK를 새로 일반화하지 않는다. root workspace 편입은 별도 PM 판단이며 새 root lock/feature graph 검증이 필요하다.

## Authority

| 관심사 | 결정권/소유권 | 현재 상태 |
| --- | --- | --- |
| Rust·SDK/CLI 우선, private/public 경계, push 금지 | 사용자/AGENTS | 확정 |
| shared S/T·단독 heavy·root index/커밋 직렬 배정 | PM | 이번 source-only, SDK r2 heavy |
| owned-process 원 owner/Deadline/Complete·Incomplete 계약 | 공통 SDK 담당, Media 협의 | 변경 제안0 |
| exact-one test 성공 정책/receipt·CLI exit | Media 제안, PM 인수 | r3/timeout 검증과 forced-success 미검증 분리 |
| public package명/standalone 여부/license·공개 채택 | PM·저장소 권리자 | 제안·미결정 |
| release player/player-cli SDK 기능 | 제품 SDK/CLI 담당 | 개발 도구로 대체하지 않음 |

## Evidence

2026-10-24 정확 manifest/소스만 읽고 조사했다. `Get-ChildItem tools -Directory`, `git ls-files tools packages/owned-process`, `git status --short --untracked-files=no`, `git check-ignore` 및 아래 명시 소스에 한정한 rg를 사용했다. target/cache/runtime 전체 재귀 탐색0이다. 당시 tracked 작업 상태에는 SDK 담당의 `src/application/mod.rs` 수정과 Meari submodule dirty가 관측됐으며 소유하지 않고 수정0이다. 현재 HEAD 숫자를 새 검증 producer로 사용하지 않았다.

| 현재 실제 경로 | 관측 책임/직접 의존성 | 상태 |
| --- | --- | --- |
| `tools/owned-test-runner/` | canonical 디렉터리 없음, Git tracked0 | 경로 이름은 private manifest의 논리 목적지일 뿐 공개 채택 아님 |
| `runtime/development/owned-test-runner-r3/source8.json` + 각 entry의 raw source | Cargo.toml/main/application/guarded_artifact/receipt/request/test_result/tests의 논리8. r2 raw6+r3 raw2 참조 | frozen parent7336E34F, 실제 기본7/OS2는 별도 SOT |
| `runtime/development/owned-test-runner-timeout-r1/source8.json` | r3 대비 tests1만 변경, 다른7 동일 | 003D22E2. 타입/list20/default10 실제 통과, timeout OS0 |
| `runtime/development/owned-test-runner-forced-success-r1/effective8.json` | app/tests2 successor, 다른6 동일 | 310535/소스2 1080, pure/type/OS/CLI actual0 |
| 같은 forced-success의 `red/effective8.json` | r3 nonzero primary 추출 app+green과 같은 tests | 0F7A1DAF, source-only control; 원r3 그대로라고 부르지 않음 |
| 공유 S의 `tools/owned-test-runner/` | previous gate 입력 자리, 원r2 bytes/mtime로 복원 | canonical 아님, 이번 읽기/쓰기0 |
| `packages/owned-process/Cargo.toml`, src/lib.rs 및 supervisor/domain/windows/evidence | 공개 workspace 공통 child-family SDK. 직접 target Windows windows=0.62.2; test-support child gated | protocol 의미는 caller 소유라는 lib 문서 확인 |
| `tools/resource-runner/Cargo.toml`, README | 독립 workspace, 외부 crate 의존성0. CPU/priority/Job admission·기동/receipt; timeout/KILL_ON_CLOSE 없음 | 이미 tracked, 별도 제품책임. owned-process 의존성0 |
| `tools/player-ui-sdk-bridge/Cargo.toml` | 독립 workspace UI 연결 검증; 제품 SDK·Meari·UI plugin 등 의존 | owned-test-runner 호출자 아님 |

runner manifest 직접 의존성은 owned-process path `../../packages/owned-process`, sha2 `=0.10.9`, serde_json `=1.0.154`, Windows windows `=0.62.2`(Foundation/Storage FileSystem)다. 현재 runner의 `[workspace]`는 독립 lock 경계를 만든다. root Cargo workspace members/default-members에는 runner가 없고 owned-process는 member이며 root dev-dependency, media-vad-executor의 production dependency다. 이 소비 관계는 시험 해석을 owned-process에 섞지 않을 근거다.

`rg owned-test-runner|owned_test_runner`를 src/tests/packages/owned-process/tools/resource-runner/tools/player-ui-sdk-bridge의 Rust/TOML/Markdown 및 root Cargo/README에 한정했을 때 직접 호출0이다. 모든 저장소 호출자0이라고 일반화하지 않는다. 실제 현재 호출은 private normal/nested OS execution-input과 중앙 운영 gate의 fixed CLI다. Request::parse는 local absolute artifact pins/정확 case/예산만 받으며 Request::arguments는 exact/thread1/ignored를 만든다. pin은 operator input이고 compilation provenance를 자체 인증하지 않는다. application은 guard→원 Supervisor→EOF/exact-one→verdict를 수행하고 main은 인수/JSON/exit 및 retained original owner foreground 관측을 한다. retain 대기를 CLI에서만 보유하는 현 상태는 lib 추출 때 정리할 책임이다.

기존 .gitignore는 tools의 Cargo/src/tests 책임 범주를 이미 허용하지만 `packages/*`는 선택한 package root만 허용한다. `git check-ignore`에서 새 packages/owned-test-runner/src/lib.rs와 runtime source8은 ignored, 가상 tools/owned-test-runner/src/main.rs는 ignored가 아니었다. 새 package에는 책임 root `!/packages/owned-test-runner/` 한 항목만 제안하며 기존 packages 공통 src/tests/Cargo/README 규칙을 소비한다. 파일별 exception 목록이나 `/runtime/`·`/.spec/` 광범위 허용은 필요 없다.

## Unverified

- 패키지 경계 제안은 아직 구현·Cargo metadata/lock/feature/상대경로 검증0이다. standalone `[workspace]`로 root 기본 graph가 실제 불변인지 별도 allocated gate에서 확인해야 한다.
- runner의 새 public Rust API 안정성/Windows 외 target 지원은 미검증이다. 현재 main은 Windows 외 exit2이며 guarded_artifact는 Windows 전용이다. crate import/진입점의 cfg 경계를 별도 설계·검사해야 하고 cross-platform 지원으로 홍보하지 않는다.
- forced-success pure/type redgreen 이후 OS selftest/직접 CLI가 미실행이다. timeout 타입/default10을 forced actual 증거로 합산하지 않는다. 두 test leaf를 합치면 expected는 **소스상23/default11/ignored12**지만 실제 merge/등록은0이다(17 parent+timeout3+forced3). 이 계획에서는 merge하지 않는다.
- root/runner/owned-process manifest에 license field가 없고 scoped git ls-files LICENSE*/NOTICE* 결과도 없었다. 권리자가 소유한 Rust 소스인지와 재배포 허가·프로젝트 license 결정을 확정하지 않았다. publish=false는 저작권 허가가 아니다. sha2/serde_json/windows 및 전이 라이선스/notice와 버전별 SPDX·원문 해시는 승인 전 정확 lock graph/공식 배포 원문으로 확인할 일이며 추정 라이선스를 채우지 않는다.
- source8의 명시 Rust 파일을 읽어 역할/입력 경계를 확인했으나 전용 secret scan·전 소스 문자열 감사·history review는 수행하지 않았다. 공개 전 token/key/내부 운영경로/개인미디어/회사 비공개 입력/타사 복사 출처를 exact proposed source manifest로 검토한다. runtime JSON·절대 사용자 경로·PID/FileID·raw 출력·고정 exe·registry 사본·Cargo producer logs는 공개 소스가 아니다.
- 새 package의 제품 필수성은 runtime dependency가 아니라 SDK/CLI 검증의 재현 가능한 개발지원이다. 이 근거로 저장소 공개가 적절한지는 PM이 결정하며 crates.io 배포·외부 계정·네트워크 작업 권한을 추론하지 않는다.

## Phases

1. **현재 후보를 그대로 검증한다.** forced GATE-PLAN대로 먼저 pure/type redgreen을 별도 배정한다. 이후 OS selftest red/green과 직접 CLI red/green은 각각 명시 배정이다. unknown-test·컴파일 실패·setup 실패는 red 반례가 아니다. 첫 실패 원 BIN/raw/owner/receipt를 보존하며 기존 예산/보호 계약을 유지한다. 완료 조건은 stage별 pin·각 실행 분모·정확 예상/실측 SOT와 슬롯 반환이다. 검증 전 통합/이동 commit 없음.
2. **통합 입력을 고정한다.** 부모r3 adoption부터 timeout test leaf, 검증된 forced app/tests를 순서대로 리뷰한다. 향후 통합 branch는 원8→timeout1→forced2의 책임 delta를 기록하고 겹치는 tests 병합은 새 명시 source manifest/patch로 만든다. 모델3을 다른순수1로 대체하지 않는다. 최종 source23/default11 예상과 기존 테스트 이름·원error·Complete계약을 대조한다. 완료 조건은 PM이 선택한 정확 논리파일별 before/after hash 및 미검증 표기다. 새 source가 생겼으므로 기존 binary 증거는 조상 근거로만 참조한다.
3. **공개 소스·license 사전 검토 묶음 C0.** 신규 public root 필요성/재배포권/secret 검토 결과를 private 정본에 기록하고 PM이 공개 경계를 결정한다. 해당 package가 없는 상태에서 .gitignore 허용만 먼저 확대하는 commit은 하지 않는다. 완료 조건은 정확 proposed public file inventory와 license/notice 출처, 제외한 private artifact 범위 및 PM 결정이다.
4. **기능을 유지한 tools 도입 묶음 C1 (선택적 중간 checkpoint).** PM이 원r3/후계의 추적 가능한 도입이 필요하다고 판단하면 논리8 `tools/owned-test-runner/{Cargo.toml,src/main.rs,src/application.rs,src/guarded_artifact.rs,src/receipt.rs,src/request.rs,src/test_result.rs,src/tests.rs}`와 Cargo.lock/README의 정확10파일을 임시 standalone checkpoint로 검증·커밋한다. license 문서가 추가되면 먼저 inventory에 정확 경로를 확정한다. 원r3만 C1a, timeout1 C1b, 검증된 forced2 C1c로 나누고 각 commit 메시지에 그 시점 실행범위/잔여 OS를 명시한다. 같은 tests 파일 delta라도 서로 다른 feature 맥락을 섞지 않는다. 도구가 canonical에 아직 없으므로 `git mv`나 기존tracked WIP 이동으로 표현하지 않는다. C1은 필수 이중도입이 아니며 직접 C2가 더 작은 검증·커밋 집합이면 생략한다.
5. **책임별 lib+CLI 패키지 묶음 C2.** 새 package의 정확 source file inventory를 PM 승인 전 확정한다. 추천 책임 경로는 `packages/owned-test-runner/Cargo.toml`, Cargo.lock, README.md, `src/lib.rs`, `src/domain/{mod.rs,request.rs,test_selection.rs}`, `src/application/{mod.rs,run_selected_test.rs}`, `src/infrastructure/{mod.rs,artifact_guard.rs,json_receipt.rs}`, `src/presentation/{mod.rs,cli_input.rs,outcome_json.rs}`, `src/bin/owned-test-runner.rs`, `tests/{policy.rs,windows_selected_test.rs}`다. 위 경로들은 제안 inventory이지 생성됨이 아니다. 기존 modules를 기계 rename하지 않고 request DTO/예산·선택정책과 OsString 파싱, 원 owner 유지·실행과 JSON 표시, Win32 guard/sink를 분리한다. policy는 CLI/lib가 동일한 application API를 호출하도록 한다. 기존 Outcome/retained carrier의 소유권을 보존하고 main은 조립/입력/출력/exit만 담당한다. 새 cancellation/router/product policy나 자동 kill 기능은 추가하지 않는다. owned-process dependency 경로만 `../owned-process`로 바꾸고 원 SDK 코드·미디어 SDK caller를 수정하지 않는다. 별도 tool shim은 실제 호출자가 요구할 때만 한시적으로 추가하며 없으면 중복CLI를 남기지 않는다.
6. **C2 검증과 exact commit.** .gitignore root 허용1과 새 package source/README/lock를 하나의 기능완결 묶음으로 두며 C1이 있었다면 그 tracked tool 파일의 제거를 같은 책임 변경에서 명시한다. root Cargo 변경이 필요하면 별도 C3 workspace decision으로 분리한다. fmt/test/strict Clippy, fixed unit/normal JSON·source provenance, lib API 소비/CLI 같은 정책의 성공·오류·retainedowner·지원범위 회귀를 중앙 슬롯에서 수행한다. 기존 Request에는 취소 명령이 없으므로 취소 지원을 새로 구현하거나 지원으로 표시하지 않고 이 부재와 원 cutoff/정리 계약을 보존한다. 순수 시험과 path이동 영향 OS는 gate 차이를 분석해 필요한 것만 별도 배정하며 이전 경로 binary로 새경로 성공을 대체하지 않는다. cargo lock root/standalone 전이를 비교하고 새 dependency download0을 지킨다. 완료 조건은 exact index allowlist diff/민감경계·license 검토·시험/SOT·원 S 복원/slot 반환이다. PM만 명시 파일 stage/commit하며 git add ./-A 및 push0이다.

모든 묶음은 실제 staged path를 위 최종 inventory의 **파일별 literal**로 한정한다. 폴더째 stage하거나 제안에 아직 없는 파일을 편의상 추가하지 않는다. shared index의 타 담당 staged 상태를 덮거나 unstage하지 않는다. C1을 생략하더라도 parent/timeout/forced의 provenance와 검증 차이는 로컬 정본 및 commit 설명에서 분리한다.

## Rollback

후속 C2 읽기 검토(2026-10-24): 아래 시험 합성과 API 제안은 rollback 대상 구현이 아직 없다. 라이선스 미결정은 외부 배포/publish 판단에 남기며 사용자가 승인한 로컬·비공개 후계 개발의 중지 gate로 확대하지 않는다.

1은 이번 gate preimage exact2 bytes/mtime 복원과 소유한 원 owner 정리 관측/슬롯 반환으로 되돌린다. 실제 red 실패 기록은 삭제하지 않는다. 2는 새 private 통합 후보를 보류하고 frozen r3/timeout/forced를 계속 별개로 참조한다. 3은 공개 경계 결정을 보류하며 public 예외0을 유지한다.

4~6 구현 전에는 rollback할 canonical 변경이 없다. 나중에 PM이 커밋했다면 공유 작업물을 reset/checkout 덮어쓰지 않고 해당 exact commit의 검토된 revert를 PM이 직렬 수행한다. tools↔packages shim/manifest/lock/.gitignore를 함께 되돌려 build 가능한 상태로 만들고 새 producer cache를 이전 소스 출처로 오인하지 않는다. fixed BIN·원 provenance·SOT는 보존하고 private ignored evidence를 삭제하지 않는다. root member 채택은 별도 C3이므로 그 전후 lock/feature diff와 역변경을 따로 검증한다. 외부 publish/push는 계획에 없으며 수행하지 않는다.

적용 스킬: `modootoday/spec-authoring/plan`의 plan.md SHA256 `091d2f8d92113d9995ab143ec583529c1ef98413017f3c4d8655406d768a9e2d`, 요구한 `modootoday/spec-authoring/plan-authoring` 본문을 읽고 여섯 섹션/측정과 제안·미검증/단계종료·복구 기준을 적용했다. frontend-first 검색 결과는 SDK/CLI 우선과 도메인이 달라 채택하지 않았다. `%USERPROFILE%/.codex/skills/architecture/SKILL.md` 및 trade-off-analysis.md를 읽고 ADR 대안/단순성/재검토 조건을 적용했다. 스킬의 예시 docs 경로 대신 프로젝트 .spec/adrs 규칙을 따랐다.

### C2 후속: exact leaf 합성 제안 (읽기만, 합성0)

세 frozen 입력의 diff hunk와 함수 이름을 읽었다. timeout tests의 원r3 대비 변경은 import 추가, 파일 앞 TimeoutFacts/TimeoutCase/TimeoutEvidence 및 helper·모델3 추가, 기존 timeout OS 본문 교체다. forced tests의 원r3 대비 변경은 `static NEXT` 뒤 새 pure1/OS1/stimulus1 추가뿐이다. 서로 같은 파일을 바꾸지만 본문 교체 위치는 겹치지 않는다. top-level timeout import와 forced pure 내부의 같은 타입 use는 서로 다른 scope여서 설계상 이름 충돌이 아니며 compiler 확인은0이다.

최소 합성은 timeout tests SHA `C813D591B288CFFF1A87A31C2E15E1806A19AB73E4127B0B6E94C6EE729FE24D`를 시험 leaf의 기준으로 삼고, forced tests SHA `A6311C14F8ACA8B3BE00B17ACCC6B915860FFCCE2B82E91932471B49CDA2FC78`에서 **새 세 함수만** NEXT 뒤에 한 번 넣는 것이다. pinned/request/retained/FailingWriter/late 지원자는 각 한 벌만 유지한다. forced tests 전체를 timeout 위에 덮어쓰면 검증된 timeout 본문을 잃으므로 하지 않는다. app는 forced SHA `EECF626ED7E9CC3756308611784AE9CAAA6261D582563EE8024DD6F78420A829`를 사용하며 timeout은 app 변경0이다. 다른6 원r3 entry는 그대로다. 논리 변경 대상은 app1+tests1 **exact2**이고, 실제 합성 bytes/hash/patch는 아직 없다. red app0F7A는 대조용 control로만 보존하고 합성본에 넣지 않는다.

semantic 경계도 충돌하지 않는다는 소스 판단이다. timeout complete validator는 forced+**nonzero**/selection없음/Other 원 오류를 요구한다. forced app는 nonzero Other 우선을 유지하고 forced+**zero**만 TimedOut으로 거절한다. timeout incomplete의 원Deadline cause·partial stored snapshot 검사는 FinishIncomplete에만 적용된다. forced Complete0의 selection1/EOF2와 섞어 넓은 OR 수락으로 바꾸지 않는다. 두 OS 사례의 Request budget도 각각 원1s/300ms와 원15s/3s를 유지한다.

합성의 예상 **기본11**은 다음의 일대일 매핑이다(실제등록0).

| 출처 | 정확 `tests::` 이름 |
| --- | --- |
| parent7 | typed_input_builds_only_exact_single_thread_arguments |
| parent7 | unknown_flag_bad_budget_and_nonlocal_path_fail_before_launch |
| parent7 | exact_output_accepts_one_pass_and_distinguishes_should_panic |
| parent7 | empty_selection_wrong_name_duplicate_summary_and_ignored_are_not_passes |
| parent7 | sink_failure_latches_original_error_without_successful_retry |
| parent7 | verdict_cutoff_uses_original_end_before_at_and_after_without_sleep |
| parent7 | late_verdict_preserves_original_error_and_bounded_secondary_overflow |
| timeout3 | timeout_complete_requires_original_forced_nonzero_and_job_failure |
| timeout3 | timeout_incomplete_preserves_cause_and_stored_partial_vs_closed_owner |
| timeout3 | timeout_never_launched_and_unobserved_facts_are_not_acceptance |
| forced1 | forced_exit_zero_is_not_selected_test_success |

예상 **ignored12**의 매핑은 아래와 같다. 같은 이름의 parent timeout 시험은 새 시험을 추가한 것이 아니라 본문 교체1이다.

| 출처/변경 | 정확 `tests::` 이름 |
| --- | --- |
| parent 유지 | delayed_verdict_keeps_stored_pre_cutoff_record_and_returns_failure |
| parent 유지 | delayed_verdict_preserves_original_receipt_failure_and_late_secondary |
| parent 유지 | fixture_hang |
| parent 유지 | fixture_nested_family |
| parent 유지 | fixture_pass |
| parent 유지 | nested_family_is_observed_by_same_outer_owner |
| parent 유지 | normal_selected_case_has_original_family_receipt_and_raw_eof |
| parent 유지 | sink_failure_retains_original_incomplete_owner_after_physical_close |
| parent 이름/timeout 본문 교체1 | timeout_stays_failure_after_original_family_is_forced_closed |
| parent 유지 | zero_selected_tests_is_failure_even_with_exit_zero_and_closed_family |
| forced 신규 | forced_exit_zero_with_one_pass_is_failure |
| forced 신규 stimulus | fixture_parent_exit_with_live_descendant |

합계23은 parent17+timeout 순수3+forced3이다. 이미 timeout에서 actual10pass가 있어도 합성 unit의 등록23/순수11 통과를 자동 승격하지 않는다. 이후 C2의 파일 분리 시 helper를 cfg(test) 모듈로 옮기되 exact case 이름은 가능한 유지하고 변경이 불가피하면 old→new 이름을 gate manifest에 매핑한다. root integration tests가 별도 crate가 되면 현재 `pub(crate)` helper 접근이 달라지므로 공개 API를 시험 편의로 넓히지 않는다. 순수 policy와 application 내부 test는 lib 내부 cfg(test), 외부 public API smoke만 integration tests로 두는 안이 더 좁다. 신규 기능 시험과 같은 단언을 이름만 바꾼 중복 시험은 만들지 않는다.

### C2 후속: 좁은 lib/API와 원 owner 계약 (제안)

우선 기존 `execute`의 소유권 동작을 유지한 opaque `RunOutcome`을 중심으로 설계한다. 개념적 서명은 `execute(validated_request) -> Result<RunOutcome, PreLaunchFailure>`다. 이것은 지금 컴파일되는 새 API가 아니라 책임 설계다. `PreLaunchFailure`는 원 io::Error를 소유하고 **원 process owner가 만들어지지 않은 pre-spawn 실패에만** 사용한다. spawn가 반환한 LaunchFailure는 owner가 Some든 None이든 원 stage/error와 함께 RunOutcome 안에 남긴다. spawn 이후의 finish/receipt/raw/selection/verdict 오류를 일반 Err로 내려 owner를 잃거나 저장 성공으로 바꾸지 않는다.

RunOutcome은 현재 Ownership의 Complete/FinishIncomplete/LaunchIncomplete와 원 request/deadline/guard/sinks를 내부에 그대로 보유한다. Clone/Serialize로 owner를 복제하거나 JSON으로 소유권을 이전하지 않는다. 외부에는 borrowed status/원cause·bounded secondary/receipt 관측·EOF/selection/physical_closed와 성공 판정만 제공한다. 관측 DTO는 소유권 ACK가 아니며 raw_json을 caller가 다시 주입할 수 없다. Completion은 SDK의 opaque 원 객체를 사용하고 모델 값으로 생성하지 않는다. SDK 자체의 Complete나 ProcessReceipt 구조는 바꾸지 않는다.

라이브러리의 원 `observe_retained(&mut self)`가 같은 원 handles/같은 Deadline만 poll하고 원 first cause를 보존한다. CLI의 기존 leak+foreground 관측을 복사한 또 다른 cleanup engine은 만들지 않는다. main은 borrowed snapshot을 presentation JSON으로 쓰고 동일 application status로 exit0/1/2를 정한다. 출력 실패가 추가돼도 원 process cause를 덮어쓰지 않고 diagnostic 실패를 구별한다. 새 취소 프로토콜·자동 retry·PID reopen·fresh deadline·전역kill은 이 리팩터의 범위 밖이다.

선택적으로 `try_into_closed(self) -> Result<ClosedOutcome, Self>` 같은 소비 경계를 둘 수 있다. 미종료이면 **같은 self 전체**를 Err로 돌려 원 owner/guard/cause가 호출자에게 남아야 한다. ClosedOutcome도 Complete 성공/Complete 실패/원Incomplete 종료후실패를 구분하고 EOF·저장 ACK를 physical_closed만으로 채우지 않는다. 이 변환은 새 final receipt 쓰기나 Incomplete→Complete 승격을 하지 않는다. 기존 closed failure carrier를 CLI 종료까지 retain하는 방식과 호환 여부는 C2 구현 시 검증하며, 필요하지 않으면 새 타입/변환을 생략하고 opaque RunOutcome만 유지한다.

`#[must_use]`나 opaque 타입만으로 Rust caller의 Drop을 막을 수는 없으므로 원 owner가 자동으로 안전해진다고 보장하지 않는다. 현재와 같은 explicit retain 계약을 문서화하고, unresolved path가 io::Error/JSON만 남긴 채 drop되는 API를 금지하는 것을 리뷰 조건으로 둔다. Drop에 새 termination/대기 정책을 추가하지 않는다. 기존 SDK lifetime에 대한 더 강한 보장이 필요하면 SDK 담당과 별도 결정한다. pure model 검증을 이 lifetime 보장의 actual 근거로 쓰지 않는다.

SRP 경계는 domain의 선택/성공 조건·validated request 값, application의 원 SDK 실행/retained 관측·오류 집계, infrastructure의 Win32 artifact guard/durable sink, presentation의 OsString parsing/JSON/exit 변환이다. 최종 public lib API는 실제 caller가 요구하는 `execute`, borrowed status, retained observation 정도로 제한하고 test injection writer/verdict hook은 cfg(test)에 둔다. 기존 CLI 인수/exit/JSON schema·nonzero우선·forced0 local cutoff 분류는 actual forced gate 뒤 최소 차이만 보존한다. 실제 반례·원 red/green GATE-PLAN은 수정하지 않았다.

### C2 r2 이후 공개 API 사용 예 판단 — 제안만

2026-10-24 PM source-only 후속, UI B4 heavy 중이다. 새 source/도구/추상화/doctest·시험 추가0, Cargo/OS0이다. C2 r2 BDF8은 r1 AEA49D12의 누락 Instant import exact1 후계이며 raw17과 expected23을 공유한다. source name resolution은 아직 미검증이다.

실제 소비자 근거는 두 종류다. 기존 중앙 normal/nested/timeout gate는 fixed binary CLI를 실행하며 Rust library caller가 아니다. 앞선 scoped source search의 외부 제품 Rust caller0 범위는 그대로이고 새 소비자를 추정하지 않는다. C2 소스 내부에서는 presentation/cli.rs가 `crate::{Request, execute}`를 import해 공통 lib facade를 호출한다. 반면 기존 merged tests 대부분은 `application::execute`를 직접 호출하며, bad-input case의 typed 두 단언도 `Request::validate()`만 검사한다. 즉 public execute→PreLaunchFailure의 소스 경로는 보이지만 그 facade 호출을 실제 시험한 근거는0이다.

현재 추천은 **실제 facade 경로를 검증할 필요가 생기면 기존 정확 시험 내 최소 단언을 제안**하는 것이다. 별도 doctest 실행 target/새 시험 이름/일반 client wrapper는 지금 만들지 않는다. 기존 `tests::unknown_flag_bad_budget_and_nonlocal_path_fail_before_launch`에서 invalid-case typed Request를 root `crate::execute`에 전달하고 Err의 borrowed original error가 InvalidInput인지 확인하는 범위가 적절하다. 이 입력은 원 Deadline 생성 뒤 domain admission에서 반환되고 artifact open/spawn 전이므로 OS fixture나 baseline child를 요구하지 않는다. 이는 기존 validate-only 단언과 구별되는 facade 연결/오류형 보존 검증이며, 추가한다면 같은 시험 이름/expected23은 유지하되 **새 source revision과 before/delta pin**이 필요하다. 지금은 제안만이고 r1/r2 tests/source18은 불변이다.

성공 경로/retained owner를 doctest에서 임의 executable·가짜 Completion·빈 fixture로 시연하지 않는다. 해당 예는 실제 fixed child/provenance/owner gate가 필요하며 원 forced actual gate와 C2 affected OS gate의 근거로만 인수한다. 공개 API 사용 문서가 추후 필요하면 README의 지원범위·명시 owner 보유 계약을 사용하고, 실제 외부 Rust caller가 정해진 뒤 그 입력/오류 처리로 최소 compile example을 작성한다. Windows cfg나 ignore/no_run doctest를 실지원 검증으로 표시하지 않는다.

producer 인수 구분은 다음과 같다. 동일 package명/시험명이 실제 artifact 출처를 같게 만들지 않는다.

| 후보/producer | 고정 소스/실제 BIN | 인정한 근거 / 미검증 |
| --- | --- | --- |
| 원r3 tools | source8 7336E34F, normal97829C7C/unitFC6900DB | 기본7 및 별도 normal/nested OS2. 후계시험/새 API 증거 아님 |
| timeout tools 후계 | source8 003D22E2, 새unit221F02C5, normal은 원978 control | fmt/strict/type/list20/default10. timeout OS0, 새normal producer0 |
| forced green/red tools | green310535/source2 1080, red0F7A | 작성/leaffmt만. pure/type/BIN/OS/directCLI0; 원 gate 그대로 |
| C2 lib+bin r1 | source18 AEA49D12, BIN없음 | 작성23/leaffmt; missing Instant 확인된 결함 보존 |
| C2 lib+bin r2 | source18 BDF8CDF8, BIN없음 | import1 source 수리/leaffmt. type/list/default23/OS/directCLI0 |

원 forced 실제 red/green을 C2로 대체하거나 timeout unit221F를 C2 libtest producer로 재사용하지 않는다. 다음 실행은 중앙 별도 배정이고, 현재는 제안 인계 후 대기한다. license 미결정은 외부 publish 판단에만 남기며 승인된 private 후계의 로컬 작업 중지 gate가 아니다. 이미 읽은 modoo QA의 실제 caller·검사 범위·미검증 구분 원칙을 적용했다.
