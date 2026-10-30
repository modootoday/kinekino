---
id: plan:kinekino/20261029113749
title: 모델 실행 없는 provenance 접합 후속
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 모델 실행 없는 provenance 접합 후속

## 최신 검사 경계

2026-10-19 독립 tool gate는 실제 실행을 통과했다. fmt, 일반12시험, strictClippy, exportbin build, 고정CLI verify/import, missing-item exit1 및 Rust actual9 scratch 부정인수1시험이 근거다. source/candidate 각 누락·동일크기변조와 companion 승격·누락을 거부하고 원문/원본 불변을 확인했다. [명령·실행 상태](historical-import-tool-gate.md), [원로그](../../evidence/verification/historical-import-2026-10-19-r1/)를 참조한다. 아래 source 준비 이력 중 tool 미실행 상태는 이 결과로 갱신한다.

2026-10-20 제품 application 부모·실제 pinned Rust reader·단일 publication의 historical binding/document3을 연결했다. 실제 CLI9import/fresh inspect와 correction/cards/highlights 각각 기술 편집·명시 채택/재요청·project capture/reopen 및 scratch 부정 입력 gate가 첫 실행을 통과했다. root396pass/22ignored·consumer47pass와 양쪽 strict/fmt·최종 pins/소유 정리는 [제품 실행 정본](../../sots/verification/application/historical-provenance-product-2026-10-20.md)을 따른다. 아래 미등록·미컴파일 표현은 준비 당시 이력이다. 새 job/attempt는 발명하지 않으며 신규 추론/download·모델 admission·사람 품질 승인·render/apply·무차단 GUI 통합을 완료로 승격하지 않는다.

회차 목록·episode/media resolve·공통 native CLI host 순서는 유지한다. 이 계획은 기존 생성 결과를 재사용하는 별도 접합 gate이며 새 모델 추론/다운로드를 허용하지 않는다. 현행 [jobs 접합 감사](../../sots/player/infrastructure/inference/research-followups/ai-executor-current-jobs-integration.md)를 기준으로 동일 JobExecutor/ExecutionControl/게시 CAS·owner/recovery를 재사용한다.

기존 출력의 형식·오류 의미는 [r3 import 현행 접합 감사](../../sots/player/infrastructure/inference/research-followups/text-r3-import-error-contract.md), 저장 provenance 인수6항목은 [model engine CLI 인계](../../sots/player/infrastructure/inference/research-followups/model-engine-cli-contract-handoff.md)를 정본으로 연결한다. 불변 source/candidate SHA→model/engine/precision manifest→create/fresh inspect→edit→accept CAS→restart/project capture를 구별한다. historical job/attempt 부재는 unknown/absent로 유지하고 새 ID·원 project revision을 발명하지 않는다. 현재 이 통합 검증은 미실행이며 추론0이다.

1. GeneratedResult(task/raw_text)와 VerifiedJobOutput(path/SHA/bytes/receipt) 사이에 source/model/engine/variant/attempt/request 및 원문 SHA를 결합하는 명시 application adapter 계약을 둔다. 원 생성 provenance가 없는 과거 결과를 새 job provenance로 위조하지 않는다.
2. 기존 원문을 그대로 유지한 typed 변환을 검증한다. correction의 proposed/uncertain_words와 DocumentCorrection(source_ref,text), 카드 headline/summary/cue_ids와 cards(front/back/source_refs), highlight ranges(cue_ids/reason)와 highlights(title/source_refs)는 의미검증이 필요한 서로 다른 계약이다. 단순 필드 이름 변경으로 성공을 만들지 않는다.
3. 제안 생성→기술 편집→명시 채택 receipt→project binding을 재현 가능한 실제 CLI gate로 연결한다. 기존 actual9 기술 fixture를 사용자 품질 승인으로 세지 않으며 사용자 결정0/rendered=false 상태를 보존한다.
4. accepted refs를 실제 media PTS/자막 파일 출력/카드 렌더/하이라이트 skip에 적용하는 후속 export/apply gate를 분리한다. accept receipt 자체는 위 결과물 제작이나 재생 적용의 증거가 아니다.

남은 근거는 기존 결과와 원문 SHA를 사용한 offline 재현, 불일치/누락 provenance 거부, project revision CAS, 명시 채택 전 자동 적용 금지, 출력 게시 후 실제 파일·시간 범위 대조다. source/model license·정밀도·HF revision 및 실제 엔진 admission은 별도 계약을 유지하며 기존 모델 품질 판단을 소급 승인하지 않는다.

## 2026-10-19 source 후속 착수

PM 즉시진행 지시 후 SDK와 신규 application/ai_provenance 소유 경계를 합의했다. AI 담당은 HistoricalEvidenceReader port·CompanionReference·검증 후 private PreparedHistoricalProposal 생성과 verify_stored를 작성했다. source/task/raw SHA·bytes 및 companion/item binding을 확인하고 draft/accepted 출력 대신 original generated를 재검증한다. DTO/GeneratedResult/root Cargo/assembly는 변경하지 않았다. 원자적인 proposal+reference 저장이 연결되지 않은 상태에서 2단계 쓰기나 제품 명령 지원을 주장하지 않는다. 등록·실제 create/inspect/edit/accept/restart/project 검증은 SDK checkpoint 뒤 조율한다.

도구의 read_verified를 분리하고 inspect-import label item-id를 작성해 검증된 companion identity·선택 evidence·source request/candidate raw UTF8를 함께 반환한다. 선택 후 다시 읽은 bytes도 재hash하며 원문을 수선하지 않는다. 새 회귀 소스 1개씩(도구/제품 경계)을 작성했다. fmtcheck만 통과했고 native test/strict Clippy/build는 중앙 슬롯 요청 상태다. 기존 고정 EXE는 아직 이 새 명령을 포함하지 않는다. modoo.today 기존 3스킬의 조건 고정/선언·관측 구분/원문 보존 원칙을 적용했으며 정확한 출처는 기존 protocol review를 따른다.

get_goal 상태는 blocked이며 update_goal에 active/resume 인자가 없다. 미완료 goal을 완료로 바꿔 교체하지 않고 실제 source 진행과 API 상태 제한을 구별한다. 신규 추론/download는 계속0이다.

제품 source fixture 보강: 신규 application 모듈의 integration_tests.rs는 고정 actual Gemma source/candidate를 include하여 공통 proposals::create/edit/accept와 직렬화 readback을 호출하도록 작성했다. 최초 NeedsRepair, 기술 edit, stale edit 거부, acceptance automatic_application_enabled=false, original raw 불변, missing companion·item mismatch·raw tamper 거부를 검증한다. Reader는 fixture이고 저장소는 메모리이므로 실제 provenance 재구성·디스크 원자 저장·fresh CLI 실행·project capture의 근거가 아니다. 신규 module은 SDK 등록 대기라 아직 컴파일/시험하지 않았고 fmtcheck만 통과했다. prepare/verify_stored는 source request도 기존 source_import로 재구성하여 caller가 임의 cue/text를 같은 SHA에 연결하지 못하도록 작성했다. 최초 import의 새 source revision은 과거 project revision으로 기록하지 않는다.

제품 인계 점검 보완: prepare/verify_stored가 reader 호출 전에 companion SHA·bytes·item ID를 공통 domain identifiers 규칙으로 검증하도록 수정했다. 중복 SHA 정책을 제거하고 source request size도 재구성 전에 제한한다. malformed reference에서 reader가 호출되면 실패하는 fixture와 상한/식별자 회귀를 작성했다. fmtcheck0이며 제품 root 등록·compile/test/strictClippy는 별도 SDK gate 대기다. 독립 tool의 검증된 소스/EXE는 변경하지 않았다.

## 제품 atomic 저장·reopen 인계 계약

이번 대조는 [MemoryStore fixture](../../../src/application/ai_provenance/integration_tests.rs), [application port](../../../src/application/ai_provenance/mod.rs), [현재 ProposalFiles](../../../src/infrastructure/proposals/mod.rs), [StoredProposal](../../../src/domain/proposals/stored.rs), [proposal 유스케이스](../../../src/application/proposals/mod.rs), [project capture 검증](../../../src/domain/projects/validation.rs)을 읽은 source 감사다. 새 시험/저장 구현/빌드를 수행한 근거가 아니다. 제품 모듈은 아직 미등록·미컴파일이다.

| 단계/불변식 | 현재 fixture 또는 제품 source | 등록 후 실제 CLI 인수에 필요한 근거 |
|---|---|---|
| 생성 identity | prepare는 task/source request 재구성/raw SHA·bytes/reference를 검사한다. fixture Reader는 고정 sample을 반환한다. | 실제 reader가 companion 전체와 pinned source manifests/objects를 재구성한 뒤 item/task/source/candidate/model 조건의 동일성을 확인한다. caller envelope나 경로 존재만으로 verified 값을 만들지 않는다. |
| 원자 게시 | PreparedHistoricalProposal은 input/reference를 반환하지만 fixture는 input만 ProposalStore에 게시하고 reference를 로컬 변수로 유지한다. | proposal ID·import source ID/revision·original raw·companion SHA/bytes/item을 같은 불변 publication 또는 동일 commit record에 결합한다. 성공 응답 전에 둘 다 재조회 가능해야 한다. prepare 결과를 분해해 reference를 버린 일반 create를 historical 지원으로 표시하지 않는다. |
| 기존 저장 규칙 | ProposalFiles는 writer lock·연속 revision·revision당 SHA 파일 하나·bounded read·sync를 사용한다. 부분 publication은 recovery 오류로 막으며 이전 source/generated 변경을 거부한다. | SDK가 envelope/schema/commit 경계를 결정한다. 기존 revision 폴더에 sidecar를 추가하면 파일 한 개 규칙을 깨므로 임의 추가하지 않는다. proposal 생성 후 별도 reference 쓰기를 성공으로 취급하지 않는다. 실패/충돌/불완전 게시 후 원본을 보존하고 기존 recovery 정책을 따른다. 전원손실 durability는 별도 근거 없이 주장하지 않는다. |
| inspect/reopen | fixture의 serde 왕복은 StoredProposal만 저장하고 별도 bound 변수를 다시 넘긴다. MemoryStore.load는 단일 slot이며 인자로 받은 proposal ID를 조회 키로 사용하지 않는다. | 새 CLI 프로세스가 persisted binding을 proposal ID와 저장 revision으로 찾아야 한다. 이전 프로세스의 ref 변수/메모리/cache에 의존하지 않는다. 다른 proposal의 companion item 교환·reference 누락·bytes/SHA 불일치가 읽기 오류이고 provenance 없는 정상 성공으로 조용히 강등되지 않아야 한다. |
| edit/CAS | fixture는 edit revision1→2와 stale edit 거부, 원문 재검증을 작성했다. 현재 ProposalFiles는 source/generated 불변을 강제한다. | binding도 동일하게 불변이며 edit는 draft만 변경한다. stale edit·다른 source revision·다른 item 재연결은 저장 revision을 늘리지 않아야 한다. 전후 원문/source/binding SHA와 기대 CAS 오류를 실제 CLI에서 보존한다. |
| accept/재요청 | 현재 accept는 기존 receipt가 expected draft/source revision과 맞으면 조기 반환한다. fixture는 accept 뒤 수동 verify_stored를 호출한다. | provenance 검증은 성공 응답 전 필수 경계다. 최초 accept뿐 아니라 기존 receipt 재반환 경로도 누락/변조 companion을 우회하지 않아야 한다. draft2/output3 및 automatic_application_enabled=false를 보존한다. 게시 후 검증 실패로 이미 변경된 상태를 정상 성공으로 포장하지 않는다. |
| restart/project capture | 프로젝트는 captured StoredProposal의 canonical hash와 working draft 분리를 검사한다. 현재 StoredProposal에는 historical reference가 없다. | SDK가 capture가 결합할 reference/commit identity를 명시하고 project CAS 저장·fresh process 재조회에서 같은 연결을 확인한다. capture_sha가 reference를 포함하지 않는 현재 형식의 해시만으로 provenance 보존을 주장하지 않는다. working draft를 original generated로 대체하지 않는다. |

### SDK 등록 뒤 바로 수행할 순서

1. 새 전용 proposal/project 저장 root와 기존 actual9 source/candidate/companion pins를 고정한다. 제품 provenance command/payload·binding 조회 위치·오류 계약은 SDK가 먼저 확정한다. 현재 `proposals.create`의 deny_unknown_fields payload에 미등록 provenance 필드를 삽입하지 않는다.
2. 실제 제품 import/create 후 정상 종료하고 새 CLI inspect에서 persisted binding·source/raw SHA·bytes·task·companion item을 읽는다. 과거 원문은 NeedsRepair를 유지한다. 로컬 import source revision1을 과거 project revision으로 승격하지 않는다.
3. 같은 root에서 명시 technical edit→stale edit 거부→accept→accept 재요청을 수행한다. 각 응답과 새 프로세스 readback의 원문/source/binding 불변 및 revision2/3·자동 적용 false를 확인한다. 이 technical edit는 사람 품질 verdict가 아니다.
4. SDK 계약에 맞춰 project capture/CAS 저장 후 새 프로세스에서 capture/binding을 읽고, 별도 working draft가 원 생성/acceptance를 덮지 않았는지 대조한다.
5. Rust 인수 harness가 별도 저장 root/scratch에서 missing companion·same-size corruption·다른 유효 item/다른 proposal의 reference 교환·부분 publication을 구성한다. inspect/reopen과 accept 재요청 모두 해당 오류를 보존하고 추가 publication이 없어야 한다. shell에 시나리오 정책을 구현하지 않는다. 기존 정상 root를 훼손하거나 자동 복구/재생성하지 않는다.
6. stdout/stderr/exit·원 저장파일 identity·전후 immutable 목록·fixed CLI SHA·resource-runner receipt를 보존한다. 도구 gate 통과와 제품 gate 통과를 별도 기록한다. 모든 실행은 추론/download 없이 기존 결과로 수행한다.

SDK가 아직 결정/구현할 항목은 module 등록, 실제 HistoricalEvidenceReader, persisted binding 형식과 atomic writer/readback, 기존 revision/schema 호환 또는 명시 migration, 성공/오류 wire 계약, project capture 결합 방식이다. AI는 이 계약을 대신 저장 구현하거나 기존 DTO에 임의 필드를 추가하지 않았다. 별도 모델 admission·라이선스·실제 precision·사람 품질·apply/render 잔여는 유지한다.
