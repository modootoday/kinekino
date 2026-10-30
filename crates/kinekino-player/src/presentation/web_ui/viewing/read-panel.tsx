import type { ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { ViewingPlayerFrame } from "./player-frame";
import { NativeTransportBar } from "./native-transport-bar";
import { CatalogBrowser } from "./catalog-browser";
import { EpisodeNavigation } from "./episode-navigation";
import { NativeStatus } from "./native-status";
import { NativeCaptionControls } from "../components/window-caption-controls";

type Props = {
  captionMinimizeDisabled: boolean; captionMaximizeDisabled: boolean; captionCloseDisabled: boolean;
  captionMaximizeLabel: string; captionMaximizeIconClass: string; captionStatus: string;
  captionStatusError: boolean;
  onWindowMinimize: () => void; onWindowToggleMaximize: () => void; onWindowUserClose: () => void;
  catalogCurrentEpisode: string; catalogConnectingEpisode: string; catalogEpisodeConnectionLabel: string;
  nativeStatusSummary: string; nativeStatusError: boolean; nativeStatusUncertain: boolean;
  localInputError: string; localInputErrorPresent: boolean;
  viewingFullscreen: boolean; viewingFrameClass: string;
  viewingRootClass: string; episodePanelClosed: boolean;
  nativeSeekRangeValue: string; nativeVolumeRangeValue: string; nativeSeekPreviewLabel: string; nativeVolumePreviewLabel: string;
  nativeSeekRangeDisabled: boolean; nativeVolumeRangeDisabled: boolean;
  onNativeSeekRangeDraft: (value: string) => void; onNativeVolumeRangeDraft: (value: string) => void;
  onNativeSeekCommit: (value: string) => void; onNativeVolumeCommit: (value: string) => void;
  diagnosticsExpanded: boolean; onToggleDiagnostics: () => void;
  transportClass: string; episodePanelClass: string; fullscreenDisabled: boolean; fullscreenLabel: string; windowError: string;
  onToggleFullscreen: () => void; onToggleEpisodes: () => void;
  automaticAvailable: boolean; automaticError: string;
  transitionNavigationDisabled: boolean; transitionCancelDisabled: boolean; transitionObserveDisabled: boolean;
  transitionPrepareDisabled: boolean; transitionCommitDisabled: boolean; transitionReadJobDisabled: boolean; transitionReleaseDisabled: boolean;
  transitionStatus: string; transitionError: string;
  onPreparePrevious: () => void; onPrepareNext: () => void; onCancelTransitionJob: () => void; onObserveTransition: () => void;
  nativeObservationPresent: boolean;
  startScreen: ReactNode;
  nativeCurrentTitle: string; nativeCurrentEpisode: string; nativeCurrentCaption: string;
  catalogSeriesRows: { key: string; title: string; disabled: boolean; }[];
  catalogEpisodeRows: { key: string; title: string; disabled: boolean; }[];
  catalogSelectedSeries: string; catalogSelectedEpisode: string; catalogStatus: string; catalogError: string;
  catalogBusy: boolean; catalogPollDisabled: boolean; catalogCancelDisabled: boolean;
  catalogReadSeriesDisabled: boolean; catalogReadEpisodesDisabled: boolean; catalogInspectDisabled: boolean;
  catalogPlayDisabled: boolean; catalogReleaseDisabled: boolean; catalogAckRetryDisabled: boolean;
  onCatalogAiring: () => void; onCatalogCompleted: () => void; onCatalogMovies: () => void;
  onCatalogSeries: (value: string) => void; onCatalogEpisode: (value: string) => void;
  onCatalogPoll: () => void; onCatalogCancel: () => void; onCatalogReadSeries: () => void; onCatalogReadEpisodes: () => void;
  onCatalogInspect: () => void; onCatalogPlay: () => void; onReleaseCatalog: () => void; onRetryCatalogAck: () => void;
  nativeControlsDisabled: boolean; nativeAckRetryDisabled: boolean;
  nativeInstanceId: string; nativeResolutionId: string; nativePositionUs: string; nativeDurationUs: string; nativePositionLabel: string; nativeDurationLabel: string; nativeVolumeLabel: string; nativeVolumePercent: string; nativePausedLabel: string; nativePlaybackDiagnostic: string;
  onNativePlay: () => void; onNativePause: () => void; onNativeSeek: (value: string) => void; onNativeVolume: (value: string) => void; onRetryNativeAck: () => void;
  nativeSeekDraft: string; nativeVolumeDraft: string; onNativeSeekDraft: (value: string) => void; onNativeVolumeDraft: (value: string) => void;
  jobsEntryDisabled: boolean; jobsEntryLabel: string; jobsEntryStatus: string; onOpenJobs: () => void;
  capturedCardChoices: ReactNode;
  proposalChooser: ReactNode;
  productName: string; connectionLabel: string; timeLabel: string; videoSemanticId: string;
  readPanelLabel: string; capabilitiesActionLabel: string; engineActionLabel: string; statusActionLabel: string;
  capabilitiesStateCode: string; capabilitiesErrorCode: string;
  engineStateCode: string; engineErrorCode: string;
  statusStateCode: string; statusErrorCode: string;
  retryAckLabel: string; refreshProjectionLabel: string; retryAckLabelKey: string; refreshProjectionLabelKey: string;
  capabilitiesRetryDisabled: boolean; engineRetryDisabled: boolean; statusRetryDisabled: boolean;
  onRetryCapabilitiesAck: () => void; onRetryEngineAck: () => void; onRetryStatusAck: () => void; onRefreshReadProjection: () => void;
  capabilitiesState: string; capabilitiesValue: string; capabilitiesError: string; capabilitiesBusy: boolean;
  engineState: string; engineValue: string; engineError: string; engineBusy: boolean;
  statusState: string; statusValue: string; statusError: string; statusBusy: boolean;
  onReadCapabilities: () => void; onReadEngine: () => void; onReadStatus: () => void;
};
export function ViewingReadRoot(props: Props) {
  return <div className={props.viewingRootClass}>
    <ViewingPlayerFrame captionStatus={props.captionStatus} captionStatusError={props.captionStatusError} captionControls={<NativeCaptionControls minimizeDisabled={props.captionMinimizeDisabled} maximizeDisabled={props.captionMaximizeDisabled} closeDisabled={props.captionCloseDisabled}
      maximizeLabel={props.captionMaximizeLabel} maximizeIconClass={props.captionMaximizeIconClass}
      onMinimize={props.onWindowMinimize} onMaximize={props.onWindowToggleMaximize} onClose={props.onWindowUserClose}/>}
      fullscreen={props.viewingFullscreen} frameClass={props.viewingFrameClass} startScreen={props.startScreen} contextLabel={props.nativeCurrentCaption} productName={props.productName} connectionLabel={props.connectionLabel} videoSemanticId={props.videoSemanticId} transport={<NativeTransportBar
      seekRangeValue={props.nativeSeekRangeValue} volumeRangeValue={props.nativeVolumeRangeValue}
      seekPreviewLabel={props.nativeSeekPreviewLabel} volumePreviewLabel={props.nativeVolumePreviewLabel}
      seekRangeDisabled={props.nativeSeekRangeDisabled} volumeRangeDisabled={props.nativeVolumeRangeDisabled}
      onSeekRangeDraft={props.onNativeSeekRangeDraft} onVolumeRangeDraft={props.onNativeVolumeRangeDraft}
      onSeekCommit={props.onNativeSeekCommit} onVolumeCommit={props.onNativeVolumeCommit}
      controlsDisabled={props.nativeControlsDisabled} ackRetryDisabled={props.nativeAckRetryDisabled}
      transportClass={props.transportClass} fullscreenDisabled={props.fullscreenDisabled} fullscreenLabel={props.fullscreenLabel} windowError={props.windowError}
      onToggleFullscreen={props.onToggleFullscreen} onToggleEpisodes={props.onToggleEpisodes}
      positionUs={props.nativePositionUs} durationUs={props.nativeDurationUs} volumePercent={props.nativeVolumePercent}
      positionLabel={props.nativePositionLabel} durationLabel={props.nativeDurationLabel}
      volumeLabel={props.nativeVolumeLabel}
      diagnosticsExpanded={props.diagnosticsExpanded} onToggleDiagnostics={props.onToggleDiagnostics}
      pausedLabel={props.nativePausedLabel} diagnostic={props.nativePlaybackDiagnostic}
      seekDraft={props.nativeSeekDraft} volumeDraft={props.nativeVolumeDraft}
      onSeekDraft={props.onNativeSeekDraft} onVolumeDraft={props.onNativeVolumeDraft}
      onPlay={props.onNativePlay} onPause={props.onNativePause} onSeek={props.onNativeSeek} onVolume={props.onNativeVolume}
      onRefresh={props.onReadEngine} onRetryAck={props.onRetryNativeAck}/>}/>
    <aside className={props.episodePanelClass} hidden={props.episodePanelClosed} aria-label="작품과 회차 목록">
    <CatalogBrowser series={props.catalogSeriesRows} episodes={props.catalogEpisodeRows}
      currentEpisode={props.catalogCurrentEpisode} connectingEpisode={props.catalogConnectingEpisode}
      connectionLabel={props.catalogEpisodeConnectionLabel} seriesTitle={props.nativeCurrentTitle} onClose={props.onToggleEpisodes}
      selectedSeries={props.catalogSelectedSeries} selectedEpisode={props.catalogSelectedEpisode}
      status={props.catalogStatus} error={props.catalogError} busy={props.catalogBusy}
      pollDisabled={props.catalogPollDisabled} cancelDisabled={props.catalogCancelDisabled}
      readSeriesDisabled={props.catalogReadSeriesDisabled} readEpisodesDisabled={props.catalogReadEpisodesDisabled}
      inspectDisabled={props.catalogInspectDisabled} playDisabled={props.catalogPlayDisabled}
      releaseDisabled={props.catalogReleaseDisabled} ackRetryDisabled={props.catalogAckRetryDisabled}
      diagnosticsExpanded={props.diagnosticsExpanded}
      volumeDraft={props.nativeVolumeDraft} onVolumeDraft={props.onNativeVolumeDraft}
      onAiring={props.onCatalogAiring} onCompleted={props.onCatalogCompleted} onMovies={props.onCatalogMovies}
      onSeries={props.onCatalogSeries} onEpisode={props.onCatalogEpisode}
      onPoll={props.onCatalogPoll} onCancel={props.onCatalogCancel}
      onReadSeries={props.onCatalogReadSeries} onReadEpisodes={props.onCatalogReadEpisodes}
      onInspect={props.onCatalogInspect} onPlay={props.onCatalogPlay}
      onRelease={props.onReleaseCatalog} onRetryAck={props.onRetryCatalogAck}/>
    </aside>
    <section className="aw-viewing-status-pane" aria-label="재생 상태와 작업">
    <p role="alert">{props.windowError}</p>
    <NativeStatus summary={props.nativeStatusSummary} error={props.nativeStatusError} uncertain={props.nativeStatusUncertain}
      localError={props.localInputError} localErrorPresent={props.localInputErrorPresent}
      refreshDisabled={props.engineBusy} ackDisabled={props.nativeAckRetryDisabled}
      onRefresh={props.onReadEngine} onRetryAck={props.onRetryNativeAck}/>
    <p role="alert">{props.automaticError}</p>
    <EpisodeNavigation navigationDisabled={props.transitionNavigationDisabled} cancelDisabled={props.transitionCancelDisabled} retryDisabled={props.transitionObserveDisabled}
      status={props.transitionStatus} error={props.transitionError}
      onPrevious={props.onPreparePrevious} onNext={props.onPrepareNext} onCancel={props.onCancelTransitionJob} onRetry={props.onObserveTransition}/>
    <p role="status">{props.catalogStatus}</p><p role="alert">{props.catalogError}</p>
    <div hidden={!props.episodePanelClosed}><Button className="aw-button" disabled={props.catalogCancelDisabled} onClick={props.onCatalogCancel}>작품 탐색 작업 취소</Button></div>
    <div>{props.capturedCardChoices}</div>
    <div>{props.proposalChooser}</div>
    <div><Button className="aw-button" disabled={props.jobsEntryDisabled} onClick={() => props.onOpenJobs()}>{props.jobsEntryLabel}</Button><p role="status">{props.jobsEntryStatus}</p></div>
    <Button className="aw-button" aria-expanded={props.diagnosticsExpanded} aria-controls="viewing-troubleshooting" onClick={props.onToggleDiagnostics}>고급 · 문제 해결</Button>
    <p>진행이 멈추거나 결과를 확인하지 못하면 고급 · 문제 해결을 열어 주세요.</p>
    <section id="viewing-troubleshooting" hidden={!props.diagnosticsExpanded} className="aw-read-panel" aria-label="고급 · 문제 해결">
      <p>자동 진행이 멈추거나 결과를 확인하지 못했을 때 아래 상태를 확인하세요. 결과가 불확정인 요청을 새로 제출하지 마세요.</p>
      <h3>{props.readPanelLabel}</h3>
      <div className="aw-read-row">
        <Button className="aw-button" disabled={props.capabilitiesBusy} onClick={() => props.onReadCapabilities()}>{props.capabilitiesActionLabel}</Button>
        <span role="status">{props.capabilitiesState}</span><pre>{props.capabilitiesValue}</pre><p>{props.capabilitiesError}</p>
        <Button className="aw-button" disabled={props.capabilitiesRetryDisabled} onClick={() => props.onRetryCapabilitiesAck()}>{props.retryAckLabel}</Button>
      </div>
      <div className="aw-read-row">
        <Button className="aw-button" disabled={props.engineBusy} onClick={() => props.onReadEngine()}>{props.engineActionLabel}</Button>
        <span role="status">{props.engineState}</span><pre>{props.engineValue}</pre><p>{props.engineError}</p>
        <Button className="aw-button" disabled={props.engineRetryDisabled} onClick={() => props.onRetryEngineAck()}>{props.retryAckLabel}</Button>
      </div>
      <div className="aw-read-row">
        <Button className="aw-button" disabled={props.statusBusy} onClick={() => props.onReadStatus()}>{props.statusActionLabel}</Button>
        <span role="status">{props.statusState}</span><pre>{props.statusValue}</pre><p>{props.statusError}</p>
        <Button className="aw-button" disabled={props.statusRetryDisabled} onClick={() => props.onRetryStatusAck()}>{props.retryAckLabel}</Button>
      </div>
      <Button className="aw-button" onClick={() => props.onRefreshReadProjection()}>{props.refreshProjectionLabel}</Button>
    </section>
    </section>
  </div>;
}

