//! Sensitive transport data, never browser replay permission. No network or Debug.
mod codec;
mod receiver;
pub use receiver::BootstrapReceiver;
use url::Url;

pub const MAX_REQUEST_BYTES: usize = 8 * 1024;
pub const REQUEST_HEADER_BYTES: usize = 52;
pub const REQUEST_MAGIC: [u8; 4] = *b"MHR1";
pub const REQUEST_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootstrapError {
    Framing,
    Version,
    Truncated,
    Limit,
    EmptyLimit,
    InvalidUrl,
    Credentials,
    Fragment,
}

/// Data received only after the parent has independently admitted an original
/// source. Constructing/decoding this value grants no execution authority.
/// Query values can be sensitive: never put these bytes in evidence or argv.
pub struct HttpBootstrap {
    binding: [u8; 32],
    maximum_bytes: u64,
    target: Url,
    referer: Option<Url>,
}
fn lengths(target: usize, referer: usize) -> Result<usize, BootstrapError> {
    if target == 0 {
        return Err(BootstrapError::Framing);
    }
    REQUEST_HEADER_BYTES
        .checked_add(target)
        .and_then(|n| n.checked_add(referer))
        .filter(|n| *n <= MAX_REQUEST_BYTES)
        .ok_or(BootstrapError::Limit)
}
pub(crate) fn https(value: &str) -> Result<Url, BootstrapError> {
    let url = Url::parse(value).map_err(|_| BootstrapError::InvalidUrl)?;
    if url.scheme() != "https" || url.host_str().is_none() {
        return Err(BootstrapError::InvalidUrl);
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(BootstrapError::Credentials);
    }
    if url.fragment().is_some() {
        return Err(BootstrapError::Fragment);
    }
    Ok(url)
}
impl HttpBootstrap {
    pub fn new(
        binding: [u8; 32],
        maximum_bytes: u64,
        target: &str,
        referer: Option<&str>,
    ) -> Result<Self, BootstrapError> {
        lengths(target.len(), referer.map_or(0, str::len))?;
        if maximum_bytes == 0 {
            return Err(BootstrapError::EmptyLimit);
        }
        let target = https(target)?;
        let referer = referer.map(https).transpose()?;
        lengths(
            target.as_str().len(),
            referer.as_ref().map_or(0, |u| u.as_str().len()),
        )?;
        Ok(Self {
            binding,
            maximum_bytes,
            target,
            referer,
        })
    }
    pub fn binding(&self) -> &[u8; 32] {
        &self.binding
    }
    pub fn maximum_bytes(&self) -> u64 {
        self.maximum_bytes
    }
    /// Sensitive request destination, not an admitted browser capability.
    pub fn target(&self) -> &Url {
        &self.target
    }
    /// Sensitive original referer. No header is inferred when absent.
    pub fn referer(&self) -> Option<&Url> {
        self.referer.as_ref()
    }
    pub fn encode(&self) -> Result<Vec<u8>, BootstrapError> {
        codec::encode(self)
    }
    pub fn decode_exact(bytes: &[u8]) -> Result<Self, BootstrapError> {
        codec::decode_exact(bytes)
    }
}
