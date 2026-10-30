//! In-process execution for the MCP surface: the same validation and local execution path as
//! `kinekino --input -`, without spawning a process.
use super::{Request, Response, execute_local, invalid};

/// Executes one request JSON (`{"command", "context", "payload"}`) and returns the response JSON.
pub fn execute_value(request: serde_json::Value) -> serde_json::Value {
    let response = match serde_json::from_value::<Request>(request) {
        Ok(request) => match request.validate() {
            Ok(()) => execute_local(&[], request),
            Err(error) => Response::failure("mcp".into(), error),
        },
        Err(_) => Response::failure(
            "mcp".into(),
            invalid("Invalid request JSON or unknown field"),
        ),
    };
    serde_json::to_value(&response).unwrap_or_default()
}
