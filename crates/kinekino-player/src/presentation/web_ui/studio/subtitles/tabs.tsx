import { TabsList, TabsTrigger } from "@/components/ui/tabs";
type Props = {};
export function SubtitleTabs(props: Props) {
  return <TabsList className="aw-subtitle-tabs" aria-label="자막 작업 종류"><TabsTrigger className="aw-subtitle-tab" value="speech">음성 인식</TabsTrigger><TabsTrigger className="aw-subtitle-tab" value="speakers">화자 분리</TabsTrigger><TabsTrigger className="aw-subtitle-tab" value="ocr">화면 OCR</TabsTrigger><TabsTrigger className="aw-subtitle-tab" value="correction">교정</TabsTrigger><TabsTrigger className="aw-subtitle-tab" value="conversion">변환</TabsTrigger></TabsList>;
}
