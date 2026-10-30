import type { ReactNode } from "react";
type Props = { modeLabel: string; media: ReactNode; waveform: ReactNode; cues: ReactNode; };
export function CorrectionWorkspace(props: Props) {
  return <section className="aw-workspace aw-correction-workspace" aria-label="자막 교정 작업"><header className="aw-workspace-heading"><h1>자막 교정</h1><p>원문을 보존하며 선택한 교정안을 비교합니다.</p><span className="aw-mode-label">{props.modeLabel}</span></header><div className="aw-subtitle-media">{props.media}</div><div className="aw-waveform-region" aria-label="선택 구간 파형">{props.waveform}</div>{props.cues}</section>;
}
