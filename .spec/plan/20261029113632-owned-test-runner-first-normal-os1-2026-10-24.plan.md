---
id: plan:kinekino/20261029113632
title: "r3 첫 normal OS1 — 입력안, 실행 미배정"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# r3 첫 normal OS1 — 입력안, 실행 미배정

baseline child 빌드 반환 뒤 중앙 요청으로 준비한 **계획만**이다. 추가 Cargo/child/OS/지연 실행0. 원 r3 source7336/기본7과 baseline child 반환4B7F를 보존하고 별도 slot/fresh 자원·고정120/graph30·alias/currentknown2 입장 승인을 받아야 한다.

## 고정물과 정확 입력

모든 경로의 공통 기반은 `D:\workspace\@myso-kr\com.kinekino.player\runtime\verification\`다.

| 역할·하위 경로 | SHA256 | bytes |
|---|---|---:|
| 정상 실행기 `owned-test-runner-r3\bin\owned-test-runner.exe` |97829C7C4F055E7E153BA464387654529FE7ACB3FB2601910F5395603EDDEE48|915456|
| unit `owned-test-runner-r3\bin\owned-test-runner-tests.exe` |FC6900DB28DB7CD2CCF195A19DABD62DB21E1CA194200EBE469C920F1F2F485E|1740288|
| baseline child `baseline-owned-process-child-r1\bin\owned-process-test-child.exe` |8440F120A37C3AD578A9BD06106C960818C59259129E32581611944523EFBC4B|293376|
| 외부 증거 index `baseline-owned-process-child-r1\execution-baseline.json` |CF4DC065BC7F9A2785E3604B4C3D86FE0A29CC47CB7EBD18412FEF1EFE49AB6E|3365|

원 resource-runner EEA9의 conservative2가 위 정상 실행기를 실행하는 입력안이다. 실행 인수는 `--test <unit 절대경로> FC6900DB28DB7CD2CCF195A19DABD62DB21E1CA194200EBE469C920F1F2F485E 1740288`, `--child <baseline child 절대경로> 8440F120A37C3AD578A9BD06106C960818C59259129E32581611944523EFBC4B 293376`, `--baseline <외부 index 절대경로> CF4DC065BC7F9A2785E3604B4C3D86FE0A29CC47CB7EBD18412FEF1EFE49AB6E 3365`, `--evidence D:\workspace\@myso-kr\com.kinekino.player\runtime\verification\owned-test-runner-first-normal-os1-r1\outer`, `--case tests::normal_selected_case_has_original_family_receipt_and_raw_eof --ignored true --total-ms 45000 --cleanup-ms 5000`다. 현재 outer/output/temp 폴더는 만들지 않았다. 같은 작업 root 아래 runner stdout/stderr/terminal receipt를 각각 별도 create_new 자료로 지정한다.

외부45s/cleanup5s(operation40s)는 **아직 승인 전 후보**이며 내부 request15s/cleanup3s는 그대로다. 둘은 각 launch 전 확정되는 별도 원 cutoff이고 phase별 재시작·원 cutoff 연장은 없다. 동기 I/O·미종료 carrier 때문에 whole-process45s 종료를 보장하지 않는다. 이번 첫 사례는 내부 지연 writer를 주입하지 않는다.

## 환경·주소·원 owner

`OWNED_TEST_BASELINE_CHILD`, `_SHA256`, `_BYTES` 세 값은 위 --child와 정확히 같은 절대경로/전체SHA/문자열293376으로 실행 단위에만 공급하고 원 입력에 기록한다. 실제 key 전체는 `OWNED_TEST_BASELINE_CHILD_SHA256`, `OWNED_TEST_BASELINE_CHILD_BYTES`다. SpawnSpec.environment=None의 상속이며 root 전체 환경/비밀값은 수집하지 않는다. 필요 시 같은 단일 실행 환경의 TEMP/TMP만 작업 root의 사전 생성·검증된 독립 `inner-temp`로 제한한다(사용자/시스템 전역 환경 변경0). 환경 선언만으로 실제 temp 해석을 입증하지 않는다.

원 outer receipt의 unit PID와 fresh exact1 프로세스의 NEXT=0으로 `inner-temp\outer-owned-{unitPID}-0\evidence` 및 그 parent snapshot을 결속한다. 실제 그 경로가 존재하고 원 고정 case/inner receipt·raw에 대응하는지 terminal 뒤 정확 경로만 관측한다. 없으면 미관측으로 남기며 다른 temp/user 디렉터리를 scan하지 않는다. 이 r3에는 evidence15 helper가 없으므로 마감 때 관측한 root FileID가 생성 시 원 root ID의 별도 증명이라고 주장하지 않는다. 원 snapshot fixture의 compiled_baseline_proof=false도 유지한다.

첫 OS1은 같은 고정 unit을 다시 `tests::fixture_pass --exact --ignored --test-threads=1`로 원 inner Job 아래 실행한다. **baseline child BIN은 guard만 하며 실제 호출하지 않는다.** top driver→unit OS1→같은 unit fixture_pass의 관계와 guarded baseline child를 분리한다. baseline child descendant 실제 호출/원 receipt identity·Job 증거는 후속 nested 사례의 별도 범위다. 빌드/첫 OS1 성공을 그 실제 호출로 더하지 않는다.

인수 관측은 runner의 원 terminal과 정상 driver exit0/최종 passed=true, outer 원 receipt child exit0/family Empty/forced=false·원 raw EOF·unit 정확1pass, inner receipt의 fixture PID/creation/Empty/원 raw EOF 및 저장된 verdict를 각각 대조한다. durable verdict는 저장 전 관측이므로 최종 Outcome과 혼동하지 않는다. 실패/partial/receipt 오류·stdout 오류·빈 선택이면 성공으로 바꾸지 않는다. 같은 원 driver/tool session과 원 handle을 terminal까지 관측하며 조회 실패를 종료로 간주하지 않는다. r3가 원 Incomplete/LaunchFailure를 보유한 채 남으면 후속50ms 관측은 기존 owner/원 cutoff 그대로이고 새 budget·새 launcher/PID 재오픈·이름별 kill·다음 사례/자동 재시도0으로 PM 주의 상태에 인계한다. resource-runner 자체에는 timeout/kill-on-close가 없다.

실행이 배정되더라도 이 exact1만 수행하고 원 raw/receipt/index·정확 inner root를 보존한 뒤 보호 감사·SOT·명시 반환한다. B/evidence15/r3 재빌드/다른 OS4/지연2/자극 전체 실행/AI0이다. 이 계획은 실제 관측·지원 완료가 아니다.
