import { Button } from "@/components/ui/button";

type Props = { timeLabel: string; };
export function TransportBar(props: Props) {
  return <section className="aw-transport" aria-label="재생 제어 · 미디어 연결 필요">
    <div className="aw-seek-disabled" aria-label="재생 위치 없음"><span className="aw-seek-thumb"/></div>
    <div className="aw-transport-row">
      <div className="aw-transport-group">
        <Button className="aw-icon-button" aria-label="이전화 · 연결 필요" disabled={true}><span className="aw-icon aw-icon-previous" aria-hidden="true"/></Button>
        <Button className="aw-icon-button" aria-label="재생 · 연결 필요" disabled={true}><span className="aw-icon aw-icon-play" aria-hidden="true"/></Button>
        <Button className="aw-icon-button" aria-label="다음화 · 연결 필요" disabled={true}><span className="aw-icon aw-icon-next" aria-hidden="true"/></Button>
        <span className="aw-time-label">{props.timeLabel}</span>
      </div>
      <div className="aw-transport-group">
        <Button className="aw-icon-button" aria-label="음량 · 연결 필요" disabled={true}><span className="aw-icon aw-icon-volume" aria-hidden="true"/></Button>
        <div className="aw-volume-disabled" aria-label="음량 관측 없음"/>
        <span className="aw-divider"/>
        <Button className="aw-icon-button" aria-label="회차 목록 · 연결 필요" disabled={true}><span className="aw-icon aw-icon-episodes" aria-hidden="true"/></Button>
        <Button className="aw-icon-button" aria-label="전체화면 · 호스트 연결 필요" disabled={true}><span className="aw-icon aw-icon-fullscreen" aria-hidden="true"/></Button>
      </div>
    </div>
  </section>;
}
