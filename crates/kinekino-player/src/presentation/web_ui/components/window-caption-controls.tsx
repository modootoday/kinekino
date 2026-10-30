import { Button } from "@/components/ui/button";
type Props = {
  minimizeDisabled: boolean; maximizeDisabled: boolean; closeDisabled: boolean;
  maximizeLabel: string; maximizeIconClass: string;
  onMinimize: () => void; onMaximize: () => void; onClose: () => void;
};
export function NativeCaptionControls(props: Props) {
  return <div className="aw-window-captions">
    <Button className="aw-caption" aria-label="최소화" disabled={props.minimizeDisabled} onClick={props.onMinimize}><span className="aw-icon aw-icon-minimize" aria-hidden="true"/></Button>
    <Button className="aw-caption" aria-label={props.maximizeLabel} disabled={props.maximizeDisabled} onClick={props.onMaximize}><span className={props.maximizeIconClass} aria-hidden="true"/></Button>
    <Button className="aw-caption aw-caption-close" aria-label="닫기" disabled={props.closeDisabled} onClick={props.onClose}><span className="aw-icon aw-icon-close" aria-hidden="true"/></Button>
  </div>;
}
