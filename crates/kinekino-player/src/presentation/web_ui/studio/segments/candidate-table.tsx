import { Button } from "@/components/ui/button";

type Props = {
  countLabel: string; emptyStateLabel: string;
  rows: { id: string; kind: "opening" | "ending" | "other"; kindLabel: string; startLabel: string; endLabel: string; durationLabel: string; stateLabel: string; selected: boolean; canEdit: boolean; editReason: string; canDelete: boolean; deleteReason: string; }[];
  onEdit: (id: string) => void; onDelete: (id: string) => void;
};
export function SegmentCandidateTable(props: Props) {
  return <section className="aw-segment-candidates" aria-label="감지된 구간 목록">
    <header><h2>감지된 구간 목록</h2><span className="aw-muted">{props.countLabel}</span></header>
    <div className="aw-segment-table-scroll"><table className="aw-segment-table"><thead><tr><th scope="col">유형</th><th scope="col">시작 시간</th><th scope="col">종료 시간</th><th scope="col">길이</th><th scope="col">상태</th><th scope="col">작업</th></tr></thead><tbody>
      {props.rows.map(row => <tr key={row.id} aria-selected={row.selected}>
        <td><span className="aw-segment-kind" data-kind={row.kind}><span className="aw-segment-dot" aria-hidden="true"/>{row.kindLabel}</span></td>
        <td className="aw-segment-time">{row.startLabel}</td><td className="aw-segment-time">{row.endLabel}</td><td className="aw-segment-time">{row.durationLabel}</td>
        <td><span className="aw-segment-badge" data-kind={row.kind}>{row.stateLabel}</span></td>
        <td><div className="aw-segment-actions"><Button className="aw-button aw-button-compact" disabled={!row.canEdit} title={row.editReason} onClick={() => props.onEdit(row.id)}>구간 수정</Button><Button className="aw-icon-button" aria-label="선택한 구간 삭제" disabled={!row.canDelete} title={row.deleteReason} onClick={() => props.onDelete(row.id)}><span className="aw-icon aw-icon-segment-delete" aria-hidden="true"/></Button></div></td>
      </tr>)}
    </tbody></table></div>
    {props.rows.length === 0 && <p className="aw-muted" role="status">{props.emptyStateLabel}</p>}
  </section>;
}
