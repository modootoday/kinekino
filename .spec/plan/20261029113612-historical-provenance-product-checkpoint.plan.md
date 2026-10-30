---
id: plan:kinekino/20261029113612
title: 제품 historical provenance 영속 연결
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 historical provenance 영속 연결

2026-10-20 PM이 [기존 접합 계획](ai-provenance-integration.md)의 구현권과 SDK single-heavy 슬롯을 배정했다. parent 등록 인수를 입력으로 삼고 기존 actual9 원문·결과만 사용한다. 추론·다운로드는 실행하지 않는다. 아래는 구현 전 설계이며 실행 근거가 아니다.

## 책임과 파일 경계

| 책임 | 변경 대상 |
| --- | --- |
| domain 불변 결합 | `src/domain/proposals/historical.rs` 신규, `stored.rs`·`mod.rs` 및 domain proposal/project 회귀 |
| application 검증·import | 기존 `src/application/ai_provenance/{mod.rs,integration_tests.rs}`, 필요하면 import/stored 하위 모듈로 분리; `application/mod.rs`, proposals 유스케이스·schema |
| 실제 evidence reader | `src/infrastructure/historical_evidence/{mod.rs,process.rs,tests.rs}` 신규 및 infrastructure 부모. 기존 Rust verifier를 명시적 어댑터로 실행하며 도구 정책을 복사하지 않음 |
| 원자 저장·재검증 | 기존 `src/infrastructure/proposals/{mod.rs,tests.rs}` 및 필요한 책임별 하위 파일. 기존 writer lock·한 revision당 한 content-addressed JSON·sync·CAS 재사용 |
| SDK/API/CLI 조립 | `src/application/api`의 capability/schema/dispatch와 facade·설정 전달, `application/launch.rs`, `src/bin/player-cli.rs` 또는 기존 CLI 책임별 하위 모듈, `src/main.rs`의 필요한 조립만 |
| project capture | 기존 `application/projects/capture.rs`, domain/projects validation 및 infrastructure/projects read 경계. live/saved capture 둘 다 증거 재검증 |
| 실제 프로세스 인수 | `tests/historical_provenance_cli.rs`와 책임별 Rust harness 하위 모듈 신규. 별도 scratch와 고정 실행 파일 사용 |

제품 presentation/UI·Meari core/native·renderer는 변경하지 않는다. StoredProposal 필드 추가로 기존 struct literal fixture의 명시 None이 필요하면 해당 UI/consumer 소유 파일의 최소 변경을 PM과 먼저 조율한다. Cargo 새 의존은 계획하지 않는다. 제품은 tools crate에 의존하지 않는다.

## 저장 호환성과 검증 경계

StoredProposal에 선택적 historical provenance를 추가하는 additive 형식을 사용한다. 일반 기존 publication은 필드 생략/None으로 읽고 다시 직렬화할 때 생략해 기존 canonical bytes와 project capture SHA를 보존한다. historical 값은 별도 version1 객체이며 companion SHA·bytes·item을 필수로 담는다. source ID/revision·원 generated raw·task는 같은 StoredProposal에 이미 있고 동일 content-addressed publication에 함께 결합된다. sidecar나 별도 create 후 쓰기는 금지한다.

새 historical publication의 모든 edit/accept revision은 binding을 불변으로 유지한다. 저장 adapter는 이전 source/generated뿐 아니라 binding 변경도 거부한다. missing/unknown version·부분 reference는 오류다. reader가 없거나 companion 원본이 없거나 pin/bytes/item/source/raw가 다르면 bound publication을 provenance 없는 정상 proposal로 강등하지 않는다. 기존 publication 전체를 다른 legacy JSON과 새 SHA로 의도적으로 다시 작성하는 공격에 대한 서명/외부 trust root는 이번 저장 checksum의 보장 범위가 아니다.

source 검토에서 단일 revision의 historical 필드만 제거하고 JSON checksum을 다시 붙이면 선택 필드만으로는 legacy와 구별할 수 없음을 확인했다. 구현 전 저장 설계를 보강한다. 새 bound proposal의 기존 document.schema_version을 `3`으로 기록하고 이 version은 historical(version1)이 필수인 publication으로 정의한다. 일반 기존 document version1/2와 source snapshot version1/2는 그대로 유지한다. binding 제거만으로 version3이 legacy로 강등되지 않으며 first/latest 결합 검사는 추가 방어다. document version까지 바꾸고 모든 checksum/이력을 재작성한 공격의 trust 한계는 별도로 유지한다. source snapshot의 schema를3으로 바꾸는 것이 아니며 response schema/capability에 새 document version을 명시한다.

기존 파일의 자동 rewrite나 일괄 migration은 하지 않는다. 기존 일반 proposal/프로젝트의 bytes/capture SHA 동일성과 새 historical capture에 binding이 포함됨을 회귀로 증명한다. 새 필드가 있는 파일을 이전 실행 파일로 읽는 것은 기존 deny_unknown_fields에 따라 거부될 수 있으므로 forward compatibility를 주장하지 않는다. API 응답 schema와 capability를 함께 갱신하고 consumer의 additive DTO 소비를 확인한다.

HistoricalEvidenceReader는 caller payload가 지정한 실행 파일/셸을 실행하지 않는다. 명시 startup 설정의 절대 verifier 경로·실행 파일 SHA·workspace root·package label을 고정한다. 기존 `player-review-sdk-export inspect-import <label> <item>`가 companion 전체·고정 package·3 source manifest·9 source/candidate를 재구성하는 정책을 재사용한다. 실행 전 executable content pin을 검사하고 제한된 출력·timeout·실패 자식 회수를 적용한다. 도구의 stdout은 어댑터 내부 데이터로만 해석하고 stderr/raw 문맥을 API 오류에 복사하지 않는다. verifier 성공과 로컬 source/raw/reference 재대조를 모두 거쳐 sample을 만든다. 이는 Rust 외부 adapter이며 모델 실행이 아니다.

### 배포 방식 비교와 선택

독립 packages library 승격은 제품·도구가 같은 Rust 정책을 직접 호출하고 subprocess 비용을 없애지만, 현재 도구 소유 source/검증/파일 IO 정책을 재분리하고 root 및 독립 도구 workspace/lock·배포를 함께 재인수해야 한다. 새 DTO 저장 변경과 그 추출을 동시에 섞는 비용이 있다. pinned binary 방식은 검증 완료한 정책을 그대로 사용하되 각 재검증의 process 비용과 별도 배포물·실패/timeout/출력 제한·자식 회수 책임이 필요하다. 이 checkpoint는 후자를 사용한다. 제품은 tools source에 직접 의존하지 않으며 개발경로 EXE를 기본값으로 검색/실행하지 않는다. verifier는 명시적으로 설치·선택한 배포물이어야 하고 없거나 pin이 다르면 지원하지 않는다. package/binary가 실제 설치됐다는 증거 없이 기본 제품 지원 완료를 주장하지 않는다. 추후 library 승격은 정책 단일 소스를 유지한 별도 추출 인수로 가능하다.

고정 검증 EXE의 SHA `e00c1726603ef7d0167c6d58deb79e694071044c20a2723de8abe8a504f45e41`를 실제 조회하고 `inspect-import sdk-text-results-r3 gemma270-correction`을 실행했다. verifier-command-first450ms/exit0, stdout3517bytes로 명령 포함을 확인했다. 이는 개발 검증 입력의 기능 확인이며 제품 reader·배포 설치 완료가 아니다. 원문과 pin/receipt는 `runtime/quality-review/historical-provenance-product-r1`에 보존했다.

`proposals.import_historical`은 명시 source/proposal identity와 companion reference를 받는다. source request와 candidate raw는 검증된 reader 결과로부터 구성하고 새 source revision을 과거 project revision/job/attempt로 표시하지 않는다. 기존 create에 미등록 필드를 몰래 허용하지 않는다. import는 원문을 수선하지 않아 NeedsRepair를 보존한다. 일반 inspect/edit/accept뿐 아니라 accept 재요청 조기 반환 전에도 bound state가 검증돼야 한다.

프로젝트 live capture는 검증된 store load를 사용한다. saved capture/reopen도 embedded bound state를 reader로 재검증하여 과거 capture가 missing/tamper companion을 우회하지 않게 한다. working draft는 별도이며 original generated/명시 acceptance와 혼합하지 않는다. evidence 검증은 품질 승인·모델 admission·render/apply 완료를 뜻하지 않는다.

## 단계와 인수

1. 부모·도메인 결합·실제 reader를 등록하고 기존 MemoryStore fixture를 컴파일한다. pin mismatch/missing/timeout/output limit/자식 회수와 실제 고정 verifier를 구분해 검사한다.
2. import·동일 JSON 원자 게시·불변 binding·load 검증을 연결한다. 최초 성공 전에 proposal/reference 둘 다 재조회 가능하게 한다. incomplete publication은 기존 recovery 오류를 유지한다.
3. API/CLI/capability/schema/startup 설정을 연결하고 새 CLI 프로세스에서 import→inspect→technical edit→stale edit 거부→accept→accept 재요청을 실행한다. raw/source/binding 불변·revision1/2/3·automatic_application_enabled=false를 보존한다.
4. project live/saved capture와 CAS→fresh reopen을 실행한다. bound reference가 capture hash에 포함되고 working draft가 별도로 유지됨을 대조한다.
5. scratch의 missing companion/source/candidate·동일크기 변조·다른 유효 item/reference 교환·부분 publication을 inspect/reopen·accept 재요청·project 경계에서 거부하고 실패 전후 publication 불변을 검사한다. 실제9의 task별 schema mismatch는 수리나 품질 승인으로 바꾸지 않는다.
6. root fmt/test/strict와 영향을 받은 consumer 회귀를 단일 heavy로 수행한다. root C/native·consumer D 전용 target을 유지한다. 첫 실패 원문·고정 source/bin pin·stdout/stderr/exit·소유 프로세스·disk를 보존한 뒤 인수 정본과 PM 반환을 기록한다.

## 단계1 진행 기록

application 부모 등록 후 module-focused-first62006ms/exit0으로 준비 모듈3시험을 처음 컴파일·실행했다. MemoryStore/fixture reader의 한계는 그대로다. 첫 fmt check는 부모의 모듈 정렬만 지적했고 수정했다.

실제 process reader 첫 all-target check는 ApiError Clone 부재 E0277로 실패했다. 첫 source/원문을 보존하고 오류 코드 전달로 고친 reader-check-r230224ms/exit0을 확인했다. reader-focused-first50569ms/exit0은 root3pass·1ignored·290filtered와 별도 jobs historical-list 회귀1pass이며 actual9 ignored 시험을 실행한 결과가 아니다.

그 사이 `cargo fmt --all`은 작성 중인 제외 Meari native의 window_lifecycle/coordinator.rs 미존재로 source 해석에 실패했다. 외부 source를 고치지 않았으며 PM 지시에 따라 이후 제품 workspace 범위의 `cargo fmt`/`cargo fmt --check`를 사용한다. 이 실패를 제품 컴파일 실패나 Meari 기능 판정으로 확대하지 않는다.

PM의 process 검토를 반영해 stdout/stderr 양쪽 초과를 동일 event 채널로 감지하고, 정리 실패 시 child/pipe reader를 Runner owner에 남기며 후속 요청은 정리만 수행하게 보강했다. Timeout은 정리 완료 영수증이 아니다. 현재 고정 verifier source가 자식을 생성하지 않는 범위만 소유하며 일반 임의 프로그램의 descendant tree 정리를 보장하지 않는다. Drop은 best effort다. 실제 자식 테스트와 kill/wait 실패 주입의 차이를 기록한다. 이 새 소스의 focused fault 회귀를 인수 중이다.

reader는 동기 process를 사용하므로 CLI-first로 연결한다. 현재 Immediate UI-thread adapter에서 무차단 동작을 보장하지 않는다. GUI 적용에는 application worker scheduling·typed Pending·terminal completion/wake를 별도로 연결해야 하며 Rc registry/GL을 worker로 보내지 않는다.

위 reader 검사 입력에는 persisted binding/API import/project capture가 아직 없었으며 실제 CLI 영속 성공을 뜻하지 않는다. 승인된3fixture의 historical None 외 UI 동작은 변경하지 않는다.

이후 reader-owner-focused-first47205ms/exit0은 실제4pass·2ignored이며 stdout/stderr 초과·timeout·nonzero의 실제 소유 자식, kill/wait 실패 주입 후 owner 보존·중복 실행 없는 명시 재정리, pipe read 실패를 확인했다. reader-actual-nine-first4069ms/exit0은 정확 ignored 시험1pass(내부9개 item의 실제 verifier 재구성·sample/source/raw 결합·swapped SHA 거부)·본문2.88s다. reader-strict-first37364ms/exit0도 통과했다. 이 결과는 다음 저장 변경 전 source이며 새 binding/CLI source에 소급하지 않는다.

PM에 heavy만 임시 반환한 뒤 domain historical binding·StoredProposal 생략 호환·같은 publication의 원자 저장과 불변 비교, application import·typed client·API schema/capability·GUI/CLI의 공통 명시 설정 조립을 source로 연결했다. 프로젝트의 기존 inspect 서명은 유지하고 reader를 받는 공통 경로에 위임하며 bound state를 무검증 반환하지 않는다. 승인3fixture에는 historical None만 추가했고 각 diff를 별도 보존했다. reader pending cleanup은 executable pin 검사보다 먼저 재시도하여 pin 변경이 기존 자식 회수를 막지 않게 했다.

Rust 실제 CLI harness는 새 전용 evidence/store와 복사 workspace·고정 CLI/검증 EXE를 사용하도록 작성했다. 9개의 명시 import/fresh PID inspect, technical edit/stale/accept 재요청, project live/saved/CAS/reopen, missing/same-size tamper·원 revision과 binding 전체 누락·단일 revision의 다른 유효 item 교환·부분 publication 거부 및 원본 pin/게시 불변을 준비했다. 현재 이 source는 미컴파일·미실행이며 새 영속 성공은 아직 없다. Meari 단일 슬롯 반환 후 root fmt/check부터 인수한다. 신규 추론·다운로드는0이다.

PM이 슬롯을 재배정한 뒤 제품 fmt/check 첫 입력이 통과했다. 그 terminal 뒤 task별 correction/cards/highlights의 독립 technical 수정·명시 채택·project capture 경로와 domain validation 선행을 보완했다. 이후 실제 결과와 첫 실패·입력 pin은 [실행 정본](../../sots/verification/application/historical-provenance-product-2026-10-20.md)에서 구분한다.
