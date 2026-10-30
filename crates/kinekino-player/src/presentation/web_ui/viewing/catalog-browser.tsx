import { Button } from "@/components/ui/button";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
type Props = {
  currentEpisode: string; connectingEpisode: string; connectionLabel: string; seriesTitle: string; onClose: () => void;
  diagnosticsExpanded: boolean;
  series: { key: string; title: string; disabled: boolean; }[];
  episodes: { key: string; title: string; disabled: boolean; }[];
  selectedSeries: string; selectedEpisode: string; status: string; error: string; busy: boolean;
  pollDisabled: boolean; cancelDisabled: boolean; readSeriesDisabled: boolean; readEpisodesDisabled: boolean;
  inspectDisabled: boolean; playDisabled: boolean; releaseDisabled: boolean; ackRetryDisabled: boolean;
  volumeDraft: string;
  onAiring: () => void; onCompleted: () => void; onMovies: () => void;
  onSeries: (key: string) => void; onEpisode: (key: string) => void;
  onPoll: () => void; onCancel: () => void; onReadSeries: () => void; onReadEpisodes: () => void;
  onInspect: () => void; onPlay: () => void; onRelease: () => void; onRetryAck: () => void;
  onVolumeDraft: (value: string) => void;
};
export function CatalogBrowser(props: Props) {
  return <section className="aw-inspector-section aw-catalog-browser" aria-label="작품과 회차 선택">
    <div className="aw-episode-heading"><h2>회차 목록</h2><Button className="aw-icon-button" aria-label="회차 목록 닫기" onClick={props.onClose}><span className="aw-icon aw-icon-close" aria-hidden="true"/></Button></div>
    <p className="aw-episode-title" title={props.seriesTitle}>{props.seriesTitle}</p>
    <div className="aw-episode-body">
    <div className="aw-inspector-heading">
      <Button className="aw-button" disabled={props.busy} onClick={() => props.onAiring()}>방영중</Button>
      <Button className="aw-button" disabled={props.busy} onClick={() => props.onCompleted()}>종영</Button>
      <Button className="aw-button" disabled={props.busy} onClick={() => props.onMovies()}>극장판</Button>
    </div>
    <p role="status">{props.status}</p><p role="alert">{props.error}</p>
    <div className="aw-catalog-lists">
      <RadioGroup value={props.selectedSeries} orientation="vertical" onValueChange={props.onSeries} aria-label="작품 선택">
        {props.series.map(row => <RadioGroupItem key={row.key} value={row.key} disabled={row.disabled} className="aw-button" aria-label={row.title}><span>{row.title}</span></RadioGroupItem>)}
      </RadioGroup>
      <RadioGroup value={props.selectedEpisode} orientation="vertical" onValueChange={props.onEpisode} aria-label="회차 선택">
        {props.episodes.map(row => <RadioGroupItem key={row.key} value={row.key} disabled={row.disabled} className="aw-button aw-episode-row" aria-label={row.title} aria-current={row.key === props.currentEpisode}>
          <span className="aw-episode-current-marker" hidden={row.key !== props.currentEpisode}><span className="aw-icon aw-icon-play" aria-hidden="true"/>현재</span>
          <span className="aw-episode-row-title" title={row.title}>{row.title}</span>
          <span className="aw-episode-connecting" hidden={row.key !== props.connectingEpisode}>{props.connectionLabel}</span>
        </RadioGroupItem>)}
      </RadioGroup>
    </div>
    <div className="aw-inspector-heading">
      <Button className="aw-button" disabled={props.cancelDisabled} onClick={() => props.onCancel()}>작업 취소</Button>
    </div>
    <div hidden={!props.diagnosticsExpanded} className="aw-inspector-section" aria-label="작품 탐색 문제 해결">
    <label htmlFor="catalog-start-volume">재생 시작 음량 (%)</label>
    <input id="catalog-start-volume" className="aw-text-input" value={props.volumeDraft} onChange={event => props.onVolumeDraft(event.currentTarget.value)}/>
    <Button className="aw-button" disabled={props.releaseDisabled} onClick={() => props.onRelease()}>종료된 작업에서 돌아가기</Button>
    <Button className="aw-button" disabled={props.ackRetryDisabled} onClick={() => props.onRetryAck()}>수신 처리 재시도</Button>
    </div>
    </div>
  </section>;
}
