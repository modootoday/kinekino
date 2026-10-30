import { Button } from "@/components/ui/button";
import { InformationNotice } from "../../components/notice";
type Props = {
  title: string; jobId: string; statusLabel: string; diagnostic: string; snapshotLabel: string;
  stateLabel: string; revisionLabel: string; attemptLabel: string; operationLabel: string;
  sourceSha256: string; sourceBytes: string; parameters: string;
  outputPath: string; outputSha256: string; outputBytes: string; failureLabel: string; remediation: string;
  refreshLabel: string; retryLabel: string; unsupportedLabel: string;
  refreshDisabled: boolean; retryDisabled: boolean; onRefreshJob: () => void; onRetryJobResult: () => void;
};
export function JobDetailsPanel(props: Props) {
  return <aside className="aw-inspector" aria-label={props.title}>
    <div className="aw-inspector-scroll">
      <div className="aw-inspector-heading"><h2>{props.title}</h2><p>{props.jobId}</p></div>
      <InformationNotice title={props.statusLabel} detail={props.diagnostic}/>
      <p className="aw-muted">{props.snapshotLabel}</p>
      <section className="aw-inspector-section"><h3>현재 기록</h3>
        <dl className="aw-metadata"><dt>상태</dt><dd>{props.stateLabel}</dd><dt>revision</dt><dd>{props.revisionLabel}</dd><dt>시도</dt><dd>{props.attemptLabel}</dd><dt>작업</dt><dd>{props.operationLabel}</dd><dt>진행률 · 경과 · 남은 시간</dt><dd>—</dd></dl>
      </section>
      <section className="aw-inspector-section"><h3>입력</h3>
        <dl className="aw-metadata"><dt>원본 SHA256</dt><dd>{props.sourceSha256}</dd><dt>원본 bytes</dt><dd>{props.sourceBytes}</dd></dl><pre>{props.parameters}</pre>
      </section>
      <section className="aw-inspector-section"><h3>검증된 출력 기록</h3>
        <dl className="aw-metadata"><dt>경로</dt><dd>{props.outputPath}</dd><dt>SHA256</dt><dd>{props.outputSha256}</dd><dt>bytes</dt><dd>{props.outputBytes}</dd></dl>
        <p>{props.failureLabel}</p><p>{props.remediation}</p>
      </section>
    </div>
    <div className="aw-inspector-footer">
      <Button className="aw-button" disabled={props.refreshDisabled} onClick={() => props.onRefreshJob()}>{props.refreshLabel}</Button>
      <Button className="aw-button" disabled={props.retryDisabled} onClick={() => props.onRetryJobResult()}>{props.retryLabel}</Button>
      <p className="aw-muted">{props.unsupportedLabel}</p>
    </div>
  </aside>;
}
