---
id: plan:kinekino/20261029113853
title: B 실제 저장 결과 읽기 재열기 최소 계획
status: active
created: 2026-10-29
updated: 2026-10-29
---
# B 실제 저장 결과 읽기 재열기 최소 계획

## 현재 상태 — B4 gate 완료

PM 후속 배정으로 source-only 단계 뒤 실제 gate를 수행했다. 기존2 A0FC + crate 내부 가시성 exact1의 composite3 F5C15D92에서 fmt/default·native strict/lib no-run 및 고정 binary의 list 대조를 마쳤고 B3+기존 D Card1이 각각1회 통과했다. 원 E0603 실패와 운영 비교 중지는 별도 보존했다. UI3 bytes/mtime 복원·staging 제거·보호 감사 완료이며 SDK7 승인 동기화는 별도 baseline으로 유지했다. 실제 관측과 한계·반환 근거는 [B4 검증 정본](../../../sots/verification/presentation/persisted-review-reopen-b4-2026-10-24.md)에 기록한다. 아래는 당시 배정별 계획 이력으로 현재 실행권을 뜻하지 않는다.

2026-10-24 PM source-only 계획 배정. A14 source F6DA49B9 및 정확12 계획은 독립 리뷰 인수 상태이며 실행·채택은 별도다. 이 계획에서 코드/시험 작성·Cargo/GUI/S/T/AI/CLI 실행0이다. A 수정 요청이 오면 우선한다. Meari가 현재 heavy 소유이며 이 문서는 B 실행권이 아니다.

## 실제 준비 가능한 입력과 출처

기존 SDK/CLI 정본 `.spec/sots/verification/application/proposals-2026-10-19.md`의 실제 nine r3 및 재시작 receipt 검증을 참조한다. 원 실제 산출물은 `runtime/quality-review/proposal-sdk-r3/actual-nine-r1/technical-store/`와 `requests/`에 남아 있다. source-only 읽기에서 task별 실제 immutable revision1/2/3 파일, CLI technical-edit/technical-accept/restart-inspect의 request/response/exit 기록을 확인했다. 입력 metadata와 파일별 SHA/bytes는 `runtime/development/review-feedback-persisted-reopen-plan/original-fixture-custody3.json` SHA256 `220B57E00F3EB6D1A920F58D812C01E13731DA5EF54DD9C93F9B908EA6FDA811`에 고정했다. 원문/생성 텍스트는 이 문서에 복제하지 않았다.

세 후보 ID는 actual-gemma270-cards / actual-gemma270-highlights / actual-gemma270-correction다. 모델명은 품질 선정 이유가 아니라 같은 검증 묶음의 fixture custody 식별자다. 원 package SHA는 659b3cb6757259d8e770f6d3c96c4a77dafbab43c6ea540bcc7e0c58e4eb0197, 당시 CLI SHA는 bc6483acf78e4a973b5716387e6c0ae59659ea8d365d7986f72941dec5594b00이다. 옛 CLI exe를 이번 B에서 실행하거나 현재 제품 BIN으로 부르지 않는다.

각 proposal의 r-1은 needs_repair, r-2는 실제 CLI edit가 저장한 editable/no acceptance 문서 revision2, r-3은 실제 CLI accept·재시작 inspect로 확인된 문서 revision3/receipt draft_revision2/output_revision3이다. 이는 `examples/proposal_actual_import/run.rs:130` 이후 원 CLI edit/accept/restart-inspect 흐름과 `tests/proposals.rs:176` 이후 재시작 비교의 이미 검증된 기능 범위다. source snapshot SHA/생성 raw 보존 및 receipt 재조회는 기존 근거를 연결하고 B에서 기능 시험으로 반복하지 않는다.

이 technical fixture는 명시 scripted 수정/채택의 실제 SDK 파일 산출물이다. 사람의 품질 판정은0이며 품질 승인false/자동 적용false/렌더 파일 생산false다. 새 원문 snapshot의 source_revision1은 복구된 원 프로젝트 최신 revision이 아니다. 실제 모델 출력과 기술적 채택 결과를 인간 승인·신뢰 provenance·원문 최신성으로 승격하지 않는다.

## 준비 포트와 가짜 seed 방지

원 `ProposalFiles::open`/`ProposalStore::load` 및 `PlayerSdk::with_proposal_store`가 유효한 읽기 준비 경계다. `ProposalStore::publish(None, state@revision3)`는 사용할 수 없다. 실제 publication.rs는 None일 때 revision1만 허용한다. 거짓 revision3 객체를 publish하거나 create/edit/accept로 다시 만드는 우회는 제외한다.

후속 B 구현을 배정받으면 fixture custody를 **원 immutable byte 파일의 격리 복사**로 준비할 제안이다. 원 root는 read-only로 취급하고 새 child 전용 store가 원 SDK에 열리기 전에 해당 proposal의 정확 r-N/sha.json leaf만 Rust의 제한된 fixture I/O로 복사한다. Card saved case는 원 r-1/r-2만, accepted case는 원 r-1/r-2/r-3을 복사한다. 경로·해시·크기·regular/non-reparse·원부재 destination을 확인하고 writer.lock/실행 파일/전체 assets/다른 proposal·runtime 제어 파일은 복사하지 않는다. 원 ID/source/ref/문서·receipt·SHA 이름·바이트를 변경하지 않는다. 원 store 또는 SDK가 열린 store에 뒤늦게 r-3을 삽입하지 않는다.

이 복사는 저장·채택 기능이나 publication/CAS 성공을 재현하는 절차가 아니라 기존 실제 SDK 산출물의 test custody 이동이다. 옛 root와 새 root의 leaf SHA 일치·선택 revision 상한 및 원 CLI response 근거를 manifest로 연결한다. 현재 원 `ProposalFiles`/SDK가 새 격리 root를 읽고 validate/inspect에 성공해야만 현재 호환 입력으로 인정한다. source-only JSON metadata 읽기만으로 현재 SDK 호환을 확인했다고 쓰지 않는다. 부재/변조/구버전 미지원/검증 오류는 첫 실패로 보존하고 조용히 임의 seed로 치환하지 않는다.

원 Supervisor의 env whitelist는 CHILD_MODE/CHILD_EVIDENCE만 별도로 전달한다. 임의 fixture 환경변수를 child에 전달하기 위해 세 번째 소스 경로를 늘리지 않는다. 원2경로 안에서 명시 gate input artifact를 S의 고정 runtime 전용 staging 경로에 미리 배치하고 원 SHA를 fixture spec에 고정하는 방식을 우선 제안한다. 실제 child는 그 read-only staging에서 자기 evidence root의 새 store로 정확 leaf를 복사한다. staging 준비와 child setup 시간을 각각 기록하고 child 내부 작업은 기존 원 deadline에 포함한다. 이 staging은 후속 gate의 명시 입력으로만 승인·보호하며 public asset/source에 실제 원문·모델 출력을 내장하지 않는다.

## 최소 task 선정 — 신규 child3 후보

| 후보 | 실제 입력 | 표시·연결에서 새로 확인할 것 |
|---|---|---|
| Card saved | 같은 actual-gemma270-cards 원 r-1/r-2, 현재 revision2·acceptance 없음 | 저장된 사람 수정 형태의 front/back·원 source/ref와 숫자 document revision2, output 없음·저장된 수정본 표시 |
| Highlight accepted | 같은 actual-gemma270-highlights 원 r-1/r-2/r-3 | title/source cue ref·document revision3 및 실제 receipt output_revision3·저장된 채택 결과 표시 |
| Correction document accepted | 같은 actual-gemma270-correction 원 r-1/r-2/r-3 | full document source_ref·실제 draft edit/editor·document revision3/output3·ProposalSource 정확 li와 결과 표시 |

task당 하나로 총3을 제안한다. Card는 saved/no-output 분기, Highlight는 accepted/output 분기, Correction은 document ref와 textarea/alert 경계를 대표한다. 같은 UI formatting 상태 전부를 세 task 각각 반복하는 것은 A pure8+typed1과 중복이라 제외한다. 다만 task마다 actual source component/slot·editor/ref가 달라 실제 B 연결을 Card 한 번으로 세 task 완료라 표시하지 않고 세 task를 각각 확인한다. Card accepted·Highlight saved·Correction saved의 모든 조합을 actual OS로 검증했다는 주장은 하지 않는다. 이 추가 조합은 A 공통 투영 근거 범위와 구별해 잔여로 둔다.

원 technical fixture는 task당 유효 draft item1이다. B에서 항목2를 만들거나 D의 기존2doc/2item 시험을 축소하지 않는다. B는 별도 case에서 같은 실제 item0를 명시 선택하며 D의 원 3task·두 문서·항목 요구와 원 성공 근거는 그대로다.

## 실제 source/frame 여정

같은 원 SDK/client/LocalHost/execution·NativeRuntime/NativeDocumentHandle을 유지하고 기본 ProductFrame choicesNone을 사용한다. TrustedLiveChoices/직접 callback/SDK 선택 권한 주입0이다.

1. 격리 store의 원 SDK read 결과와 fixture custody가 준비된 뒤 actual Viewing retained frame을 기다린다.
2. 현재 source의 List→정확 proposal ID/metadata 현재 revision이 표시된 실제 행→명시 item0→Open으로 첫 entry를 만든다. 목록 순서의 첫 항목이나 cached path/key를 사용하지 않는다.
3. 원 current inspect/ACK 및 실제 retained entry/clean 활성 뒤 현재 ProposalSource metadata의 정확 revision 표시, 원 source identity/ref, 해당 editor 값과 A result status를 검증한다. GeneratedRaw 숫자/전체 contains나 요청 selected 값만으로 양성 처리하지 않는다. document-ref는 실제 active Stored draft/index/ref와 current ProposalSource li를 대조한다.
4. 수정·Save·Accept 없이 실제 source VIEW→원 새 Viewing receipt/proof/frame→실제 release→기존 bounded fresh redraw/다른 actual ticket을 기다린다.
5. fresh List로 **같은 proposal**의 현재 metadata revision을 다시 읽고 새 generation의 document/item key를 명시 선택→Open한다. 이전 callback/key/receipt 재사용0이다.
6. 두 번째 current inspect/ACK·retained entry/clean 활성·현재 header/result/editor/source/ref가 원 persisted 결과와 맞는지 확인하고 원 close/cleanup으로 끝낸다. 두 번째 VIEW 반복은 요구하지 않는다.

이 case 동안 실제 SDK mutation command는0이다. setup의 원 fixture 읽기와 여정의 read transcript를 구분한다. 여정 예상 최소는 List2+chooser inspect2+R1 inspect2이며 실제 transcript/원 response와 대조하고 결과에 맞춰 expected를 무조건 바꾸지 않는다. 현재 SDK load는 원 publication 검증 경계를 사용한다. receipt가 있다고 품질/자동 적용 성공으로 표시하지 않는 기존 quality_unknown 문구도 유지한다.

새 fixture는 처음부터 current revision2 또는3으로 열리므로 실행 중2→3 publication 갱신을 입증하지 않는다. 최신 저장본을 새로 읽어 같은 proposal을 fresh generation으로 다시 연다는 범위다. live 외부 writer에 의한 revision 경쟁/동시 저장/CAS/강한 취소는 별도 SDK 근거이며 이번 B로 확대하지 않는다.

## 정확 경로·시험명 후보 — 아직 미작성

기존2만 제안한다.

- `src/assembly/meari/viewer_native_launch/no_media/tests/end_to_end/live_retained.rs`: 고정 fixture custody spec·원 byte 복사/현재 SDK 준비·원 Recorded client/parent_run/typed startup와 cleanup·3개 exact child.
- `src/assembly/meari/viewer_native_probe/live_retained.rs`: B opt-in 기대값/단계·현재 source metadata/ref/editor/result 관측. D/기존 Live 의미와 기존 성공·단언은 보존한다.

새 source/SDK/CLI/store adapter/일반 launcher/supervision 경로 변경은 현재 제안하지 않는다. 두 경로로 표현되지 않으면 구현 전에 정확 추가 이유·경로부터 PM 검토한다.

후보 전체 이름은 아래3이다. 아래 최초 계획 이후 PM 배정으로 private source-only authored3이 되었으며 실제 등록/실행은 계속0이다. 확정 작성 명단은 runtime/development/review-feedback-persisted-reopen-r1/expected3.json이다.

- assembly::meari::viewer_native_launch::no_media::tests::end_to_end::live_retained::persisted_card_saved_result_reopens_same_proposal_with_current_feedback
- assembly::meari::viewer_native_launch::no_media::tests::end_to_end::live_retained::persisted_highlight_acceptance_reopens_same_proposal_with_output_revision
- assembly::meari::viewer_native_launch::no_media::tests::end_to_end::live_retained::persisted_document_correction_reopens_same_proposal_with_exact_ref_feedback

각각 ignored=true·실제 원 supervised window child다. A 순수9+영향3을 B child마다 다시 실행하지 않는다. 실제 source 공통 probe 변경의 직접 영향은 기존 D Card1 등 최소 대표를 별도 이유/정확 명단으로 먼저 제안하고 자동 추가하지 않는다.

## 시간·실패·반환

기존 total20초/cleanup reserve2초/run18초·NoMedia expiry15초/frame64/event80/payload4096을 유지한다. 원 immutable file custody/setup/SDK read/query/frame/cleanup 시간을 구분해 기록한다. 현재 D3 내부6.5~8.9초는 과거 측정일 뿐 B의 시간 성공 근거가 아니다. fixture 크기·marker를 이유로 예산을 임의 늘리지 않는다.

원 stdout/stderr/inner record/family identity/receipt·원 retained proof/ticket 및 원 cleanup owner를 보존한다. 잘못된 current revision/receipt/ref/editor/표시·원 SDK 오류·예산/child 실패는 첫 실패 중지다. SDK 실패를 표시 확인 성공으로 바꾸지 않는다. 강제 종료 physical Complete/Family Empty와 SDK ACK/worker cleanup 미확인은 구분한다. 정상 child0/nonforced/Empty·원 WorkersQuiescent(Closed)·프로토콜/원 raw 관측 범위가 없으면 완료로 표시하지 않는다. 자동 재시도/추가 wake polling/가짜 proof/검사 축소0이다.

실행은 A 인수 및 당시 heavy 반환·PM 별도 입장/동기화/단독 배정 이후다. 원 S/T/carry/alias-known 예외·source2 exact 복원/원 fixed BIN·증거 귀속·process/owner/원 슬롯 명시 반환은 기존 D 계약을 유지한다. 사용자 demo player는 실행/종료하지 않으며 수동 시각·OS pointer/IME 인수·일반 noargs launcher·모델 품질·사람 승인으로 합산하지 않는다.

## 스킬과 현재 상태

같은 세션에서 본문을 읽은 modootoday/software-qa/test-fixture-and-mock-build의 실제 persisted fixture provenance/격리 custody·원 I/O·실제/모델·teardown 구분을 적용했다. frontend-first-journey의 실제 source/frame/사용자 기능과 기술 입력 범위 구분을 적용했다. PM의 A14 인수 뒤 승인된 existing2/new0 private 후보를 runtime/development/review-feedback-persisted-reopen-r1에 작성했다. source2/before2/delta2/expected3/leaf2 및 HANDOFF/GATE-PLAN을 고정했다. rustfmt exit0만 관측했으며 Cargo·type/Clippy·등록·OS child 실행0, 원 SDK fixture/code/A14 정본 변경0이다. PM은 별도 heavy 입장·당시 baseline 동기화·정확 실행 명단을 배정해야 한다.
