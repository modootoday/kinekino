---
id: evidence:kinekino/20261029114749
title: IPC 선행 baseline 실행기 — C2 격리 검증 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# IPC 선행 baseline 실행기 — C2 격리 검증 2026-10-27

후속 실제 범위는 [잔여7 OS self-test](source-ipc-first-os-20261027.md)의 통과로 pure11+OSselftest8이 됐다. 아래 정상OS1만 완료/잔여 미실행 문구는 이전 시점의 기록이다. 내부 negative·forced·late 상태를 정상 작업 성공으로 계상하지 않는다. baseline source41 전후 동일도851에서 확인했다. B 후보 actual은 별도 echo1/나머지9미실행이다.

IPC OS10을 감독할 실행기는 candidate B의 Supervisor를 소비하면 안 된다. 미검증 C2 후계 raw18(원 AEA49D12C219CD858D393C91A0A5505518AF6A836391F7B1AE1216D026CD041E)을 최신 제품의 **IPC 없는** owned-process src 복사본과 별도 문맥에 결합했다. root·원C2·원B·제품alias·root/Meari lock은 변경하지 않았다. baseline은 canonical 승인이라는 뜻이 아니며 소유된 고정 입력이라는 범위다.

## 입력과 첫 실패

격리 root=`runtime/development/source-ipc-baseline-runner-r1`, target=`runtime/build-cache/native/owned-test-runner-baseline-r1`. C2의 standalone lock0B2562E8D295AEE5602407228C3A5A21DBD5E478299A685CE94780FADE49226B를 복사했다. owned-process 복사본은 workspace 경계만 추가하고 자체 lock을 offline 생성했다(820). B source_pipe/source_clock/IPC 모듈은 이 dependency에 없다. 822 원 입력41개 고정 후823 check는 outcome_record.rs의 누락 Instant import(E0433)로 exit101이었다. 원 leaf는 `runtime/verification/source-ipc-baseline-runner-repair-r1/outcome-record-before-type.rs`에 보존하고 원C2는 그대로 뒀다. 격리본에 import1줄만 추가했다.

824 R2 입력은41개, SOURCE.md SHA256=F266556CC34F4126B5668A157C587F770FF7408877D7090A6C665BEFCE713067, inventory=4C4448D3C1D188BCA5495F10CF646FF2F0472AAA85BDB329D09E19B5A361C2C1. 격리 root의 runtime/verification/input-r2에 보존했다. snapshot의 generic canonical 제목은 root 적용이 아니다.

## 실제 gate와 producer

로그 `runtime/verification/source375-datalab-b0-r1-logs/820~835-baseline-*`:

- 821 원 fmt,825 R2 all-targets check,826 R2 all-targets strict -D warnings,827 library no-run,832 R2 fmt는 exit0.
- 828 실제 등록23.829 기본11pass/12ignored, OS/자극 전체 실행이 아니다. forced-zero/timeout의 논리 판정 모델은 검증했지만 실제 forced 경로 성공으로 계상하지 않는다.
- 830 정상 실행기와831 baseline test child는 profile.test=false의 Cargo JSON producer로 빌드했다. 제품 source/B의 실행기·child로 대체하지 않았다.
- 835 R2 전체41 입력 content/identity 동일 exit0.

고정 copy는 `runtime/build-cache/native/owned-test-runner-baseline-r1/fixed-r2/`에 있으며 이후 candidate B 빌드와 공유하지 않는다.

| 파일 | SHA256 | bytes |
|---|---|---:|
| owned-test-runner.exe |7E7AD25832F177143A3D0D3376EEFC317AE1EF38356E1DC6606B5AECCD5F5F44|924672|
| runner-unit.exe |AE488649F213F605BCE4234C2FCD9A2E47055B8494D44D1EA34B9254F7D73E04|1849344|
| baseline-child.exe |80888B58A3F3F68A59BB237C0BDA801BDDF0E603500884A1ED98F4B0816EE551|293376|
| baseline-index.json |4289009CB457D1E155DEA2357C66686676203175D8B3CEC9C110DB4C717CFFCB|1524|

index는 operator 자료이며 compiled_baseline_proof=false다. 실제 compilation 근거는 고정 source/Cargo JSON이고 index의 자기 선언으로 대체하지 않는다. CLI guard의 실제 path/SHA/bytes/FileID는834최종 관측에 있다. driver/unit/child의 서로 다른 feature/producer 역할을 혼동하지 않는다. 생성 시점은 B candidate 빌드 뒤였지만 의존성은 별도로 고정한 IPC 없는 원 source 복사본이며 B candidate library로 자기 감독하지 않는다. 옛 ‘B 빌드 전에 driver 고정’의 producer 오염 방지 목적은 이 독립 graph/고정 copy로 유지하고 실제 시간 순서를 바꾸어 기록하지 않는다.

## 첫 정상 OS self-test

833 첫 시도는 준비되지 않은 evidence directory를 DirectoryGuard가 거절해 os2/exit2/not launched였다. child 실행0이며 파일 경로/원 실패 로그를 보존했다. source를 바꾸지 않고 새로운 준비된 디렉터리로834를 실행했다.

834는 `tests::normal_selected_case_has_original_family_receipt_and_raw_eof --exact --ignored`만 선택했다. 외부45s/cleanup5s, 내부15s/cleanup3s는 각 launch 전에 한 번 설정되고 재시작하지 않았다. 전역 환경 변경 없이 baseline child3변수와 TMP/TEMP만 실행 단위로 지정했다.

- 원 resource-runner terminal exit0/11316ms. 정상 driver는 passed/selected_one/physical_closed/stdout_eof/stderr_eof=true, first_error=null, ownership=complete, process/verdict sink 각1을 보고했다.
- outer 원 unit PID5628/creation134375525795261944, exit0/familyEmpty/forced=false. 원 stdout은 정확1pass,ignored0,should_panic=false다.
- 정확 inner 경로는 `runtime/build-cache/tmp/baseline-runner-normal-r3/outer-owned-5628-0/evidence`다. parent와 함께 `runtime/verification/baseline-runner-normal-r3/inner-observed/`에 복사 보존했다. inner fixture PID8864/creation134375525906755297, exit0/familyEmpty/forced=false·원 raw EOF, 실제 `tests::fixture_pass`1pass다. outer 자료는 같은 verification root의 outer/에 있다.
- 이 사례에서 baseline-child.exe는 guard-only다. 실제 실행된 프로세스는 driver→unit normal self-test→같은 unit fixture_pass이며 baseline child의 descendant 호출로 계상하지 않는다. inner baseline-fixture.json의 compiled_baseline_proof=false도 유지한다.

## 남은 조건

현재 실제 인수는 pure11+정상OS1뿐이다. nested family·zero selection·sink failure·timeout/forced/nonzero·forced-zero·late verdict2는 별도 exact 사례로 검증해야 한다. 예상 실패를 검사한 unit pass를 inner 작업 성공으로 바꾸지 않는다. 원 Incomplete/receipt 실패가 남으면 같은 owner를 보존하고 다음 사례를 중단한다. 이 검증 전 candidate B OS10은 계속0이며 증거 보존 옵션의 OS 동작도 미검증이다. 다운로드/HTTP/replay·전체CLI/AI/GUI 지원을 부여하지 않는다. WARP·추론·다운로드·수동GUI·push 보류 유지.

기존 읽은 modoo.today fixture(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100)와 change-review(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)의 원 owner/실제 범위·고정 input·첫 실패 보존을 적용했다.
