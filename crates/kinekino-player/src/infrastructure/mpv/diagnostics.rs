use super::Mpv;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        catalog::{CaptureMetadata, CatalogArtifacts, CatalogCaptureKind},
        playback_session::NativeEngineDiagnostics,
    },
    domain::api_values::DecimalU64,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha256};

pub(super) fn record(
    mpv: &Mpv,
    source_artifact: &str,
    source_url: &str,
    store: &dyn CatalogArtifacts,
    worker_exited: bool,
) -> Result<NativeEngineDiagnostics, ApiError> {
    let snapshot = mpv
        .stderr
        .lock()
        .map_err(|_| ApiError::new(ErrorCode::Internal, "Native diagnostics unavailable"))?
        .clone();
    let stderr_sha256 = format!("{:x}", Sha256::digest(&snapshot.bytes));
    let stdout = mpv
        .stdout
        .lock()
        .map_err(|_| ApiError::new(ErrorCode::Internal, "Native diagnostics unavailable"))?
        .clone();
    let stdout_sha256 = format!("{:x}", Sha256::digest(&stdout.bytes));
    let truncated = snapshot.truncated || stdout.truncated;
    let read_failed = snapshot.read_failed || stdout.read_failed;
    let complete = collection_complete(&snapshot, &stdout, worker_exited);
    let bytes = serde_json::to_vec(&serde_json::json!({
        "schema_version":"1","kind":"native_engine_output",
        "engine_instance_id":DecimalU64(mpv.control_handle().instance_id()),
        "resolution_artifact_id":source_artifact,
        "stderr_encoding":"base64","stderr_base64":STANDARD.encode(&snapshot.bytes),
        "stderr_bytes":DecimalU64(snapshot.bytes.len() as u64),"stderr_sha256":stderr_sha256,
        "stdout_encoding":"base64","stdout_base64":STANDARD.encode(&stdout.bytes),
        "stdout_bytes":DecimalU64(stdout.bytes.len() as u64),"stdout_sha256":stdout_sha256,
        "stderr_collector_started":snapshot.collector_started,"stdout_collector_started":stdout.collector_started,
        "stderr_eof_observed":snapshot.eof_observed,"stdout_eof_observed":stdout.eof_observed,
        "stderr_truncated":snapshot.truncated,"stdout_truncated":stdout.truncated,
        "stderr_read_failed":snapshot.read_failed,"stdout_read_failed":stdout.read_failed,
        "complete":complete,"worker_exited":worker_exited,"truncated":truncated,"read_failed":read_failed,
    })).map_err(|_| ApiError::new(ErrorCode::Internal, "Native diagnostics encoding failed"))?;
    let metadata = CaptureMetadata {
        requested_url: source_url.into(),
        final_url: source_url.into(),
        redirects: vec![],
        observed_at: time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .map_err(|_| ApiError::new(ErrorCode::Internal, "Native capture time unavailable"))?,
        parser_revision: "native-mpv-output-v1".into(),
        capture_kind: CatalogCaptureKind::NativeEngineDiagnostic,
        status: None,
        content_type: "application/json".into(),
        content_encoding: None,
        body_sha256: format!("{:x}", Sha256::digest(&bytes)),
        body_bytes: DecimalU64(bytes.len() as u64),
        browser: None,
    };
    Ok(NativeEngineDiagnostics {
        artifact: store.save(&bytes, &metadata)?,
        stderr_sha256,
        stderr_bytes: DecimalU64(snapshot.bytes.len() as u64),
        stdout_sha256,
        stdout_bytes: DecimalU64(stdout.bytes.len() as u64),
        complete,
        truncated,
        read_failed,
    })
}

fn collection_complete(
    stderr: &super::process_output::OutputSnapshot,
    stdout: &super::process_output::OutputSnapshot,
    workers_exited: bool,
) -> bool {
    workers_exited
        && stderr.collector_started
        && stdout.collector_started
        && stderr.eof_observed
        && stdout.eof_observed
        && !stderr.truncated
        && !stdout.truncated
        && !stderr.read_failed
        && !stdout.read_failed
}

#[cfg(test)]
mod tests {
    use super::super::process_output::OutputSnapshot;
    use super::*;
    #[test]
    fn complete_requires_both_observed_eofs_and_worker_exit_without_output_loss() {
        let unobserved = OutputSnapshot::default();
        let observed = OutputSnapshot {
            collector_started: true,
            eof_observed: true,
            ..Default::default()
        };
        assert!(!collection_complete(&unobserved, &observed, true));
        assert!(!collection_complete(&observed, &unobserved, true));
        assert!(!collection_complete(&observed, &observed, false));
        assert!(collection_complete(&observed, &observed, true));
        let mut failed = observed.clone();
        failed.read_failed = true;
        assert!(!collection_complete(&failed, &observed, true));
        failed.read_failed = false;
        failed.truncated = true;
        assert!(!collection_complete(&observed, &failed, true));
    }
}
