import { Button } from "@/components/ui/button";

type Props = { productName: string; connectionLabel: string; };
export function ViewingStart(props: Props) {
  return <section className="aw-viewing-start" aria-label="시청 시작">
    <div className="aw-start-content">
      <span className="aw-start-mark" aria-hidden="true"/>
      <h1 className="aw-start-title">{props.productName}</h1>
      <div className="aw-start-actions">
        <Button className="aw-button aw-button-outline-accent" disabled={true}><span className="aw-icon aw-icon-folder" aria-hidden="true"/>파일 열기</Button>
        <Button className="aw-button" disabled={true}><span className="aw-icon aw-icon-search" aria-hidden="true"/>작품 탐색</Button>
      </div>
      <p className="aw-start-shortcut">Ctrl+O로 파일 열기</p>
      <p className="aw-connection-status" role="status">{props.connectionLabel}</p>
    </div>
  </section>;
}
