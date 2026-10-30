//! Thin presentation consumer of the original chooser owner; no SDK request or route policy.
use super::*;
use crate::{
    application::{launch::native_proposal::NativeProposalTarget, proposals::list::ListProposals},
    assembly::meari::viewer_callbacks::chooser::ChooserPhase,
    domain::proposals::ProposalTask,
    presentation::web_ui_adapter::player_shell::{
        ShellPhase,
        document::proposal_chooser::{self, ChoiceRow, ChooserLabels, ChooserView, OpenAffordance},
    },
};
use meari::{
    application::{
        host_callbacks::LocalDispatchError,
        plugins::{PluginError, PluginErrorCode},
    },
    domain::{Value, plugins::PluginScope},
};
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct Projection {
    committed: Option<(PluginScope, BTreeMap<String, Value>)>,
}
impl Projection {
    fn changed(&self, scope: &PluginScope, props: &BTreeMap<String, Value>) -> bool {
        self.committed
            .as_ref()
            .is_none_or(|(old_scope, old_props)| old_scope != scope || old_props != props)
    }
    pub(super) fn committed(&mut self, scope: PluginScope, props: BTreeMap<String, Value>) {
        self.committed = Some((scope, props));
    }
}

pub(super) fn projection_error(error: impl Into<String>) -> ViewerHostError {
    ViewerHostError::Callback(Arc::new(PluginError::new(PluginErrorCode::Adapter, error)))
}

fn task_label(task: ProposalTask) -> &'static str {
    match task {
        ProposalTask::Cards => "카드",
        ProposalTask::Highlights => "하이라이트",
        ProposalTask::Correction => "자막 교정",
    }
}

fn item_label(target: &NativeProposalTarget) -> String {
    match target {
        NativeProposalTarget::Card { index } => format!("카드 {}", u64::from(*index) + 1),
        NativeProposalTarget::Highlight { index } => {
            format!("하이라이트 {}", u64::from(*index) + 1)
        }
        NativeProposalTarget::CueCorrection { index, .. } => {
            format!("자막 {}", u64::from(*index) + 1)
        }
        NativeProposalTarget::DocumentCorrection { index, .. } => {
            format!("문서 구간 {}", u64::from(*index) + 1)
        }
        NativeProposalTarget::DocumentRepair { .. } => "문서 복구".into(),
        NativeProposalTarget::ManualRepair { task } => format!("수동 수정 · {task:?}"),
    }
}

fn status(phase: ChooserPhase) -> &'static str {
    match phase {
        ChooserPhase::Idle => "목록을 조회한 뒤 문서와 항목을 선택하세요",
        ChooserPhase::Listing => "제안 목록 조회 중",
        ChooserPhase::PageReady => "제안 문서를 선택하세요",
        ChooserPhase::Inspecting => "선택한 문서 확인 중",
        ChooserPhase::DocumentReady => "제안 항목을 선택하세요",
        ChooserPhase::Prepared => "선택한 제안을 열 수 있습니다",
        ChooserPhase::Cancelling => "조회 취소 대기 중",
        ChooserPhase::Failed => "제안 조회 실패",
        ChooserPhase::Closing => "조회 정리 중",
    }
}

/// All selections and enablement are display affordances; the original owner authenticates input.
pub(super) fn display(
    owner: &mut ViewerCallbacks,
    scope: Option<&PluginScope>,
) -> Result<Option<BTreeMap<String, Value>>, ViewerHostError> {
    let Some(phase) = owner.chooser_phase() else {
        return Ok(None);
    };
    let occupied = owner.has_jobs_entry() || owner.has_live_entry();
    let busy = occupied
        || matches!(
            phase,
            ChooserPhase::Listing
                | ChooserPhase::Inspecting
                | ChooserPhase::Cancelling
                | ChooserPhase::Closing
        );
    let prepared = owner.chooser_prepared();
    let selected_document_key = owner
        .chooser_document()
        .and_then(|document| {
            owner
                .chooser_documents()
                .iter()
                .find(|row| row.metadata.proposal_id == document.document.proposal_id)
                .map(|row| row.key.clone())
        })
        .unwrap_or_default();
    let selected_item_key = prepared
        .and_then(|prepared| {
            owner
                .chooser_items()
                .iter()
                .find(|row| row.target == prepared.selection.target)
                .map(|row| row.key.clone())
        })
        .unwrap_or_default();
    let view = ChooserView {
        documents: owner
            .chooser_documents()
            .iter()
            .map(|row| ChoiceRow {
                key: row.key.clone(),
                title: format!(
                    "{} · {}",
                    task_label(row.metadata.task),
                    row.metadata.proposal_id
                ),
                detail: format!(
                    "수정본 {} · 항목 {}개",
                    row.metadata.draft_revision.0, row.metadata.typed_item_count
                ),
                disabled: busy,
            })
            .collect(),
        items: owner
            .chooser_items()
            .iter()
            .map(|row| ChoiceRow {
                key: row.key.clone(),
                title: item_label(&row.target),
                detail: String::new(),
                disabled: busy,
            })
            .collect(),
        selected_document_key,
        selected_item_key,
        status: status(phase).into(),
        error: owner
            .chooser_failure()
            .or_else(|| {
                owner
                    .chooser_terminal()
                    .and_then(|terminal| terminal.owner_ack.as_ref().err())
            })
            .map_or_else(String::new, |error| error.message.clone()),
        list_disabled: busy,
        next_disabled: busy
            || phase != ChooserPhase::PageReady
            || owner
                .chooser_page()
                .is_none_or(|page| page.next_cursor.is_none()),
        open: None,
    };
    // Validate untrusted display strings before registering even an owner-local Open row.
    let mut values =
        proposal_chooser::props(view, ChooserLabels::korean()).map_err(projection_error)?;
    if !busy && phase == ChooserPhase::Prepared {
        let scope = scope.ok_or(ViewerHostError::ForeignCheckpoint)?;
        let key = owner
            .register_prepared_open(scope)
            .map_err(|error| ViewerHostError::Callback(Arc::new(error)))?;
        let open = OpenAffordance {
            key,
            disabled: false,
        };
        values.insert("openKey".into(), Value::String(open.key));
        values.insert("openDisabled".into(), Value::Bool(open.disabled));
    }
    Ok(Some(values))
}

impl ViewerHost {
    pub(crate) fn configure_default_chooser(
        &mut self,
        root: &mut BTreeMap<String, Value>,
    ) -> Result<(), ViewerHostError> {
        let mut owner = self
            .callbacks
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        configure_initial(&mut owner, root)
    }
    pub(super) fn observed_viewing_for_chooser(&self) -> bool {
        !self.shutdown_started
            && self.product_frame.is_some()
            && self.route_observation.as_ref().is_some_and(|observer| {
                observer.try_borrow().is_ok_and(|observer| {
                    observer.state.phase() == ShellPhase::Observed
                        && observer
                            .state
                            .observed_route()
                            .is_some_and(|route| route.as_str() == "view.start")
                })
            })
    }

    pub(super) fn admit_chooser(
        &mut self,
        stamp: &HostStamp,
        callback: &PendingCallback,
    ) -> Option<IntentAdmission<ViewerHostError>> {
        if !matches!(
            callback.invocation.prop.as_str(),
            "onChooserList" | "onChooserNext" | "onChooserDocument" | "onChooserItem"
        ) {
            return None;
        }
        if !self.observed_viewing_for_chooser()
            || self.document.as_ref().is_none_or(|document| {
                document.lifecycle() != HostLifecycle::Open
                    || document.stamp().ok().as_ref() != Some(stamp)
            })
        {
            return Some(IntentAdmission::Rejected(
                ViewerHostError::ForeignCheckpoint,
            ));
        }
        let mut owner = match self.callbacks.try_borrow_mut() {
            Ok(owner) => owner,
            Err(_) => return Some(IntentAdmission::Capacity),
        };
        // Property names route to the original authenticator; they never authenticate the callback.
        Some(match owner.queue_chooser(&stamp.owner, callback) {
            Ok(()) => IntentAdmission::Accepted,
            Err(LocalDispatchError::Invalid(error)) if error.code == PluginErrorCode::Busy => {
                IntentAdmission::Capacity
            }
            Err(LocalDispatchError::Invalid(error)) => {
                IntentAdmission::Rejected(ViewerHostError::Callback(Arc::new(error)))
            }
            Err(LocalDispatchError::NotLocal) => {
                IntentAdmission::Rejected(ViewerHostError::UnsupportedCallback)
            }
        })
    }

    pub(super) fn seed_chooser_projection(
        &mut self,
        document: &NativeDocumentHandle,
        root: &BTreeMap<String, Value>,
    ) {
        if let Some(values) = proposal_chooser::staged_props(root)
            && let Ok(stamp) = document.stamp()
        {
            self.viewing_chooser.committed(stamp.owner, values.clone());
        }
    }

    pub(super) fn chooser_changed(
        &self,
        scope: &PluginScope,
        values: &BTreeMap<String, Value>,
    ) -> bool {
        self.viewing_chooser.changed(scope, values)
    }
}

pub(super) fn configure_initial(
    owner: &mut ViewerCallbacks,
    root: &mut BTreeMap<String, Value>,
) -> Result<(), ViewerHostError> {
    owner
        .configure_proposal_chooser(ListProposals::default())
        .map_err(|error| ViewerHostError::Callback(Arc::new(error)))?;
    if let Some(values) = display(owner, None)? {
        proposal_chooser::stage_root(root, values).map_err(projection_error)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
