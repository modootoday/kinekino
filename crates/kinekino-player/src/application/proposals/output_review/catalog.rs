//! Fixed text-review scope from accepted-output-review-criteria.md, version 1.
//! This catalog neither evaluates output nor authorizes media/application behavior.
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::proposals::{
        ProposalTask,
        output_review::{CriteriaDocument, CriteriaIdentity, OutputCriterion},
    },
};

pub fn criteria_for(task: ProposalTask) -> Result<CriteriaDocument, ApiError> {
    let (criteria_id, purpose, clauses) = match task {
        ProposalTask::Correction => (
            "accepted-correction-source-review",
            "선택한 채택 교정 텍스트를 고정 원문과 비교하여 참조·의미 보존·읽기 적합성을 검토한다. 원음의 정답이나 자막 시간 정확도는 판정하지 않는다.",
            [
                (
                    "reference",
                    "참조 대조",
                    "cue 교정은 canonical cue ID, document 교정은 source asset SHA/outer pointer/extraction/inner pointer 전체로 원문 위치를 확인한다. 문서 위치를 가짜 cue/time으로 바꾸지 않는다. 잘못된/중복 참조의 기술 거부와 읽기 비교를 구분한다.",
                ),
                (
                    "meaning",
                    "의미 보존",
                    "선택 output의 각 수정 텍스트를 해당 원문과 대조한다. 원문에 없는 사실 추가, 의미 반전, 문장 잘림·불필요 반복·중요 내용 누락이 없는지 확인하고 문제 위치와 이유를 기록한다. ASR 원문 자체가 불명확하면 추측으로 정답을 확정하지 않고 미확인으로 남긴다.",
                ),
                (
                    "change-scope",
                    "변경 범위의 가시성",
                    "구두점·대소문자·단어/문장 변경을 구분해 확인한다. 현재 미정인 punctuation-only의 대소문자·어휘 변경 허용 정책을 이 기준이 대신 승인하지 않는다. 별도 허용 정책 준수까지 요구되는 검토는 정책 identity가 준비되기 전 지원하지 않는다.",
                ),
                (
                    "partial-reading",
                    "부분 수정과 읽기",
                    "output은 부분 edits를 허용한다. 수정 대상만 검토한 결과를 원문 전체의 교정 완료로 표시하지 않는다. 선택 수정문의 가독성을 확인하되 텍스트만으로 원음 인식 정확도·전체 자막·동기화를 승인하지 않는다.",
                ),
            ],
        ),
        ProposalTask::Cards => (
            "accepted-cards-source-review",
            "선택한 채택 카드의 앞·뒷면 문구를 참조 원문 cue와 비교하여 근거·의미·가독성을 검토한다. 별도 학습 목적·게시 목적·렌더 결과는 추정하지 않는다.",
            [
                (
                    "source-support",
                    "문구별 원문 근거",
                    "각 카드 front/back의 사실·설명을 그 카드 source_refs의 실제 원문과 대조한다. ID 존재만으로 의미 근거가 일치한다고 하지 않으며 본문과 무관한 참조·원문 없는 주장은 위치와 이유를 기록한다.",
                ),
                (
                    "meaning-reading",
                    "의미와 가독성",
                    "원문 의미가 왜곡되지 않고 문장 연결이 자연스러운지 확인한다. 앞·뒷면이 서로 모순되거나 설명이 잘려 뜻을 읽을 수 없는지 대조한다. 원문 오류 가능성은 별도 미확인 사항이다.",
                ),
                (
                    "redundancy",
                    "불필요한 중복",
                    "같은 주장이나 문구의 반복이 내용 전달을 방해하는지 확인한다. front/back의 공통 단어·질문과 답의 대응·추출형 표현 자체를 일괄 실패로 보지 않는다. 단순 복사를 근거 없이 환각으로 분류하지 않는다.",
                ),
                (
                    "supported-scope",
                    "지원 범위 명시",
                    "이 purpose에는 미정인 학습 효과·사용자별 추출/요약 목적·글자 수·한 문장 규칙이 포함되지 않는다. 과거 모델 request의 headline/summary 조건을 현재 front/back에 자동 이식하지 않는다. 별도 조건이 필요한 판정은 조건을 갖춘 새 기준으로 재제시해야 한다.",
                ),
            ],
        ),
        ProposalTask::Highlights => (
            "accepted-highlights-source-review",
            "선택한 채택 하이라이트의 제목과 참조 cue를 비교하여 텍스트 근거와 선택 설명의 정합성을 검토한다. 흥미도·숏폼 적합성·실제 영상 범위나 재생 적용은 판정하지 않는다.",
            [
                (
                    "title-support",
                    "제목·선택의 텍스트 근거",
                    "각 title과 source_refs의 실제 cue를 함께 읽어 원문 없는 사건·설명·인물 관계를 추가했는지, 참조와 제목이 의미상 대응하는지 확인한다. 현재 typed output에 reason 필드가 없으므로 검토 메모를 모델이 생성한 reason으로 저장하거나 원 candidate reason을 새 output의 일부로 가장하지 않는다.",
                ),
                (
                    "selection-note",
                    "선택 설명의 정합성",
                    "검토자는 선택 cue와 제목이 어떻게 대응하는지 note에 근거를 남긴다. 참조 전체를 고르는 결과도 독립 선정 목적·범위 기준이 없는 상태에서 자동 실패로 보지 않는다. 의미 근거 확인을 흥미도·선별성 승인으로 확대하지 않는다.",
                ),
                (
                    "reading-unknown",
                    "읽기와 명시적 미확인",
                    "제목의 문장 완결성·가독성·반복으로 인한 오해를 확인한다. 텍스트만으로 자연스러운 음성 쉼·장면 전환·실제 영상 흥미도를 검증했다고 하지 않는다.",
                ),
                (
                    "time-application",
                    "시간과 적용 경계",
                    "source cue의 시간, package range, proposal의 선택적 시간, 실제 media PTS를 구분한다. 현재 실제9 package range와 수입 proposal 시간은 null을 유지한다. 고정 원 request에 시간이 있어도 생성 후보·선택 output의 검증된 영상 구간으로 자동 승격하지 않는다. 최대2범위·연속성·길이·자동 skip는 이 catalog의 판정 범위가 아니다.",
                ),
            ],
        ),
    };
    let document = CriteriaDocument {
        schema_version: "1".into(),
        criteria_id: criteria_id.into(),
        version: "1".into(),
        task,
        purpose: purpose.into(),
        clauses: clauses
            .into_iter()
            .map(|(id, how, expected)| OutputCriterion {
                criterion_id: id.into(),
                how_to_review: how.into(),
                expected: expected.into(),
            })
            .collect(),
    };
    document
        .validate()
        .map_err(|message| ApiError::new(ErrorCode::Internal, message))?;
    Ok(document)
}

/// Select trusted content by task, then compare the whole presented identity.
/// Caller-supplied criteria prose never replaces the compiled catalog.
pub fn require_current_criteria(identity: &CriteriaIdentity) -> Result<CriteriaDocument, ApiError> {
    let document = criteria_for(identity.task)?;
    let current = document
        .identity()
        .map_err(|message| ApiError::new(ErrorCode::Internal, message))?;
    if *identity != current {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Output review criteria changed; present current criteria again",
        ));
    }
    Ok(document)
}
