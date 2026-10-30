import { Button } from "@/components/ui/button";
type Props = { rowId: string; title: string; sourceLabel: string; inputLabel: string; statusLabel: string; progressLabel: string; outputLabel: string; selected: boolean; };
export function JobRow(props: Props) {
  return <div id={props.rowId} className="aw-job-row" role="row" aria-selected={props.selected}>
    <div className="aw-job-task" role="cell"><span className="aw-icon aw-icon-subtitles" aria-hidden="true"/><div className="aw-job-task-label"><span className="aw-job-title">{props.title}</span><span className="aw-muted">{props.sourceLabel}</span></div></div>
    <div className="aw-job-cell" role="cell">{props.inputLabel}</div>
    <div className="aw-job-cell" role="cell">{props.statusLabel}</div>
    <div className="aw-job-cell aw-numeric" role="cell">{props.progressLabel}</div>
    <div className="aw-job-cell" role="cell"><Button className="aw-link-button" disabled={true} aria-label="출력 조회 · SDK 연결 필요">{props.outputLabel}</Button></div>
  </div>;
}
