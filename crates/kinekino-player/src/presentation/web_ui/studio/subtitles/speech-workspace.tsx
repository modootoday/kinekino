import type { ReactNode } from "react";

type Props = {
  modeLabel: string;
  inputStateLabel: string;
  media: ReactNode;
  waveform: ReactNode;
  editToolbar: ReactNode;
  cues: ReactNode;
};

export function SpeechWorkspace(props: Props) {
  return <section className="aw-workspace aw-speech-workspace" aria-label="음성 인식 작업">
    <header className="aw-workspace-heading"><h1>음성 인식</h1><p>오디오의 인식 결과와 원문·교정안을 구분해 검토합니다.</p><span className="aw-mode-label">{props.modeLabel}</span></header>
    <div className="aw-subtitle-media">{props.media}</div>
    <div className="aw-waveform-region" aria-label="오디오 파형과 선택 구간">{props.waveform}</div>
    <p className="aw-muted" role="status">{props.inputStateLabel}</p>
    {props.editToolbar}
    {props.cues}
  </section>;
}
