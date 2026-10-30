import { useId } from "react";
import { Button } from "@/components/ui/button";
type Props = {
  rawReviewPackage: string; rawReviewItem: string; rawReviewNote: string;
  rawReviewSource: string; rawReviewCandidate: string; rawReviewStatus: string;
  rawReviewError: string; rawReviewVerdict: string; rawReviewSaved: string;
  rawReviewModel: string; rawReviewSelection: string; rawReviewCriteria: string[]; rawReviewRangeStatus: string;
  rawReviewRetainedIdentity: string; rawReviewRetainedResponse: string; rawReviewAttempt: string; rawReviewRecovery: string;
  rawReviewCanInspectRetained: boolean; rawReviewCanHistoryRetained: boolean; rawReviewCanReleaseRetained: boolean;
  rawReviewOnInspectRetained: () => void; rawReviewOnHistoryRetained: () => void; rawReviewOnReleaseRetained: () => void;
  rawReviewItems: string[]; rawReviewIssues: string[]; rawReviewHistory: string[];
  rawReviewFieldsDisabled: boolean; rawReviewCanRead: boolean; rawReviewCanRecord: boolean;
  rawReviewOnPackageChange: (text: string) => void; rawReviewOnItemChange: (text: string) => void; rawReviewOnNoteChange: (text: string) => void;
  rawReviewOnOpenPackage: () => void; rawReviewOnOpenItem: () => void;
  rawReviewOnReadSource: () => void; rawReviewOnReadCandidate: () => void;
  rawReviewOnHistory: () => void; rawReviewOnRecord: () => void;
  rawReviewOnUsable: () => void; rawReviewOnNeedsChanges: () => void; rawReviewOnDeferred: () => void;
};
export function RawReviewPanel(props: Props) {
  const packageId = useId();
  const itemId = useId();
  return <section className="aw-inspector-section" aria-label="원본 후보와 판정 이력 다시 열기">
    <h4>원본·후보와 판정 이력</h4>
    <p>원래 검토 자료와 후보 코드를 입력하세요. 현재 제안의 편집·채택 판정과 구분해 표시합니다.</p>
    <p role="status">{props.rawReviewStatus}</p><p role="alert">{props.rawReviewError}</p>
    <label htmlFor={packageId}>검토 자료 코드</label><input id={packageId} className="aw-text-input" value={props.rawReviewPackage} disabled={props.rawReviewFieldsDisabled} onChange={event => props.rawReviewOnPackageChange(event.currentTarget.value)}/>
    <Button type="button" disabled={props.rawReviewFieldsDisabled} onClick={props.rawReviewOnOpenPackage}>검토 자료 다시 열기</Button>
    <ul>{props.rawReviewItems.map(text => <li key={text}>{text}</li>)}</ul>
    <label htmlFor={itemId}>자료 안의 후보 코드</label><input id={itemId} className="aw-text-input" value={props.rawReviewItem} disabled={props.rawReviewFieldsDisabled} onChange={event => props.rawReviewOnItemChange(event.currentTarget.value)}/>
    <Button type="button" disabled={props.rawReviewFieldsDisabled} onClick={props.rawReviewOnOpenItem}>원 후보 확인</Button>
    <p role="status">{props.rawReviewSelection}</p><p>{props.rawReviewModel}</p>
    <ul>{props.rawReviewCriteria.map(text => <li key={text}>{text}</li>)}</ul>
    <ul>{props.rawReviewIssues.map(text => <li key={text}>{text}</li>)}</ul>
    <p role="status">{props.rawReviewRangeStatus}</p>
    <Button type="button" disabled={!props.rawReviewCanRead} onClick={props.rawReviewOnReadSource}>원 요청·원문 읽기</Button><pre>{props.rawReviewSource}</pre>
    <Button type="button" disabled={!props.rawReviewCanRead} onClick={props.rawReviewOnReadCandidate}>원 후보 읽기</Button><pre>{props.rawReviewCandidate}</pre>
    <Button type="button" disabled={!props.rawReviewCanRead} onClick={props.rawReviewOnHistory}>같은 후보 판정 이력 읽기</Button>
    <ul>{props.rawReviewHistory.map(text => <li key={text}>{text}</li>)}</ul>
    <p>{props.rawReviewAttempt}</p>
    <pre>{props.rawReviewRetainedIdentity}</pre><pre>{props.rawReviewRetainedResponse}</pre><pre>{props.rawReviewRecovery}</pre>
    <Button type="button" disabled={!props.rawReviewCanInspectRetained} onClick={props.rawReviewOnInspectRetained}>보관한 자료 다시 확인</Button>
    <Button type="button" disabled={!props.rawReviewCanHistoryRetained} onClick={props.rawReviewOnHistoryRetained}>보관한 후보 이력 읽기</Button>
    <Button type="button" disabled={!props.rawReviewCanReleaseRetained} onClick={props.rawReviewOnReleaseRetained}>확인한 보관 결과 정리</Button>
  </section>;
}
