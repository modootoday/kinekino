import { useId } from "react";
import { Button } from "@/components/ui/button";

type Props = {
  proposalId: string; sourceChoice: string; sourceOptions: string[]; sourceText: string;
  generatedRaw: string; importError: string; resultLabel: string;
  front: string; back: string; title: string; workingText: string;
  selectionDisabled: boolean; startDisabled: boolean;
  cardDisabled: boolean; highlightDisabled: boolean; correctionDisabled: boolean;
  canSave: boolean; canAccept: boolean; busy: boolean; refreshRequired: boolean;
  isCard: boolean; isHighlight: boolean; isCorrection: boolean;
  rawReviewPackage: string; rawReviewItem: string; rawReviewNote: string;
  rawReviewSource: string; rawReviewCandidate: string; rawReviewStatus: string;
  rawReviewError: string; rawReviewVerdict: string; rawReviewSaved: string;
  rawReviewModel: string; rawReviewSelection: string; rawReviewCriteria: string[]; rawReviewRangeStatus: string;
  rawReviewRetainedIdentity: string; rawReviewRetainedResponse: string;
  rawReviewAttempt: string;
  rawReviewRecovery: string;
  rawReviewCanInspectRetained: boolean; rawReviewCanHistoryRetained: boolean; rawReviewCanReleaseRetained: boolean;
  rawReviewOnInspectRetained: () => void; rawReviewOnHistoryRetained: () => void; rawReviewOnReleaseRetained: () => void;
  rawReviewItems: string[]; rawReviewIssues: string[]; rawReviewHistory: string[];
  rawReviewFieldsDisabled: boolean; rawReviewCanRead: boolean; rawReviewCanRecord: boolean;
  rawReviewOnPackageChange: (text: string) => void; rawReviewOnItemChange: (text: string) => void;
  rawReviewOnNoteChange: (text: string) => void;
  rawReviewOnOpenPackage: () => void; rawReviewOnOpenItem: () => void;
  rawReviewOnReadSource: () => void; rawReviewOnReadCandidate: () => void;
  rawReviewOnHistory: () => void; rawReviewOnRecord: () => void;
  rawReviewOnUsable: () => void; rawReviewOnNeedsChanges: () => void; rawReviewOnDeferred: () => void;
  outputReviewCanRecord: boolean; outputReviewCanRetry: boolean; outputReviewFieldsDisabled: boolean;
  outputReviewStatusLabel: string; outputReviewErrorLabel: string; outputReviewNote: string;
  outputReviewReviewerLabel: string;
  outputReviewHistoryRecords: string[];
  outputReviewAvailable: boolean; outputReviewBusy: boolean; outputReviewHasInspection: boolean;
  outputReviewSelected: boolean; outputReviewResultUnknown: boolean; outputReviewHistoryNotChecked: boolean;
  outputReviewVerdictLabel: string; outputReviewReviewerId: string; outputReviewClaimedTime: string;
  outputReviewTargetLines: string[]; outputReviewCriteriaLines: string[];
  outputReviewSourceText: string; outputReviewGeneratedRaw: string; outputReviewDraftText: string;
  outputReviewAcceptedText: string; outputReviewEvidenceLabel: string;
  outputReviewAttemptedLabel: string; outputReviewConfirmedLabel: string;
  outputReviewOnRetry: () => void;
  outputReviewOnReviewerIdChange: (text: string) => void;
  outputReviewOnClaimedTimeChange: (text: string) => void;
  onSourceChoice: (text: string) => void; onStartManual: () => void;
  onFrontChange: (text: string) => void; onBackChange: (text: string) => void;
  onTitleChange: (text: string) => void; onDraftChange: (text: string) => void;
  onSave: () => void; onAccept: () => void; onRefresh: () => void;
  outputReviewOnInspect: () => void; outputReviewOnHistory: () => void;
  outputReviewOnSelect: () => void;
  outputReviewOnUsable: () => void; outputReviewOnNeedsChanges: () => void;
  outputReviewOnDeferred: () => void; outputReviewOnRecord: () => void;
  outputReviewOnNoteChange: (text: string) => void;
  outputReviewOnReviewerLabelChange: (text: string) => void;
};

export function ManualRepair(props: Props) {
  const sourceId = useId();
  const frontId = useId();
  const backId = useId();
  const titleId = useId();
  const correctionId = useId();
  const noteId = useId();
  const reviewerId = useId();
  const sourceHintId = useId();
  const rawPackageId = useId();
  const rawItemId = useId();
  const rawNoteId = useId();
  const reviewerIdentityId = useId();
  const claimedTimeId = useId();
  return <section className="aw-inspector-section" aria-label="AI 원본 비교와 수동 수정" aria-busy={props.busy}>
    <h3>원본 비교 · 수동 수정</h3><p>{props.proposalId}</p>
    <p>직접 수정한 내용을 저장한 뒤 채택할 수 있어요. 품질 판정은 따로 기록해요.</p>
    <ul>{props.sourceOptions.map(text => <li key={text}>{text}</li>)}</ul>
    <label htmlFor={sourceId}>직접 검토할 원문 항목 번호</label>
    <p id={sourceHintId}>원문 번호를 선택한 다음 빈 수정본에 직접 입력해요.</p>
    <input id={sourceId} aria-describedby={sourceHintId} className="aw-text-input" value={props.sourceChoice} disabled={props.selectionDisabled} onChange={event => props.onSourceChoice(event.currentTarget.value)}/>
    <pre>{props.sourceText}</pre>
    <h4>모델 원결과 · 읽기 전용</h4><pre>{props.generatedRaw}</pre><p role="alert">{props.importError}</p>
    <Button type="button" disabled={props.startDisabled} onClick={props.onStartManual}>선택한 원문으로 직접 수정하기</Button>
    <div hidden={!props.isCard}><label htmlFor={frontId}>카드 앞면</label><input id={frontId} className="aw-text-input" value={props.front} disabled={props.cardDisabled} onChange={event => props.onFrontChange(event.currentTarget.value)}/>
    <label htmlFor={backId}>카드 뒷면</label><input id={backId} className="aw-text-input" value={props.back} disabled={props.cardDisabled} onChange={event => props.onBackChange(event.currentTarget.value)}/></div>
    <div hidden={!props.isHighlight}><label htmlFor={titleId}>하이라이트 제목</label><input id={titleId} className="aw-text-input" value={props.title} disabled={props.highlightDisabled} onChange={event => props.onTitleChange(event.currentTarget.value)}/></div>
    <div hidden={!props.isCorrection}><label htmlFor={correctionId}>교정 내용</label><input id={correctionId} className="aw-text-input" value={props.workingText} disabled={props.correctionDisabled} onChange={event => props.onDraftChange(event.currentTarget.value)}/></div>
    <Button type="button" disabled={!props.canSave} onClick={props.onSave}>수정본 저장</Button>
    <Button type="button" disabled={!props.canAccept} onClick={props.onAccept}>저장한 수정본 채택</Button>
    <Button type="button" disabled={props.busy} onClick={props.onRefresh}>저장한 내용 다시 열기</Button><p role="status">{props.resultLabel}</p>
    <section className="aw-inspector-section" aria-label="원 모델 결과의 직접 품질 판정">
      <h4>원 모델 결과의 품질 판정</h4><p>수정하거나 채택하지 않고도 이 후보의 판정을 기록할 수 있어요. 모델 전체 승인은 아니에요.</p>
      <p role="status">{props.rawReviewStatus}</p><p role="alert">{props.rawReviewError}</p>
      <pre>{props.rawReviewRetainedIdentity}</pre><pre>{props.rawReviewRetainedResponse}</pre>
      <label htmlFor={rawPackageId}>검토 자료 코드</label><input id={rawPackageId} className="aw-text-input" value={props.rawReviewPackage} disabled={props.rawReviewFieldsDisabled} onChange={event => props.rawReviewOnPackageChange(event.currentTarget.value)}/>
      <Button type="button" disabled={props.rawReviewFieldsDisabled} onClick={props.rawReviewOnOpenPackage}>검토 자료 열기</Button>
      <ul>{props.rawReviewItems.map(text => <li key={text}>{text}</li>)}</ul>
      <label htmlFor={rawItemId}>자료 안의 후보 코드</label><input id={rawItemId} className="aw-text-input" value={props.rawReviewItem} disabled={props.rawReviewFieldsDisabled} onChange={event => props.rawReviewOnItemChange(event.currentTarget.value)}/>
      <Button type="button" disabled={props.rawReviewFieldsDisabled} onClick={props.rawReviewOnOpenItem}>이 후보 선택</Button>
      <p role="status">{props.rawReviewSelection}</p><p>{props.rawReviewModel}</p>
      <ul>{props.rawReviewCriteria.map(text => <li key={text}>{text}</li>)}</ul>
      <Button type="button" disabled={!props.rawReviewCanRead} onClick={props.rawReviewOnReadSource}>원 요청과 원문 읽기</Button><pre>{props.rawReviewSource}</pre>
      <Button type="button" disabled={!props.rawReviewCanRead} onClick={props.rawReviewOnReadCandidate}>모델 원결과 읽기</Button><pre>{props.rawReviewCandidate}</pre>
      <ul>{props.rawReviewIssues.map(text => <li key={text}>{text}</li>)}</ul>
      <p role="status">{props.rawReviewRangeStatus}</p><p role="status">{props.rawReviewVerdict}</p>
      <Button type="button" disabled={!props.rawReviewCanRead} onClick={props.rawReviewOnUsable}>사용 가능 선택</Button>
      <Button type="button" disabled={!props.rawReviewCanRead} onClick={props.rawReviewOnNeedsChanges}>수정 필요 선택</Button>
      <Button type="button" disabled={!props.rawReviewCanRead} onClick={props.rawReviewOnDeferred}>보류 선택</Button>
      <label htmlFor={rawNoteId}>원결과 판정 메모</label><input id={rawNoteId} className="aw-text-input" value={props.rawReviewNote} disabled={props.rawReviewFieldsDisabled} onChange={event => props.rawReviewOnNoteChange(event.currentTarget.value)}/>
      <Button type="button" disabled={!props.rawReviewCanRecord} onClick={props.rawReviewOnRecord}>원결과 판정 기록</Button><p role="status">{props.rawReviewSaved}</p>
      <p>{props.rawReviewAttempt}</p>
      <pre role="status">{props.rawReviewRecovery}</pre>
      <Button type="button" disabled={!props.rawReviewCanInspectRetained} onClick={props.rawReviewOnInspectRetained}>보관한 원자료 다시 조회</Button>
      <Button type="button" disabled={!props.rawReviewCanHistoryRetained} onClick={props.rawReviewOnHistoryRetained}>원 후보 이력 확인</Button>
      <Button type="button" disabled={!props.rawReviewCanReleaseRetained} onClick={props.rawReviewOnReleaseRetained}>조회 후 보관 결과 정리</Button>
      <Button type="button" disabled={!props.rawReviewCanRead} onClick={props.rawReviewOnHistory}>이 후보 판정 이력 열기</Button>
      <ul>{props.rawReviewHistory.map(text => <li key={text}>{text}</li>)}</ul>
    </section>
    <h4>채택 출력 품질 판정 · 이력</h4><p role="status">{props.outputReviewStatusLabel}</p><p role="alert">{props.outputReviewErrorLabel}</p>
    <section aria-label="채택 출력 검토 상태" aria-busy={props.outputReviewBusy}>
      <p hidden={props.outputReviewAvailable}>출력 검토 연결을 기다리고 있어요.</p>
      <p hidden={props.outputReviewHasInspection}>판정 대상을 먼저 확인해 주세요.</p>
      <p hidden={!props.outputReviewSelected}>명시적으로 선택한 출력이에요.</p>
      <p role="alert" hidden={!props.outputReviewResultUnknown}>기록 결과를 확인하지 못했어요. 같은 판정 요청의 상태를 확인해 주세요.</p>
      <p hidden={!props.outputReviewHistoryNotChecked}>이력에 연결된 현재 근거는 아직 확인하지 않았어요.</p>
      <ul>{props.outputReviewTargetLines.map(text => <li key={text}>{text}</li>)}</ul>
      <ul>{props.outputReviewCriteriaLines.map(text => <li key={text}>{text}</li>)}</ul>
      <h5>원문</h5><pre>{props.outputReviewSourceText}</pre>
      <h5>모델 원결과</h5><pre>{props.outputReviewGeneratedRaw}</pre>
      <h5>저장한 수정본</h5><pre>{props.outputReviewDraftText}</pre>
      <h5>채택한 출력</h5><pre>{props.outputReviewAcceptedText}</pre><p>{props.outputReviewEvidenceLabel}</p>
      <p>{props.outputReviewVerdictLabel}</p><pre>{props.outputReviewAttemptedLabel}</pre><pre>{props.outputReviewConfirmedLabel}</pre>
      <Button type="button" disabled={!props.outputReviewCanRetry} onClick={props.outputReviewOnRetry}>같은 판정 요청 결과 다시 확인</Button>
    </section>
    <Button type="button" disabled={props.busy} onClick={props.outputReviewOnInspect}>판정 대상 확인</Button>
    <Button type="button" disabled={props.outputReviewFieldsDisabled} onClick={props.outputReviewOnSelect}>확인한 출력 판정 대상으로 선택</Button>
    <Button type="button" disabled={props.busy} onClick={props.outputReviewOnHistory}>이력 조회</Button>
    <Button type="button" disabled={props.outputReviewFieldsDisabled} onClick={props.outputReviewOnUsable}>사용 가능</Button>
    <Button type="button" disabled={props.outputReviewFieldsDisabled} onClick={props.outputReviewOnNeedsChanges}>수정 필요</Button>
    <Button type="button" disabled={props.outputReviewFieldsDisabled} onClick={props.outputReviewOnDeferred}>보류</Button>
    <label htmlFor={noteId}>판정 메모</label><input id={noteId} className="aw-text-input" value={props.outputReviewNote} disabled={props.outputReviewFieldsDisabled} onChange={event => props.outputReviewOnNoteChange(event.currentTarget.value)}/>
    <label htmlFor={reviewerId}>검토자 표시 이름</label><input id={reviewerId} className="aw-text-input" value={props.outputReviewReviewerLabel} disabled={props.outputReviewFieldsDisabled} onChange={event => props.outputReviewOnReviewerLabelChange(event.currentTarget.value)}/>
    <label htmlFor={reviewerIdentityId}>검토자 식별값 · 선택</label><input id={reviewerIdentityId} className="aw-text-input" value={props.outputReviewReviewerId} disabled={props.outputReviewFieldsDisabled} onChange={event => props.outputReviewOnReviewerIdChange(event.currentTarget.value)}/>
    <label htmlFor={claimedTimeId}>검토 시각 · Unix ms · 선택</label><input id={claimedTimeId} className="aw-text-input" value={props.outputReviewClaimedTime} disabled={props.outputReviewFieldsDisabled} onChange={event => props.outputReviewOnClaimedTimeChange(event.currentTarget.value)}/>
    <Button type="button" disabled={!props.outputReviewCanRecord} onClick={props.outputReviewOnRecord}>명시적 판정 기록</Button>
    <ul>{props.outputReviewHistoryRecords.map(text => <li key={text}>{text}</li>)}</ul>
  </section>;
}
