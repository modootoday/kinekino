import { Button } from "@/components/ui/button";

type Props = {
  selectedMode: string; available: boolean; maskAvailable: boolean; reason: string;
  onOriginal: () => void; onMask: () => void; onComposite: () => void;
};

export function SubjectViewModes(props: Props) {
  return <div className="aw-subject-view-modes" role="group" aria-label="피사체 프레임 보기" title={props.reason}>
    <Button className="aw-button aw-subject-mode" aria-pressed={props.selectedMode === "original"} disabled={!props.available} onClick={() => props.onOriginal()}>원본</Button>
    <Button className="aw-button aw-subject-mode" aria-pressed={props.selectedMode === "mask"} disabled={!props.maskAvailable} onClick={() => props.onMask()}>마스크</Button>
    <Button className="aw-button aw-subject-mode" aria-pressed={props.selectedMode === "composite"} disabled={!props.maskAvailable} onClick={() => props.onComposite()}>합성</Button>
  </div>;
}
