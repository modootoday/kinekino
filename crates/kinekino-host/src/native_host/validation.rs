//! Trusted feature-only CLI assembly. No request payload or ambient env activation.
use kinekino_player::{
    application::api::{ApiError, ApplicationClientHandle, ErrorCode},
    infrastructure::local_host::{
        LocalHost, TransportOptions,
        validation::{self, ProofSink, TRACE_BYTES, ValidationDeliveryLoss},
    },
};
use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};
const DROP: &str = "--validation-export-drop-request-id";
const EVIDENCE: &str = "--validation-export-evidence";
struct Settings {
    request_id: String,
    path: PathBuf,
}
pub(super) struct Prepared {
    request_id: String,
    sink: ProofSink,
}
pub(super) struct Report(Arc<ValidationDeliveryLoss>);
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Validation export requires one request id and a fresh absolute proof path",
    )
}
fn parse(args: &[String]) -> Result<(Vec<String>, Option<Settings>), ApiError> {
    let mut normal = Vec::new();
    let mut id = None;
    let mut path = None;
    let mut index = 0;
    while index < args.len() {
        let slot = match args[index].as_str() {
            DROP => Some(&mut id),
            EVIDENCE => Some(&mut path),
            _ => None,
        };
        if let Some(slot) = slot {
            index += 1;
            let value = args
                .get(index)
                .filter(|value| !value.starts_with("--"))
                .ok_or_else(invalid)?;
            if slot.replace(value.clone()).is_some() {
                return Err(invalid());
            }
        } else {
            normal.push(args[index].clone());
        }
        index += 1;
    }
    match (id, path) {
        (None, None) => Ok((normal, None)),
        (Some(request_id), Some(path)) => {
            validation::validate_request_id(&request_id)?;
            let path = PathBuf::from(path);
            if !path.is_absolute() {
                return Err(invalid());
            }
            Ok((normal, Some(Settings { request_id, path })))
        }
        _ => Err(invalid()),
    }
}
struct Output {
    file: File,
    written: usize,
}
pub(super) fn prepare(args: &[String]) -> Result<(Vec<String>, Option<Prepared>), ApiError> {
    let (normal, settings) = parse(args)?;
    let Some(settings) = settings else {
        return Ok((normal, None));
    };
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&settings.path)
        .map_err(|_| {
            ApiError::new(
                ErrorCode::EngineFailed,
                "Cannot create fresh validation evidence",
            )
        })?;
    let output = Arc::new(Mutex::new(Output { file, written: 0 }));
    let sink: ProofSink = Arc::new(move |bytes| {
        let mut output = output
            .lock()
            .map_err(|_| io::Error::other("Validation output poisoned"))?;
        let next = output
            .written
            .checked_add(bytes.len())
            .filter(|next| *next <= TRACE_BYTES)
            .ok_or_else(|| io::Error::other("Validation output limit exceeded"))?;
        output.file.write_all(bytes)?;
        output.file.flush()?;
        output.written = next;
        Ok(())
    });
    Ok((
        normal,
        Some(Prepared {
            request_id: settings.request_id,
            sink,
        }),
    ))
}
pub(super) fn start(
    client: ApplicationClientHandle,
    options: TransportOptions,
    prepared: Option<Prepared>,
    probe: impl Fn() -> Result<(u64, u64), ApiError> + Send + Sync + 'static,
) -> Result<(LocalHost, Option<Report>), ApiError> {
    match prepared {
        None => LocalHost::start(client, options).map(|host| (host, None)),
        Some(prepared) => {
            let gate = Arc::new(ValidationDeliveryLoss::new(
                prepared.request_id,
                Arc::new(probe),
                prepared.sink,
            )?);
            LocalHost::start_validation_delivery(client, options, gate.clone())
                .map(|host| (host, Some(Report(gate))))
        }
    }
}
pub(super) fn finish(
    report: Option<Report>,
    original: Result<(), ApiError>,
) -> Result<(), ApiError> {
    let Some(report) = report else {
        return original;
    };
    let proof = report.0.finish().map_err(|_| {
        ApiError::new(
            ErrorCode::EngineFailed,
            "Original validation evidence could not be completed",
        )
    });
    crate::host_shutdown::preserve_first_error(original.err(), proof)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_pair_is_required_and_normal_arguments_are_unchanged() {
        let base = vec![
            "--serve-native".into(),
            "--catalog-root".into(),
            "D:/catalog".into(),
        ];
        let (same, settings) = parse(&base).unwrap();
        assert_eq!(same, base);
        assert!(settings.is_none());
        let mut valid = base.clone();
        valid.extend([
            DROP.into(),
            "original-request".into(),
            EVIDENCE.into(),
            "D:/proof.jsonl".into(),
        ]);
        let (same, settings) = parse(&valid).unwrap();
        assert_eq!(same, base);
        assert_eq!(settings.unwrap().request_id, "original-request");
        let mut incomplete = base;
        incomplete.extend([DROP.into(), "original-request".into()]);
        assert!(parse(&incomplete).is_err());
        valid.extend([DROP.into(), "duplicate".into()]);
        assert!(parse(&valid).is_err());
    }
    #[test]
    fn relative_evidence_and_invalid_selector_are_rejected_before_file_creation() {
        assert!(
            parse(&[
                DROP.into(),
                "original-request".into(),
                EVIDENCE.into(),
                "relative.jsonl".into()
            ])
            .is_err()
        );
        assert!(
            parse(&[
                DROP.into(),
                "x".repeat(129),
                EVIDENCE.into(),
                "D:/proof.jsonl".into()
            ])
            .is_err()
        );
    }
}
