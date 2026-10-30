type Props = { title: string; losses: { id: string; text: string; }[]; };
export function ConversionLossNotice(props: Props) {
  return <section className="aw-loss-notice" aria-label="변환 손실 확인"><span className="aw-icon aw-icon-warning" aria-hidden="true"/><div><h3>{props.title}</h3><ul>{props.losses.map(loss => <li key={loss.id}>{loss.text}</li>)}</ul></div></section>;
}
