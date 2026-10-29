//! Protocol families and majors. Crate semver and protocol versions move separately; a
//! protocol major bump forces a crate major bump (minor while 0.x).

/// `source/1`: catalog, search, episode and stream resolution.
pub const SOURCE_API: &str = "source/1";
/// `document/1`: page fetch and parsing, multiplexed on the same executable.
pub const DOCUMENT_PROTOCOL: &str = "document/1";
/// Request lines carry `document/1` methods under this field.
pub const PROTOCOL_FIELD: &str = "protocol";
