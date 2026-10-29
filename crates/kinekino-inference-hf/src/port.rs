//! Transport port for the HF adapter (tests use a loopback mock server).
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub url: String,
    pub bearer: Option<String>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportError {
    Timeout(String),
    Connect(String),
    Refused(String),
}

pub trait HttpTransport {
    fn post_json(
        &self,
        request: &HttpRequest,
        connect_timeout: Duration,
        total_timeout: Duration,
    ) -> Result<HttpResponse, TransportError>;
}
