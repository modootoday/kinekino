import type { ReactNode } from "react";
import { Button } from "@/components/ui/button";

type Props = {
  rows: ReactNode; stateLabel: string; canAdd: boolean; addReason: string;
  onAdd: () => void;
};

export function SubjectList(props: Props) {
  return <section className="aw-inspector-section" aria-label="피사체 목록">
    <div className="aw-inspector-heading"><h2>피사체 목록</h2><Button className="aw-button" disabled={!props.canAdd} title={props.addReason} onClick={() => props.onAdd()}><span className="aw-icon aw-subject-icon-include" aria-hidden="true"/>새 피사체 추가</Button></div>
    <ul className="aw-subject-list">{props.rows}</ul>
    <p role="status" className="aw-muted">{props.stateLabel}</p>
    <p className="aw-muted">이름은 사용자 라벨이며 신원 인식 결과가 아닙니다.</p>
  </section>;
}
