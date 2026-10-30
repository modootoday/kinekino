//! One existing document, fixed module references and current callback export supplied by host.
use crate::presentation::web_ui_adapter::studio::{
    inspector_state::InspectorViewState,
    jobs::{labels, list::JobsListProjection, selection::JobsSelection},
};
use meari::domain::{
    Value,
    build::BuildPath,
    host_callbacks::{CallbackReference, CallbackSignature},
    source_modules::ComponentReference,
};
use meari_native::infrastructure::document::InteractiveDocument;
use std::{collections::BTreeMap, num::NonZeroU64};

pub(super) struct JobsFrameInput<'a> {
    pub caption: &'a crate::presentation::web_ui_adapter::player_shell::window_caption::CaptionView,
    pub product_name: &'a str,
    pub context_label: &'a str,
    pub route: NonZeroU64,
    pub list: &'a JobsListProjection,
    pub selection: &'a JobsSelection,
    pub detail: Option<&'a BTreeMap<String, Value>>,
    pub inspector: InspectorViewState,
    pub blocked: bool,
    pub retry_list: bool,
    pub retry_detail: bool,
    pub connection_retry: bool,
    pub diagnostic: Option<&'a str>,
    pub status: Option<&'a str>,
}
fn reference(module: &str, export: &str) -> Result<ComponentReference, String> {
    Ok(ComponentReference {
        module: BuildPath::try_from(module.to_owned())?,
        export: export.into(),
    })
}
pub(super) fn props(
    document: &InteractiveDocument,
    input: JobsFrameInput<'_>,
) -> Result<BTreeMap<String, Value>, String> {
    let mut table = input.list.props(input.selection)?;
    if let Some(message) = input.diagnostic {
        table.insert(
            "status".into(),
            Value::String("조회 결과 확인을 마치지 못했습니다.".into()),
        );
        table.insert("diagnostic".into(), Value::String(message.into()));
    }
    if let Some(status) = input.status {
        table.insert("status".into(), Value::String(status.into()));
    }
    table.insert("retryDisabled".into(), Value::Bool(!input.retry_list));
    table.insert(
        "connectionRetryDisabled".into(),
        Value::Bool(!input.connection_retry),
    );
    table.insert(
        "onRetryJobsConnection".into(),
        callback("onRetryJobsConnection", CallbackSignature::Action),
    );
    table.insert(
        "onRetryJobsResult".into(),
        callback("onRetryJobsResult", CallbackSignature::Action),
    );
    if input.blocked {
        table.insert("refreshDisabled".into(), Value::Bool(true));
        if let Some(Value::Array(rows)) = table.get_mut("rows") {
            for row in &mut rows.items {
                if let Value::Record(row) = row {
                    row.fields.insert("disabled".into(), Value::Bool(true));
                }
            }
        }
    }
    let workspace = document.node_slot(
        &format!("jobs-table-{}", input.route),
        reference("studio/jobs/read-workspace.tsx", "ReadJobsWorkspace")?,
        table,
    )?;
    let inspector = if let Some(detail) = input.detail {
        let selected = input
            .selection
            .active()
            .ok_or("Detail has no committed selection")?;
        let mut detail = detail.clone();
        if input.blocked {
            detail.insert("refreshDisabled".into(), Value::Bool(true));
        }
        detail.insert("retryDisabled".into(), Value::Bool(!input.retry_detail));
        document.node_slot(
            &format!("jobs-detail-{}-{}", input.route, selected.generation()),
            reference("studio/jobs/detail-panel.tsx", "JobDetailsPanel")?,
            detail,
        )?
    } else {
        document.node_slot(
            &format!("jobs-no-selection-{}", input.route),
            reference("components/notice.tsx", "InformationNotice")?,
            BTreeMap::from([
                (
                    "title".into(),
                    Value::String("선택한 작업이 없습니다.".into()),
                ),
                ("detail".into(), Value::String(labels::NO_SELECTION.into())),
            ]),
        )?
    };
    let inspector_state = input.inspector.projection();
    let count = input
        .list
        .page()
        .map(|page| format!("현재 페이지 {}개", page.jobs.len()))
        .unwrap_or_else(|| "목록 미조회".into());
    let mut props = BTreeMap::from([
        (
            "productName".into(),
            Value::String(input.product_name.into()),
        ),
        (
            "contextLabel".into(),
            Value::String(input.context_label.into()),
        ),
        ("workspace".into(), workspace),
        ("inspector".into(), inspector),
        ("narrow".into(), Value::Bool(inspector_state.modal)),
        (
            "inspectorOpen".into(),
            Value::Bool(inspector_state.drawer_open),
        ),
        ("queueSummaryLabel".into(), Value::String(count)),
        (
            "onNavigate".into(),
            callback("onNavigate", CallbackSignature::Text),
        ),
        (
            "onInspectorOpenChange".into(),
            callback("onInspectorOpenChange", CallbackSignature::Boolean),
        ),
    ]);
    props.extend(input.caption.props());
    Ok(props)
}
fn callback(name: &str, signature: CallbackSignature) -> Value {
    Value::CallbackReference(CallbackReference {
        binding: name.into(),
        signature,
    })
}
