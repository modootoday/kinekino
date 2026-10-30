import { Button } from "@/components/ui/button";
import { Range } from "../components/ui/range";

type Props = {
  seekRangeValue: string; volumeRangeValue: string; seekPreviewLabel: string; volumePreviewLabel: string;
  seekRangeDisabled: boolean; volumeRangeDisabled: boolean;
  onSeekRangeDraft: (value: string) => void; onVolumeRangeDraft: (value: string) => void;
  onSeekCommit: (value: string) => void; onVolumeCommit: (value: string) => void;
  diagnosticsExpanded: boolean; onToggleDiagnostics: () => void;
  positionLabel: string; durationLabel: string; volumeLabel: string;
  transportClass: string; fullscreenDisabled: boolean; fullscreenLabel: string; windowError: string;
  onToggleFullscreen: () => void; onToggleEpisodes: () => void;
  controlsDisabled: boolean; ackRetryDisabled: boolean;
  positionUs: string; durationUs: string; volumePercent: string; pausedLabel: string; diagnostic: string;
  onPlay: () => void; onPause: () => void; onSeek: (value: string) => void;
  onVolume: (value: string) => void; onRefresh: () => void; onRetryAck: () => void;
  seekDraft: string; volumeDraft: string; onSeekDraft: (value: string) => void; onVolumeDraft: (value: string) => void;
};
export function NativeTransportBar(props: Props) {
  return <section className={props.transportClass} aria-label="원본 엔진 재생 제어">
    <div className="aw-seek-range">
      <Range label="재생 위치" value={props.seekRangeValue} max="1000000" disabled={props.seekRangeDisabled}
        previewId="native-range-seek-preview" preview={props.seekPreviewLabel}
        onDraft={props.onSeekRangeDraft} onCommit={props.onSeekCommit}/>
    </div>
    <div className="aw-transport-row">
      <div className="aw-transport-group">
        <Button className="aw-icon-button" aria-label="재생" disabled={props.controlsDisabled} onClick={() => props.onPlay()}><span className="aw-icon aw-icon-play" aria-hidden="true"/></Button>
        <Button className="aw-icon-button" aria-label="일시 정지" disabled={props.controlsDisabled} onClick={() => props.onPause()}><span className="aw-icon aw-icon-pause" aria-hidden="true"/></Button>
        <span className="aw-time-label">{props.pausedLabel}</span>
        <span className="aw-time-label"><span id="native-range-position">{props.positionLabel}</span> / {props.durationLabel}</span>
        <span className="aw-time-label"><span className="aw-icon aw-icon-volume" aria-hidden="true"/> 음량 <span id="native-range-volume">{props.volumeLabel}</span></span>
      </div>
      <div className="aw-transport-group">
        <div className="aw-volume-range">
          <Range label="음량" value={props.volumeRangeValue} max="1000" disabled={props.volumeRangeDisabled}
            previewId="native-range-volume-preview" preview={props.volumePreviewLabel}
            onDraft={props.onVolumeRangeDraft} onCommit={props.onVolumeCommit}/>
        </div>
        <Button className="aw-icon-button" aria-label="회차 목록 열기 또는 닫기" onClick={props.onToggleEpisodes}><span className="aw-icon aw-icon-episodes" aria-hidden="true"/></Button>
        <Button className="aw-icon-button" aria-label={props.fullscreenLabel} disabled={props.fullscreenDisabled} onClick={props.onToggleFullscreen}><span className="aw-icon aw-icon-fullscreen" aria-hidden="true"/></Button>
        <Button className="aw-button" aria-expanded={props.diagnosticsExpanded} onClick={props.onToggleDiagnostics}>고급 · 문제 해결</Button>
      </div>
    </div>
    <div hidden={!props.diagnosticsExpanded} className="aw-transport-group" aria-label="고급 재생 제어">
        <label htmlFor="native-seek-position">이동 위치 (시:분:초.밀리초)</label>
        <input id="native-seek-position" className="aw-text-input" value={props.seekDraft} disabled={props.controlsDisabled} onChange={event => props.onSeekDraft(event.currentTarget.value)}/>
        <Button className="aw-button" disabled={props.controlsDisabled} onClick={() => props.onSeek(props.seekDraft)}>이동</Button>
        <label htmlFor="native-volume-value">음량 (%)</label>
        <input id="native-volume-value" className="aw-text-input" value={props.volumeDraft} disabled={props.controlsDisabled} onChange={event => props.onVolumeDraft(event.currentTarget.value)}/>
        <Button className="aw-button" disabled={props.controlsDisabled} onClick={() => props.onVolume(props.volumeDraft)}>음량 적용</Button>
        <span id="native-range-volume-raw" role="status">{props.volumePercent}</span>
        <Button className="aw-button" onClick={() => props.onRefresh()}>엔진 관측 갱신</Button>
        <Button className="aw-button" disabled={props.ackRetryDisabled} onClick={() => props.onRetryAck()}>수신 처리 재시도</Button>
        <p>SDK 원관측: <span id="native-range-position-raw">{props.positionUs}</span> / {props.durationUs} μs</p>
    </div>
    <p hidden={!props.diagnosticsExpanded} role="status">{props.diagnostic}</p>
    <p role="alert">{props.windowError}</p>
  </section>;
}
