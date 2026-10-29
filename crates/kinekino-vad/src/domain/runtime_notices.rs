use super::invalid;
use kinekino_media_contracts::errors::ApiError;

/// Native notices are separate from the model's license document.
pub const MAX_RUNTIME_NOTICE_BYTES: u64 = 1024 * 1024;
/// Preserve the former worst-case budget: sixteen 64 KiB notices.
pub const MAX_RUNTIME_NOTICES_TOTAL_BYTES: u64 = 1024 * 1024;
pub const MAX_RUNTIME_NOTICE_FILES: usize = 16;

/// Only sizes/count are policy here. Required presence belongs to each caller;
/// an absent optional deployment must not become a native admission request.
pub fn validate_runtime_notice_sizes(sizes: impl IntoIterator<Item = u64>) -> Result<(), ApiError> {
    let mut count = 0usize;
    let mut total = 0u64;
    for bytes in sizes {
        count = count
            .checked_add(1)
            .ok_or_else(|| invalid("Native notice count overflow"))?;
        if count > MAX_RUNTIME_NOTICE_FILES || bytes == 0 || bytes > MAX_RUNTIME_NOTICE_BYTES {
            return Err(invalid("Native notice file size or count exceeds bound"));
        }
        total = total
            .checked_add(bytes)
            .ok_or_else(|| invalid("Native notice total overflow"))?;
        if total > MAX_RUNTIME_NOTICES_TOTAL_BYTES {
            return Err(invalid("Native notice total exceeds bound"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ArtifactPin, MAX_LICENSE_BYTES, VadProfile};
    use kinekino_media_contracts::api_values::DecimalU64;

    #[test]
    fn acquired_notice_sizes_and_prior_maximum_budget_fit() {
        validate_runtime_notice_sizes([338_088, 1073]).unwrap();
        validate_runtime_notice_sizes([65_536; 16]).unwrap();
    }
    #[test]
    fn single_notice_exact_bound_and_one_byte_over_are_distinct() {
        validate_runtime_notice_sizes([MAX_RUNTIME_NOTICE_BYTES]).unwrap();
        assert!(validate_runtime_notice_sizes([MAX_RUNTIME_NOTICE_BYTES + 1]).is_err());
        assert!(validate_runtime_notice_sizes([0]).is_err());
        assert!(validate_runtime_notice_sizes([u64::MAX]).is_err());
    }
    #[test]
    fn total_limit_rejects_two_individually_valid_notices() {
        let half = MAX_RUNTIME_NOTICES_TOTAL_BYTES / 2;
        validate_runtime_notice_sizes([half, half]).unwrap();
        assert!(validate_runtime_notice_sizes([half, half + 1]).is_err());
    }
    #[test]
    fn optional_empty_list_is_preserved_but_count_is_bounded() {
        validate_runtime_notice_sizes([]).unwrap();
        validate_runtime_notice_sizes([1; 16]).unwrap();
        assert!(validate_runtime_notice_sizes([1; 17]).is_err());
    }
    #[test]
    fn native_notice_limit_does_not_expand_the_model_license_limit() {
        assert_eq!(MAX_LICENSE_BYTES, 65_536);
        let license = ArtifactPin::new(
            "license".into(),
            "a".repeat(64),
            DecimalU64(MAX_LICENSE_BYTES + 1),
        )
        .unwrap();
        let runtime = ArtifactPin::new("worker".into(), "b".repeat(64), DecimalU64(16)).unwrap();
        assert!(VadProfile::cached_silero(license, runtime).is_err());
    }
}
