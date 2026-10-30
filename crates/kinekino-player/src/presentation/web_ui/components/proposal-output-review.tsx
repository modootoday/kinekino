import { useId } from "react";
import { Button } from "@/components/ui/button";

// Declaration only: Rust owns selection, typed input, request lifetime and SDK calls.
type Props = {
  available: boolean;
  busy: boolean; hasInspection: boolean; selected: boolean; resultUnknown: boolean;
  canRecord: boolean; canRetry: boolean; fieldsDisabled: boolean;
  statusLabel: string; errorLabel: string; verdictLabel: string;
  targetLines: string[]; criteriaLines: string[]; sourceText: string; generatedRaw: string;
  draftText: string; acceptedText: string; evidenceLabel: string;
  note: string; reviewerLabel: string; reviewerId: string; claimedTime: string;
  historyRecords: string[]; historyNotChecked: boolean; confirmedLabel: string; attemptedLabel: string;
  onInspect: () => void; onSelect: () => void; onHistory: () => void;
  onUsable: () => void; onNeedsChanges: () => void; onDeferred: () => void;
  onNoteChange: (text: string) => void; onReviewerLabelChange: (text: string) => void;
  onReviewerIdChange: (text: string) => void; onClaimedTimeChange: (text: string) => void;
  onRecord: () => void; onRetry: () => void;
};

export function ProposalOutputReview(props: Props) {
  const noteId = useId();
  const reviewerId = useId();
  const optionalId = useId();
  const claimedId = useId();
  return <section className="aw-inspector-section" aria-label="채택 출력 품질 판정">
    <h3>채택 출력 품질 판정</h3>
    <p className="aw-muted">명시적 채택과 별도의 판정입니다. 자동 적용이나 원 모델의 품질 승인을 실행하지 않습니다.</p>
    <Button className="aw-button" disabled={props.busy || !props.available} onClick={() => props.onInspect()}>현재 판정 대상 조회</Button>
    <p role="status">{props.statusLabel}</p><p role="alert">{props.errorLabel}</p>
    <section hidden={!props.hasInspection}>
      <h4>선택할 출력</h4><ul>{props.targetLines.map(line => <li key={line} className="aw-evidence-body">{line}</li>)}</ul>
      <p className="aw-muted">{props.evidenceLabel}</p>
      <details><summary>같은 조회의 원문·생성 결과·저장 초안·채택 출력 비교</summary>
        <h4>원문</h4><p className="aw-evidence-body">{props.sourceText}</p>
        <h4>생성 원문</h4><p className="aw-evidence-body">{props.generatedRaw}</p>
        <h4>저장 초안</h4><p className="aw-evidence-body">{props.draftText}</p>
        <h4>판정할 채택 출력</h4><p className="aw-evidence-body">{props.acceptedText}</p>
      </details>
      <h4>전체 검토 기준</h4><ul>{props.criteriaLines.map(line => <li key={line} className="aw-evidence-body">{line}</li>)}</ul>
      <Button className="aw-button" disabled={props.busy || props.selected || !props.available} onClick={() => props.onSelect()}>이 출력과 기준 선택</Button>
      <p>판정: {props.verdictLabel}</p>
      <Button className="aw-button" disabled={props.fieldsDisabled} onClick={() => props.onUsable()}>사용 가능</Button>
      <Button className="aw-button" disabled={props.fieldsDisabled} onClick={() => props.onNeedsChanges()}>수정 필요</Button>
      <Button className="aw-button" disabled={props.fieldsDisabled} onClick={() => props.onDeferred()}>판정 보류</Button>
      <label htmlFor={noteId}>검토 메모 (필수)</label><textarea id={noteId} className="aw-text-input aw-draft-input" value={props.note} disabled={props.fieldsDisabled} onChange={event => props.onNoteChange(event.currentTarget.value)}/>
      <label htmlFor={reviewerId}>선언 검토자 (필수)</label><input id={reviewerId} className="aw-text-input" value={props.reviewerLabel} disabled={props.fieldsDisabled} onChange={event => props.onReviewerLabelChange(event.currentTarget.value)}/>
      <label htmlFor={optionalId}>검토자 ID (선택)</label><input id={optionalId} className="aw-text-input" value={props.reviewerId} disabled={props.fieldsDisabled} onChange={event => props.onReviewerIdChange(event.currentTarget.value)}/>
      <label htmlFor={claimedId}>주장하는 검토 시각 (선택, Unix ms)</label><input id={claimedId} className="aw-text-input" value={props.claimedTime} disabled={props.fieldsDisabled} onChange={event => props.onClaimedTimeChange(event.currentTarget.value)}/>
      <p className="aw-muted">선언 이름은 인증 신원이 아닙니다. 저장 관측 시각은 adapter가 기록하며 주장한 검토 시각과 구별합니다.</p>
      <Button className="aw-button aw-button-primary" disabled={!props.canRecord} onClick={() => props.onRecord()}>이 출력의 품질 판정 기록</Button>
      <p className="aw-evidence-body" hidden={!props.resultUnknown}>{props.attemptedLabel}</p>
      <Button className="aw-button" hidden={!props.resultUnknown} disabled={!props.canRetry} onClick={() => props.onRetry()}>동일 판정 요청 명시적 재시도</Button>
    </section>
    <p role="status">{props.confirmedLabel}</p>
    <Button className="aw-button" disabled={props.busy || !props.available} onClick={() => props.onHistory()}>저장된 판정 이력 조회</Button>
    <p className="aw-muted" hidden={!props.historyNotChecked}>이력 조회는 현재 근거를 재검증하지 않았습니다. 기록은 채택 출력에만 한정되며 모델 표본 성공 수에 합산하지 않습니다.</p>
    <ul>{props.historyRecords.map(record => <li key={record} className="aw-evidence-body">{record}</li>)}</ul>
  </section>;
}
