import type { ReactNode } from "react";

type Props = { productName: string; contextLabel: string; captionControls: ReactNode; captionStatus: string; captionStatusError: boolean; };
export function WindowChromeFrame(props: Props) {
  return <div className="aw-window-chrome" role="banner">
    <div className="aw-window-brand"><span className="aw-icon aw-icon-brand" aria-hidden="true"/><span className="aw-product-name">{props.productName}</span><span className="aw-window-context" title={props.contextLabel}>{props.contextLabel}</span></div>
    <span className="aw-window-caption-status" title={props.captionStatus} role="status" hidden={props.captionStatusError}>{props.captionStatus}</span>
    <span className="aw-window-caption-status" title={props.captionStatus} role="alert" hidden={!props.captionStatusError}>{props.captionStatus}</span>
    <div className="aw-window-captions" aria-label="창 제어">{props.captionControls}</div>
  </div>;
}
