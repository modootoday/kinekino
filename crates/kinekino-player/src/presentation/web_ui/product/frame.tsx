import type { ReactNode } from "react";
type Props = { workspace: ReactNode; };
export function ProductFrame(props: Props) {
  return <div className="aw-product-route-root"><div className="aw-product-route-workspace">{props.workspace}</div></div>;
}
