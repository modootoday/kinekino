type Props = { sourceText: string; proposedText: string; stateLabel: string; };
export function TextComparison(props: Props) {
  return <section className="aw-text-comparison" aria-label="원문과 교정안 비교"><dl><dt>원문</dt><dd>{props.sourceText}</dd><dt>교정안</dt><dd className="aw-proposed-text">{props.proposedText}</dd></dl><p className="aw-comparison-state">{props.stateLabel}</p></section>;
}
