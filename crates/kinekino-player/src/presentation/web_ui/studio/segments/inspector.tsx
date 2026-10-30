import { useId } from "react";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { InformationNotice } from "../../components/notice";

type Props = {
  sourceLabel: string; scopeLabel: string; stateLabel: string; busy: boolean;
  opModeId: string; edModeId: string;
  opModes: { id: string; label: string; available: boolean; reason: string; }[];
  edModes: { id: string; label: string; available: boolean; reason: string; }[];
  canChangeOpMode: boolean; opModeReason: string; canChangeEdMode: boolean; edModeReason: string;
  verificationLabel: string; verificationDetail: string;
  nextEpisodeEnabled: boolean; canChangeNextEpisode: boolean; nextEpisodeReason: string;
  episodeConfirmationRequired: boolean; canChangeEpisodeConfirmation: boolean; episodeConfirmationReason: string;
  afterSkipId: string; afterSkipOptions: { id: string; label: string; available: boolean; reason: string; }[];
  noticeTimingId: string; noticeTimingOptions: { id: string; label: string; available: boolean; reason: string; }[];
  canChangePlaybackOptions: boolean; playbackOptionsReason: string;
  recentActionLabel: string; recentActionDetail: string; canUndo: boolean; undoReason: string;
  detectionMethodId: string; detectionMethods: { id: string; label: string; available: boolean; reason: string; }[];
  canChangeDetectionMethod: boolean; detectionReason: string;
  modelLabel: string; modelRevisionLabel: string; precisionLabel: string;
  canAnalyze: boolean; analyzeReason: string;
  canEditRange: boolean; editRangeReason: string; canSave: boolean; saveReason: string;
  canResetOverride: boolean; resetOverrideReason: string;
  resultLabel: string; hasError: boolean; errorLabel: string;
  onOpModeChange: (id: string) => void; onEdModeChange: (id: string) => void;
  onNextEpisodeChange: (value: boolean) => void; onEpisodeConfirmationChange: (value: boolean) => void;
  onAfterSkipChange: (id: string) => void; onNoticeTimingChange: (id: string) => void;
  onDetectionMethodChange: (id: string) => void;
  onUndo: () => void; onAnalyze: () => void; onEditRange: () => void;
  onSaveEpisode: () => void; onResetOverride: () => void;
};

export function SkipPolicyInspector(props: Props) {
  const opId = useId();
  const edId = useId();
  const opReasonId = useId();
  const edReasonId = useId();
  const nextId = useId();
  const nextReasonId = useId();
  const confirmationId = useId();
  const confirmationReasonId = useId();
  const afterSkipId = useId();
  const noticeId = useId();
  const playbackReasonId = useId();
  const detectionId = useId();
  const detectionReasonId = useId();
  return <aside className="aw-inspector" aria-label="구간 건너뛰기 설정" aria-busy={props.busy}>
    <div className="aw-inspector-scroll">
      <div className="aw-inspector-heading"><h2>구간 건너뛰기 설정</h2></div>
      <p className="aw-muted">{props.sourceLabel}</p>
      <p role="status">{props.stateLabel}</p>

      <section className="aw-inspector-section" aria-label="오프닝과 엔딩 설정">
        <label htmlFor={opId}>오프닝 설정</label>
        <select id={opId} className="aw-text-input" value={props.opModeId} disabled={!props.canChangeOpMode || props.busy} aria-describedby={opReasonId} onChange={event => props.onOpModeChange(event.currentTarget.value)}>
          <option value="" disabled={true}>설정을 선택하세요</option>
          {props.opModes.map(mode => <option key={mode.id} value={mode.id} disabled={!mode.available} title={mode.reason}>{mode.label}</option>)}
        </select>
        <p id={opReasonId} className="aw-muted">{props.opModeReason}</p>
        <label htmlFor={edId}>엔딩 설정</label>
        <select id={edId} className="aw-text-input" value={props.edModeId} disabled={!props.canChangeEdMode || props.busy} aria-describedby={edReasonId} onChange={event => props.onEdModeChange(event.currentTarget.value)}>
          <option value="" disabled={true}>설정을 선택하세요</option>
          {props.edModes.map(mode => <option key={mode.id} value={mode.id} disabled={!mode.available} title={mode.reason}>{mode.label}</option>)}
        </select>
        <p id={edReasonId} className="aw-muted">{props.edModeReason}</p>
        <InformationNotice title={props.verificationLabel} detail={props.verificationDetail}/>
      </section>

      <section className="aw-inspector-section" aria-label="재생 옵션">
        <h3>재생 옵션</h3>
        <div className="aw-inspector-heading"><label htmlFor={nextId}>다음화 자동 재생</label><Switch id={nextId} checked={props.nextEpisodeEnabled} disabled={!props.canChangeNextEpisode || props.busy} aria-describedby={nextReasonId} onCheckedChange={value => props.onNextEpisodeChange(value)}/></div>
        <p id={nextReasonId} className="aw-muted">{props.nextEpisodeReason}</p>
        <div className="aw-inspector-heading"><label htmlFor={confirmationId}>회차별 확인</label><Switch id={confirmationId} checked={props.episodeConfirmationRequired} disabled={!props.canChangeEpisodeConfirmation || props.busy} aria-describedby={confirmationReasonId} onCheckedChange={value => props.onEpisodeConfirmationChange(value)}/></div>
        <p id={confirmationReasonId} className="aw-muted">{props.episodeConfirmationReason}</p>
      </section>

      <section className="aw-inspector-section" aria-label="추가 설정">
        <h3>추가 설정</h3>
        <label htmlFor={afterSkipId}>건너뛰기 후 동작</label>
        <select id={afterSkipId} className="aw-text-input" value={props.afterSkipId} disabled={!props.canChangePlaybackOptions || props.busy} aria-describedby={playbackReasonId} onChange={event => props.onAfterSkipChange(event.currentTarget.value)}>
          <option value="" disabled={true}>동작을 선택하세요</option>
          {props.afterSkipOptions.map(option => <option key={option.id} value={option.id} disabled={!option.available} title={option.reason}>{option.label}</option>)}
        </select>
        <label htmlFor={noticeId}>미리 알림 표시</label>
        <select id={noticeId} className="aw-text-input" value={props.noticeTimingId} disabled={!props.canChangePlaybackOptions || props.busy} aria-describedby={playbackReasonId} onChange={event => props.onNoticeTimingChange(event.currentTarget.value)}>
          <option value="" disabled={true}>알림을 선택하세요</option>
          {props.noticeTimingOptions.map(option => <option key={option.id} value={option.id} disabled={!option.available} title={option.reason}>{option.label}</option>)}
        </select>
        <p id={playbackReasonId} className="aw-muted">{props.playbackOptionsReason}</p>
      </section>

      <section className="aw-inspector-section" aria-label="최근 작업 기록">
        <h3>최근 작업 기록</h3>
        <InformationNotice title={props.recentActionLabel} detail={props.recentActionDetail}/>
        <Button className="aw-link-button" disabled={!props.canUndo || props.busy} title={props.undoReason} onClick={() => props.onUndo()}>되돌리기</Button>
        <p className="aw-muted">{props.undoReason}</p>
      </section>

      <section className="aw-inspector-section" aria-label="구간 분석">
        <h3>구간 분석</h3>
        <label htmlFor={detectionId}>감지 방식</label>
        <select id={detectionId} className="aw-text-input" value={props.detectionMethodId} disabled={!props.canChangeDetectionMethod || props.busy} aria-describedby={detectionReasonId} onChange={event => props.onDetectionMethodChange(event.currentTarget.value)}>
          <option value="" disabled={true}>감지 방식을 선택하세요</option>
          {props.detectionMethods.map(method => <option key={method.id} value={method.id} disabled={!method.available} title={method.reason}>{method.label}</option>)}
        </select>
        <dl className="aw-metadata"><dt>모델</dt><dd>{props.modelLabel}</dd><dt>모델 revision</dt><dd>{props.modelRevisionLabel}</dd><dt>정밀도</dt><dd>{props.precisionLabel}</dd></dl>
        <p id={detectionReasonId} className="aw-muted">{props.detectionReason}</p>
        <Button className="aw-button" disabled={!props.canAnalyze || props.busy} title={props.analyzeReason} onClick={() => props.onAnalyze()}>구간 분석 시작</Button>
        <p className="aw-muted">{props.analyzeReason}</p>
      </section>

      <section className="aw-inspector-section" aria-label="적용 범위">
        <h3>적용 범위</h3><p>{props.scopeLabel}</p>
        <Button className="aw-link-button" disabled={!props.canResetOverride || props.busy} title={props.resetOverrideReason} onClick={() => props.onResetOverride()}>작품 예외 초기화</Button>
        <p className="aw-muted">{props.resetOverrideReason}</p>
      </section>
    </div>
    <div className="aw-inspector-footer">
      <Button className="aw-button" disabled={!props.canEditRange || props.busy} title={props.editRangeReason} onClick={() => props.onEditRange()}>구간 수정</Button>
      <Button className="aw-button aw-button-primary" disabled={!props.canSave || props.busy} title={props.saveReason} onClick={() => props.onSaveEpisode()}>현재 회차 저장</Button>
      <p className="aw-muted">{props.editRangeReason}</p><p className="aw-muted">{props.saveReason}</p>
      <p role="status">{props.resultLabel}</p><p role="alert" hidden={!props.hasError}>{props.errorLabel}</p>
    </div>
  </aside>;
}
