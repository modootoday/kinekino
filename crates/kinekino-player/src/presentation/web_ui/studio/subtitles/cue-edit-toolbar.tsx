import { Button } from "@/components/ui/button";

type Props = {
  canSplit: boolean; splitReason: string;
  canMerge: boolean; mergeReason: string;
  canAdd: boolean; addReason: string;
  canDelete: boolean; deleteReason: string;
  canUndo: boolean; undoReason: string;
  canRedo: boolean; redoReason: string;
  onSplit: () => void; onMerge: () => void; onAdd: () => void;
  onDelete: () => void; onUndo: () => void; onRedo: () => void;
};

export function CueEditToolbar(props: Props) {
  return <div className="aw-cue-edit-toolbar" role="group" aria-label="자막 편집">
    <div className="aw-cue-edit-group" role="group" aria-label="cue 편집">
      <Button className="aw-button aw-button-compact" disabled={!props.canSplit} title={props.splitReason} onClick={() => props.onSplit()}><span className="aw-icon aw-icon-cue-split" aria-hidden="true"/>분할</Button>
      <Button className="aw-button aw-button-compact" disabled={!props.canMerge} title={props.mergeReason} onClick={() => props.onMerge()}><span className="aw-icon aw-icon-cue-merge" aria-hidden="true"/>병합</Button>
      <Button className="aw-button aw-button-compact" disabled={!props.canAdd} title={props.addReason} onClick={() => props.onAdd()}><span className="aw-icon aw-icon-cue-add" aria-hidden="true"/>추가</Button>
      <Button className="aw-button aw-button-compact" disabled={!props.canDelete} title={props.deleteReason} onClick={() => props.onDelete()}><span className="aw-icon aw-icon-cue-delete" aria-hidden="true"/>삭제</Button>
    </div>
    <span className="aw-cue-edit-divider" aria-hidden="true"/>
    <div className="aw-cue-edit-group" role="group" aria-label="편집 기록">
      <Button className="aw-button aw-button-compact" disabled={!props.canUndo} title={props.undoReason} onClick={() => props.onUndo()}><span className="aw-icon aw-icon-cue-undo" aria-hidden="true"/>되돌리기</Button>
      <Button className="aw-button aw-button-compact" disabled={!props.canRedo} title={props.redoReason} onClick={() => props.onRedo()}><span className="aw-icon aw-icon-cue-redo" aria-hidden="true"/>다시 실행</Button>
    </div>
  </div>;
}
