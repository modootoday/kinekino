//! Product identifiers and display names. No other code may hard-code these literals.

/// Stable machine identifier; never changes with branding.
pub const PRODUCT_ID: &str = "kinekino";
/// Korean display name.
pub const DISPLAY_NAME_KO: &str = "키네키노";
/// English display name.
pub const DISPLAY_NAME_EN: &str = "Kinekino";
/// Reverse-domain bundle identifier.
pub const BUNDLE_ID: &str = "com.kinekino.app";
/// Prefix for environment variables (`KINEKINO_<CONTEXT>_<KEY>`).
pub const ENV_PREFIX: &str = "KINEKINO_";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_values_are_consistent_with_product_id() {
        assert!(BUNDLE_ID.ends_with(".app") && BUNDLE_ID.contains(PRODUCT_ID));
        assert_eq!(ENV_PREFIX, format!("{}_", PRODUCT_ID.to_uppercase()));
        assert_eq!(DISPLAY_NAME_EN.to_lowercase(), PRODUCT_ID);
    }
}
