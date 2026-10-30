import { Button } from "@/components/ui/button";
type Props = {
  summary: string; error: boolean; uncertain: boolean;
  localError: string; localErrorPresent: boolean;
  refreshDisabled: boolean; ackDisabled: boolean;
  onRefresh: () => void; onRetryAck: () => void;
};
export function NativeStatus(props: Props) {
  return <section className="aw-read-panel" aria-label="기본 재생 상태">
    <p role="alert" hidden={!props.error}>{props.summary}</p>
    <p role="status" hidden={props.error}>{props.summary}</p>
    <p hidden={!props.uncertain}>원래 요청의 결과가 아직 확인되지 않았습니다. 같은 요청을 다시 보내지 말고 수신 처리를 확인하세요.</p>
    <p role="alert" hidden={!props.localErrorPresent}>{props.localError}</p>
    <div className="aw-read-row">
      <Button className="aw-button" disabled={props.refreshDisabled} onClick={props.onRefresh}>재생 상태 다시 확인</Button>
      <Button className="aw-button" disabled={props.ackDisabled} onClick={props.onRetryAck}>원 요청 수신 처리 확인</Button>
    </div>
  </section>;
}
