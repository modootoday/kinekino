//! Worker-owned response, retained before metadata validation. No request sender.
use crate::worker_buffer::WorkerBuffer;
use reqwest::{
    Response,
    header::{CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResponseError {
    Order,
    Status,
    Encoding,
    Length,
    Limit,
    Body,
    Url,
    ContentType,
}

/// No Debug/Clone: headers and URL stay in the original worker owner.
/// Drop releases this response only; it is not a DNS/Job/IPC close receipt.
pub struct HttpResponseOwner {
    response: Response,
    expected_url: url::Url,
    buffer: WorkerBuffer,
    maximum: u64,
    expected: Option<u64>,
    received: u64,
    validated: bool,
    eof: bool,
    failure: Option<ResponseError>,
}
impl HttpResponseOwner {
    /// Export only after local response checks. Parent must recheck original URL,
    /// size and binding; this payload creates no trusted hash or replay grant.
    pub fn wire_metadata(&self) -> Result<crate::http_metadata::HttpMetadata, ResponseError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if !self.validated {
            return Err(ResponseError::Order);
        }
        let content_type = self
            .response
            .headers()
            .get(CONTENT_TYPE)
            .map(|v| v.to_str().map_err(|_| ResponseError::ContentType))
            .transpose()?;
        crate::http_metadata::HttpMetadata::new(
            self.response.url().as_str(),
            self.expected,
            content_type,
        )
        .map_err(|_| ResponseError::ContentType)
    }
    /// Store the response first. Even rejected metadata remains owned by self.
    pub fn new(response: Response, maximum: u64, expected_url: url::Url) -> Self {
        Self {
            response,
            expected_url,
            buffer: WorkerBuffer::default(),
            maximum,
            expected: None,
            received: 0,
            validated: false,
            eof: false,
            failure: None,
        }
    }
    pub fn validate(&mut self) -> Result<Option<u64>, ResponseError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if self.validated {
            return Err(ResponseError::Order);
        }
        let result = self.metadata();
        match result {
            Ok(length) => {
                self.expected = length;
                self.validated = true;
                Ok(length)
            }
            Err(error) => {
                self.failure = Some(error);
                Err(error)
            }
        }
    }
    fn metadata(&self) -> Result<Option<u64>, ResponseError> {
        if self.maximum == 0 {
            return Err(ResponseError::Limit);
        }
        if self.response.status().as_u16() != 200 {
            return Err(ResponseError::Status);
        }
        if self.response.url() != &self.expected_url {
            return Err(ResponseError::Url);
        }
        let headers = self.response.headers();
        if headers.contains_key(CONTENT_RANGE) {
            return Err(ResponseError::Status);
        }
        let mut types = headers.get_all(CONTENT_TYPE).iter();
        if let Some(value) = types.next() {
            let value = value.to_str().map_err(|_| ResponseError::ContentType)?;
            if value.len() > 256 || value.to_ascii_lowercase().contains("html") {
                return Err(ResponseError::ContentType);
            }
        }
        if types.next().is_some() {
            return Err(ResponseError::ContentType);
        }
        for value in headers.get_all(CONTENT_ENCODING) {
            if value.as_bytes() != b"identity" {
                return Err(ResponseError::Encoding);
            }
        }
        let mut lengths = headers.get_all(CONTENT_LENGTH).iter();
        let length = match lengths.next() {
            None => None,
            Some(value) => {
                let value = value.to_str().map_err(|_| ResponseError::Length)?;
                if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(ResponseError::Length);
                }
                Some(value.parse::<u64>().map_err(|_| ResponseError::Length)?)
            }
        };
        if lengths.next().is_some() {
            return Err(ResponseError::Length);
        }
        if length.is_some_and(|n| n > self.maximum) {
            return Err(ResponseError::Limit);
        }
        Ok(length)
    }
    /// The worker must check original current/cutoff/control before AND after
    /// awaiting this method. No read-ahead while a chunk or IPC offer is pending.
    pub async fn read_next(&mut self) -> Result<bool, ResponseError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if !self.validated || self.eof || !self.buffer.can_poll_upstream() {
            return Err(ResponseError::Order);
        }
        let result = self.read_inner().await;
        if let Err(error) = result {
            self.failure = Some(error);
        }
        result
    }
    pub fn can_read(&self) -> bool {
        self.validated && !self.eof && self.failure.is_none() && self.buffer.can_poll_upstream()
    }
    async fn read_inner(&mut self) -> Result<bool, ResponseError> {
        let chunk = self
            .response
            .chunk()
            .await
            .map_err(|_| ResponseError::Body)?;
        let Some(chunk) = chunk else {
            if self.expected.is_some_and(|n| n != self.received) {
                return Err(ResponseError::Length);
            }
            self.eof = true;
            return Ok(false);
        };
        // Return to the worker checkpoint even for an empty upstream frame.
        if chunk.is_empty() {
            return Ok(true);
        }
        let total = self
            .received
            .checked_add(chunk.len() as u64)
            .ok_or(ResponseError::Limit)?;
        if total > self.maximum {
            return Err(ResponseError::Limit);
        }
        if self.expected.is_some_and(|n| total > n) {
            return Err(ResponseError::Length);
        }
        self.buffer
            .install(chunk)
            .map_err(|_| ResponseError::Limit)?;
        self.received = total;
        Ok(true)
    }
    pub fn offer(&mut self, credit: u64) -> Result<Option<bytes::Bytes>, ResponseError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if !self.validated {
            return Err(ResponseError::Order);
        }
        self.buffer.offer(credit).map_err(|_| ResponseError::Order)
    }
    pub fn complete(&mut self, length: usize) -> Result<(), ResponseError> {
        self.buffer
            .complete(length)
            .map_err(|_| ResponseError::Order)
    }
    /// Actual response EOF plus full buffer drain; never IPC EOF/physical ACK.
    pub fn body_ended(&self) -> bool {
        self.eof && self.failure.is_none() && self.buffer.can_poll_upstream()
    }
}
