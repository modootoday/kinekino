import type { ReactNode } from "react";
import { useId } from "react";
import { Button } from "@/components/ui/button";

type Props = {
  subjects: ReactNode; modelPicker: ReactNode; reviewDetails: ReactNode;
  selectedOperation: string; canDetect: boolean; canSegment: boolean; canTrack: boolean;
  detectionReason: string; segmentationReason: string; trackingReason: string;
  rangeStartLabel: string; rangeEndLabel: string; canEditRange: boolean; rangeReason: string;
  opacityLabel: string; opacityValue: string; featherLabel: string; featherValue: string;
  hasMaskSettings: boolean; canEditMask: boolean; maskReason: string;
  modelLabel: string; modelReason: string; busy: boolean; canRun: boolean;
  canCancel: boolean; canReview: boolean; canSave: boolean;
  runReason: string; cancelReason: string; reviewReason: string; saveReason: string;
  resultLabel: string;
  onDetect: () => void; onSegment: () => void; onTrack: () => void;
  onEditRange: () => void; onOpacityChange: (value: string) => void; onFeatherChange: (value: string) => void;
  onRun: () => void; onCancel: () => void; onReview: () => void; onSave: () => void;
};

export function SubjectInspector(props: Props) {
  const rangeReasonId = useId();
  const maskReasonId = useId();
  const opacityId = useId();
  const featherId = useId();
  return <aside className="aw-inspector" aria-label="피사체 설정" aria-busy={props.busy}>
    <div className="aw-inspector-scroll">
      {props.subjects}
      <section className="aw-inspector-section" aria-label="탐지 분할 추적 작업 설정">
        <h2>작업 설정</h2>
        <div className="aw-subject-operation" role="group" aria-label="피사체 분석 작업">
          <Button className="aw-button aw-subject-mode" aria-pressed={props.selectedOperation === "detect"} disabled={!props.canDetect || props.busy} title={props.detectionReason} onClick={() => props.onDetect()}>탐지</Button>
          <Button className="aw-button aw-subject-mode" aria-pressed={props.selectedOperation === "segment"} disabled={!props.canSegment || props.busy} title={props.segmentationReason} onClick={() => props.onSegment()}>분할</Button>
          <Button className="aw-button aw-subject-mode" aria-pressed={props.selectedOperation === "track"} disabled={!props.canTrack || props.busy} title={props.trackingReason} onClick={() => props.onTrack()}>추적</Button>
        </div>
        <div className="aw-inspector-section" aria-label="선택 구간">
          <h3>선택 구간</h3><div className="aw-subject-range aw-numeric"><span>{props.rangeStartLabel}</span><span aria-hidden="true">~</span><span>{props.rangeEndLabel}</span></div>
          <Button className="aw-button" disabled={!props.canEditRange || props.busy} aria-describedby={rangeReasonId} onClick={() => props.onEditRange()}>구간 수정</Button><p id={rangeReasonId} className="aw-muted">{props.rangeReason}</p>
        </div>
        <div className="aw-inspector-section" aria-label="마스크 표시와 경계">
          <label htmlFor={opacityId}>마스크 불투명도</label><p>{props.opacityLabel}</p>
          <input id={opacityId} className="aw-subject-opacity" type="range" min="0" max="100" value={props.opacityValue} hidden={!props.hasMaskSettings} disabled={!props.canEditMask || props.busy} aria-describedby={maskReasonId} onChange={event => props.onOpacityChange(event.currentTarget.value)}/>
          <label htmlFor={featherId}>경계 세부 보정 · 원본 px</label><p>{props.featherLabel}</p>
          <input id={featherId} className="aw-text-input" type="text" inputMode="decimal" value={props.featherValue} hidden={!props.hasMaskSettings} disabled={!props.canEditMask || props.busy} aria-describedby={maskReasonId} onChange={event => props.onFeatherChange(event.currentTarget.value)}/>
          <p id={maskReasonId} className="aw-muted">{props.maskReason}</p>
        </div>
        <section className="aw-inspector-section" aria-label="모델 선택"><h3>모델 선택</h3>{props.modelPicker}<p>{props.modelLabel}</p><p className="aw-muted">{props.modelReason}</p></section>
        {props.reviewDetails}
      </section>
    </div>
    <div className="aw-inspector-footer">
      <Button className="aw-button aw-button-primary" disabled={!props.canRun || props.busy} title={props.runReason} onClick={() => props.onRun()}>인식 시작</Button>
      <Button className="aw-button" disabled={!props.canCancel} title={props.cancelReason} onClick={() => props.onCancel()}>작업 취소</Button>
      <div className="aw-subject-footer-actions"><Button className="aw-button" disabled={!props.canReview || props.busy} title={props.reviewReason} onClick={() => props.onReview()}>마스크 검토</Button><Button className="aw-button" disabled={!props.canSave || props.busy} title={props.saveReason} onClick={() => props.onSave()}>저장</Button></div>
      <p role="status">{props.resultLabel}</p>
    </div>
  </aside>;
}
