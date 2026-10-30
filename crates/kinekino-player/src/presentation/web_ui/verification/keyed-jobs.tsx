// Preserved consumer requirement; awaits Meari's typed collection contract.
type Props = { rows: { id: string; title: string; }[]; };
export function KeyedJobs(props: Props) {
  return <div role="table">{props.rows.map(row => <div key={row.id} role="row"><span role="cell">{row.title}</span></div>)}</div>;
}
