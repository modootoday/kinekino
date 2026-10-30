import { Button } from "@/components/ui/button";
type Props = {
  navigationDisabled: boolean; cancelDisabled: boolean; retryDisabled: boolean;
  status: string; error: string;
  onPrevious: () => void; onNext: () => void; onCancel: () => void; onRetry: () => void;
};
export function EpisodeNavigation(props: Props) {
  return <section className="aw-read-panel" aria-label="회차 이동">
    <div className="aw-read-row">
      <Button className="aw-button" disabled={props.navigationDisabled} onClick={() => props.onPrevious()}>이전 회차</Button>
      <Button className="aw-button" disabled={props.navigationDisabled} onClick={() => props.onNext()}>다음 회차</Button>
      <Button className="aw-button" disabled={props.cancelDisabled} onClick={() => props.onCancel()}>회차 이동 취소</Button>
      <Button className="aw-button" disabled={props.retryDisabled} onClick={() => props.onRetry()}>다시 확인</Button>
    </div>
    <p role="status">{props.status}</p><p role="alert">{props.error}</p>
  </section>;
}
