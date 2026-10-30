import type { ReactNode } from "react";
import { StudioFrame } from "./frame";
import { StudioNavigation } from "./navigation";
import { InspectorDock } from "./inspector-dock";
import { QueueFooter } from "./queue-footer";

type Props = {
  captionMinimizeDisabled: boolean; captionMaximizeDisabled: boolean; captionCloseDisabled: boolean;
  captionMaximizeLabel: string; captionMaximizeIconClass: string; captionStatus: string; captionStatusError: boolean;
  onWindowMinimize: () => void; onWindowToggleMaximize: () => void; onWindowUserClose: () => void;
  productName: string; contextLabel: string; workspaceId: string;
  workspace: ReactNode; inspector: ReactNode; inspectorTitle: string;
  narrow: boolean; inspectorOpen: boolean;
  queueSummaryLabel: string; queueDetailsLabel: string;
  onNavigate: (workspaceId: string) => void;
  onInspectorOpenChange: (open: boolean) => void;
};

export function StudioApplication(props: Props) {
  return <StudioFrame
    captionMinimizeDisabled={props.captionMinimizeDisabled} captionMaximizeDisabled={props.captionMaximizeDisabled} captionCloseDisabled={props.captionCloseDisabled}
    captionMaximizeLabel={props.captionMaximizeLabel} captionMaximizeIconClass={props.captionMaximizeIconClass} captionStatus={props.captionStatus} captionStatusError={props.captionStatusError}
    onWindowMinimize={props.onWindowMinimize} onWindowToggleMaximize={props.onWindowToggleMaximize} onWindowUserClose={props.onWindowUserClose}
    productName={props.productName}
    contextLabel={props.contextLabel}
    navigation={<StudioNavigation selectedId={props.workspaceId} onNavigate={props.onNavigate}/>}
    workspace={props.workspace}
    inspector={<InspectorDock narrow={props.narrow} open={props.inspectorOpen} title={props.inspectorTitle} onOpenChange={props.onInspectorOpenChange}>{props.inspector}</InspectorDock>}
    queue={<QueueFooter summaryLabel={props.queueSummaryLabel} detailsLabel={props.queueDetailsLabel}/>}
  />;
}
