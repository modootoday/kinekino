import type { ReactNode } from "react";
import { Button } from "@/components/ui/button";

type Props = {
  trackId: string; userLabel: string; hasUserLabel: boolean; reviewLabel: string; thumbnail: ReactNode;
  selected: boolean; canSelect: boolean; canOpenMenu: boolean;
  selectReason: string; menuReason: string;
  onSelect: () => void; onOpenMenu: () => void;
};

export function SubjectRow(props: Props) {
  return <li className="aw-subject-row" data-selected={props.selected}>
    <Button className="aw-subject-row-select" aria-pressed={props.selected} disabled={!props.canSelect} title={props.selectReason} onClick={() => props.onSelect()}>
      <span className="aw-subject-thumbnail" aria-hidden="true">{props.thumbnail}</span>
      <span className="aw-subject-row-text"><span hidden={!props.hasUserLabel}>{props.userLabel}</span><span hidden={props.hasUserLabel}>이름 미지정</span><span className="aw-muted">{props.trackId}</span><span className="aw-muted">{props.reviewLabel}</span></span>
    </Button>
    <Button className="aw-icon-button" aria-label="피사체 작업 메뉴" disabled={!props.canOpenMenu} title={props.menuReason} onClick={() => props.onOpenMenu()}><span className="aw-icon aw-subject-icon-menu" aria-hidden="true"/></Button>
  </li>;
}
