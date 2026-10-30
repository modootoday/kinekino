import { Button } from "@/components/ui/button";
import { InformationNotice } from "../../components/notice";
import { ReadJobRow } from "./read-row";
type Props = {
  rows: { key: string; operation: string; source: string; state: string; output: string; selected: boolean; disabled: boolean; }[];
  status: string; diagnostic: string; snapshot: string; refreshDisabled: boolean; retryDisabled: boolean; connectionRetryDisabled: boolean;
  onRefreshJobs: () => void; onRetryJobsResult: () => void; onRetryJobsConnection: () => void; onSelectJob: (jobId: string) => void;
};
export function ReadJobsWorkspace(props: Props) {
  return <section className="aw-workspace" aria-label="작업 대기열">
    <header className="aw-workspace-heading"><h1>작업 대기열</h1><p>작업을 선택해 입력과 결과를 확인합니다.</p></header>
    <InformationNotice title={props.status} detail={props.diagnostic}/>
    <p className="aw-muted">{props.snapshot}</p>
    <div className="aw-job-toolbar"><Button className="aw-button" disabled={props.refreshDisabled} onClick={() => props.onRefreshJobs()}>목록 새로고침</Button><Button className="aw-button" disabled={props.retryDisabled} onClick={() => props.onRetryJobsResult()}>조회 결과 다시 확인</Button><Button className="aw-button" disabled={props.connectionRetryDisabled} onClick={() => props.onRetryJobsConnection()}>화면 연결 다시 시도</Button><span className="aw-muted">목록은 자동 갱신되지 않습니다.</span></div>
    <div className="aw-job-table" role="table" aria-label="조회한 작업 목록">
      <div className="aw-job-table-head" role="row"><span role="columnheader">작업</span><span role="columnheader">입력</span><span role="columnheader">상태</span><span role="columnheader">진행</span><span role="columnheader">출력</span></div>
      <div>{props.rows.map(row => <ReadJobRow key={row.key} jobId={row.key} operation={row.operation} source={row.source} state={row.state} output={row.output} selected={row.selected} disabled={row.disabled} onSelect={props.onSelectJob}/>)}</div>
    </div>
  </section>;
}
