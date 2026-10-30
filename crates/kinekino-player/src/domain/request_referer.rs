use serde::{Deserialize, Serialize};

/// Header absence/empty/value observed in the network stack. Unknown is Option::None outside this type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum RequestReferer {
    Absent,
    Empty,
    Url(String),
}
impl RequestReferer {
    pub fn observed(value: Option<String>) -> Self {
        match value {
            None => Self::Absent,
            Some(value) if value.is_empty() => Self::Empty,
            Some(value) => Self::Url(value),
        }
    }
    pub fn allowed(&self) -> bool {
        match self {
            Self::Absent | Self::Empty => true,
            Self::Url(value) => {
                value.len() <= 8192
                    && crate::domain::navigation::navigation_allowed(value)
                    && !value.chars().any(char::is_control)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observed_absence_and_empty_are_distinct_and_unsafe_values_stay_rejected() {
        assert_eq!(RequestReferer::observed(None), RequestReferer::Absent);
        assert_eq!(
            RequestReferer::observed(Some(String::new())),
            RequestReferer::Empty
        );
        for value in [
            "http://media.example.test/",
            "https://user:password@media.example.test/",
            "https://media.example.test/\r\nInjected: value",
            "file:///private",
        ] {
            assert!(!RequestReferer::observed(Some(value.into())).allowed());
        }
    }
}
