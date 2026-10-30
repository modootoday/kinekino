import type { ReactNode } from "react";

type Props = { modeLabel: string; sourceLabel: string; media: ReactNode; timeline: ReactNode; candidates: ReactNode; };
export function SegmentsWorkspace(props: Props) {
  return <section className="aw-workspace" aria-label="오프닝 엔딩 구간 감지">
    <header className="aw-workspace-heading"><h1>구간 감지 · 오프닝 엔딩 제어</h1><p>감지 후보를 검토하고 현재 회차의 구간을 관리합니다.</p><span className="aw-mode-label">{props.modeLabel}</span><p>{props.sourceLabel}</p></header>
    <div className="aw-segment-media">{props.media}</div>
    <div className="aw-segment-timeline" aria-label="영상 썸네일과 감지 구간 시간축">{props.timeline}</div>
    {props.candidates}
  </section>;
}
