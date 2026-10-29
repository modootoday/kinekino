//! Versioned Metadata payload; untrusted transport observations, never authority.
mod codec;
use crate::request::{MAX_REQUEST_BYTES, REQUEST_HEADER_BYTES, https};

pub const METADATA_HEADER_BYTES: usize = 20;
pub const MAX_URL_BYTES: usize = MAX_REQUEST_BYTES - REQUEST_HEADER_BYTES;
pub const MAX_TYPE_BYTES: usize = 256;
pub const MAX_HTTP_METADATA_BYTES: usize = METADATA_HEADER_BYTES + MAX_URL_BYTES + MAX_TYPE_BYTES;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetadataError {
    Framing,
    Version,
    Limit,
    Url,
    ContentType,
    Length,
}

/// No Debug/Clone/Serialize: the original URL may contain sensitive query values.
/// v1 represents a whole, identity-encoded response; no trusted digest is carried.
pub struct HttpMetadata {
    url: url::Url,
    length: Option<u64>,
    content_type: Option<String>,
}
impl HttpMetadata {
    pub fn new(
        url: &str,
        length: Option<u64>,
        content_type: Option<&str>,
    ) -> Result<Self, MetadataError> {
        if url.len() > MAX_URL_BYTES {
            return Err(MetadataError::Limit);
        }
        let url = https(url).map_err(|_| MetadataError::Url)?;
        if url.as_str().len() > MAX_URL_BYTES {
            return Err(MetadataError::Limit);
        }
        if let Some(value) = content_type
            && (value.is_empty()
                || value.len() > MAX_TYPE_BYTES
                || !value.bytes().all(|b| (32..=126).contains(&b))
                || value.to_ascii_lowercase().contains("html"))
        {
            return Err(MetadataError::ContentType);
        }
        Ok(Self {
            url,
            length,
            content_type: content_type.map(str::to_owned),
        })
    }
    pub fn url(&self) -> &url::Url {
        &self.url
    }
    pub fn response_length(&self) -> Option<u64> {
        self.length
    }
    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }
    /// Parent rechecks against its original admitted request; this is not replay
    /// issuance, source identity proof, media probing, or trusted hash admission.
    pub fn check_request(
        &self,
        expected_url: &url::Url,
        maximum: u64,
    ) -> Result<(), MetadataError> {
        if &self.url != expected_url {
            return Err(MetadataError::Url);
        }
        if maximum == 0 || self.length.is_some_and(|n| n > maximum) {
            return Err(MetadataError::Length);
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>, MetadataError> {
        codec::encode(self)
    }
    pub fn decode_exact(bytes: &[u8]) -> Result<Self, MetadataError> {
        codec::decode_exact(bytes)
    }
}
