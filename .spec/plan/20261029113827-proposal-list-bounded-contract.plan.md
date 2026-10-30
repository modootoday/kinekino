---
id: plan:kinekino/20261029113827
title: proposals.list — bounded 읽기 계약 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# proposals.list — bounded 읽기 계약 제안

2026-10-23 root `579f5db0778a9d48f2eb2afc251c1f232cca7de3` 소스 대조. R0의 [계약 조사](proposal-live-selection-r0-sdk-review.md)를 구체화한 **미구현 제안**이다. 새 코드/Cargo/S/T/실제 저장소 열거/CLI/네트워크 실행0. 제품14는 불변이며 UI가 sole-heavy다. create/import 결과의 명시 host handoff는 UI 별도 선행 경로로 유지한다. 목록은 mutation 권한을 발급하지 않는다.

## 현재 포트로 가능한 것과 추가가 필요한 것

| 실제 경로 | 관측 | 적용 판단 |
|---|---|---|
| `src/infrastructure/proposals/reservation.rs` | acquire는 기존 writer.lock의 share_mode0 파일을 보유하며 check는 store.path를 비교한다. lock 생성 옵션이 있다 | 페이지 하나를 원 writer 아래 읽어 SDK publication과 직렬화한다. read operation이지만 writer.lock 최초 생성 부작용은 명시한다. busy는 기존 RevisionConflict, 자동 retry/sleep 없음 |
| `immutable_storage_paths.rs`, `ProposalFiles::_guards` | ordinary local root/역순 ancestor handles, reparse 거절, directory share_mode3·file share_mode1 | 원 root guard를 유지한다. arbitrary path/UNC/renderer path를 받지 않는다. 경로문자열을 dataset identity로 사용하지 않는다 |
| `storage_directory_identity.rs` | original FileIdInfo의 volume/file identity | `_guards.last()`의 **같은 원 root handle**에서 읽기 전후 관측한다. unsupported filesystem은 Unsupported. 목록은 origin metadata를 새 발급하지 않는다 |
| `state_read.rs::revisions/read/read_current_locked` | revision 전체 수집·정렬에 개수 상한 없음, revision 내 파일도 전체 collect. state는 4MiB 한정/hash/typed 검증 및 first/latest immutable 대조 | 이 함수를 그대로 반복 호출하고 행 수만 제한하면 bounded 목록이 아니다. bounded budget을 받는 공통 내부 reader가 추가로 필요하다. 기존 load/mutation의 허용 범위는 바꾸지 않는다 |
| `ProposalFiles::load`, `ai_provenance::verify_bound` | bound historical state는 configured verifier를 동기 호출 | 목록은 structural metadata 읽기이며 current historical evidence를 검증하지 않는다. 외부 verifier를 문서마다 실행하지 않는다. 결과에 `current_evidence:not_checked`를 명시하고 명시 inspect/open의 기존 검증은 그대로 유지한다 |
| `sdk_execution/control.rs`, `worker.rs`, `close.rs` | queued cancel은 실행 전 Cancelled. running cancel은 flag만 설정하고 `client.execute`에 token을 전달하지 않음. close는 terminal ACK/원 worker join을 관측 | 현 포트로 실행 중 scan 중단/즉시 close를 약속할 수 없다. 원 요청은 끝까지 보유·terminal 소비/ACK·join 후 해제한다. timeout/PID 부재를 물리 종료 ACK로 바꾸지 않는다 |

## 선택한 v1 계약

새 `proposals.list`는 configured ProposalStore의 선택적 읽기 capability다. 기존 provider 기본값은 Unsupported이며 일반 available=true를 목록 지원으로 승격하지 않는다. 기존 inspect/edit/accept/create/import 지원은 영향받지 않는다.

입력은 strict `{task?:correction|cards|highlights, limit?:integer, cursor?:string}`이다. 기본50/1..200행, unknown field·명시 null·빈 cursor·0/초과 limit·unknown task는 InvalidArgument. cursor가 없으면 첫 페이지다. RequestContext의 expected_revision/idempotency_key/timeout_ms는 기존 proposals 읽기와 같이 허용하지 않는다. 현재 operation deadline을 제공한다고 표시하지 않는다.

행은 proposal_id/task/draft_revision/draft_state/source_id/source_revision/source_sha256/typed_item_count를 포함한 **문서 metadata**다. NeedsRepair의 count는0이며 repair는 명시 source-ref 선택 단계가 필요하다. 목록에 원문/generated raw/draft payload/파일 path/origin/편집 가능 권한을 넣지 않는다. 행의 최신 revision과 source witness를 host에 보유하고 명시 inspect 결과와 대조한 뒤 item/ref를 선택한다. 목록 응답이 native panel 입력을 직접 구성하지 않는다.

응답은 schema_version/rows/next_cursor/dataset_witness/snapshot_witness/current_evidence=`not_checked`/selection_authority=`none`을 갖는다. 정렬은 proposal ID의 ASCII 오름차순이며 task filter를 적용한 다음 cursor의 after_id 뒤에서 limit만 반환한다. 빈 첫 페이지와 마지막 페이지는 next_cursor 없음이다. 데이터 읽기·모든 정책 검증이 끝나기 전 일부 rows를 성공으로 내보내지 않는다. current_evidence가 not_checked인 것을 live editable/quality approved로 해석하지 않는다.

### Dataset·cursor·snapshot

- dataset_witness는 원 directory handle의 volume/FileID를 versioned canonical 형식으로 SHA256한 비밀 아닌 witness다. `.proposal-store-origin.v1.json`이나 path의 hash를 대신 쓰지 않는다. legacy store도 새 origin 파일 발급 없이 목록화할 수 있다. 같은 물리 directory의 명시 rename/reopen은 identity가 같을 수 있고 copied/replaced root는 다른 dataset이다. FileID를 영구 보안 식별자로 주장하지 않는다.
- snapshot_witness는 **전체 configured proposal 집합**의 정렬된 `(proposal_id,latest_revision,validated_current_state_sha256)`와 dataset을 versioned canonical hash한 값이다. filter 밖 문서 변경도 snapshot을 바꾼다. raw directory ordering/mtime/서브폴더명만으로 snapshot을 만들지 않는다. writer 밖의 비협조적 조작에 대한 보안 transaction은 약속하지 않는다. root/파일 guard와 hash/typed 검증이 관측한 위반은 실패로 반환한다.
- cursor는 version/dataset/snapshot/query(task,limit,order)/after_id를 가진 최대2048B opaque encoding이다. 클라이언트가 보내는 cursor는 항상 불신하고 strict decode·크기·모든 필드와 after_id의 snapshot membership/filter 정합을 검사한다. 서명된 권한이나 원 host handle로 표시하지 않는다. 잘 구성된 read witness가 있어도 mutation은 기존 guard를 다시 거쳐야 한다.
- 페이지마다 동일 원 store writer를 잠깐 보유해 전체 bounded manifest를 재계산한다. dataset 다름은 SourceExpired, 같은 dataset의 snapshot 다름은 RevisionConflict(`restart_proposal_list`). create/edit/accept 또는 삭제/추가가 있으면 이전 continuation을 자동 갱신하지 않는다. query/limit을 바꾼 cursor·형식 위조는 InvalidArgument다. 다른 root의 복사본이 같은 내용이어도 cursor를 재사용하지 않는다.
- 응답 후 writer/reader는 해제한다. cursor를 위해 원 writer나 OS enumeration handle을 페이지 사이에 남기지 않는다. 따라서 다른 CLI 프로세스에서 continuation이 가능하다. 목록 시점의 일관성은 같은 SDK writer 규약을 지키는 publication에 대한 것이며, 목록→inspect→edit 사이의 transaction이 아니다. 그 사이 변화는 기존 정확 revision/ref 검사가 거절한다.

## 자원·파일 검증 상한

아래 값은 **제안한 v1 상수**로 아직 성능 측정되지 않았다. 요청이 임의로 늘릴 수 없다.

| 자원 | 제안 상한/실패 |
|---|---|
| root entries | reserved entries 포함4096개+초과 검출1개. 중간 성공 반환 없이 ResourceExhausted |
| proposal candidates |4096개. invalid/noncanonical p-ID는 NeedsInput, 이름별 정렬은 이 범위 안에서만 |
| revision enumeration | proposal당4096개, 호출 전체8192개. 양쪽 상한을 모든 iteration에 적용, 정렬/연속1..latest 검증 전에 이미 부과 |
| revision content entries |2개까지만 열거해 정확1개인지 판정. 기존 read의 무제한 collect를 그대로 호출하지 않음 |
| state bytes | 파일당 기존4MiB, 호출 전체 first/latest의 실제 읽은 bytes 합32MiB. read 전에 남은 budget 검사·초과 검출용1B도 별도 한정. 이름/metadata file length만으로 성공 판정하지 않음 |
| rows/response |1..200행, 행당1024B, 전체 API envelope256KiB 이내. rows를 조용히 truncate하지 않고 초과는 ResourceExhausted. 기존 transport4MiB 및 더 작은 ExecutionBudget을 우회하지 않음 |
| retained memory/handles | 전체 state를 누적 보유하지 않고 한 state씩 검증 후 metadata만 유지. 전체 metadata/canonical snapshot 입력도 고정 byte budget 내에서 계산. 페이지 종료/오류 시 원 request의 readers/writer 모두 해제 |

root의 알려진 reserved 항목은 writer.lock/선택적 origin metadata/선택적 output-decisions다. origin 내용과 output history는 목록에서 재검증·재귀 탐색하지 않지만 이름·ordinary 파일/디렉터리·reparse 정책을 검사한다. origin 내용이 없거나 불완전하다는 이유만으로 legacy 읽기를 mutation 권한으로 승격하거나 origin을 복구하지 않는다. 예상 밖 root entries·incomplete proposal revision은 NeedsInput으로 보존하고 자동 삭제/skip하지 않는다. 최상위 root entries 상한은 무관한 항목도 센다.

page filter로 제외되는 proposal도 manifest/source state 구조·hash·revision 검증 및 scan budget을 소비한다. 일부 손상을 필터 뒤에 숨겨 전체 목록 성공으로 만들지 않는다. payload 검증은 기존 source/task/ref 규칙을 재사용한다. historical binding은 구조만 확인하고 증거의 재현/현재 실행 가능 여부는 `not_checked`다. 그 뒤 inspect가 Unsupported/오류를 반환하면 UI는 해당 원 오류를 표시한다.

writer는 요청이 성공/실패할 때 해제하며 pagination 동안 계속 소유하지 않는다. 기존 helpers의 read/lock이 실제 OS I/O에서 막힐 수 있으므로 파일 수/bytes 상한을 hard wall-clock 제한이라고 표시하지 않는다. 대규모 저장소는 이 v1에서 ResourceExhausted이며 이후 index/transaction 설계가 별도 필요하다.

## 실패·취소·close의 정확한 의미

| 관측/입력 | 반환/정리 |
|---|---|
| absent backend 또는 provider에 list 미등록 | Unsupported, capabilities/schema의 list만 미지원 |
| configured root 없음/읽는 도중 expected entry 부재 | NotFound. 자동 root 생성0, 일관된 성공 빈목록으로 치환0 |
| busy writer | 기존 RevisionConflict, 자동 wait/retry0 |
| 잘못된 cursor/request | InvalidArgument, store 열기/scan 전 거절 가능한 필드는 선검사 |
| dataset 변경 | SourceExpired, 원 selection/cursor 자동 대체0 |
| 같은 dataset의 snapshot 변경 | RevisionConflict와 명시 재목록 remediation. 기존 rows를 최신으로 자동 승격0 |
| hash·revision 연속성·typed source/ref/immutable 불일치·incomplete publication | NeedsInput, 원 파일 보존. 행 skip0/원 raw path 노출0 |
| reparse/nonordinary/접근 거부 | PermissionDenied, path fallback0 |
| 안정 root identity 미지원 | Unsupported. 문자열/mtime identity로 대체0 |
| scan/byte/response budget 초과 | ResourceExhausted, partial success/cursor 발급0 |
| SDK queued cancel | 현재 queued Cancelled, 실제 scan0. 원 terminal 소비/ACK 유지 |
| SDK running cancel·transport timeout·close 요청 | 원 scan 중단 ACK가 아님. running Requested/불확실한 transport 결과를 유지하고 원 terminal·ACK·worker join까지 소유. 새 worker/중복 scan/retry0 |

강한 실행 중 취소/고정 deadline이 필요하면 현 exact 목록 변경에 숨기지 않는다. RequestContext는 wire일 뿐 cancel token이 아니며 jobs의 AttemptScope를 일반 SDK 읽기에 임의 재사용하지 않는다. 별도 application execution-control port 및 `sdk_execution/{worker,control,types}.rs`, `api/mod.rs`, client 계약의 명시 배정·회귀가 필요하다. 그 단계 전 capability에 in_flight_cancel=false를 명시한다. 단순 queued 취소 성공을 running scan 취소 성공으로 표시하지 않는다.

## exact 경로와 최소 단계

원 R0 exact8 후보의 책임은 유지한다: 신규 application/proposals/list.rs, 기존 application/proposals/mod.rs, 신규 infrastructure/proposals/list.rs, 기존 infrastructure/proposals/mod.rs, 기존 api/client.rs·capabilities.rs·proposals/schema.rs, 신규 tests/proposal_list.rs.

**원8만으로는 완결되지 않는다.** 다음 추가 경로를 제안하며 아직 배정/작성하지 않았다.

| 추가 경로 | 필요한 이유 |
|---|---|
| 기존 `src/application/api/mod.rs` | 현재 schema::describe/capabilities에 store.available 하나만 전달한다. optional list 지원을 기존 create/inspect 지원과 분리하려면 실제 호출부 변경이 필요 |
| 기존 `src/infrastructure/proposals/state_read.rs` | bounded revision/content 열거·aggregate byte accounting을 기존 hash/typed/first/latest 검증과 공유한다. 새 list에서 정책을 복사하거나 기존 무제한 helper를 bounded라고 표시하지 않음. 기존 public load/mutation 상한은 불변 |
| 신규 `tests/support/proposal_list_fixture.rs` | 실제 CLI+configured store의 명시 setup/원 child/raw/teardown/재개방·lock 회수 관측을 tests/proposal_list.rs의 요구 단언과 분리. 기존 owned-process helper 사용 |

따라서 최소 완결 제안은 exact11(원8+추가3)이다. source-only 제안이며 전체 구현 권한이 아니다. 추가 unit test 파일이 필요하면 행정적으로 숨겨 넣지 않고 책임/경로를 별도 제안한다.

1. exact 계약/상한/오류·snapshot 정책을 먼저 승인·동결한다. create/import handoff와 UI R1/R2는 목록 완성을 기다린다는 이유로 기존 입력 포트를 복제하지 않는다.
2. SDK/application·storage exact10 구현/optional provider·typed/schema/capability와 구조화된 fake provider 시험. 실행 배정 뒤 fmt/test/clippy 및 실제 등록을 확인한다.
3. 위 CLI fixture 추가 exact1로 실제 standalone CLI 여러 프로세스의 목록→continuation→명시 inspect를 검증한다. scope/root 변경·writer 경합·손상/상한·queued/running cancellation과 원 close 관측은 아래처럼 구별한다.
4. 인수 후 UI async registration/manifest/pending_host에 별도 exact 배정으로 연결한다. 목록 rows는 원 host에 두고 callback opaque key만 노출한다. stale/error/빈 상태를 명시하고 자동 첫 선택·origin/current inspect만으로 captured live 승격0을 유지한다.

## 실제 CLI 시험 경로·입장과 근거 제안

`tests/proposal_list.rs`는 기존 `tests/support/owned_process_evidence.rs`로 producer/hash/bytes가 고정된 player-cli와 --proposal-root를 호출한다. `tests/support/proposal_list_fixture.rs`는 기존 ProposalFiles/공통 SDK로 고정 Correction/Card/Highlight 및 repair/역사 structural fixture를 준비한다. 새 JavaScript/PowerShell 검증 정책0. 각 case는 새 evidence namespace, raw stdin/stdout/stderr·원 process receipt·지원/error code/request_id를 보존한다. historical metadata 목록은 verifier 프로세스를 시작하지 않았음을 별도로 관측한다.

| 제안 시험 | 요구 관측 |
|---|---|
| empty/last/filter/order/repair | 빈 next_cursor, ASCII 정렬, limit 경계, task 분리, repair count0. CLI 결과와 typed client 동일 |
| cross-process continuation | 같은 root로 별도 CLI2가 cursor 소비. 중복/누락 없는 명시 fixture ID 집합, inspect exact revision/ref 확인 |
| concurrent create/edit/accept | page1 뒤 별도 원 store mutation→page2 RevisionConflict. same filter 밖 변경도 stale, 자동 새 cursor0 |
| dataset copy/rename/replacement | 다른 root 동일 bytes cursor→SourceExpired. 같은 물리 directory rename은 guard 해제 뒤 명시 reopen으로 시험. 원 handle 유지 중 replacement 거부와 재개방 identity 교체를 별도 관측 |
| corruption/reparse/incomplete/foreign cursor | 원 파일 그대로·명시 오류·partial rows/cursor0, 외부 파일을 읽거나 고치지 않음 |
| root/revision/bytes/response bounds | 경계 바로 아래/정확/초과. 작은 budget 주입으로 정상 테스트를 빠르게 하되 production 상수와 연결 시험을 따로 남김 |
| writer custody | fixture가 실제 writer를 보유할 때 list 실패, drop 뒤 같은 root list 재성공. 같은 ID·새 owner로 성공을 조작하지 않음 |
| cancellation/close | SDK 원 ExecutionOwner로 queued cancelled scan0·running Requested를 분리. running 중 future scan 작업을 막는 barrier는 시험 adapter 안에서만 보유. 원 결과 소비/ACK 전 close pending, barrier 해제→terminal→ACK→Joined/lock 재획득을 확인. CLI child 중단을 SDK scan 완료 근거로 삼지 않음 |
| legacy/optional backend | origin 없는 store listing이 origin 파일을 발급하지 않음. 일반 store available=true/list unavailable provider가 create/inspect는 유지하고 list만 Unsupported |

지연/스캔 횟수·메모리·실제 처리시간은 미측정이다. 계약 시험 fixture는 OS hard I/O timeout, 대규모 저장소 성능, 사용자 전체 GUI 여정 또는 historical current evidence 검증을 입증하지 않는다. GUI 목록 기능 인수는 SDK/CLI 목록 gate 뒤 별도 원 host/route/proof 검증을 요구한다.

## 스킬 적용과 검색 판단

modoo.today에서 pagination/파일 읽기/정리 계약으로 탐색했다. 최초 추천 frontend-first-journey·agent state audit·rule table은 이번 단일 SDK 읽기 계약에 맞지 않아 채택하지 않았다. software-qa로 좁힌 탐색의 `modootoday/software-qa/test-fixture-and-mock-build` 본문(SHA256 a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100)을 실제 읽었다. 정의된 상태·빈/마지막 페이지·실제 writer 점유/해제/재획득·원 child/raw·미측정 성능과 fixture 한계의 명시에 적용했다. 스킬을 계약 설계 자체의 권위나 기존코드 실행 성공 근거로 사용하지 않았다.
