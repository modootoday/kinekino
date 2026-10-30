type Props = { title: string; detail: string; };
export function InformationNotice(props: Props) {
  return <div className="aw-notice aw-notice-info" role="status"><span className="aw-icon aw-icon-info" aria-hidden="true"/><div className="aw-notice-content"><p className="aw-notice-title">{props.title}</p><p className="aw-notice-detail">{props.detail}</p></div></div>;
}
