import type { ReactNode } from "react";

type Props = {
  sourceLabel: string; stateLabel: string; modeLabel: string;
  canvas: ReactNode; tools: ReactNode; viewModes: ReactNode;
  transport: ReactNode; trackingFilmstrip: ReactNode;
};

export function SubjectsWorkspace(props: Props) {
  return <section className="aw-workspace aw-subject-workspace" aria-label="피사체 인식 작업">
    <header className="aw-workspace-heading"><h1>피사체 인식</h1><p>영상에서 피사체를 탐지·분할하고 추적 결과를 검토합니다.</p><span className="aw-mode-label">{props.modeLabel}</span><p>{props.sourceLabel}</p></header>
    <div className="aw-subject-preview">
      <div className="aw-subject-canvas" aria-label="원본 프레임과 피사체 마스크">{props.canvas}</div>
      <div className="aw-subject-tools-position">{props.tools}</div>
      <div className="aw-subject-modes-position">{props.viewModes}</div>
    </div>
    <div className="aw-subject-transport">{props.transport}</div>
    <div className="aw-subject-tracking" aria-label="추적 썸네일과 재검토 구간">{props.trackingFilmstrip}</div>
    <p className="aw-muted" role="status">{props.stateLabel}</p>
  </section>;
}
