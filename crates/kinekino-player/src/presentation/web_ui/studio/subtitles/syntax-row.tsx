import { Button } from "@/components/ui/button";
type Props = { cueId: string; numberLabel: string; timeLabel: string; text: string; syntax: string; selected: boolean; onSelect: (cueId: string) => void; };
export function SubtitleSyntaxRow(props: Props) {
  return <article className="aw-syntax-row" aria-current={props.selected}><Button className="aw-syntax-number" aria-label={props.numberLabel} onClick={() => props.onSelect(props.cueId)}>{props.numberLabel}</Button><div className="aw-syntax-content"><p className="aw-cue-time">{props.timeLabel}</p><p className="aw-syntax-text">{props.text}</p><pre className="aw-syntax-source">{props.syntax}</pre></div></article>;
}
