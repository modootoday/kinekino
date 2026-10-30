import { TransportBar } from "../components/transport-bar";
import { ViewingPlayerFrame } from "./player-frame";
import { ViewingStart } from "./start-screen";
import { DisabledWindowCaptionControls } from "../components/disabled-window-caption-controls";

type Props = { productName: string; connectionLabel: string; timeLabel: string; videoSemanticId: string; };
export function ViewingPlayer(props: Props) {
  return <ViewingPlayerFrame captionStatus="" captionStatusError={false} captionControls={<DisabledWindowCaptionControls/>} fullscreen={false} frameClass="aw-player-shell" startScreen={<ViewingStart productName={props.productName} connectionLabel={props.connectionLabel}/>} productName={props.productName} connectionLabel={props.connectionLabel} contextLabel="" videoSemanticId={props.videoSemanticId} transport={<TransportBar timeLabel={props.timeLabel}/>}/>;
}
