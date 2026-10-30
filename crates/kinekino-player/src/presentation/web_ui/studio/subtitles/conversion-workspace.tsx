import type { ReactNode } from "react";
type Props = { modeLabel: string; media: ReactNode; sourcePreview: ReactNode; outputPreview: ReactNode; lossNotice: ReactNode; };
export function ConversionWorkspace(props: Props) {
  return <section className="aw-workspace" aria-label="자막 변환"><header className="aw-workspace-heading"><h1>자막 변환</h1><p>원본 문법과 변환 미리보기를 비교하고 손실 항목을 확인합니다.</p><span className="aw-mode-label">{props.modeLabel}</span></header><div className="aw-subtitle-media">{props.media}</div><div className="aw-conversion-comparison"><section className="aw-syntax-panel" aria-label="원본 자막 문법">{props.sourcePreview}</section><section className="aw-syntax-panel" aria-label="출력 자막 문법 미리보기">{props.outputPreview}</section></div>{props.lossNotice}</section>;
}
