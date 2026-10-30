---
id: evidence:kinekino/20261029113535
title: baseline owned-process child — 정상 빌드·고정
status: active
created: 2026-10-29
updated: 2026-10-29
---
# baseline owned-process child — 정상 빌드·고정

PM 단독 배정으로 원 S 명시 cwd/기존 T에서 `cargo build --offline --locked -p owned-process --features test-support --bin owned-process-test-child --message-format=json -j1` 한 번만 실행했다. 원 runner EEA9/conservative2/CPU2/BelowNormal/descendant Job, fresh RAM16GiB/GPU3GiB/C12GiB 이상/writer0 조건이다. 01-normal-build receipt terminal0/3245ms다. receipt의 child_pid는 Cargo 프로세스이지 검증용 child 실제 호출이 아니다. 소스·dependency·lock 변경/metadata/update/clean/새 target/추가 Cargo·시험0이다.

canonical root `314d468e7a43b3ef3b70126cf3f0ed9ec11d5d19`, Meari `b532fe0eed06d8526b755b11bc36416070244ef7`, root lock `8F2C1D56C8A2C22D36D9E57A285D727C7EE9C7D2D1160902B66CC6630B042C67`다. S는24d25 계보이며 canonical과 다른13은 승인 UI 전용 경로로 실제24d25 Git blob/신규 부재에 대조했다. owned-process 전체23의 canonical/S/원 snapshot bytes와 workspace Cargo.toml/lock은 같고 추가 dirty graph는 없다. UI13을 S에 동기화하거나 B/evidence15/r3 overlay를 적용하지 않았다.

Cargo JSON은 package owned-process0.1.0, normal profile.test=false, features=[test-support], fresh=false다. 정상 child 고정본은 `runtime/verification/baseline-owned-process-child-r1/bin/owned-process-test-child.exe`, SHA256 `8440F120A37C3AD578A9BD06106C960818C59259129E32581611944523EFBC4B`, 293376B다. 원 producer와 고정 copy의 FileID·SHA/bytes를 각각 보존했다. 기존 `exit` mode는7이며 정상0으로 오인하지 않는다. **이 BIN은 실행하지 않았다.**

compiler-artifact graph19의 rlib/rmeta/DLL 파일30을 별도 copy/SHA/bytes/FileID로 보존했다. 그중 owned-process의 test-support rlib `A8D9C2510A73768CC2291ABA291D9DA8B79ADF287BCE8E1767A6440B5922E30F`/5215896B와 rmeta `8B07639A035CFB79E3F49AA9313C2F1AB01AD2A8DCF0E3088CEB57EADBBE2C12`/305037B도 fresh=false다. registry15의 기존 cached archive checksum은 root lock 항목과 일치한다. graph pin manifest SHA `D5B4F4C6FC98DB801A06A21C770CDCE65F009C20BF24AE820C5DFAF23D4FDACB`다. 이는 실제 Cargo producer/원 sourcegraph 결속 자료이며 전체 unpacked registry source·toolchain의 독립 재현 인증을 주장하지 않는다.

root1377/library493/S5808/index·manifest·lock4/fixed119/protected alias-known6의 bytes/mtime 및 protected6 FileID 차0이다. S 신규0, T 기존 exe581 중 정상 child root/deps alias2만 실제 Cargo producer로 변경됐고 다른579는 동일·새 exe0·미귀속 변경0이다. 고정 child1을 더한 carry120 SHA `E14C7F8E764F566DF555640A9922019304CD9B21074C5D8906CA196629EE315A`와 graph30을 인계한다. known2 현재 bytes 사본을 보존했고 historical producer 미귀속2/all_cache_protected=false는 유지한다. 원 프로세스/session/handle/pending0으로 slot을 반환했다. canonical/stage/commit/push0이다.

사전 보호 자료 선택에서 `63-after-protected6` summary를 path 목록으로 잘못 읽어 null LiteralPath가 발생했다. Cargo 시작 전 그 사실을 기록하고 summary changes=[]와 원 before path 목록을 대조해 fresh protected6를 다시 읽었다. 실제 파일 차이나 후보 코드 수리·빌드 실패는 아니며 `admission-inventory-correction.json`에 원 사실을 남겼다.

원자료는 `runtime/verification/baseline-owned-process-child-r1/`의 source23·root314d-vs-S24d-UI13·Cargo/lock·normal-build JSON/receipt·fixed-child·compiler-artifact-graph·registry-graph-admission·rlib-graph-pins·보호·T-producer-attribution·carry120·slot-return이다. `execution-baseline.json` SHA `CF4DC065BC7F9A2785E3604B4C3D86FE0A29CC47CB7EBD18412FEF1EFE49AB6E`는 이 자료의 operator index이며 그 파일 자체가 컴파일 출처 권한을 발급하지 않는다. r3 driver default/standalone lock0B graph와 이번 child test-support/root lock8F graph도 구별했다.

후속 첫 normal OS1은 같은 r3 unit의 fixture_pass를 실행하며 baseline child는 guard만 한다. 빌드와 그 OS1을 baseline child 실제 호출로 합산하지 않는다. baseline descendant 실제 호출·Job/receipt identity·outer/inner raw EOF·r3 지연2·B/evidence custody15·추론/모델/다운로드/worker/GUI는 별도 배정 전0이다. modoo.today test-fixture-and-mock-build 본문 SHA `a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100`(추가 필수 참조 없음)의 fixture/실제 호출·원 source custody/고정물 구분 원칙을 적용했다. 전체 Media 목표는 미완료다.
