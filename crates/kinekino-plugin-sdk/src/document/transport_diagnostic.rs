use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportPhase {
    #[default]
    Unknown,
    ClientBuild,
    RequestSend,
    BodyRead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticEdge {
    Root,
    Source,
    IoPayload,
}

/// Connector labels are hints, not proof of DNS/proxy/TLS execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticHint {
    DnsConnector,
    TcpConnector,
    ProxyTunnel,
    UnexpectedEof,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorNode {
    pub parent: Option<usize>,
    pub edge: DiagnosticEdge,
    pub io_kind: Option<String>,
    pub os_code: Option<i32>,
    pub hints: Vec<DiagnosticHint>,
    pub unknown: bool,
    pub label_truncated: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReqwestFlags {
    pub builder: bool,
    pub request: bool,
    pub connect: bool,
    pub timeout: bool,
    pub body: bool,
    pub decode: bool,
    pub redirect: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportDiagnostic {
    pub connect: bool,
    pub timeout: bool,
    pub io_kind: Option<String>,
    pub os_code: Option<i32>,
    #[serde(default)]
    pub phase: TransportPhase,
    #[serde(default)]
    pub reqwest_flags: Option<ReqwestFlags>,
    #[serde(default)]
    pub nodes: Vec<ErrorNode>,
    #[serde(default)]
    pub duplicate_nodes: usize,
    #[serde(default)]
    pub truncated: bool,
}
