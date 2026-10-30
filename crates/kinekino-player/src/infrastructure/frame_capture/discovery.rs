//! Bounded discovery diagnostics; never expose backend stdout/stderr or infer success.
use super::process::ProcessOutput;
use crate::application::api::ApiError;

pub(super) fn observe(
    output: Result<ProcessOutput, ApiError>,
    stage: &str,
    failures: &mut Vec<String>,
) -> Option<ProcessOutput> {
    output
        .map_err(|error| failures.push(format!("{stage}: {:?}", error.code)))
        .ok()
}

pub(super) fn version(
    output: Result<ProcessOutput, ApiError>,
    stage: &str,
) -> Result<String, String> {
    let output = output.map_err(|error| format!("{stage}: {:?}", error.code))?;
    String::from_utf8(output.stdout)
        .map_err(|_| format!("{stage}: invalid UTF-8"))?
        .lines()
        .next()
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("{stage}: empty response"))
}

pub(super) fn reason(failures: &[String]) -> String {
    if failures.is_empty() {
        "Trusted local FFmpeg and FFprobe executables were not found or required versions/features were absent".into()
    } else {
        format!(
            "Native frame dependency checks failed: {}",
            failures.join("; ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::api::ErrorCode;

    #[test]
    fn failure_stage_and_code_survive_without_backend_details() {
        let mut failures = Vec::new();
        let mut error = ApiError::new(ErrorCode::Timeout, "private backend text");
        error.details = serde_json::json!({"secret":"not a public diagnostic"});
        assert!(observe(Err(error), "ffmpeg_filters", &mut failures).is_none());
        assert_eq!(
            reason(&failures),
            "Native frame dependency checks failed: ffmpeg_filters: Timeout"
        );
    }

    #[test]
    fn version_parse_failure_is_distinct_from_process_timeout() {
        let output = ProcessOutput {
            stdout: vec![255],
            stderr: b"private".to_vec(),
        };
        assert_eq!(
            version(Ok(output), "ffprobe_version").unwrap_err(),
            "ffprobe_version: invalid UTF-8"
        );
        assert_eq!(
            version(
                Err(ApiError::new(ErrorCode::Timeout, "private")),
                "ffprobe_version"
            )
            .unwrap_err(),
            "ffprobe_version: Timeout"
        );
    }
}
