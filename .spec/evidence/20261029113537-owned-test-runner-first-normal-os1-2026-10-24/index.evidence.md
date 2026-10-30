---
id: evidence:kinekino/20261029113537
title: r3 첫 normal OS1 — 실제1 인수
status: active
created: 2026-10-29
updated: 2026-10-29
---
# r3 첫 normal OS1 — 실제1 인수

PM 최신 배정 root `c351b5d6e87120d6455944e10ff74bc134bd3171`, Meari b532fe0eed06d8526b755b11bc36416070244ef7, lock8F2C에서 계획074385의 exact1만 실행했다. Cargo/overlay/소스 변경/다른 OS/지연/B/evidence15/AI0이다. 원 runner EEA9/conservative2/CPU2/BelowNormal/descendant Job 및 fresh RAM16GiB/GPU3GiB/C12GiB/writer0, S5856/carry122(30CB4B86)/protected6·FileID/currentknown2 사전 pin을 확인했다.

고정 입력은 정상 driver `97829C7C4F055E7E153BA464387654529FE7ACB3FB2601910F5395603EDDEE48`/915456B, unit `FC6900DB28DB7CD2CCF195A19DABD62DB21E1CA194200EBE469C920F1F2F485E`/1740288B, guard-only child `8440F120A37C3AD578A9BD06106C960818C59259129E32581611944523EFBC4B`/293376B, 외부 index `CF4DC065BC7F9A2785E3604B4C3D86FE0A29CC47CB7EBD18412FEF1EFE49AB6E`/3365B다. r3 effective8 7336와 baseline source23·고정 graph30을 보존했다. 외부45s/cleanup5s와 내부15s/cleanup3s는 각 원 launch 전 확정했고 재시작/확대하지 않았다. 실행 단위 TEMP/TMP만 검증된 이 작업의 inner-temp에 설정하고 child env3은 --child와 정확히 같은 path/SHA/bytes로 기록했다. 전역 환경 변경0이다.

## 실제 사실

`tests::normal_selected_case_has_original_family_receipt_and_raw_eof`를 --exact/--ignored/thread1로 실행한 원 unit stdout은1pass/0fail/0ignored다. 원 runner receipt는 정상 driver PID35208의 terminal exit0/12402ms를 기록했다. 최종 normal stdout은 passed=true/ownership=complete/physical_closed=true/first_error=null/secondary=[]와 stdout/stderr EOF=true, process/verdict sink records1이다. 기록된 시간은 이번 실행 관측이며 성능 상한을 뜻하지 않는다.

| 원 Rust owner의 process | PID / creation_filetime | 원 receipt |
|---|---|---|
| OS1 unit |76664 / 134372611393315982|exit0, family Empty, forced=false|
| fixture_pass를 실행한 같은 고정 unit |57668 / 134372611505712815|exit0, family Empty, forced=false|

두 receipt의 stdin_eof_requested=true는 요청 사실이고 child가 stdin을 읽었다는 ACK가 아니다. inner stdout의 fixture_pass도1pass/0fail/0ignored지만 이는 OS1의 입력 자극이므로 독립 인수2개로 합산하지 않는다. 양쪽 Rust SDK 원 raw_bytes 관측은 stdout/stderr EOF=true였고 기존 정상 OS1 단언도 통과했다.

원 outer receipt unit PID76664와 fresh NEXT0에 대응하는 정확 `runtime/verification/owned-test-runner-first-normal-os1-r1/inner-temp/outer-owned-76664-0/evidence`를 실제 확인했다. 그곳의 fixture receipt PID57668·same fixed unit path·raw/outcome와 부모 baseline-fixture.json(48B, compiled_baseline_proof=false)을 대조했다. 환경 선언만으로 주소 성공을 가정하거나 다른 temp를 scan하지 않았다. root FileID는 terminal 뒤 관측했으며 생성 시 독립 ID 증명으로 표시하지 않는다.

outer/inner 각 stdout.raw·stderr.raw·원 outer-receipt.jsonl·outcome.jsonl과 fixture snapshot 총9개 원문/SHA/bytes/mtime/FileID를 그대로 보존했다. durable verdict에는 저장 전 verdict_sink_records0/원 remaining이 기록되고 최종 normal stdout에는 저장 완료 records1이 있다. 저장 전 관측과 최종 Outcome을 구분하며 이번 정상 경로의 시간 여유가 실제 late verdict 실패를 입증하지 않는다. 외부 index도 공급 bytes 검증이며 스스로 컴파일 출처 권한을 발급하지 않는다.

## 범위·보호·반환

이번 실제 계층은 정상 driver→OS1 unit→같은 unit fixture_pass다. **baseline owned-process-test-child BIN은 양쪽에서 guard만 하고 실제 호출0**이다. descendant/nested 자극의 별도 실제 호출·원 identity/Job 검증은 미완료다. timeout/빈 선택/receipt failure 등 나머지 OS4·지연2·자극 전체 실행·B IPC/evidence custody·제품 GUI/미디어·모델 지원을 이 정상1로 승격하지 않는다. 원 미종료 owner 보존·같은 handle 후속 관측·hard deadline 미보장 경계는 유지하며 이번에는 해당 실패 경로가 발생하지 않았다.

root1377/library493/index·manifest·lock4/S5856/T 기존 exe581/fixed122/protected alias-known6/input4/graph30의 검사 범위 bytes/mtime 및 protected/input/graph FileID 차0, S 신규0/T 신규0이다. 새 고정 BIN/새 producer0이다. known2 원 bytes 사본도 유지했고 과거 cache4709 producer 미귀속2/all_cache_protected=false는 별개로 남겼다. 원 session30174 terminal0 및 최종 관련 process/session/handle/pending0으로 명시 반환했다. kill/새 owner/새 budget/재시도/추가 실행0, canonical/stage/commit/push0이다.

원자료는 `runtime/verification/owned-test-runner-first-normal-os1-r1/`의 admission·input4-pins·execution-input·01-normal-os1 stdout/stderr/receipt·outer·정확 inner-temp root·observed-evidence9·actual-observation·protection-summary·graph30-protection·final-process-observation·slot-return이다. modoo.today test-fixture-and-mock-build 본문 SHA `a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100`(추가 필수 참조 없음)의 원 fixture/실제 호출·원 자원 해제/증거 보존·실측/미검증 분리 원칙을 적용했다. 전체 Media 목표는 미완료다.
