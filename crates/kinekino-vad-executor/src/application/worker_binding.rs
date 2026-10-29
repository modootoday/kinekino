use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::domain::{ArtifactPin, VadProfile};

/// Call only after the existing trusted admission authorizes `approved`.
/// This validates release composition, not file bytes, process identity or rights.
pub fn bind_static_worker(
    approved: &VadProfile,
    requested: &VadProfile,
    executable: &ArtifactPin,
) -> Result<(), ApiError> {
    if requested != approved || approved.runtime() != executable {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "VAD worker differs from the admitted static release",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinekino_media_contracts::api_values::DecimalU64;

    fn pin(id: &str, digest: char) -> ArtifactPin {
        ArtifactPin::new(id.into(), digest.to_string().repeat(64), DecimalU64(16)).unwrap()
    }
    fn profile(worker: ArtifactPin, revision: char) -> VadProfile {
        VadProfile::new(
            "fixture".into(),
            "onnx-community/silero-vad".into(),
            revision.to_string().repeat(40),
            pin("weights", 'a'),
            pin("license", 'b'),
            worker,
        )
        .unwrap()
    }
    #[test]
    fn matching_admitted_release_binds() {
        let worker = pin("worker", 'c');
        let approved = profile(worker.clone(), 'd');
        assert!(bind_static_worker(&approved, &approved, &worker).is_ok());
    }
    #[test]
    fn same_size_other_digest_is_rejected() {
        let approved = profile(pin("worker", 'c'), 'd');
        assert!(bind_static_worker(&approved, &approved, &pin("worker", 'e')).is_err());
    }
    #[test]
    fn cross_release_with_same_worker_is_rejected() {
        let worker = pin("worker", 'c');
        let approved = profile(worker.clone(), 'd');
        let requested = profile(worker.clone(), 'e');
        assert!(bind_static_worker(&approved, &requested, &worker).is_err());
    }
    #[test]
    fn same_content_other_identity_is_rejected() {
        let approved = profile(pin("worker", 'c'), 'd');
        assert!(bind_static_worker(&approved, &approved, &pin("alias", 'c')).is_err());
    }
}
