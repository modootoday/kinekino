import { Button } from "@/components/ui/button";
import { CardCopyFields } from "../../components/card-copy-fields";
import { CaptureActions } from "../../components/capture-actions";
import { ProposalSource } from "../../components/proposal-source";

type Props = {
  captureCanReturn: boolean; returnViewingLabel: string; onCaptureReturnViewing: () => void;
  captureCanRevert: boolean; revertLabel: string; onCaptureRevert: () => void;
  captureTitle: string; saveLabel: string; acceptLabel: string; consumeLabel: string; discardLabel: string; retryLabel: string;
  historicalReferenceLabel: string; historicalVerificationLabel: string;
  proposalId: string; revisionLabel: string; sourceIdentityLabel: string; sourceRefs: string[];
  sourceText: string; generatedRaw: string; qualityLabel: string; selectedCardLabel: string;
  front: string; back: string; canEdit: boolean; canSave: boolean; canAccept: boolean;
  busy: boolean; refreshRequired: boolean; resultLabel: string;
  captureCanConsume: boolean; captureCanDiscard: boolean; captureCanRetry: boolean;
  onFrontChange: (text: string) => void; onBackChange: (text: string) => void;
  onSave: () => void; onAccept: () => void;
  onCaptureConsume: () => void; onCaptureDiscard: () => void; onCaptureRetry: () => void;
};
export function CapturedCardPanel(props: Props) {
  return <section aria-label={props.captureTitle}>
    <Button className="aw-button" disabled={!props.captureCanReturn} onClick={() => props.onCaptureReturnViewing()}>{props.returnViewingLabel}</Button>
    <div className="aw-inspector-heading"><h3>카드 문구 제안</h3><p>{props.selectedCardLabel}</p></div>
    <ProposalSource proposalId={props.proposalId} revisionLabel={props.revisionLabel} sourceIdentityLabel={props.sourceIdentityLabel} sourceRefs={props.sourceRefs} sourceText={props.sourceText} generatedRaw={props.generatedRaw} qualityLabel={props.qualityLabel} historicalReferenceLabel={props.historicalReferenceLabel} historicalVerificationLabel={props.historicalVerificationLabel}/>
    <CardCopyFields front={props.front} back={props.back} canEdit={props.canEdit} busy={props.busy} refreshRequired={props.refreshRequired} onFrontChange={props.onFrontChange} onBackChange={props.onBackChange}/>
    <CaptureActions canSave={props.canSave} canAccept={props.canAccept} busy={props.busy} canConsume={props.captureCanConsume} canDiscard={props.captureCanDiscard} canRetry={props.captureCanRetry} canRevert={props.captureCanRevert} resultLabel={props.resultLabel} saveLabel={props.saveLabel} acceptLabel={props.acceptLabel} consumeLabel={props.consumeLabel} discardLabel={props.discardLabel} retryLabel={props.retryLabel} revertLabel={props.revertLabel} onSave={props.onSave} onAccept={props.onAccept} onConsume={props.onCaptureConsume} onDiscard={props.onCaptureDiscard} onRetry={props.onCaptureRetry} onRevert={props.onCaptureRevert}/>
  </section>;
}
