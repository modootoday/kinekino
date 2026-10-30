import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
type Props = {
  label: string; hint: string;
  selectedKey: string;
  items: { key: string; number: string; front: string; back: string; label: string; disabled: boolean; }[];
  onSelect: (key: string) => void;
};
export function CardSelectionRail(props: Props) {
  return <section className="aw-inspector-section" aria-label={props.label}>
    <p className="aw-muted">{props.hint}</p>
    <RadioGroup value={props.selectedKey} orientation="vertical" onValueChange={props.onSelect} aria-label={props.label}>
      {props.items.map(item => <RadioGroupItem key={item.key} value={item.key} disabled={item.disabled} className="aw-button" aria-label={item.label}>
        <span>{item.number}</span><span>{item.front}</span><span>{item.back}</span>
      </RadioGroupItem>)}
    </RadioGroup>
  </section>;
}
