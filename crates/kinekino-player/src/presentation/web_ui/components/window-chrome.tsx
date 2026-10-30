import { WindowChromeFrame } from "./window-chrome-frame";
import { DisabledWindowCaptionControls } from "./disabled-window-caption-controls";

type Props = { productName: string; contextLabel: string; };
export function WindowChrome(props: Props) {
  return <WindowChromeFrame productName={props.productName} contextLabel={props.contextLabel} captionStatus="" captionStatusError={false} captionControls={<DisabledWindowCaptionControls/>}/>;
}
