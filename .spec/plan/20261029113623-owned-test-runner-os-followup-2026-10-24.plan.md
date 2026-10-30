---
id: plan:kinekino/20261029113623
title: baseline driver OS5·late verdict 후속 제안 — 실행 권한 아님
status: active
created: 2026-10-29
updated: 2026-10-29
---
# baseline driver OS5·late verdict 후속 제안 — 실행 권한 아님

## 2026-10-24 후속 읽기 검토 — 입장 누락과 순서

중앙 최신 순서는 UI 마지막 감사 반환 뒤 SDK chooser gate 우선이다. 아래는 실행·수정 권한이 없는 준비 검토다. r3 effective8 `7336E34F1155BF6DD4441DBC4C6E8077527063C95CEE09773C9FFCD3422A876B`와 B r4 `6B16E911C0DAAE6E6829966EE061D1F260E126D67D4E95B25168B8BF37B17CDF`의 pin 불변을 다시 읽어 확인했다. r3은 leaf fmt2/작성4·컴파일/list/actual0, r2만 실제 타입·기본5가 있다. r3을 사용할 gate라면 별도 타입·목록·기본7 및 새 normal/unit Cargo JSON 고정을 먼저 해야 한다. 아래 역사 r2 BIN을 r3 검증 BIN으로 이름만 바꾸어 쓰지 않는다.

실행 전 중앙이 확정해야 할 누락은 다음과 같다.

1. **baseline child 생산과 B child를 분리한다.** B 적용 전 최신 승인 baseline source23/manifest/원 root lock을 고정한 후 `cargo build --offline --locked -p owned-process --features test-support --bin owned-process-test-child --message-format=json -j1`의 정상 profile.test=false producer를 별도 고정 copy로 만든다(아직 미실행). 실제 Cargo features는 JSON으로 확인하며 driver default graph의 빈 feature와 혼동하지 않는다. 경로/SHA/bytes/FileID·Cargo JSON·sourcegraph/lock과 필요한 원 rlib graph 사본을 함께 보존한다. B 빌드가 같은 T alias/rlib를 갱신할 수 있으므로 baseline 고정 child/BIN을 공유 alias로 대체하거나 B 이후 재빌드하지 않는다. 원 child의 `exit` mode는 exit7이고 정상0 자극이 아니다. 정상은 기존 eof/fixture_pass, descendant는 실제 별도 process 자극이며 서로 구별한다.
2. **guard pin과 상속 환경의 결속을 별도 기록한다.** driver `--child`는 파일을 guard하지만 `SpawnSpec.environment=None`이므로 OS selftest의 `OWNED_TEST_BASELINE_CHILD/SHA256/BYTES`는 기존 환경을 상속한다. 같은 세 값이 외부 --child pin과 정확히 같은 baseline 고정 copy를 가리키도록 실행 단위 환경·입력 자료를 보존해야 한다. 이것만으로 실제 child 호출이 입증되지는 않는다. nested fixture의 원 child receipt/identity와 raw를 대조한다. B fixture는 이 환경 대신 compile-time CARGO_BIN_EXE alias를 쓰므로 B child producer/alias pin을 별도로 보유한다. baseline env를 B child 출처로 해석하지 않는다.
3. **중첩 cutoff 예산의 여유를 별도 입장한다.** 새 지연2는 내부 Deadline15s/cleanup3s의 원 end까지 writer 반환을 지연한다. 외부 driver도15s이면 외부의 준비·spawn 시간이 먼저 소비돼 내부 단언/EOF 전에 operation cutoff가 올 수 있다. 외부는 launch 전 단일 예산을 별도 중앙 배정하고 내부15s와 준비/관측/정리 여유를 포함해야 한다. 예를 들어 외부45s/cleanup5s는 기존 입력 상한60s 안의 검토 후보일 뿐 승인·성능 보장이 아니다. 기존 내부 예산 확대/수신 시 budget 재시작은 없다. nested child3s/1s, timeout1s/300ms, B5s/1s, sentinel 자체 원 cutoff도 별도로 표기한다. 파일 sync/OS 동기 호출은 여전히 선점 불가이며 전체 hard deadline을 주장하지 않는다.
4. **inner evidence의 주소와 보관을 확정한다.** driver selftest는 temp의 `outer-owned-{unitPID}-{counter}`와 nested의 `outer-nested-{stimulusPID}`에 저장하고 해당 경로를 최상위 stdout에 직접 내보내지 않는다. 실행별 독립 temp root 상속을 사용할 경우 실제 생성 경로와 원 receipt PID로 결속하고 그 정확 경로만 보관한다. 환경 변경만으로 temp 해석을 입증하거나 실패 시 전역 temp scan으로 대신하지 않는다. 현재 selftest의 보관과 달리 B 정상 fixture·기존 windows_family는 성공 뒤 remove_dir_all을 호출하므로 outer raw만으로 inner raw를 보존했다고 할 수 없다. inner 전체 보관이 필수면 기존 시험 leaf의 keep/evidence-root나 명시 원자료 인계가 별도 승인돼야 하며 지금은 변경0이다.
5. **bootstrap·관측·회수를 성공 수와 분리한다.** 정상고정/목록 확인→정상 OS1→빈 선택 negative→timeout negative→nested→sink failure 순서로 exact 사례마다 새 원 wrapper evidence를 사용하되 전체 --ignored 실행은 하지 않는다. 새 지연2는 순수 회귀와 기존 OS5 뒤 별도 배정한다. 정상 driver 자체는 처음에는 OS 검증 전이므로 원 runner의 terminal 사실과 wrapper의 아직 미입증 판정을 분리해 bootstrap 한계를 보존한다. 원 owner/receipt 실패·미종료·출처 불일치에서 다음 사례를 멈추며 새 cleanup owner/전역 kill/새 budget/재시도 성공으로 바꾸지 않는다. 같은 carrier·guard를 보존하고 중앙 주의 상태로 인계한다. parent-exit 회귀의 sentinel은 피시험 inner Job 밖이지만 동일 outer containment 안이므로 사용자 앱/outer 전체 밖 sentinel의 안전 증명으로 확대하지 않는다.

반환은 원 terminal/receipt와 child·family·forced·raw EOF를 분리해 확인하고, baseline/B별 고정물·alias/원 FileID·source/lock·S 비입력·known2/역사 미귀속을 각각 대조한 뒤 SOT·명시 반환으로 한다. 실제 OS gate 전 독립읽기와 중앙 슬롯·위 조건의 관측 수단을 확정해야 한다. 이번 검토는 코드/Cargo/S/T/OS/모델/네트워크 변경·실행0이다. modoo fixture skill의 고정 자극·원 자원 인수/해제·fixture/실제 근거 구분 원칙을 적용했다.

## 최초 제안 이력

### 성공 inner evidence 삭제 gap의 최소 exact leaf 제안

2026-10-24 PM 후속 요청의 **설계 제안만**이며 코드/Cargo/S/T/OS 실행 권한이 아니다. 수정 후보는 `packages/owned-process/tests/source_ipc.rs`(B r4를 원본으로 하는 별도 revision), `packages/owned-process/tests/windows_family.rs`(실행 시 승인 baseline preimage)의 **기존 exact2**다. production crate/launch/owner/IPC/RawFile API, Cargo/dependency, frozen B r4/r3 driver에는 손대지 않는다. 공통 신규 helper 경로가 필요해지면 exact2 안에 거대 모듈을 만들지 않고 다시 경로 승인을 요청한다.

**권장 최소 동작은 복사·이동 없이 원 root를 보존하는 test-only opt-in**이다. 예를 들어 `OWNED_TEST_KEEP_EVIDENCE=1` 한 값을 준비 전에 엄격히 읽고, 미설정은 기존 동작·다른 값은 launch 전 오류로 한다. 환경은 실행 admission에 이 한 key만 기록한다. 각 prepared가 만든 기존 임시 root의 위치·정체성은 바꾸지 않는다. 원 child/family/IPC reaping·named-pipe 재획득·EOF와 원 성공 단언은 그대로 수행한 뒤, 성공 마감의 remove_dir_all만 keep 모드에서 생략한다. OS 자원 정리 완료와 증거 디렉터리 삭제는 다른 사실로 기록한다. 기본 모드의 삭제·완료 동작은 유지한다. Windows의 invalid_root/sentinel_root 등 한 사례가 만든 모든 root도 빠짐없이 같은 선택을 적용한다. 전역 TEMP/TMP 재작성·다른 temp root 탐색은 필요하지 않다.

원 root custody는 준비 직후(원 process 생성 전) 일반 directory 여부·reparse 부재를 확인하고 원 directory handle/volume+file ID를 보유해 마감 때 같은 root임을 대조하는 범위다. 새 launcher/cleanup owner를 만들지 않는다. 이미 생성된 `PreparedEvidence.paths`의 원 RawFile clone을 보존하고, child/family 실제 종료 후 `RawFile::open_read`의 원 file identity 검사로만 raw를 읽는다. path만 재열거나 일반 recursive directory copy로 대체하지 않는다. 실패/미완료 carrier가 있으면 root 보유물도 그 원 carrier와 함께 보존하고 assertion 전에 Drop하지 않도록 시험 소유 구조를 검토한다. 이 작업이 기존 intentional Drop/exit-probe 회귀의 의미를 바꾸게 된다면 그 사례는 별도 설계로 돌리고 자동 확대하지 않는다.

bounded 보관 후보는 한 fixture root의 고정 `stdout.raw`/`stderr.raw`(B 각각64KiB, windows_family 각각128KiB), 존재하는 원 `receipt.txt`(64KiB), 새 test-only 인계 manifest(16KiB 이내)뿐이다. 정확 파일 수를 한정하고 symlink/reparse/하위 디렉터리·stdin/개인 미디어·환경 전체·임의 argv는 포함하지 않는다. 원 owner의 현재 raw-output bound를 낮추거나 초과 raw를 잘라 성공으로 만들지 않는다. bounded read는 원 file에서 최대+1과 EOF/크기를 확인하고 초과/증가/identity 변경이면 원 파일을 삭제·수정하지 않은 채 evidence 보관 실패로 마감한다. 최대 크기 판정은 보존된 bytes의 완전성을 위한 것이며 child 출력량의 물리 hard cap이라고 주장하지 않는다.

manifest는 고정 schema와 실행 case/target·unit PID·fixture counter·원 root의 handle identity·알려진 상대 파일명/크기/EOF/존재·원 receipt 상태·보존 선택만 담는다. root 주소는 private 검증 로그에 필요한 범위로 한 번만 전달하고 경로 길이/전체 record 상한을 넘으면 실패한다. 개인 사용자 경로·식별자는 공개 commit/제품 결과에 넣지 않는다. source8/source13·lock·Cargo JSON·test/child BIN pin의 인증 주체는 외부 원 gate bundle이며 fixture manifest가 자체 출처를 발급하지 않는다. inner bytes SHA가 필요하면 기존 의존성/허용 API 안에서 가능한지 별도 검토한다. 새 hash crate/도구를 이 두 leaf 권한으로 추가하지 않는다. 원 file identity·EOF와 외부 실제 hash 관측을 구별한다.

`source_ipc`는 원 `receipt.txt`를 그대로 두고 `run` 및 partial-assignment 성공 마감의 삭제만 동일 helper를 거치게 한다. diagnostic create/write/flush/panic negative는 이미 보존하므로 새 성공 receipt·재시도로 덮지 않는다. `windows_family`의 `Sink(Vec<ProcessReceipt>)`는 **원래 durable receipt 파일을 만들지 않는다**. 정상 `finish`에서 받은 원 ProcessReceipt를 인계할 경우 '원 메모리 receipt의 관측 사본/기존 durable sink 증거 아님'으로 명시하고 bounded 별도 manifest에 넣는다. query/launch 실패의 receipt 부재는 부재로 남긴다. 기존 drop-fallback 회귀를 새 강한 cleanup 증거로 승격하지 않는다.

manifest create_new/write/flush/sync/출력 실패는 모두 fallible이다. root/owner가 살아 있는 상태에서 unwrap으로 unwind하지 않고 원 first error/action panic/Incomplete를 우선 보존하며 보관 오류를 별도 secondary로 남긴다. 정상 기능 단언이 통과했어도 evidence 인계 실패면 전체 gate 인수 실패다. 기존 오류/진단을 새 성공 sink로 바꾸거나 같은 destination을 재열어 성공할 때까지 반복하지 않는다. 이미 있는 파일 overwrite·rename·삭제·실패 뒤 자동 재시도0이다. 실패가 난 root는 정확 주소/원 상태를 보존해 PM 주의 대상으로 반환한다. stdout에 경로를 찍은 것만으로 내구 보관 성공이라고 표시하지 않는다.

test-only 검증 제안은 기본 모드의 원 삭제 보존, keep 모드의 원 root/file identity·동일 raw bytes/EOF·자원 종료 후 보존, sentinel/invalid 등 복수 root 누락0, 잘못된 opt-in 값의 launch 전 거절, manifest 실패/상한 초과 시 원 오류·원 raw 보존/재시도0을 분리하는 것이다. 새 시험 수/코드는 아직 정하지 않았고 실제0이다. 실제 gate는 outer runner/driver 결과와 내부 인계 manifest·원 raw/receipt를 함께 회수한다. 임시 root 보존은 독립 장기 archive copy의 완료가 아니다. 필요 시 반환 전에 중앙이 정확 root 목록만 기존 비공개 검증 bundle로 수거·대조한 뒤 별도 명시 삭제하며, 그 회수/보존 기간·한 실행의 전체 case/root 수와 총 bytes 상한도 입장 계약에서 먼저 정한다. 별도 수거 로직이 필요하면 Rust로 작성하고 새 경로/자원 권한을 요청한다.

이 최소안은 modoo fixture skill의 고정 자극·원 자원 해제와 evidence teardown 분리·mock/actual 및 실패 원문 보존 원칙을 적용한다. bounded 메모리·파일 보관과 private 경계를 정했을 뿐 원 source custody·DurableWrite/OS 종료·새 API 지원을 실제 입증하지 않았다. SDK sole-heavy와 외부45s/cleanup5s 후보의 미배정 상태는 유지한다.

타입·순수5 마감 후 중앙에 제출하는 source-only 제안이다. 다음 heavy는 UI 우선이며 추가 Cargo/OS/회귀 코드 작성은 아직 승인되지 않았다. r2 source8 5FCE/정합 lock0B/기존 baseline source23/Cargo JSON/고정 BIN은 원 검증 자료 그대로 보존한다.

## OS5의 최소 입장

- 정상 driver 고정 `runtime/verification/owned-test-runner-r2/bin/owned-test-runner.exe` SHA58690FF6/914944B와 unit 고정 `owned-test-runner-tests.exe` SHA0057C9E9/1573888B를 사용한다. 정상 BIN과 libtest BIN을 바꾸어 쓰지 않는다.
- 별도 배정 후 **B overlay 전에** baseline owned-process의 test-support normal child를 기존 S/T에서 offline/locked Cargo JSON(test=false)으로 빌드·고정한다. child sourcegraph/lock/producer path/SHA/bytes/FileID와 env OWNED_TEST_BASELINE_CHILD/SHA256/BYTES를 외부 입력에 결속한다. 현재 child build/고정0이며 기존 미확인 alias로 대신하지 않는다.
- 정상 driver가 고정 unit의 OS exact1만 선택하도록 하고 각 child 자극은 기존 application Supervisor의 원 owner 아래에서만 선택한다. 전체 --ignored 금지다. OS5는 정상 결과/timeout 강제종료 후 실패 유지/빈 선택0exit 거절/nested Job/첫 receipt 실패 carrier 보존이다. stimulus3은 별도 인수 성공 수로 더하지 않는다.
- 원 입력 snapshot은 fixture compiled_baseline_proof=false임을 유지한다. 외부 sourcegraph와 컴파일 근거가 권한을 소유하며 내부 fixture가 스스로 출처를 인증하지 않는다. alias/env pin과 실제 nested child 호출의 process 근거도 별도로 대조한다.
- 원 case deadline15s/cleanup3s, timeout 자극1s/300ms, nested 자극3s/1s를 원 코드대로 구분한다. driver 전체의 고정15s 종료나 성능 보장은 아니다. 같은 baseline owner/원 Deadline을 사용하고 receipt·원 first error/Incomplete·child exit/family Empty/forced·raw stdout/stderr EOF를 각각 보존한다. 임의 PID 재오픈/이름별 kill/새 cleanup owner/새 budget/성공 receipt 대체0이다.
- 기존 resource-runner는 timeout/kill-on-close를 제공하지 않는다. 미종료 원 carrier는 driver foreground에 남을 수 있으므로 이 제약을 포함한 중앙 관측·주의 절차가 확정돼야 OS gate를 배정할 수 있다. 종료 요청과 실제 terminal을 혼동하지 않는다. 원 임시 evidence 경로·snapshot/outer-receipt/outcome/raw와 최상위 stdout/stderr/receipt를 각 사례별 원문 그대로 보존한다.

## late verdict 최소 회귀 제안

후속 수정 후보는 **기존 exact2** `tools/owned-test-runner/src/application.rs`, `tools/owned-test-runner/src/tests.rs`다. 새 경로·main/launcher/제품 정책 변경은 제안하지 않는다. 승인 전 source8 수정0이다.

1. 순수 회귀: 원 now/cutoff와 기존 first error를 입력받는 작은 판정 seam을 분리할 수 있을지 검토한다. cutoff 전/동일/후와 기존 첫 오류 유지·late secondary/overflow를 결정적으로 검사하며 process/실제 sleep이 필요 없다. 이는 실제 durable sync 지연의 입증은 아니다.
2. 명시 OS 회귀: 별도 test-only verdict writer 주입으로 실제 write/flush/sync 중 원 cutoff를 넘기는 고정 지연을 만들어 저장 전 durable 관측과 반환 Outcome의 late error/passed=false를 대조한다. 원 process physical close/EOF와 첫 receipt 오류 유지도 구별한다. worker나 budget을 새로 만들지 않는다. sync의 hard-preemption/고정 latency 보장으로 확대하지 않는다.
3. 원 owner 후속 관측은 기존 OS5만으로 entry/main 분기 전체를 입증하지 못한다. 원 Incomplete/LaunchFailure carrier를 유지하는 exact 사례와 같은 handle 관측·오류 상태·물리 종료/미종료를 독립 검토한 뒤 추가 경로/사례 수를 중앙에 다시 제안한다. 기존5 통과를 이 동작의 증거로 합치지 않는다.

기능 판단은 Rust SDK/CLI로 수행하며 GUI 우회0이다. modoo fixture skill의 고정 입력·실제 자원 회수·mock/실측 구별 원칙을 적용한다. root index/commit은 PM 소유이며 push0이다.
