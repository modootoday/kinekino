import type { ReactNode } from "react";
import { StudioApplication } from "../application";
type Props = {
  captionMinimizeDisabled: boolean; captionMaximizeDisabled: boolean; captionCloseDisabled: boolean;
  captionMaximizeLabel: string; captionMaximizeIconClass: string; captionStatus: string; captionStatusError: boolean;
  onWindowMinimize: () => void; onWindowToggleMaximize: () => void; onWindowUserClose: () => void;
  productName: string; contextLabel: string; workspace: ReactNode; inspector: ReactNode;
  narrow: boolean; inspectorOpen: boolean; queueSummaryLabel: string;
  onNavigate: (workspaceId: string) => void; onInspectorOpenChange: (open: boolean) => void;
};
export function JobsApplication(props: Props) {
  return <StudioApplication productName={props.productName} contextLabel={props.contextLabel}
    captionMinimizeDisabled={props.captionMinimizeDisabled} captionMaximizeDisabled={props.captionMaximizeDisabled} captionCloseDisabled={props.captionCloseDisabled}
    captionMaximizeLabel={props.captionMaximizeLabel} captionMaximizeIconClass={props.captionMaximizeIconClass} captionStatus={props.captionStatus} captionStatusError={props.captionStatusError}
    onWindowMinimize={props.onWindowMinimize} onWindowToggleMaximize={props.onWindowToggleMaximize} onWindowUserClose={props.onWindowUserClose}
    workspaceId="JOB" workspace={props.workspace} inspector={<div>{props.inspector}</div>} inspectorTitle="작업 정보"
    narrow={props.narrow} inspectorOpen={props.inspectorOpen}
    queueSummaryLabel={props.queueSummaryLabel} queueDetailsLabel="현재 조회한 작업 목록"
    onNavigate={props.onNavigate} onInspectorOpenChange={props.onInspectorOpenChange}/>;
}
