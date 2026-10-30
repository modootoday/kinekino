use super::super::{
    bootstrap::BootstrapLabels, correction_panel::CorrectionLabels, panel_model::PanelLabels,
};

#[derive(Clone)]
pub struct ProductWindowLabels {
    pub panel: PanelLabels,
    pub source_count: String,
    pub unselected_source: String,
}
impl ProductWindowLabels {
    /// Presentation default; callers may inject translated labels without changing SDK data.
    pub fn korean() -> Self {
        Self {
            panel: PanelLabels {
                feedback: crate::presentation::web_ui_adapter::proposals::review_feedback::FeedbackLabels::korean(),
                revision: "제안 문서".into(),
                source_identity: "저장된 원본".into(),
                quality_unknown: "품질 미판정".into(),
                selected_item: "명시 선택 항목".into(),
                range: "원본 참조".into(),
                result: String::new(),
            },
            source_count: "저장된 문서 원문".into(),
            unselected_source: "원문을 선택하세요".into(),
        }
    }
    pub fn bootstrap(&self) -> BootstrapLabels {
        BootstrapLabels {
            panel: self.panel.clone(),
            source_count_label: self.source_count.clone(),
            unselected_source_text: self.unselected_source.clone(),
        }
    }
    pub fn correction(&self) -> CorrectionLabels {
        CorrectionLabels {
            panel: self.panel.clone(),
            source_count: self.source_count.clone(),
            unselected_source: self.unselected_source.clone(),
        }
    }
}
