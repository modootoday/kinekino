import { useId } from "react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";

type Props = {
  candidateId: string; title: string; rangeLabel: string; durationLabel: string;
  sourceLabel: string; evidenceKind: "subtitle" | "scene" | "manual" | "unknown"; evidenceKindLabel: string; evidenceText: string; qualityLabel: string;
  hasThumbnail: boolean; thumbnailSrc: string; thumbnailAlt: string; thumbnailStateLabel: string;
  previewSelected: boolean; included: boolean; busy: boolean;
  canSelect: boolean; selectLabel: string; selectReason: string;
  canInclude: boolean; includeLabel: string; inclusionReason: string;
  canOpenSource: boolean; sourceActionLabel: string; sourceReason: string;
  canPreview: boolean; previewActionLabel: string; previewReason: string;
  canEdit: boolean; editActionLabel: string; editReason: string;
  canExclude: boolean; excludeActionLabel: string; excludeReason: string;
  onSelect: () => void; onIncludedChange: (value: boolean) => void;
  onOpenSource: () => void; onPreview: () => void; onEdit: () => void; onExclude: () => void;
};

export function ClipRecommendationItem(props: Props) {
  const titleId = useId();
  const inclusionId = useId();
  const inclusionReasonId = useId();
  const selectReasonId = useId();
  const sourceReasonId = useId();
  const previewReasonId = useId();
  const editReasonId = useId();
  const excludeReasonId = useId();
  return <article className="aw-clip-recommendation" aria-labelledby={titleId} aria-busy={props.busy} data-preview-selected={props.previewSelected}>
    <div className="aw-clip-recommendation-inclusion">
      <Checkbox id={inclusionId} checked={props.included} disabled={!props.canInclude || props.busy} aria-label={props.includeLabel} aria-describedby={inclusionReasonId} onCheckedChange={value => props.onIncludedChange(value)}/>
      <p id={inclusionReasonId} className="aw-clip-recommendation-reason">{props.inclusionReason}</p>
    </div>

    <div className="aw-clip-recommendation-artwork">
      <Button className="aw-clip-recommendation-select" aria-label={props.selectLabel} aria-pressed={props.previewSelected} aria-describedby={selectReasonId} disabled={!props.canSelect || props.busy} title={props.selectReason} onClick={() => props.onSelect()}>
        {props.hasThumbnail && <img className="aw-clip-recommendation-thumbnail" src={props.thumbnailSrc} alt={props.thumbnailAlt}/>}
        {!props.hasThumbnail && <span className="aw-clip-recommendation-thumbnail aw-clip-recommendation-placeholder">{props.thumbnailStateLabel}</span>}
      </Button>
      <p id={selectReasonId} className="aw-clip-recommendation-reason">{props.selectReason}</p>
    </div>

    <div className="aw-clip-recommendation-details">
      <h3 id={titleId}>{props.title}</h3>
      <p className="aw-clip-recommendation-time"><span>{props.rangeLabel}</span><span>{props.durationLabel}</span></p>
      <p className="aw-clip-recommendation-source-label">{props.sourceLabel}</p>
      <Button className="aw-clip-recommendation-source" aria-label={props.sourceActionLabel} aria-describedby={sourceReasonId} disabled={!props.canOpenSource || props.busy} title={props.sourceReason} onClick={() => props.onOpenSource()}><span className="aw-icon aw-clip-recommendation-icon-source" aria-hidden="true"/>원본 구간으로 이동</Button>
      <p id={sourceReasonId} className="aw-clip-recommendation-reason">{props.sourceReason}</p>
    </div>

    <div className="aw-clip-recommendation-evidence">
      <p className="aw-clip-recommendation-evidence-heading">선정 근거</p>
      <p className="aw-clip-recommendation-evidence-kind" data-evidence-kind={props.evidenceKind}><span className="aw-icon aw-clip-recommendation-icon-evidence" aria-hidden="true"/>{props.evidenceKindLabel}</p>
      <p className="aw-clip-recommendation-evidence-text">{props.evidenceText}</p>
      <p className="aw-clip-recommendation-quality">{props.qualityLabel}</p>
    </div>

    <div className="aw-clip-recommendation-actions">
      <Button className="aw-button" aria-label={props.previewActionLabel} aria-describedby={previewReasonId} disabled={!props.canPreview || props.busy} title={props.previewReason} onClick={() => props.onPreview()}><span className="aw-icon aw-icon-play" aria-hidden="true"/>미리보기</Button>
      <Button className="aw-button" aria-label={props.editActionLabel} aria-describedby={editReasonId} disabled={!props.canEdit || props.busy} title={props.editReason} onClick={() => props.onEdit()}><span className="aw-icon aw-clip-recommendation-icon-edit" aria-hidden="true"/>편집</Button>
      <Button className="aw-button" aria-label={props.excludeActionLabel} aria-describedby={excludeReasonId} disabled={!props.canExclude || props.busy} title={props.excludeReason} onClick={() => props.onExclude()}><span className="aw-icon aw-clip-recommendation-icon-exclude" aria-hidden="true"/>제외</Button>
      <p id={previewReasonId} className="aw-clip-recommendation-reason">{props.previewReason}</p>
      <p id={editReasonId} className="aw-clip-recommendation-reason">{props.editReason}</p>
      <p id={excludeReasonId} className="aw-clip-recommendation-reason">{props.excludeReason}</p>
    </div>
  </article>;
}
