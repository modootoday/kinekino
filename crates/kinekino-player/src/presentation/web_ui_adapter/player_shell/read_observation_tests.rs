use super::*;

#[test]
fn local_retry_initial_props_are_disabled_without_exhausted_ticket_authority() {
    let props = local_action_props();
    for name in [
        "capabilitiesRetryDisabled",
        "engineRetryDisabled",
        "statusRetryDisabled",
    ] {
        assert_eq!(props.get(name), Some(&Value::Bool(true)));
    }
    assert_eq!(
        props.get("retryAckLabelKey"),
        Some(&Value::String("viewer.read.retry_ack".into()))
    );
    assert_ne!(
        props.get("retryAckLabelKey"),
        props.get("refreshProjectionLabelKey")
    );
}

#[test]
fn error_key_is_stable_and_raw_owner_diagnostic_is_not_rendered() {
    let mut panel = ReadPanel::default();
    panel.rows[0].error_key = Some("viewer.read.ack_failed");
    panel.rows[0].diagnostic = Some("raw internal endpoint".into());
    let props = panel.props();
    assert_eq!(
        props.get("capabilitiesErrorCode"),
        Some(&Value::String("viewer.read.ack_failed".into()))
    );
    assert_ne!(
        props.get("capabilitiesError"),
        Some(&Value::String("raw internal endpoint".into()))
    );
}
#[test]
fn unobserved_values_have_no_numeric_playback_defaults() {
    let props = ReadPanel::default().props();
    assert_eq!(
        props.get("statusValue"),
        Some(&Value::String("미관측".into()))
    );
    assert_eq!(props.get("statusBusy"), Some(&Value::Bool(false)));
}
#[test]
fn pending_preserves_previous_observation() {
    let mut panel = ReadPanel::default();
    panel.rows[0].value = Some("previous".into());
    panel.rows[0].phase = ReadPhase::Pending;
    assert_eq!(
        panel.props().get("capabilitiesValue"),
        Some(&Value::String("previous".into()))
    );
    assert_eq!(
        panel.props().get("capabilitiesBusy"),
        Some(&Value::Bool(true))
    );
}
