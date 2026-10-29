use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

/// Lossless wire representation for source microseconds, revisions and sequences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DecimalU64(pub u64);

impl Serialize for DecimalU64 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for DecimalU64 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(D::Error::custom("Expected unsigned decimal string"));
        }
        value.parse().map(Self).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_integers_beyond_javascript_precision() {
        let value = DecimalU64(u64::MAX);
        let wire = serde_json::to_string(&value).unwrap();
        assert_eq!(wire, "\"18446744073709551615\"");
        assert_eq!(serde_json::from_str::<DecimalU64>(&wire).unwrap(), value);
        for wire in ["1", "\"+1\"", "\"-1\"", "\"18446744073709551616\""] {
            assert!(serde_json::from_str::<DecimalU64>(wire).is_err());
        }
    }
}
