import { useId } from "react";

type Props = {
  label: string; value: string; max: string; disabled: boolean;
  previewId: string; preview: string;
  onDraft: (value: string) => void; onCommit: (value: string) => void;
};
export function Range(props: Props) {
  const id = useId();
  return <div className="aw-transport-group">
    <label htmlFor={id}>{props.label}</label>
    <input id={id} className="aw-range-input" type="range" data-meari-range-parts="compact" min="0" max={props.max} step="1"
      value={props.value} aria-valuemin="0" aria-valuemax={props.max} aria-valuenow={props.value} disabled={props.disabled}
      aria-describedby={props.previewId}
      onChange={event => props.onDraft(event.currentTarget.value)}
      onValueCommit={props.onCommit}/>
    <span id={props.previewId}>{props.preview}</span>
  </div>;
}
