import { useId } from "react";
import { Button } from "@/components/ui/button";
import { ProposalReviewIntent } from "../../verification/proposal-review-intent";

type Props = {
  sourceCountLabel: string; sourceChoice: string; selectedSourceText: string;
  generatedRaw: string; importError: string; selectionDisabled: boolean; startDisabled: boolean;
  proposalId: string; sourceSha256: string; documentRevision: string; workingText: string;
  errorText: string; inputDisabled: boolean; saveDisabled: boolean; acceptDisabled: boolean; refreshDisabled: boolean;
  onSourceChoice: (value: string) => void; onStartManual: () => void;
  onDraftChange: (value: string) => void; onSave: () => void; onAccept: () => void; onRefresh: () => void;
};

export function DocumentRepair(props: Props) {
  const sourceChoiceId = useId();
  return <main aria-label="문서 원문 비교와 수동 수정">
    <section className="aw-inspector-section" aria-label="원문 선택">
      <h2>원문 선택</h2><p>{props.sourceCountLabel}</p>
      <label htmlFor={sourceChoiceId}>원문 항목 번호</label>
      <input id={sourceChoiceId} className="aw-text-input" value={props.sourceChoice} disabled={props.selectionDisabled} onChange={event => props.onSourceChoice(event.currentTarget.value)}/>
      <p className="aw-evidence-body">{props.selectedSourceText}</p>
      <Button className="aw-button" disabled={props.startDisabled} onClick={() => props.onStartManual()}>선택한 원문에서 직접 수정 시작</Button>
    </section>
    <section className="aw-inspector-section" aria-label="가져온 생성 원문">
      <h2>생성 원문</h2><p className="aw-evidence-body">{props.generatedRaw}</p>
      <p role="status">{props.importError}</p>
    </section>
    <ProposalReviewIntent proposalId={props.proposalId} sourceSha256={props.sourceSha256} documentRevision={props.documentRevision} workingText={props.workingText} errorText={props.errorText} inputDisabled={props.inputDisabled} saveDisabled={props.saveDisabled} acceptDisabled={props.acceptDisabled} refreshDisabled={props.refreshDisabled} onDraftChange={props.onDraftChange} onSave={props.onSave} onAccept={props.onAccept} onRefresh={props.onRefresh}/>
  </main>;
}
