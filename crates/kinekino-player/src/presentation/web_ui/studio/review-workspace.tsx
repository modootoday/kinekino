import type { ReactNode } from "react";
type Props = { editor: ReactNode; rawReview: ReactNode; };
export function ReviewWorkspace(props: Props) {
  return <div className="aw-workspace">{props.editor}{props.rawReview}</div>;
}
