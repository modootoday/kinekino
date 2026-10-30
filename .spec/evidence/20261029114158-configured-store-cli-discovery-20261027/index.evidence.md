---
id: evidence:kinekino/20261029114158
title: Project/Proposal/Job 저장소 조합 CLI 발견 — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Project/Proposal/Job 저장소 조합 CLI 발견 — 2026-10-27

제품 정책 변경 없이 `tests/cli_discovery_inventory.rs`를 확장해 명시 `--project-root/--proposal-root/--job-root` 조합의 실제 capability와 모든 command schema를 수집했다. worker·historical verifier·catalog/model/review root는 주입하지 않았다. 모델/미디어 다운로드·추론·수동GUI·WARP·push 없음. 발견 성공을 기능·품질 승인으로 계상하지 않는다.

## 실행 근거와 첫 실패

로그 `runtime/verification/source375-datalab-b0-r1-logs/764~771-store-discovery-*` 및 원 증거 `runtime/build-cache/tmp/cli-store-discovery-34200/`, 보존 복사 `runtime/verification/configured-store-discovery-r1/`에 요청 인자에 대응한98 stdout/stderr 쌍과 inventory가 있다. CLI SHA256=1DF8ADD0F6210B4E7C38CEC46943D0EA6647530086902DCB752752D4C735378D를 전후 비교했다. query는 각 child의15초 deadline/원 wait/reap을 사용한다.

764 no-run 후 765 실제 시험은 import_historical을 verifier 없이 supported로 가정해 실패했다. 실제 capability는 unsupported였다. 원 결과는 tmp/cli-store-discovery-4724와765 로그에 보존했다. 766은 import 기대를 고친 중간 빌드이며 실제 시험은 실행하지 않았다. 코드 확인에서 JobFiles::open/read_only가 최초 job-events/job-sources/jobs-writer.lock을 초기화함을 확인해 잘못된 empty-root 가정도 실행 전 수정했다. 초기 capability 이후 디렉터리·파일 SHA inventory를 잡고97개 schema 조회 뒤 동일함을 비교했다. 초기화 자체가 readonly 무생성이라는 주장은 하지 않는다.

767 최종 no-run exit0/10128ms, 768 actual1/1·CLI98회 exit0(시험15.09초), 769 해당 Native test strict exit0, 770 fmt exit0. inventory는 supported28/unsupported69이며 명령 중복 없음/각schema 상태 일치/원CLI hash 유지/초기화 후 세 저장소의 경로와 파일 내용 동일을 확인했다.

| 구성에서 확인한 명령군 | 지원 | 미지원·입장 제한 |
|---|---|---|
| projects | create/inspect/save_revision/reconnect_source | export 미지원, reconnect는 locator-only |
| proposals | create/list/inspect/edit/accept/export_subtitles/inspect_output_review_target/record_output_decision/output_decisions/edit_capture/accept_capture | import_historical은 verifier 부재로 미지원; 지원 명령도 historical bound 입장 검증을 생략하지 않음 |
| jobs | get/list/watch | submit/cancel/retry는 worker 부재로 미지원 |
| 그 외 | 기본 file capability와 proposal-root가 공급하는 subtitles.edit_cues 포함 | 모델/카탈로그/리뷰/재생 기능 인수 근거 아님 |

## 핀·범위

772 전후 검증은 exit0/41820ms, 전체2244 content/identity 동일로 종료했다. 원 exact 로그를 추가 감사한 결과 jobs6은 관측/정리 정책 시험이며 actual_cli_explicit_host_and_sdk_share_durable_jobs_without_gui는 등록 목록에만 있고 exact 실행 대상에는 없다. 담당의 처음 ‘기존190 내 실제종단’ 추정은 잘못되어 정정한다. 등록과 실행을 구별하고 다음 별도 실제 시험으로 배정한다. 아래 매핑 제안은 조사 중 기록이다.

771 입력 고정은 완료했다. SOURCE.md SHA256=28B8A1E922C867B25BB5ED08E489A057B13F23306131866BB6BEA04F668E7DBC, inventory SHA256=C2E18E1B936CC2EC9CBF651BC7FCB08A93C990326A02B6C7C52EA39F5F08D522, files2244. `runtime/verification/source375-store-discovery-input-r1/`에 있다. 772 전후 검증은 진행 중이다. 기존 tests/jobs.rs의 actual_cli_explicit_host_and_sdk_share_durable_jobs_without_gui가 host/get/list/watch/재시작/fresh readonly get을 다루므로 다음에는 해당 원190 로그를 먼저 매핑하고 실제 공백만 보완한다. 아래 진행 중/다음 문장은 초기 기록이다.

시험 SHA256=B90CAF6ED09EFB13A31FD8E46CC66287F5036B38D520F302C0E71002119F9018. `runtime/verification/store-discovery-preimage-r1/cli_discovery_inventory.rs` SHA256=52C0773CC3037FB2D6B415CE997949D7BAB2E88D98BFF1008AD92395FD401DB4. 제품 코드·기존 WIP/소유권·원본 증거·보류 유지. 771 새2244 입력 고정은 진행 중이다.190/정상bins/전체strict는 SOURCE268E3482의 원 실행이며 이번 시험 변경 후 재실행으로 옮기지 않는다.

기존 읽은 modoo.today test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100)의 실제 fixture·초기화/관측·기능 검증 한계를 적용했다. 다음은 이 구성의 실제 job read 시나리오와 명시 host/operation의 미구현 공급 경로를 구분하는 것이다. 네트워크 다운로드 producer를 arbitrary URL/선택 SHA만으로 대체하지 않는다.
