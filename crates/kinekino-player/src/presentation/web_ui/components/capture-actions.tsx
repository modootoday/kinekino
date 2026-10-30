import { Button } from "@/components/ui/button";
type Props = {
  canSave: boolean; canAccept: boolean; busy: boolean; canConsume: boolean; canDiscard: boolean; canRetry: boolean;
  resultLabel: string;
  saveLabel: string; acceptLabel: string; consumeLabel: string; discardLabel: string; retryLabel: string;
  canRevert: boolean; revertLabel: string; onRevert: () => void;
  onSave: () => void; onAccept: () => void; onConsume: () => void; onDiscard: () => void; onRetry: () => void;
};
export function CaptureActions(props: Props) {
  return <div className="aw-inspector-footer">
    <Button className="aw-button" disabled={!props.canSave || props.busy} onClick={() => props.onSave()}>{props.saveLabel}</Button>
    <Button className="aw-button aw-button-primary" disabled={!props.canAccept || props.busy} onClick={() => props.onAccept()}>{props.acceptLabel}</Button>
    <Button className="aw-button" disabled={!props.canConsume} onClick={() => props.onConsume()}>{props.consumeLabel}</Button>
    <Button className="aw-button" disabled={!props.canDiscard} onClick={() => props.onDiscard()}>{props.discardLabel}</Button>
    <Button className="aw-button" disabled={!props.canRetry} onClick={() => props.onRetry()}>{props.retryLabel}</Button>
    <Button className="aw-button" disabled={!props.canRevert} onClick={() => props.onRevert()}>{props.revertLabel}</Button>
    <p role="status">{props.resultLabel}</p>
  </div>;
}
