---
id: plan:kinekino/20261029113714
title: 구간 자막 Job · queue/fresh CLI 인수 계획
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 구간 자막 Job · queue/fresh CLI 인수 계획

상태: 검증 완료·PM 채택 대기, 2026-10-21. 중앙 PM의 단일 shared/heavy 슬롯과 exact8 독립감사·fresh CLI2 실행 승인을 인수했다. root d0d185faf09b56120ce5cf960a95fdcfa04f6029 기준 snapshot 보호1336 일치 후 exact8만 합류했다. 원 r2 source8 934A375105DB2DABB8AA97E5345C6E99A8FA6ABE1D4BBFFA8C167F4A213CC6BC를 보존하며 unused import만 제거한 r3 source8은08B82DAF7A0872C0538F5D43B110227DC86DF750DF2E076FD94EE6E930DF356B다. 신규14와 기존17은 실제 통과31, 기존 ignored1이다. 양 strict/fmt0·최종보호 차이0을 확인하고 슬롯을 반환했다. 원 로그·receipt는 runtime/verification/subtitle-range-job-r2에 보존한다. goal 저장 도구는 과거 blocked 상태로 생성 거절되어 재시도하지 않으며, 이 계획과 PM 명시 배정을 실행 목표로 사용했다.

## Outcome — 결과

실제 --serve-jobs owner와 서로 다른 fresh CLI 요청으로 range 작업의 admission→terminal→새 파일/원 digest/receipt를 확인한다. caps/schema/ready는 실제 실행의 전제이며 독립 성공 근거가 아니다. 새 모델·브라우저·미디어·GUI와 renderer/range 정책 구현은 없다.

## Authority — 범위

PM이 최신 root d0d185f+harness 인수 base/Meari·snapshot·heavy 슬롯을 배정했다. exact8은 원6과 승인된 tests/jobs.rs 등록+신규 tests/jobs/range_export.rs이며 기존 Fixture/Host/Request helper를 재사용한다. 실제 CLI 취소를 확정하려면 별도 기존 cancel harness의 range 적용이 필요하며 production timing 경합으로 성공을 만들지 않는다. 신규 추론·다운로드·AI worker 빌드·GUI 확장과 root index 변경은 이 배정에 포함하지 않는다.

## Evidence — 로컬 fixture와 고정 기대값

`tests/subtitle_range_cli.rs::SOURCE`의 기존 UTF8/LF SRT를 private planned-fixtures/source.srt로 그대로 보존했다. 이는 새 미디어나 다운로드가 아니다. 예상 VTT는 기존 domain renderer를 읽어 기록한 **기대 자산**이며 실행 결과가 아니다.

| 자산 | bytes | canonical SHA256 |
| --- | --- | --- |
| source.srt |94|2840bb1f0ba39a911fe040b9bb79861486628f0f2bb2e7a5bbd57432782017aa|
| expected-range.vtt |98|f85e351d515582490a97eb64a3febd6479851fd1993c5bee268262718125288c|

경로는 `runtime/coordination/subtitle-range-job-r1/planned-fixtures/`, metadata는 pins.json이다. 범위 [2000000,5000000)μs는 기존 cue [1s,3s)/[4s,6s)를 clip/rebase하여 [0s,1s)/[2s,3s)로 만든다. 내용 첫째 原文/둘째 한글, BOM0/LF·마지막 빈 줄, cue2/손실0을 기대한다. hashes는 파일 메타데이터 산출만 했고 feature/Job 실행0이다.

기존 `tests/jobs/fixture.rs`는 실제 CLI 복사 SHA/expected/source 동등성, 요청 JSON·stdout/stderr/exit 보존, Host stdin EOF·원 Child wait를 제공한다. JOB_EVIDENCE_DIR는 새 절대 경로로 설정하고 JOB_CLI_EXE/소문자 JOB_CLI_SHA256는 한 canonical artifact JSON에서 소비한다. 기존 대소문자 입력 실패 이력을 반복하지 않는다.

## Unverified — 차단·보장 한계

최종 r3 신규14와 기존 영향 회귀17은 실제 통과31이며 기존 corpus ignored1은 유지한다. 양 strict/fmt도0이다. 상세 원 근거는 runtime/coordination/subtitle-range-job-r3/handoff.md와 runtime/verification/subtitle-range-job-r2다. heavy/shared 슬롯은 PM에게 명시 반환했다. 생산 --serve-jobs에서 즉시 cancel을 보내는 것은 worker가 먼저 publication에 도달할 수 있어 결정적 취소 인수가 아니다. 실제 coordinator CAS 시험과 CLI wire 취소를 구별한다. SourceExpired-after-admission/deadline/readback fault도 작은 production 입력에서 임의 타이밍을 가정하지 않고, 제어 포트 seam의 근거와 실제 CLI 근거를 따로 남긴다.

Unsupported/InvalidArgument 같은 submit 실패는 CLI nonzero이며 job이 없다. 정상 submit 뒤 Job NeedsInput/Failed는 jobs.get의 API exit0일 수 있다. query 성공을 작업 성공으로 해석하지 않는다. 실제 crop/loss/render는 media-subtitles 원 usecase를 사용한다.

## Phases — 명령과 인수 조건

### 1. 입력·작성 회귀

중앙 snapshot을 보호하고 source 정확 경로/before/mtime/bytes를 동결한다. 수정본을 인수하면 원 r1을 덮어쓰지 않는다. runner EEA9/conservative2/offline/locked/-j1/thread1, snapshot cwd와 절대 snapshot/Cargo.toml manifest·기존 project-9dc296c4/native target를 명시한다.

- `cargo test ... -p media-jobs --lib infrastructure::subtitle_executor::tests:: -- --test-threads=1 --nocapture` — 작성10.
- `cargo test ... -p kinekino-player --lib application::api::job_schema::tests:: -- --test-threads=1 --nocapture` — 작성2.
- 기존 convert 회귀는 `-p media-jobs --lib infrastructure::files::tests::actual_`의 원 conversion/loss/cancel 시험, root `--test subtitle_conversion` 및 `--test subtitle_range_cli`로 책임을 분리한다. 실제 선택명·수로 정정하고 broad test --list는 하지 않는다.

### 2. 실제 queue + fresh CLI 성공

아래 literal 명령은 Rust Fixture가 원 Child handle/stdin을 소유하여 실행한다. shell의 새 테스트/검증 로직을 만들지 않는다. Request.json은 기존 Request::new/serde로 쓰고 context/request schema를 복제하지 않는다.

```text
player-cli.exe --serve-jobs --job-root <fresh-absolute-store>
player-cli.exe --endpoint <observed-ready-endpoint> --input <Rust-serialized-Request.json>
```

1. ready.execution_operations에 convert/export_range, fresh capabilities의 실행 host와 jobs.submit schema의 range 원 schema를 확인한다.
2. fresh CLI jobs.submit의 payload는 아래와 같다. context.idempotency_key는 새 명시 key, timeout은30000이다.

```json
{
  "operation":"subtitles.export_range",
  "parameters":{
    "input_path":"<fresh-source.srt>",
    "output_path":"<fresh-job-range.vtt>",
    "input_format":"srt","output_format":"vtt",
    "range":{"in_us":"2000000","out_us":"5000000"},
    "allow_loss":false,"speaker_policy":"omit"
  },
  "expected_source_sha256":"2840bb1f0ba39a911fe040b9bb79861486628f0f2bb2e7a5bbd57432782017aa",
  "execution_timeout_ms":"30000"
}
```

3. submit API exit0/created=true/job ID·원 input.source.sha256/bytes94를 보존한다. 대기는 기존 bounded SDK get로 줄일 수 있지만 **최종 terminal은 새 CLI jobs.get**으로 인수한다. 자동 retry/새 submit을 하지 않는다.
4. 기대 terminal은 completed, output.path=요청 경로, output.sha256=f85e…288c, output.bytes="98"이다. receipt는 output_format=vtt/cue_count2/output_bytes98/losses[]/encoding=utf-8/overwrite=false다. source94B/hash는 그대로이며 원 blob도 보존한다. actual output은 고정 expected98B 및 기존 동기 usecase 결과와 각각 대조하되 동기 성공을 Job 근거로 대체하지 않는다.
5. 같은 key/같은 normalized input은 created=false/같은 ID다. valid convert payload에서 range 필드를 빼고 같은 key를 쓰거나 range bound를 바꾸면 idempotency_conflict/RevisionConflict다. 새로운 queued/publication을 만들면 안 된다.
6. 모든 job terminal 뒤 EOF로 host를 닫는다. 원 ready/stop JSON, LocalHost pipe/sdk 각각 joined·dispatch 의미, dependency receipt, 원 Child wait/exit0·CLI artifact를 보존한다. process exit와 양 join은 구별한다.

### 3. 실패·손실·취소

| 경우 | 기대·근거 |
| --- | --- |
| 잘못된 expected SHA | fresh submit SourceExpired/nonzero, job/output0. source digest 불변 |
| 기존 output | submit은 accepted일 수 있음. terminal NeedsInput/RevisionConflict, 기존 bytes/hash 보존 |
| empty/out-of-range 또는 in>=out | 실행 phase의 원 InvalidArgument→NeedsInput, output0. 임의 early validation 정책 추가0 |
| μs precision loss | allow_loss=false이면 NeedsInput/losses·output0. 새 key의 명시 allow_loss=true만 별도 job으로 출력; 기존 job/retry의 immutable 입력을 수정하지 않음 |
| source admission 뒤 변경 | 결정적 package seam에서 SourceExpired/보존 snapshot/output0. 실제 CLI에 gate 없이 성립했다고 주장0 |
| 취소 before/after publication permit | 작성 coordinator CAS의 before=Cancelled/output0, after=PublicationInProgress/원 완료·파일 보존. 실제 CLI는 기존 Rust cancel Gate를 range에 적용할 별도 시험 인수 전까지 미검증 |
| deadline/readback | 작성 포트 주입/실 publish_before·readback fault 근거와 실제 CLI를 분리. 1ms production timeout 같은 불안정한 시험으로 보장 주장0 |

CLI 실패/cancel 근거를 추가 작성하려면 r1 동결과 분리해 승인된 테스트 범위에 넣고 원 production 정책은 그대로 둔다. 테스트 marker/네트워크/브라우저/모델 플래그는 추가하지 않는다.

### 4. strict·보호 마감

default/native-proposal-window all-target strict와 fmt를 수행한다. source/before/비중첩보호·CLI SHA·원 request/response/exit·파일/digest/Job receipt/취소 disposition·원 PID identity를 연결한다. 각 원 exec terminal 후 다음 target을 수행하며 실패는 원 log/pin을 보존한다. 관측된 pass만 수에 넣고 반복 동기/Job 결과를 중복하지 않는다.

## Rollback — 원본 보존·반환

기존 output/source와 captured blob을 임의 삭제하거나 overwrite하지 않는다. 부분 publish/readback 실패의 파일은 원 정책에 따라 보존한다. 새 fixture root만 원 owner가 정리하며 evidence root는 남긴다. 무관한 프로세스를 kill하지 않는다. source6 또는 수정 source의 독립 인수/실행 실패는 live에 반영하지 않고, 보호·원 identity 종료를 마감해 PM에 snapshot/heavy를 명시 반환한다. modoo.today 계획/변경 검증 본문·참조의 관측/기대/미검증 분리를 적용했다.
