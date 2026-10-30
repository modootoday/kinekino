import { Button } from "@/components/ui/button";
type Props = { rows: { id: string; numberLabel: string; startLabel: string; endLabel: string; original: string; proposed: string; selected: boolean; }[]; onSelect: (id: string) => void; };
export function SubtitleCueTable(props: Props) {
  return <div className="aw-cue-table-scroll"><table className="aw-cue-table"><thead><tr><th scope="col">번호</th><th scope="col">시작 시간</th><th scope="col">종료 시간</th><th scope="col">원문</th><th scope="col">교정안</th></tr></thead><tbody>{props.rows.map(row => <tr key={row.id} aria-selected={row.selected}><td><Button className="aw-cue-select" aria-label={row.numberLabel} onClick={() => props.onSelect(row.id)}>{row.numberLabel}</Button></td><td className="aw-cue-time">{row.startLabel}</td><td className="aw-cue-time">{row.endLabel}</td><td>{row.original}</td><td className="aw-proposed-text">{row.proposed}</td></tr>)}</tbody></table></div>;
}
