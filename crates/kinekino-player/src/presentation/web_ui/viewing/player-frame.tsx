import type { ReactNode } from "react";
import { WindowChromeFrame } from "../components/window-chrome-frame";

type Props = { captionStatus: string; captionStatusError: boolean; captionControls: ReactNode; fullscreen: boolean; frameClass: string; productName: string; connectionLabel: string; contextLabel: string; startScreen: ReactNode; videoSemanticId: string; transport: ReactNode; };
export function ViewingPlayerFrame(props: Props) {
  return <main className={props.frameClass}>
    <div className="aw-viewing-chrome" hidden={props.fullscreen}><WindowChromeFrame productName={props.productName} contextLabel={props.contextLabel} captionControls={props.captionControls} captionStatus={props.captionStatus} captionStatusError={props.captionStatusError}/></div>
    <div className="aw-viewing-stage">
      <video id={props.videoSemanticId} className="aw-viewing-video" tabIndex="-1" aria-hidden="true"/>
      {props.startScreen}
    </div>
    {props.transport}
  </main>;
}
