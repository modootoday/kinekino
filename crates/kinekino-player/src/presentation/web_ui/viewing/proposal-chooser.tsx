import { Button } from "@/components/ui/button";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";

type Props = {
  label: string; documentsLabel: string; itemsLabel: string;
  listLabel: string; nextLabel: string; openLabel: string;
  status: string; error: string;
  selectedDocumentKey: string; selectedItemKey: string; openKey: string;
  listDisabled: boolean; nextDisabled: boolean; openDisabled: boolean;
  documents: { key: string; title: string; detail: string; disabled: boolean; }[];
  items: { key: string; title: string; detail: string; disabled: boolean; }[];
  onChooserList: () => void; onChooserNext: () => void;
  onChooserDocument: (key: string) => void; onChooserItem: (key: string) => void;
  onOpenLiveProposal: (key: string) => void;
};

export function ProposalChooser(props: Props) {
  return <section className="aw-inspector-section" aria-label={props.label}>
    <div className="aw-inspector-heading">
      <Button className="aw-button" disabled={props.listDisabled} onClick={() => props.onChooserList()}>{props.listLabel}</Button>
      <Button className="aw-button" disabled={props.nextDisabled} onClick={() => props.onChooserNext()}>{props.nextLabel}</Button>
    </div>
    <p role="status">{props.status}</p>
    <p role="alert">{props.error}</p>
    <RadioGroup value={props.selectedDocumentKey} orientation="vertical" onValueChange={props.onChooserDocument} aria-label={props.documentsLabel}>
      {props.documents.map(row => <RadioGroupItem key={row.key} value={row.key} disabled={row.disabled} className="aw-button" aria-label={row.title}>
        <span>{row.title}</span><span className="aw-muted">{row.detail}</span>
      </RadioGroupItem>)}
    </RadioGroup>
    <RadioGroup value={props.selectedItemKey} orientation="vertical" onValueChange={props.onChooserItem} aria-label={props.itemsLabel}>
      {props.items.map(row => <RadioGroupItem key={row.key} value={row.key} disabled={row.disabled} className="aw-button" aria-label={row.title}>
        <span>{row.title}</span><span className="aw-muted">{row.detail}</span>
      </RadioGroupItem>)}
    </RadioGroup>
    <Button className="aw-button" disabled={props.openDisabled} onClick={() => props.onOpenLiveProposal(props.openKey)}>{props.openLabel}</Button>
  </section>;
}
