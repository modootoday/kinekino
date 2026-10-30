use super::{
    api_values::DecimalU64,
    identifiers::{valid_id, valid_request_id},
    navigation,
    player_document::PlayerDocumentLocation,
};
use kinekino_core::source::document::{PageLocation, ResourceKind};
use serde::{Deserialize, Serialize};

pub const MAX_CAPTURE_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_COMMANDS: usize = 16;
pub const MAX_RETAINED_TICKETS: usize = 32;
pub const MAX_NAVIGATIONS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserOperation {
    ReadCatalog,
    ResolveEpisode,
    ReadPlayerDocument,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserRequest {
    pub request_id: String,
    pub operation: BrowserOperation,
    pub url: String,
    pub timeout_ms: DecimalU64,
    pub blocking_enabled: bool,
}
#[derive(Debug, Clone)]
pub struct ValidatedBrowserRequest(BrowserRequest);
impl ValidatedBrowserRequest {
    pub fn new(mut request: BrowserRequest) -> Result<Self, &'static str> {
        if !valid_request_id(&request.request_id)
            || request.url.len() > 8192
            || !(1..=120_000).contains(&request.timeout_ms.0)
        {
            return Err("Invalid browser request identity, URL length or timeout");
        }
        if request.operation == BrowserOperation::ReadPlayerDocument {
            request.url = PlayerDocumentLocation::parse(&request.url)?.as_str().into();
            return Ok(Self(request));
        }
        let location = PageLocation::parse(&request.url)
            .map_err(|_| "Unsupported browser request location")?;
        let valid = match request.operation {
            BrowserOperation::ReadCatalog => matches!(
                location.kind(),
                ResourceKind::Category | ResourceKind::Series
            ),
            BrowserOperation::ResolveEpisode => location.kind() == ResourceKind::Episode,
            BrowserOperation::ReadPlayerDocument => unreachable!("handled above"),
        };
        if !valid {
            return Err("Browser operation does not match its location");
        }
        request.url = location.as_str().into();
        Ok(Self(request))
    }
    pub fn request(&self) -> &BrowserRequest {
        &self.0
    }
    pub fn navigation_allowed(&self, url: &str) -> bool {
        if url.len() > 8192 {
            return false;
        }
        match self.0.operation {
            BrowserOperation::ReadCatalog => PageLocation::parse(url).is_ok_and(|target| {
                PageLocation::parse(&self.0.url).is_ok_and(|start| start.kind() == target.kind())
            }),
            BrowserOperation::ResolveEpisode => navigation::navigation_allowed(url),
            BrowserOperation::ReadPlayerDocument => {
                PlayerDocumentLocation::parse(url).is_ok_and(|target| target.as_str() == self.0.url)
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "TicketFields")]
pub struct BrowserTicket {
    session_id: String,
    request_id: String,
    request_generation: DecimalU64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TicketFields {
    session_id: String,
    request_id: String,
    request_generation: DecimalU64,
}
impl TryFrom<TicketFields> for BrowserTicket {
    type Error = &'static str;
    fn try_from(value: TicketFields) -> Result<Self, Self::Error> {
        Self::new(
            value.session_id,
            value.request_id,
            value.request_generation.0,
        )
    }
}
impl BrowserTicket {
    pub fn new(
        session_id: String,
        request_id: String,
        request_generation: u64,
    ) -> Result<Self, &'static str> {
        if !valid_id(&session_id) || !valid_request_id(&request_id) || request_generation == 0 {
            return Err("Invalid browser ticket");
        }
        Ok(Self {
            session_id,
            request_id,
            request_generation: DecimalU64(request_generation),
        })
    }
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
    pub fn request_id(&self) -> &str {
        &self.request_id
    }
    pub fn request_generation(&self) -> u64 {
        self.request_generation.0
    }
}
