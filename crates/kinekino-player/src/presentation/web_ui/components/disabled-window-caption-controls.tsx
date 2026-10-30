import { Button } from "@/components/ui/button";

type Props = {};
export function DisabledWindowCaptionControls() {
  return <div className="aw-window-captions" aria-label="창 제어 · 호스트 연결 필요">
      <Button className="aw-caption" aria-label="최소화 · 호스트 연결 필요" disabled={true}><span className="aw-icon aw-icon-minimize" aria-hidden="true"/></Button>
      <Button className="aw-caption" aria-label="최대화 · 호스트 연결 필요" disabled={true}><span className="aw-icon aw-icon-maximize" aria-hidden="true"/></Button>
      <Button className="aw-caption aw-caption-close" aria-label="닫기 · 호스트 연결 필요" disabled={true}><span className="aw-icon aw-icon-close" aria-hidden="true"/></Button>
  </div>;
}
