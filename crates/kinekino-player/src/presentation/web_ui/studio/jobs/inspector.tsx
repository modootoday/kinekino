import { Button } from "@/components/ui/button";
import { InformationNotice } from "../../components/notice";
type Props = { title: string; sourceLabel: string; currentStage: string; stageDetail: string; progressLabel: string; elapsedLabel: string; remainingLabel: string; sourcePath: string; languageLabel: string; modelLabel: string; outputLabel: string; };
export function JobInspector(props: Props) {
  return <aside className="aw-inspector" aria-label="선택한 작업 정보">
    <div className="aw-inspector-scroll">
      <div className="aw-inspector-heading"><span className="aw-icon aw-icon-subtitles" aria-hidden="true"/><div><h2>{props.title}</h2><p className="aw-muted">{props.sourceLabel}</p></div></div>
      <ol className="aw-job-stages" aria-label="작업 단계 구성"><li><span>1</span>입력 검사</li><li><span>2</span>대기</li><li><span>3</span>실행</li><li><span>4</span>검증</li><li><span>5</span>완료</li></ol>
      <section className="aw-inspector-section"><h3>현재 단계</h3><InformationNotice title={props.currentStage} detail={props.stageDetail}/><dl className="aw-metadata"><dt>진행률</dt><dd>{props.progressLabel}</dd><dt>경과 시간</dt><dd>{props.elapsedLabel}</dd><dt>남은 시간</dt><dd>{props.remainingLabel}</dd></dl></section>
      <section className="aw-inspector-section"><h3>작업 입력</h3><dl className="aw-metadata"><dt>영상 파일</dt><dd>{props.sourcePath}</dd><dt>언어</dt><dd>{props.languageLabel}</dd><dt>모델</dt><dd>{props.modelLabel}</dd><dt>출력 형식</dt><dd>{props.outputLabel}</dd></dl></section>
    </div>
    <div className="aw-inspector-footer"><Button className="aw-button" disabled={true}>다시 시도</Button><Button className="aw-button" disabled={true}>취소</Button><Button className="aw-button" disabled={true}>출력 폴더</Button><p className="aw-muted">SDK capability와 작업 revision 연결 후 활성화합니다.</p></div>
  </aside>;
}
