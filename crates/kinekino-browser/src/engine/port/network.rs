//! Request interception contract (adblock and provider policy plug in here).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterceptedRequest {
    pub url: String,
    pub method: String,
    pub resource_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestDecision {
    Allow,
    Block,
}

pub trait NetworkInterceptor: Send + Sync {
    fn decide(&self, request: &InterceptedRequest) -> RequestDecision;
}
