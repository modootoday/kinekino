import { Button } from "@/components/ui/button";
import { JobRow } from "./job-row";
type Props = { modeLabel: string; rowId: string; jobTitle: string; sourceLabel: string; inputLabel: string; statusLabel: string; progressLabel: string; outputLabel: string; selected: boolean; };
export function JobsWorkspace(props: Props) {
  return <section className="aw-workspace" aria-label="작업 대기열">
    <header className="aw-workspace-heading"><h1>작업 대기열</h1><p>추가한 작업을 순서대로 처리합니다.</p><span className="aw-mode-label">{props.modeLabel}</span></header>
    <div className="aw-job-toolbar"><div className="aw-filter-group" aria-label="작업 상태 필터"><Button className="aw-filter" disabled={true}>전체</Button><Button className="aw-filter" disabled={true}>실행 중</Button><Button className="aw-filter" disabled={true}>입력 필요</Button><Button className="aw-filter" disabled={true}>실패</Button><Button className="aw-filter" disabled={true}>완료</Button></div><div className="aw-job-run"><span className="aw-muted">동시 실행 수 —</span><Button className="aw-button aw-button-primary" disabled={true}>전체 시작</Button></div></div>
    <div className="aw-job-table" role="table" aria-label="작업 목록"><div className="aw-job-table-head" role="row"><span role="columnheader">작업</span><span role="columnheader">입력</span><span role="columnheader">상태</span><span role="columnheader">진행</span><span role="columnheader">출력</span></div><JobRow rowId={props.rowId} title={props.jobTitle} sourceLabel={props.sourceLabel} inputLabel={props.inputLabel} statusLabel={props.statusLabel} progressLabel={props.progressLabel} outputLabel={props.outputLabel} selected={props.selected}/></div>
  </section>;
}
