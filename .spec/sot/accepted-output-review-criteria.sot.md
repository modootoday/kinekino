---
id: sot:kinekino/accepted-output-review-criteria
title: 채택 출력의 작업별 검토 기준 초안
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 채택 출력의 작업별 검토 기준 초안

2026-10-20 PM 배정 독립 설계 검토. **미인수 초안이며 구현된 catalog·사용자 판정·품질 승인·합격 기준 확정이 아니다.** SDK의 [최소 포트/criteria 제안](../../../plans/player/accepted-output-review-sdk.md)과 [대상·저장 인수 조건](../../../plans/player/edited-output-quality-decision.md)을 따른다. 정확 schema/문구·version의 SDK/PM 인수 후 Rust catalog로 옮길 수 있는 내용 제안이다. 지금 사용자의 평가·승인을 요구하지 않는다.

근거는 [기능별 검토 정책](model-quality-review.md), [현재 세 기능 API 시나리오](api/proposals.md#2026-10-20-세-기능의-sdkcli-인수-시나리오), [사람 검토 준비](../infrastructure/inference/research-followups/model-quality-final-review-readiness.md), [기존 내용 감사](../../verification/ai/text-review-content-audit.md)다. 원 모델 request schema와 제품 typed output을 동형으로 간주하지 않는다.

## 공통 대상과 판정 한계

대상은 현재 선택된 acceptance의 proposal/task/draft revision/output revision/canonical typed output hash 및 source/generated/historical identity다. 원 candidate, working draft, 실제 렌더·export 파일과 별개다. accepted라는 이유만으로 human-edited라 표시하지 않는다. 원문·원 candidate·선택 output을 나란히 읽되 원 candidate의 usable/자동검사/ModelCondition을 새 output에 상속하지 않는다. 원문은 입력 기준선이며 독립 사람 정답이 아니다.

catalog 내용은 `criteria_id/version/task/purpose/clauses` 전체를 identity에 포함하는 SDK 안에 맞춘다. 아래 version `1`은 **첫 인수 후보**이며 현재 배포된 version이 아니다. purpose는 별도 사용자 목적을 추정하지 않는 좁은 텍스트 검토 범위다. clause 번호는 문서의 안정적인 대조용이며 SDK field/type을 새로 확정하지 않는다. 문구·purpose·clauses가 바뀌면 새 identity/version과 재제시가 필요하고 기존 decision을 재해석하지 않는다. 최종 직렬화/hash 규칙이 인수되기 전 content SHA를 발명하지 않는다.

공통 필수 note에는 검토한 정확 output/기준, 확인한 문제나 미확인 사항의 문장·cue·문서 위치를 남긴다. 시간 근거가 없는 위치에 timestamp를 만들지 않는다. 개발자가 먼저 알려진 구조/누락/근거 오류를 제시하고, 사람은 준비된 자료에서 의미·가독성을 판단한다. 구조 검사 성공·모든 clause 표시·technical acceptance만으로 usable이나 sample_review_passed를 자동 생성하지 않는다. 이 초안은 clause 수의 합계·점수·자동 통과 threshold를 도입하지 않는다.

## 작업별 catalog 후보

| criteria_id | version 후보 | task | 고정 purpose 문구 후보 |
|---|---|---|---|
| accepted-correction-source-review | 1 | correction | 선택한 채택 교정 텍스트를 고정 원문과 비교하여 참조·의미 보존·읽기 적합성을 검토한다. 원음의 정답이나 자막 시간 정확도는 판정하지 않는다. |
| accepted-cards-source-review | 1 | cards | 선택한 채택 카드의 앞·뒷면 문구를 참조 원문 cue와 비교하여 근거·의미·가독성을 검토한다. 별도 학습 목적·게시 목적·렌더 결과는 추정하지 않는다. |
| accepted-highlights-source-review | 1 | highlights | 선택한 채택 하이라이트의 제목과 참조 cue를 비교하여 텍스트 근거와 선택 설명의 정합성을 검토한다. 흥미도·숏폼 적합성·실제 영상 범위나 재생 적용은 판정하지 않는다. |

### correction clauses 후보

task는 domain의 correction이며 cue payload의 `correction`과 document payload의 `document_correction` 모두 포함한다. 서로 다른 source 참조 형식을 합치거나 document를 cue로 변환하는 기준이 아니다.

1. **참조 대조:** cue 교정은 canonical cue ID, document 교정은 source asset SHA/outer pointer/extraction/inner pointer 전체로 원문 위치를 확인한다. 문서 위치를 가짜 cue/time으로 바꾸지 않는다. 잘못된/중복 참조의 기술 거부와 읽기 비교를 구분한다.
2. **의미 보존:** 선택 output의 각 수정 텍스트를 해당 원문과 대조한다. 원문에 없는 사실 추가, 의미 반전, 문장 잘림·불필요 반복·중요 내용 누락이 없는지 확인하고 문제 위치와 이유를 기록한다. ASR 원문 자체가 불명확하면 추측으로 정답을 확정하지 않고 미확인으로 남긴다.
3. **변경 범위의 가시성:** 구두점·대소문자·단어/문장 변경을 구분해 확인한다. 현재 미정인 punctuation-only의 대소문자·어휘 변경 허용 정책을 이 기준이 대신 승인하지 않는다. 별도 허용 정책 준수까지 요구되는 검토는 정책 identity가 준비되기 전 지원하지 않는다.
4. **부분 수정과 읽기:** output은 부분 edits를 허용한다. 수정 대상만 검토한 결과를 원문 전체의 교정 완료로 표시하지 않는다. 선택 수정문의 가독성을 확인하되 텍스트만으로 원음 인식 정확도·전체 자막·동기화를 승인하지 않는다.

### cards clauses 후보

1. **문구별 원문 근거:** 각 카드 front/back의 사실·설명을 그 카드 source_refs의 실제 원문과 대조한다. ID 존재만으로 의미 근거가 일치한다고 하지 않으며 본문과 무관한 참조·원문 없는 주장은 위치와 이유를 기록한다.
2. **의미와 가독성:** 원문 의미가 왜곡되지 않고 문장 연결이 자연스러운지 확인한다. 앞·뒷면이 서로 모순되거나 설명이 잘려 뜻을 읽을 수 없는지 대조한다. 원문 오류 가능성은 별도 미확인 사항이다.
3. **불필요한 중복:** 같은 주장이나 문구의 반복이 내용 전달을 방해하는지 확인한다. front/back의 공통 단어·질문과 답의 대응·추출형 표현 자체를 일괄 실패로 보지 않는다. 단순 복사를 근거 없이 환각으로 분류하지 않는다.
4. **지원 범위 명시:** 이 purpose에는 미정인 학습 효과·사용자별 추출/요약 목적·글자 수·한 문장 규칙이 포함되지 않는다. 과거 모델 request의 headline/summary 조건을 현재 front/back에 자동 이식하지 않는다. 별도 조건이 필요한 판정은 조건을 갖춘 새 기준으로 재제시해야 한다.

### highlights clauses 후보

1. **제목·선택의 텍스트 근거:** 각 title과 source_refs의 실제 cue를 함께 읽어 원문 없는 사건·설명·인물 관계를 추가했는지, 참조와 제목이 의미상 대응하는지 확인한다. 현재 typed output에 reason 필드가 없으므로 검토 메모를 모델이 생성한 reason으로 저장하거나 원 candidate reason을 새 output의 일부로 가장하지 않는다.
2. **선택 설명의 정합성:** 검토자는 선택 cue와 제목이 어떻게 대응하는지 note에 근거를 남긴다. 참조 전체를 고르는 결과도 독립 선정 목적·범위 기준이 없는 상태에서 자동 실패로 보지 않는다. 의미 근거 확인을 흥미도·선별성 승인으로 확대하지 않는다.
3. **읽기와 명시적 미확인:** 제목의 문장 완결성·가독성·반복으로 인한 오해를 확인한다. 텍스트만으로 자연스러운 음성 쉼·장면 전환·실제 영상 흥미도를 검증했다고 하지 않는다.
4. **시간과 적용 경계:** source cue의 시간, package range, proposal의 선택적 시간, 실제 media PTS를 구분한다. 현재 실제9 package range와 수입 proposal 시간은 null을 유지한다. 고정 원 request에 시간이 있어도 생성 후보·선택 output의 검증된 영상 구간으로 자동 승격하지 않는다. 최대2범위·연속성·길이·자동 skip는 이 catalog의 판정 범위가 아니다.

## 인수 사례와 미지원 범위

| 사례 | 기준 해석 / 기대 한계 |
|---|---|
| 원 candidate가 잘린 JSON이고 사람이 유효한 draft를 저장·채택 | 잘린 raw는 그대로 비교 자료로 보존. 검토 대상은 별도 exact accepted output. raw 수선 성공·원 모델 품질 개선으로 집계하지 않음 |
| correction에서 일부 문장만 변경 | 해당 edits의 참조·의미·가독성만 검토. 전 문장 수정/ASR 정확도/시간 일치 pass 생성0 |
| turk→Turk 같은 대소문자 변경 | 변경 유형을 보이되 punctuation-only 허용 여부는 미결정. 이 초안이 사용자 정책 대신 정답을 정하지 않음 |
| 카드 refs는 존재하지만 문구와 무관함 | 구조 유효성과 별개로 의미 근거 문제를 note에 기록. usable 자동 생성0 |
| 카드 front/back이 같은 핵심 단어를 씀 | 단어 중복만으로 실패시키지 않고 전달 방해·의미 대응을 검토. 별도 길이/문장 수 합격을 주장하지 않음 |
| 하이라이트가 전체 cue를 선택하거나 음성 쉼을 주장 | 전체 선택 자체의 자동 실패0. 음성 연결 없이 쉼 주장은 미확인이고 영상 구간 사용 가능 판정으로 확대0 |
| 새 purpose·clauses를 저장 직전에 교환 | catalog 전체 identity 불일치로 재제시/거부해야 하는 SDK 후속 인수 사례. 기존 승인 문구/decision의 의미를 덮어쓰지 않음 |
| 원 candidate usable·technical receipt 있음 | accepted-output의 새 사람 판정을 자동 상속하지 않음. 사람 편집 여부가 unknown이어도 원 모델의 추가 성공/독립 표본에 합산0 |

이 초안의 좁은 텍스트 purpose로 판정할 수 있는 것과 미디어/목적별 승인을 구분한다. 자료 부족은 근거/미확인 사항으로 제시하며 deferred/needs_changes/usable의 명시 선택은 후속 SDK 계약과 실제 검토 행위에 따른다. 미지원 항목에 N/A를 붙였다는 이유로 전체 기능 합격을 만들지 않는다. quality decision과 technical acceptance, 실제 apply/export, 모델/license/runtime precision admission은 별개다.

우선 인수할 사항은 세 purpose가 첫 catalog의 제한된 범위임을 명시하는 것, criteria 전체 내용/hash/read-back 결합, clause별 결과를 별도 저장할지 note로만 남길지의 SDK 결정이다. 현재 schema에는 clause별 outcome 저장이 없으므로 이 문서의 번호가 저장 지원을 뜻하지 않는다. reviewer label/claimed time은 선언, adapter observed time은 로컬 관측이며 인증 신원·신뢰 시각 증명이 아니다.

적용 스킬은 modoo.today `modootoday-free/ai-quality-plus/llm-output-eval-suite` 본문(추가 참조 없음)이다. task별 사례·기대/금지와 실패 slice를 고정하는 부분을 적용했다. 새 표본 수집·모델 탐색·반복 추론·LLM judge·점수/통과 threshold를 추가하지 않았다. 이번 산출물은 문서 초안뿐이며 제품 코드·Cargo·브라우저·Pages·원자료 복사·판정 저장은0이다.
