import type { ReactNode } from "react";
import { WindowChromeFrame } from "../components/window-chrome-frame";
import { NativeCaptionControls } from "../components/window-caption-controls";
type Props = { productName: string; contextLabel: string; navigation: ReactNode; workspace: ReactNode; inspector: ReactNode; queue: ReactNode;
  captionMinimizeDisabled: boolean; captionMaximizeDisabled: boolean; captionCloseDisabled: boolean;
  captionMaximizeLabel: string; captionMaximizeIconClass: string; captionStatus: string; captionStatusError: boolean;
  onWindowMinimize: () => void; onWindowToggleMaximize: () => void; onWindowUserClose: () => void; };
export function StudioFrame(props: Props) {
  return <main className="aw-studio-shell"><WindowChromeFrame productName={props.productName} contextLabel={props.contextLabel} captionStatus={props.captionStatus} captionStatusError={props.captionStatusError}
    captionControls={<NativeCaptionControls minimizeDisabled={props.captionMinimizeDisabled} maximizeDisabled={props.captionMaximizeDisabled} closeDisabled={props.captionCloseDisabled}
      maximizeLabel={props.captionMaximizeLabel} maximizeIconClass={props.captionMaximizeIconClass} onMinimize={props.onWindowMinimize} onMaximize={props.onWindowToggleMaximize} onClose={props.onWindowUserClose}/>}/>
    <div className="aw-studio-layout">{props.navigation}<div className="aw-workspace-slot">{props.workspace}</div>{props.inspector}</div>{props.queue}</main>;
}
