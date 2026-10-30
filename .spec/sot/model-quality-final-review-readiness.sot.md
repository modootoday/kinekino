---
id: sot:kinekino/model-quality-final-review-readiness
title: 제품 연결 뒤 마지막 모델 검토를 위한 준비 상태
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 제품 연결 뒤 마지막 모델 검토를 위한 준비 상태

2026-10-19 연구 세션 소유 준비 문서다. 사용자 결정에 따라 **자막 교정·카드 문구·하이라이트의 원문 비교→사람 수정→명시적 채택 제안 기능을 먼저 연결하고, 모델 검토는 제품 연결·통합 검증 뒤 마지막 단계**에 둔다. 이는 현재 결과의 모델 품질 승인이나 자동 적용 승인이 아니다. 지금 사용자에게 평가·승인을 요구하지 않는다.

읽은 기준은 [application 검토 계약](../../../application/model-quality-review.md), [내용 감사](../../../../verification/ai/text-review-content-audit.md), [SDK 실제 패키지](../../../../verification/ai/text-review-sdk-actual.md), [23역할 상태](../../../../verification/ai/model-status.md)와 r3 package다. 이번 작업은 기존 감사의 결함을 재정리하고 package의 item/criteria/assets를 확인한 것이며, 모든 원시 출력을 새로 독립 평가하거나 판정을 저장한 것은 아니다. Pages·새 추론·다운로드·제품/공통 정본 수정은 하지 않았다. 후속 추론·취득은 root 한 worker 정책을 유지한다.

## 2026-10-20 제품 사람 검토 인수표

이 절은 현재 source의 계약 대조이며 새 CLI 실행·사람 판정 저장 근거가 아니다. 아래의 기존 2026-10-19 화면/실행본 기록은 과거 범위다. 현재 실행본·저장소·UI 인수로 소급하지 않는다. 기능 인수는 공통 Rust SDK/CLI를 먼저 검증하며 GUI는 표시·입력·접근성·결과/오류 정합성만 검토한다. 제품 historical 연결의 실제 범위는 [실행 정본](../../../../verification/application/historical-provenance-product-2026-10-20.md)을 따른다.

| 대상·정확 식별 | 기존 SDK/CLI 포트와 가능 범위 | 증거·화면 표현 | 사람이 확인·승인하는 대상 / gap |
|---|---|---|---|
| 원 review package/item과 candidate | `inspect_review` / `reviews.inspect {package_sha256}`; `read_review_text_asset` / `reviews.read_text_asset {package_sha256, asset_sha256}`. `package.items[item_id].source/baseline/candidate.assets/criteria/known_issues` 조회 가능 | package SHA + item ID + 각 asset SHA/bytes로 원문과 candidate를 구분. `evidence_verified`는 해당 package/assets 무결성, `review_ready`는 baseline 존재이며 baseline은 human gold가 아님 | 원 candidate의 해당 task·조건·기준에 대한 의미 품질. 잘림·근거 불일치 등 알려진 실패를 먼저 표시. 원 bytes를 수정 draft로 덮지 않음 |
| 저장된 historical reference | `inspect_proposal` / `proposals.inspect {proposal_id}` 응답의 `StoredProposal.historical.reference {sha256,bytes,item_id}`. UI `historical_reference`는 이 참조만 투영 | SHA는 **companion** 식별이며 review package SHA가 아님. 현재 UI는 “저장된 참조”와 검증/현재 작업/정밀도/라이선스/품질 미확인을 표시 | 저장 참조 표시 자체는 사람이 승인하는 품질 대상이 아님. `historical.is_some()`·구조 validate만으로 증거 검증 완료를 표시하지 않음 |
| canonical inspect의 bound evidence 검증 관측 | `proposals.import_historical {proposal_id,source_id,source_revision,reference}` 및 bound `proposals.inspect/edit/accept`에서 pinned reader 검증. 성공 inspect는 그 요청 시점의 source/raw/binding 검증 관측 | request ID·명령·성공/오류와 반환 proposal/revision을 연결해야 함. 현재 `StoredProposal`에는 독립 verify receipt/시점/범위 필드가 없고 UI projection도 이를 생산하지 않음 | canonical 성공 응답의 관측과 오프라인 저장 참조를 구분. 관측 성공을 license 승인·runtime dtype·current job/attempt/output 결합으로 확대하지 않음 |
| 사용자 작업 중 draft와 저장 draft | `edit_proposal` / `proposals.edit {proposal_id,draft}` + `context.expected_revision`; `inspect_proposal`로 `document.draft_revision/draft/state` 재조회 가능. working draft는 아직 저장되지 않은 입력 | 원 `source`, `generated.raw_text`, 저장 draft, 미저장 working draft를 나눔. stale CAS는 재조회 후 사용자가 다시 판단. NeedsRepair와 import_error를 숨기지 않음 | 사람이 source↔candidate↔수정 draft의 의미·근거·누락을 확인. edit 성공은 typed 저장/구조 유효성이고 의미 품질 verdict가 아님 |
| 명시 technical acceptance와 편집 후 output | `accept_proposal` / `proposals.accept {proposal_id,target_source_revision}` + `context.expected_revision`; `AcceptanceReceipt {proposal_id,draft_revision,source_id,source_revision,source_sha256,output_revision,output,automatic_application_enabled}` 재조회 가능 | “제안 채택”과 선택한 draft/output revision 표시. receipt 재반환도 bound 검증을 선행. 자동 적용 false 유지 | 해당 draft를 technical output으로 채택하는 행위. receipt는 quality decision이 아니며 실제 자막/카드/미디어 적용 완료도 아님. receipt에는 output asset SHA나 decision ID가 없음 |
| 원 candidate의 명시 품질 판정 | `record_review_decision` / `reviews.record_decision {package_sha256,item_id,decision_id,verdict,note,issue_ranges}`; `review_decisions` / `reviews.decisions {package_sha256,item_id}`로 `StoredDecision {decision_sha256,decision}` 읽기 가능 | `usable/needs_changes/deferred`, 필수 note, `scope=sample_only`, candidate condition 표시. package SHA가 candidate assets와 criteria를 간접 고정 | **그 package/item의 candidate**에 대한 판정만 승인. `sample_review_passed = baseline 존재 && candidate/baseline 자동검사 모두 Passed && verdict=usable`; usable 단독과도 구별. 실제9 range=null이므로 가짜 issue 시간 대신 note에 cue/문장 위치를 기록 |
| 편집 후 output의 별도 quality decision | 현재 `RecordReviewDecision`에는 proposal ID/draft revision/output revision/output SHA/acceptance 식별 입력이 없음. 기존 포트로 **임의 편집 output을 직접 판정하는 결합은 누락** | “편집 결과 품질 미판정”을 유지. 원 candidate decision을 화면 가까이에 표시하더라도 대상이 원 candidate임을 명시. `quality_decision_known`을 원 decision 존재만으로 true로 바꾸지 않음 | 새 불변 output을 candidate로 포함한 별도 package를 정식 준비하면 기존 review 포트의 새 대상이 될 수 있으나, 현재 proposal→derived package 생성/계보 결합 포트와 실제 인수는 없음. 자동 생성·품질 승계로 가정하지 않음 |

CLI는 `player-cli --input FILE|- --json`의 공통 Request(`command`, `payload`, `context`)를 사용한다. review 명령은 명시 절대 `--review-root`, proposal은 `--proposal-root`가 필요하며 historical reader 설정은 [API 계약](../../../application/api/historical-provenance.md)을 따른다. `reviews.*`에는 expected_revision/idempotency_key/timeout_ms를 붙이지 않는다. proposal edit/accept는 expected_revision이 필수이며 import/inspect에는 붙이지 않는다. 이 절은 명령·스키마 매핑이며 새 명령 실행이나 현재 설치 성공을 주장하지 않는다.

### 정확 인수 기준과 남은 연결

1. 원 candidate 판정 저장 전 동일 package/item의 source·candidate assets·criteria·known_issues를 읽는다. 식별 오류/자산 누락/잘린 JSON을 정상 출력으로 가리지 않고 source는 독립 정답이 아님을 표시한다. 판정 저장 뒤 별도 `reviews.decisions` read-back에서 decision ID/SHA·대상 package/item·verdict/note 일치를 확인한다. 테스트용 deferred를 실제 사용자 판정으로 재사용하지 않는다.
2. 수정·채택은 조회한 proposal/draft/source revision에만 수행한다. 새 조회에서 원 generated/source/historical 불변과 정확 acceptance.output/revision을 확인한다. 원 candidate의 usable/needs_changes를 편집 후 output에 자동 전이하지 않는다. 편집 후 output 판정의 대상 식별·불변 bytes·기준·명시 decision/read-back 연결이 없는 동안 품질 승인 완료로 표시하지 않는다.
3. bound inspect 성공의 evidence 검증은 그 요청의 canonical 결과와 결합된 관측으로만 설명한다. 저장 참조의 존재, package evidence_verified, technical acceptance를 서로 대체하지 않는다. 현재 UI storedref 표시의 별도 source 수리는 이 관측 receipt gap을 해소한 구현이 아니다. UI 담당도 이 구분과 두 bool의 false 유지를 확인했다.
4. license acceptance·HF revision·실제 runtime weight/KV dtype·current job/attempt/output은 각각 정본 관측 필드가 있어야 한다. historical 누락은 계속 absent이며 `ModelCondition.precision` 선언·검증 성공으로 채우지 않는다. verifier 설치/배포와 프로젝트 자체 license 선택은 별도 잔여이고 로컬 검토 포트의 존재와 구별한다.
5. correction의 허용 편집/의미 보존, cards의 문장별 supporting cue/반복·길이, highlights의 선정 목적/근거를 작업별로 먼저 고정한다. 잘림·문장 누락·근거 불일치는 개발 결함으로 선표시한다. 한 표본·한 task의 결과를 전체 모델/23역할/자동 적용 승인으로 확대하지 않는다. 실제 사람 검토는 제품 연결 준비 뒤 별도 단계다.

현재 `ReviewDecision`/`AcceptanceReceipt`에는 명시 reviewer identity·행위 시각 필드가 없다. decision ID·필수 note와 불변 기록만으로 “누가 언제 승인”까지 검증됐다고 하지 않는다. 이 인수표의 사람 행위는 최종 검토 시 실제 행위/저장/read-back으로 증명해야 하며, 추가 감사 필드는 SDK 후속 계약 판단 사항이다.

미구현 후속은 [편집 output 품질 판정 계획](../../../../../plans/player/edited-output-quality-decision.md)으로 분리한다. target/hash/lineage·stale/tamper·legacy 호환과 선언된 검토자/로컬 기록 시각만 설계하며 인증된 신원 증명을 새 필수 흐름으로 요구하지 않는다.

근거 source: `src/application/api/client.rs`, `src/application/model_review/mod.rs`, `src/domain/model_review/{package,decision}.rs`, `src/application/proposals/mod.rs`, `src/application/ai_provenance/mod.rs`, `src/domain/proposals/{mod,historical}.rs`, `src/presentation/web_ui_adapter/proposals/projection.rs`, `src/bin/player-cli/runner.rs`. 제품 코드·DTO는 각 소유자가 유지하며 이번 변경은 이 준비 정본만 보완한다.

적용 스킬: modoo.today `modootoday-free/ai-quality-plus/llm-output-eval-suite`를 검색 설명과 본문으로 확인했다(본문에 추가 참조 없음). 첫 검색의 공개 라벨링·규제 문서·이미지 납품 스킬은 이 책임과 달라 채택하지 않았다. 선택 스킬의 case별 기대/금지 기준, task별 slice와 알려진 실패를 적용해 원 candidate·편집 output의 판정 단위를 고정했다. 새20~50표본 수집·반복 추론·LLM judge·CI 실행은 중앙 보류 범위로 실행하지 않았다.

## 현재 수치와 준비도

| 구분 | 근거 있는 상태 |
|---|---|
| 실제 모델 실행·출력 확인 | 23개 역할/모델 묶음. 정밀도·반복·참조 엔진·후속 진단을 새 모델 수로 더하지 않음 |
| 제품 모델 품질/자동 적용 승인 | 0개. 사용자 미응답 때문이 아니라 개발 검증·패키지·기능별 품질 게이트 미완료 |
| 이번 텍스트 package | Gemma270/Llama1B/Qwen0.6 × correction/cards/highlights = 실제9표본 |
| r3 package |51138B/SHA256 `659b3cb6757259d8e770f6d3c96c4a77dafbab43c6ea540bcc7e0c58e4eb0197`|
| SDK 검토 준비 |inspect 실제9개·evidence_verified/review_ready=true, automatic_application_enabled=false. 사람 의미 판정 통과와 다름 |
| text asset 실제 전달 |Gemma 잘린 JSON516B/Qwen 한글261B/원문JSON2129B의3자산 확인. 모든9개 GUI 표시 통과와 다름 |
| native MODEL |계약에 기록된 r153/r154의 일부 표본·복제 저장소 저장/재시작/760폭 검증 범위. 현재 root의 진행 중 구현을 최종 완료로 소급 표시하지 않음 |
| 본 세션 수행 |문서 준비·SOT/package 읽기. 모델 실행0/사용자 판정 저장0/제품 변경0 |

읽기 전용 브라우저 화면은 판정 저장 화면이 아니며 실행 중 서버도 전제다. 최종 사용자 안내에는 root가 실제 통합 검증한 실행본·저장소·진입 방법만 제공한다. 과거 포트/EXE/테스트 저장소를 현재 실행 중인 것으로 안내하지 않는다. 품질 검토를 제품 연결의 차단 조건으로 삼지 않는다.

## 9표본의 결함·재제시 조건

아래는 기존 개발 내용 감사다. 사용자 verdict나 모델 전반 순위가 아니다. “수정 전 재승인 불가”는 **현재 실패를 그대로 사용 가능으로 제시하지 않는다**는 뜻이며 과거 모델 품질 승인이 있었다는 뜻이 아니다. 모든 historical candidate bytes는 유지한다.

| item_id | 알려진 실제 관측 | 수정 전 제시·판정 한계 | 다시 검토할 때 필요한 근거 |
|---|---|---|---|
|gemma270-correction|proposed `thir`, uncertain_words 조각 반복, JSON 미완결, 원문6문장 미보존|교정 결과로 사용 가능/자동 적용 제시 불가. 실패 예시·수동 수정 출발점으로는 명확히 표시 가능|완결 출력,6문장 보존 diff,반복/의미누락0 검사,새 revision 및 동일입력 대조|
|llama1b-correction|첫 문장만 proposed,나머지가 uncertain_words 반복 후 잘림|동일하게 교정 사용 가능 제시 불가. 미완성 JSON을 조용히 복구해 모델 성공으로 표시 금지|원문 보존·완결성·반복 검사와 수정본의 별도 출처|
|qwen06-correction|6문장 결합, turk→Turk,나머지 의심 ASR 유지,uncertain_words 빈 배열|ASR 오인식 교정·불확실성 검출 성공으로 승인 불가. punctuation-only 대소문자 정책 미정|허용 편집 범위 명시·word diff·원음/사람 정답이 필요한 오류는 미검증 표시|
|gemma270-cards|headline cue1/summary cue2 복사,근거는1~7 전체|새 시각 사건 환각으로 단정하지 않음. 넓은 근거를 정확한 최소근거라고 승인 불가|문장별 실제 supporting cue,복사/요약 목적 기준,가독성·근거 최소성|
|llama1b-cards|본문은cue1/4인데근거6/7|참조 ID 존재 통과와 별개로 근거 연결 실패. 수정 전 해당 연결 사용 가능 제시 불가|본문-근거 의미 대응 수정 및 각문장↔cue 대조|
|qwen06-cards|`바보로 다니다` 부자연스러움,headline/summary 반복,한문장 조건 위반|수정 전 요청 제약 준수·문구 편집 완료로 승인 불가. 새로운 사건 환각으로 일괄 단정 금지|자연스러운 문구·중복 제거·한문장/길이 검사·의미 보존 diff|
|gemma270-highlights|cue1~7 전체,원문 없는 가족 이야기,혼합 문자 깨짐|근거·가독성 실패. 해당 이유를 수정 전 사용 가능 제시 불가|원문 근거 있는 이유·언어/문장 완결성;선별성 기준은 별도|
|llama1b-highlights|cue3~6과cue7의두범위,flow/natural pause 설명|범위 수 조건 통과. 전부선택 모델로 오기 금지. 음성 쉼 검증은 불가|실제 음성 연결 없으면 pause 주장을 미검증/텍스트 근거 설명으로 구분;편집 목적·선정 기준|
|qwen06-highlights|cue1~7 전체 한범위,이유는원문구절 조합|근거 문구 존재와흥미도는별개. 미정의기준으로전체선택만자동실패 금지|사용자가쓸편집목적/독립선정기준·범위길이/중복/선별성 기준|

강한 결함(잘림·문장 누락·본문-근거 불일치·원문 없는 주장)은 개발 담당이 먼저 표시·수정한다. 기준 미정 항목(대소문자·전체범위 선택 허용·흥미도·추출형/요약형 목적)은 root가 제안 기능의 작업 조건으로 명시한 뒤 최종 검토 자료에 넣는다. 현 품질불량을 사용자가 발견·진단하도록 넘기지 않는다.

## 최종 제시 패키지의 원문·결과·기준·판단

9개 모두 range=null이다. cue 안의 시각 문자열을 검증된 원본 영상 범위로 승격하지 않는다. 현재 텍스트만으로 음성 쉼·장면 선택·자막 동기화 정확도를 판단할 수 없다. source는 실제 입력/ASR 초안이며 독립 사람 gold가 아니다.

| 패널/항목 | 실제로 제공할 데이터 | 판단/보존 조건 |
|---|---|---|
|원문|고정 source asset 원문,task input조건,문장/cue ID,입력SHA;baseline reference/source-text 표시|기준선이 인간 정답이 아님을 명시. 원문 덮어쓰기 금지|
|모델 결과|고정candidate의바이트 그대로;잘린JSON도text/plain 실패상태로표시;model/revision/precision/engine/request조건|수정한텍스트를모델원본인것처럼덮지않음|
|변경 비교|원문↔candidate,원문↔사람수정본,필요시candidate↔수정본의추가·삭제·대소문자·문장/cue 차이|“단순 punctuation”와단어/의미변경 구분. 자동검사는의미정답대체가아님|
|근거|카드각문장/하이라이트이유↔실제source cue;ID존재와의미대응을별도표시|근거없는문구는개발결함으로명시;timestamp/media unavailable을숨기지않음|
|기준|package criterion_id grounded-content와작업별체크리스트·새허용편집정책 버전|결과를본뒤기준을바꿔통과시키지않음|
|자원/실행|model/enginepin·request조건·측정. batch비용과request시간·모델load포함peak를구분|성능순위·GPU/노트북조건으로확대금지|
|알려진문제|위표의표본별결함과아직없는판단자료|모든모델을같은환각/전체cue실패로일괄분류금지|
|사람수정|편집전후내용·수정자행위·수정시점·대상원본candidateSHA·derivedasset별도ID/SHA|구현할기능의필요데이터이며현재저장완료라고표현하지않음|
|표본판정|사용가능/수정필요/판단보류+필수메모·문제위치/이유·model/task/item/criterion revision|보류는통과아님. 개발감사와사용자verdict 구분|
|명시적채택|사용자가선택한**결과 revision/사람수정본**과대상작업의채택기록|verdict만으로자동적용금지. “제안채택”과모델품질/자동적용승인은별도|

최종 사용자에게는 위자료를 같은작업의3후보로 비교할수있게 제시하되 modelname만으로선택시키지않는다. developer가준비한알려진실패표시·구조검사·근거대조결과를먼저제공한다. 주관적가독성/편집적합성은준비후사람이판단한다. “사용가능”은해당표본/조건의판정이며전체모델·자동스킵·자동자막교체·자동출력허가가아니다.

## 실제 asset 식별과 재검토 데이터

r3 package를기준으로다음3개의baseline을공유한다. 같은task간원본이같다는것과다른task의입력표현이완전히같다는것은구분한다.

| task | baseline asset SHA / bytes |
|---|---|
|correction|b282db497133af492690d5d8726549cbafa7dd203517476e64e915c8101b2e27 /1232B|
|cards|47aecd7b66ef847c6e4a555650d3e39f57e9bfecf5cd00e64082e3da61d49cab /2129B|
|highlights|658e5fbded1a1203472c937b8b7188c64915859446cb6c4e9b827d877b0d8093 /5623B|

candidate는 package의 각 item.candidate.assets를 정본으로 참조한다. Gemma/Llama correction은516B/1115B의 실패 원문을 보존하고 Qwen correction271B/cards261B를 혼동하지 않는다. 수정 결과는 새 asset·package/derived revision으로 남기고 r3/과거자동검사/known_issues를 덮지 않는다.

추가 필요 데이터는 다음과 같다. 이번 세션에서 생성·실행한 데이터가 아니다.

1. 교정:6문장별 허용 편집 명세(구두점만/대소문자 포함 여부)와 모호한 ASR 어휘 위치. 원음 오인식까지 평가하려면 identity·구간을 고정한 원음과 독립 사람 정답이 필요하다.
2. 카드:추출/요약 목적,문장 수·길이·반복 허용량,문장별 최소 supporting cue와 의미 불일치 반례.
3. 하이라이트:대화 발췌/숏폼 등 목적,최대2범위 외 길이·선별성 기준과 독립 사람 선정/이유. media 시간 상한을 쓰려면 실제 대응 구간이 필요하며 range=null을 보정하지 않는다.
4. 수정 후 재검증:같은 입력 before/after와 추가 독립 입력. prompt/constraint/budget/model 변경은 별도 revision으로 root 한 worker가 필요할 때 실행한다. 기존9표본 반복을 새로운 독립 품질 표본으로 늘리지 않는다.
5. 최종 저장소:테스트 판정 없는 품질 review store,실제 SDK 저장 응답과 read-back,원본 candidate/사람 수정본/채택 대상을 구분한 이력. UI 테스트의 판단 보류를 사용자 품질 verdict로 재사용하지 않는다.

## 제품 연결 뒤 최종 검토에 들어갈 준비 gate

GUI/CLI가 같은 application/domain 비교·편집·명시적 채택 계약을 사용하고,9개 전체 읽기·키보드/복사·오류/자산 누락/저장 실패·재시작 read-back을 실제 검증해야 한다. 아직 없는 버튼·명령을 사용 가능으로 안내하지 않는다. SDK inspect의 review_ready만으로 편집/채택 기능의 실제 동작을 통과시키지 않는다.

그 뒤 개발 담당이 알려진 결함을 수정하고 자동검사·원문대조·필요데이터가 갖춰진 대상만 마지막 모델 검토로 넘긴다. 미준비는 개발 측 준비 미완료이며 사용자 승인 대기가 아니다. 제안 기능부터 연결하기로 한 결정을 유지하며 모델 품질 개선이 모두 끝날 때까지 제품 연결을 멈추는 조건으로 되돌리지 않는다.

나머지23역할에는 음성/정렬/화자/검색/OCR/영상 등의 별도 패키지·품질 gate가 있다. 텍스트9개의 SDK/GUI 전달을 다른 역할로 일반화하지 않는다. 역할별 원본 재생·독립 정답·지표·실패 표시가 필요하며 실행23/품질승인0을 구분한다. Gemma 경계·내부 trace 통과도 문장 품질 통과가 아니다.

본 문서는 최종 검토의 준비 상태와 필요한 데이터만 기록한다. 새 모델 채택·품질 verdict·자동 적용 허가를 부여하지 않으며 root의 제품/공통 정본과 Pages를 변경하지 않았다.
