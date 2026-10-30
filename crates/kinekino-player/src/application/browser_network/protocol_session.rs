use crate::application::api::{ApiError, ErrorCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum ProtocolSession {
    #[default]
    Default,
    Target(String),
    Unavailable,
}
impl ProtocolSession {
    pub fn from_observed(value: Option<&str>) -> Result<Self, ApiError> {
        match value {
            None => Ok(Self::Unavailable),
            Some("") => Ok(Self::Default),
            Some(value) if value.len() <= 128 && !value.chars().any(char::is_control) => {
                Ok(Self::Target(value.into()))
            }
            Some(_) => Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid observed protocol session identity",
            )),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_session_is_not_promoted_to_default_or_another_target() {
        assert!(matches!(
            ProtocolSession::from_observed(None).unwrap(),
            ProtocolSession::Unavailable
        ));
        assert!(matches!(
            ProtocolSession::from_observed(Some("")).unwrap(),
            ProtocolSession::Default
        ));
        assert!(matches!(
            ProtocolSession::from_observed(Some("target-session")).unwrap(),
            ProtocolSession::Target(_)
        ));
        assert!(ProtocolSession::from_observed(Some("target\nother")).is_err());
    }
}
