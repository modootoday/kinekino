//! Canonical lookup values only. Parsing never creates an issuer or control authority.
use crate::application::api::{ApiError, ErrorCode};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::{fmt, num::NonZeroU64, str::FromStr};

pub const NATIVE_ISSUER_SCOPE_V1_BYTES: usize = 37;
pub const NATIVE_RESULT_REF_V1_BYTES: usize = 54;

#[derive(Clone, Copy, Debug)]
pub struct NativeExportValueError {
    pub code: ErrorCode,
}
impl fmt::Display for NativeExportValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid bounded native export value")
    }
}
impl std::error::Error for NativeExportValueError {}
impl From<ApiError> for NativeExportValueError {
    fn from(error: ApiError) -> Self {
        Self { code: error.code }
    }
}
impl From<NativeExportValueError> for ApiError {
    fn from(error: NativeExportValueError) -> Self {
        ApiError::new(error.code, "Invalid bounded native export value")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NativeIssuerScopeV1([u8; 16]);
impl NativeIssuerScopeV1 {
    pub(crate) fn from_original_nonce(nonce: [u8; 16]) -> Result<Self, ApiError> {
        if nonce == [0; 16] {
            return Err(invalid());
        }
        Ok(Self(nonce))
    }
}
impl FromStr for NativeIssuerScopeV1 {
    type Err = ApiError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() != NATIVE_ISSUER_SCOPE_V1_BYTES || !value.starts_with("nsi1.") {
            return Err(invalid());
        }
        Self::from_original_nonce(decode_nonce(&value.as_bytes()[5..])?)
    }
}
impl fmt::Display for NativeIssuerScopeV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("nsi1.")?;
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NativeResultRefV1 {
    scope: NativeIssuerScopeV1,
    row: NonZeroU64,
}
impl NativeResultRefV1 {
    pub fn scope(&self) -> NativeIssuerScopeV1 {
        self.scope
    }
    #[allow(dead_code, reason = "P1 resolves the existing native row number")]
    pub(crate) fn original_row(&self) -> NonZeroU64 {
        self.row
    }
    pub(crate) fn from_original_row(scope: NativeIssuerScopeV1, row: NonZeroU64) -> Self {
        Self { scope, row }
    }
}
impl FromStr for NativeResultRefV1 {
    type Err = ApiError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let bytes = value.as_bytes();
        if bytes.len() != NATIVE_RESULT_REF_V1_BYTES
            || !bytes.starts_with(b"nsr1.")
            || bytes[37] != b'.'
        {
            return Err(invalid());
        }
        let scope = NativeIssuerScopeV1::from_original_nonce(decode_nonce(&bytes[5..37])?)?;
        let mut row = 0_u64;
        for byte in &bytes[38..] {
            row = (row << 4) | u64::from(nibble(*byte)?);
        }
        Ok(Self::from_original_row(
            scope,
            NonZeroU64::new(row).ok_or_else(invalid)?,
        ))
    }
}
impl fmt::Display for NativeResultRefV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("nsr1.")?;
        for byte in self.scope.0 {
            write!(f, "{byte:02x}")?;
        }
        write!(f, ".{:016x}", self.row.get())
    }
}
fn nibble(byte: u8) -> Result<u8, ApiError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(invalid()),
    }
}
fn decode_nonce(bytes: &[u8]) -> Result<[u8; 16], ApiError> {
    if bytes.len() != 32 {
        return Err(invalid());
    }
    let mut nonce = [0; 16];
    for (index, pair) in bytes.as_chunks::<2>().0.iter().enumerate() {
        nonce[index] = (nibble(pair[0])? << 4) | nibble(pair[1])?;
    }
    Ok(nonce)
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Invalid canonical native export v1 lookup value",
    )
}

macro_rules! lookup_serde {
    ($kind:ty) => {
        impl Serialize for $kind {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.collect_str(self)
            }
        }
        impl<'de> Deserialize<'de> for $kind {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                struct LookupVisitor;
                impl de::Visitor<'_> for LookupVisitor {
                    type Value = $kind;
                    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        f.write_str("a canonical native export v1 lookup string")
                    }
                    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                        value.parse().map_err(|_: ApiError| {
                            E::custom("invalid native export v1 lookup string")
                        })
                    }
                }
                deserializer.deserialize_str(LookupVisitor)
            }
        }
    };
}
lookup_serde!(NativeIssuerScopeV1);
lookup_serde!(NativeResultRefV1);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_scope_and_original_u64_reference_roundtrip_at_exact_widths() {
        let scope = NativeIssuerScopeV1::from_original_nonce([0xab; 16]).unwrap();
        for row in [1, u64::MAX] {
            let reference =
                NativeResultRefV1::from_original_row(scope, NonZeroU64::new(row).unwrap());
            assert_eq!(scope.to_string().len(), 37);
            assert_eq!(reference.to_string().len(), 54);
            assert_eq!(
                scope.to_string().parse::<NativeIssuerScopeV1>().unwrap(),
                scope
            );
            assert_eq!(
                reference.to_string().parse::<NativeResultRefV1>().unwrap(),
                reference
            );
            assert_eq!(reference.original_row().get(), row);
            assert_eq!(
                serde_json::from_str::<NativeResultRefV1>(
                    &serde_json::to_string(&reference).unwrap()
                )
                .unwrap(),
                reference
            );
        }
    }
    #[test]
    fn scope_rejects_zero_version_case_unicode_whitespace_and_width_changes() {
        let valid = "nsi1.abcdefabcdefabcdefabcdefabcdefab";
        for value in [
            "nsi1.00000000000000000000000000000000".to_owned(),
            valid.replace("nsi1", "nsi2"),
            valid.to_uppercase(),
            format!(" {valid}"),
            format!("{valid}.x"),
            valid.replace('a', "가"),
            valid[..36].to_owned(),
        ] {
            assert_eq!(
                value.parse::<NativeIssuerScopeV1>().unwrap_err().code,
                ErrorCode::InvalidArgument
            );
        }
    }
    #[test]
    fn reference_rejects_noncanonical_row_and_pair_shape() {
        let value = "nsr1.abcdefabcdefabcdefabcdefabcdefab.0000000000000001";
        for bad in [
            value.replace("0000000000000001", "0000000000000000"),
            value.replace("nsr1", "nsr2"),
            value.to_uppercase(),
            format!("{value} "),
            value.replace(".0000000000000001", ".1"),
            value.replace(".0000000000000001", ".g000000000000001"),
        ] {
            assert_eq!(
                bad.parse::<NativeResultRefV1>().unwrap_err().code,
                ErrorCode::InvalidArgument
            );
        }
        assert!(serde_json::from_str::<NativeResultRefV1>("17").is_err());
    }
}
