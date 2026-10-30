//! Stable provider dataset identity; not a secret or a mutation permission.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposalStoreOrigin {
    schema_version: String,
    origin_id: String,
}
impl ProposalStoreOrigin {
    /// Only validates the data shape. Provider/root verification is separate.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != "proposal-origin/1"
            || self.origin_id.len() != 32
            || !self
                .origin_id
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        {
            return Err("Invalid proposal store origin");
        }
        Ok(())
    }
    pub fn origin_id(&self) -> &str {
        &self.origin_id
    }
    pub(crate) fn issued(origin_id: String) -> Result<Self, &'static str> {
        let value = Self {
            schema_version: "proposal-origin/1".into(),
            origin_id,
        };
        value.validate()?;
        Ok(value)
    }
}
