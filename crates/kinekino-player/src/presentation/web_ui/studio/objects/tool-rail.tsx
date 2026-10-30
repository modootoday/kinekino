import { Button } from "@/components/ui/button";

type Props = {
  selectedTool: string; canSelect: boolean; canInclude: boolean;
  canExclude: boolean; canBrush: boolean; canErase: boolean;
  selectReason: string; includeReason: string; excludeReason: string;
  brushReason: string; eraseReason: string;
  onSelect: () => void; onInclude: () => void; onExclude: () => void;
  onBrush: () => void; onErase: () => void;
};

export function SubjectToolRail(props: Props) {
  return <div className="aw-subject-tool-rail" role="group" aria-label="피사체 마스크 편집 도구">
    <Button className="aw-icon-button aw-subject-tool" aria-label="피사체 선택" aria-pressed={props.selectedTool === "select"} disabled={!props.canSelect} title={props.selectReason} onClick={() => props.onSelect()}><span className="aw-icon aw-subject-icon-select" aria-hidden="true"/></Button>
    <Button className="aw-icon-button aw-subject-tool" aria-label="포함 점 추가" aria-pressed={props.selectedTool === "include"} disabled={!props.canInclude} title={props.includeReason} onClick={() => props.onInclude()}><span className="aw-icon aw-subject-icon-include" aria-hidden="true"/></Button>
    <Button className="aw-icon-button aw-subject-tool" aria-label="제외 점 추가" aria-pressed={props.selectedTool === "exclude"} disabled={!props.canExclude} title={props.excludeReason} onClick={() => props.onExclude()}><span className="aw-icon aw-subject-icon-exclude" aria-hidden="true"/></Button>
    <Button className="aw-icon-button aw-subject-tool" aria-label="마스크 브러시" aria-pressed={props.selectedTool === "brush"} disabled={!props.canBrush} title={props.brushReason} onClick={() => props.onBrush()}><span className="aw-icon aw-subject-icon-brush" aria-hidden="true"/></Button>
    <Button className="aw-icon-button aw-subject-tool" aria-label="마스크 지우개" aria-pressed={props.selectedTool === "erase"} disabled={!props.canErase} title={props.eraseReason} onClick={() => props.onErase()}><span className="aw-icon aw-subject-icon-erase" aria-hidden="true"/></Button>
  </div>;
}
