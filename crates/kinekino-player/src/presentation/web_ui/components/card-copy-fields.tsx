import { useId } from "react";
type Props = { front: string; back: string; canEdit: boolean; busy: boolean; refreshRequired: boolean; onFrontChange: (text: string) => void; onBackChange: (text: string) => void; };
export function CardCopyFields(props: Props) {
  const frontId = useId();
  const backId = useId();
  return <section className="aw-inspector-section">
    <label htmlFor={frontId}>앞면 문구</label><textarea id={frontId} className="aw-text-input aw-draft-input" value={props.front} disabled={!props.canEdit || props.busy || props.refreshRequired} onChange={event => props.onFrontChange(event.currentTarget.value)}/>
    <label htmlFor={backId}>뒷면 문구</label><textarea id={backId} className="aw-text-input aw-draft-input" value={props.back} disabled={!props.canEdit || props.busy || props.refreshRequired} onChange={event => props.onBackChange(event.currentTarget.value)}/>
    <p className="aw-muted">문구 채택은 카드 이미지 저장이나 게시를 실행하지 않습니다.</p>
  </section>;
}
