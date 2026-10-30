import { Button } from "@/components/ui/button";
type Props = {
  jobId: string; operation: string; source: string; state: string; output: string;
  selected: boolean; disabled: boolean; onSelect: (jobId: string) => void;
};
export function ReadJobRow(props: Props) {
  return <div className="aw-job-row" role="row" aria-selected={props.selected}>
    <div className="aw-job-task" role="cell"><Button className="aw-link-button" disabled={props.disabled} onClick={() => props.onSelect(props.jobId)} aria-label={props.jobId}>{props.operation}</Button></div>
    <div className="aw-job-cell" role="cell">{props.source}</div>
    <div className="aw-job-cell" role="cell">{props.state}</div>
    <div className="aw-job-cell aw-numeric" role="cell">—</div>
    <div className="aw-job-cell" role="cell">{props.output}</div>
  </div>;
}
