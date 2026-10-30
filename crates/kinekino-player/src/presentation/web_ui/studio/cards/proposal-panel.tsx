import { CardCopyFields } from "../../components/card-copy-fields";
import { ProposalActions } from "../../components/proposal-actions";
import { ProposalOutputReview } from "../../components/proposal-output-review";
import { ProposalSource } from "../../components/proposal-source";

type Props = {
  outputReviewAvailable: boolean;
  outputReviewBusy: boolean;
  outputReviewHasInspection: boolean;
  outputReviewSelected: boolean;
  outputReviewResultUnknown: boolean;
  outputReviewCanRecord: boolean;
  outputReviewCanRetry: boolean;
  outputReviewFieldsDisabled: boolean;
  outputReviewHistoryNotChecked: boolean;
  outputReviewStatusLabel: string;
  outputReviewErrorLabel: string;
  outputReviewVerdictLabel: string;
  outputReviewSourceText: string;
  outputReviewGeneratedRaw: string;
  outputReviewDraftText: string;
  outputReviewAcceptedText: string;
  outputReviewEvidenceLabel: string;
  outputReviewNote: string;
  outputReviewReviewerLabel: string;
  outputReviewReviewerId: string;
  outputReviewClaimedTime: string;
  outputReviewConfirmedLabel: string;
  outputReviewAttemptedLabel: string;
  outputReviewTargetLines: string[];
  outputReviewCriteriaLines: string[];
  outputReviewHistoryRecords: string[];
  outputReviewOnInspect: () => void;
  outputReviewOnSelect: () => void;
  outputReviewOnHistory: () => void;
  outputReviewOnUsable: () => void;
  outputReviewOnNeedsChanges: () => void;
  outputReviewOnDeferred: () => void;
  outputReviewOnRecord: () => void;
  outputReviewOnRetry: () => void;
  outputReviewOnNoteChange: (text: string) => void;
  outputReviewOnReviewerLabelChange: (text: string) => void;
  outputReviewOnReviewerIdChange: (text: string) => void;
  outputReviewOnClaimedTimeChange: (text: string) => void;
  historicalReferenceLabel: string; historicalVerificationLabel: string;
  proposalId: string; revisionLabel: string; sourceIdentityLabel: string; sourceRefs: string[];
  sourceText: string; generatedRaw: string; qualityLabel: string; selectedCardLabel: string;
  front: string; back: string; canEdit: boolean; canSave: boolean; canAccept: boolean;
  busy: boolean; refreshRequired: boolean; resultLabel: string;
  onFrontChange: (text: string) => void; onBackChange: (text: string) => void;
  onSave: () => void; onAccept: () => void; onRefresh: () => void;
};

export function CardProposalPanel(props: Props) {
  return <section aria-label="카드 문구 제안 검토">
    <div className="aw-inspector-heading"><h3>카드 문구 제안</h3><p>{props.selectedCardLabel}</p></div>
    <ProposalSource proposalId={props.proposalId} revisionLabel={props.revisionLabel} sourceIdentityLabel={props.sourceIdentityLabel} sourceRefs={props.sourceRefs} sourceText={props.sourceText} generatedRaw={props.generatedRaw} qualityLabel={props.qualityLabel} historicalReferenceLabel={props.historicalReferenceLabel} historicalVerificationLabel={props.historicalVerificationLabel}/>
    <CardCopyFields front={props.front} back={props.back} canEdit={props.canEdit} busy={props.busy} refreshRequired={props.refreshRequired} onFrontChange={props.onFrontChange} onBackChange={props.onBackChange}/>
    <ProposalActions canSave={props.canSave} canAccept={props.canAccept} busy={props.busy} refreshRequired={props.refreshRequired} resultLabel={props.resultLabel} onSave={props.onSave} onAccept={props.onAccept} onRefresh={props.onRefresh}/>
    <ProposalOutputReview
      available={props.outputReviewAvailable}
      busy={props.outputReviewBusy}
      hasInspection={props.outputReviewHasInspection}
      selected={props.outputReviewSelected}
      resultUnknown={props.outputReviewResultUnknown}
      canRecord={props.outputReviewCanRecord}
      canRetry={props.outputReviewCanRetry}
      fieldsDisabled={props.outputReviewFieldsDisabled}
      historyNotChecked={props.outputReviewHistoryNotChecked}
      statusLabel={props.outputReviewStatusLabel}
      errorLabel={props.outputReviewErrorLabel}
      verdictLabel={props.outputReviewVerdictLabel}
      sourceText={props.outputReviewSourceText}
      generatedRaw={props.outputReviewGeneratedRaw}
      draftText={props.outputReviewDraftText}
      acceptedText={props.outputReviewAcceptedText}
      evidenceLabel={props.outputReviewEvidenceLabel}
      note={props.outputReviewNote}
      reviewerLabel={props.outputReviewReviewerLabel}
      reviewerId={props.outputReviewReviewerId}
      claimedTime={props.outputReviewClaimedTime}
      confirmedLabel={props.outputReviewConfirmedLabel}
      attemptedLabel={props.outputReviewAttemptedLabel}
      targetLines={props.outputReviewTargetLines}
      criteriaLines={props.outputReviewCriteriaLines}
      historyRecords={props.outputReviewHistoryRecords}
      onInspect={props.outputReviewOnInspect}
      onSelect={props.outputReviewOnSelect}
      onHistory={props.outputReviewOnHistory}
      onUsable={props.outputReviewOnUsable}
      onNeedsChanges={props.outputReviewOnNeedsChanges}
      onDeferred={props.outputReviewOnDeferred}
      onRecord={props.outputReviewOnRecord}
      onRetry={props.outputReviewOnRetry}
      onNoteChange={props.outputReviewOnNoteChange}
      onReviewerLabelChange={props.outputReviewOnReviewerLabelChange}
      onReviewerIdChange={props.outputReviewOnReviewerIdChange}
      onClaimedTimeChange={props.outputReviewOnClaimedTimeChange}
    />
  </section>;
}

