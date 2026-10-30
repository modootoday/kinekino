//! `kinekino mcp serve`: a minimal MCP server over stdio (newline-delimited JSON-RPC 2.0).
//!
//! Tools are the commands that `capabilities` reports as supported; each tool's input schema is
//! the request `payload` schema from `schema`. Calls run in process through the same validation
//! and execution path as `kinekino --input -`. Hand-written instead of the `rmcp` crate to keep
//! the dependency graph small (P9 plan note). Policy comes from the `mcp` settings section.
use kinekino_core::port::{ErrorCode, PortError, PortResult};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io::{BufRead, Write};

/// Protocol revisions this server speaks; the newest is offered when the client asks for another.
pub const PROTOCOL_VERSIONS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];

/// Commands whose action only reads state.
const READ_ACTIONS: [&str; 12] = [
    "version",
    "capabilities",
    "schema",
    "inspect",
    "list",
    "status",
    "get",
    "search",
    "read",
    "read_raw",
    "decisions",
    "output_decisions",
];

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AllowedTools {
    #[default]
    ReadOnly,
    All,
}

/// `settings.mcp`: off by default, read-only tools by default, confirmation for writes.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, default)]
pub struct McpSettings {
    pub enabled: bool,
    pub allowed_tools: AllowedTools,
    pub confirm_destructive: bool,
}

impl Default for McpSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_tools: AllowedTools::ReadOnly,
            confirm_destructive: true,
        }
    }
}

impl McpSettings {
    pub fn set(mut self, key: &str, raw: &str) -> Result<Self, String> {
        let flag = || match raw {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(format!("{key}: expected true|false")),
        };
        match key {
            "enabled" => self.enabled = flag()?,
            "confirm_destructive" => self.confirm_destructive = flag()?,
            "allowed_tools" => {
                self.allowed_tools = serde_json::from_value(json!(raw))
                    .map_err(|_| format!("{key}: expected read_only|all"))?
            }
            other => return Err(format!("mcp.{other}: unknown key")),
        }
        Ok(self)
    }
}

pub fn is_read_only(command: &str) -> bool {
    let action = command.rsplit('.').next().unwrap_or(command);
    READ_ACTIONS.contains(&action)
}

/// Executes one API request JSON and returns the response JSON.
pub type Execute = fn(Value) -> Value;

pub struct Server {
    settings: McpSettings,
    execute: Execute,
    counter: u64,
}

fn request(command: &str, id: String, payload: Value) -> Value {
    json!({"command": command, "context": {"request_id": id, "api_version": "1"}, "payload": payload})
}

fn rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

impl Server {
    pub fn new(settings: McpSettings, execute: Execute) -> Self {
        Self {
            settings,
            execute,
            counter: 0,
        }
    }

    fn call(&mut self, command: &str, payload: Value) -> Value {
        self.counter += 1;
        (self.execute)(request(command, format!("mcp-{}", self.counter), payload))
    }

    fn allowed(&self, command: &str) -> bool {
        self.settings.allowed_tools == AllowedTools::All || is_read_only(command)
    }

    fn needs_confirm(&self, command: &str) -> bool {
        self.settings.confirm_destructive && !is_read_only(command)
    }

    /// Supported, policy-allowed commands in capability order.
    fn commands(&mut self) -> Vec<String> {
        let caps = self.call("capabilities", json!({}));
        caps["data"]["commands"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|c| c["status"] == "supported")
            .filter_map(|c| c["command"].as_str().map(str::to_owned))
            .filter(|c| self.allowed(c))
            .collect()
    }

    fn tool(&mut self, command: &str) -> Value {
        let schema = self.call("schema", json!({ "command": command }));
        let mut input = schema["data"]["request"]["properties"]["payload"].clone();
        if !input.is_object() {
            input = json!({"type": "object"});
        }
        if self.needs_confirm(command) {
            input["properties"]["confirm"] = json!({"const": true, "description": "Required: confirms a call that writes files or state"});
            match input["required"].as_array_mut() {
                Some(required) => required.push(json!("confirm")),
                None => input["required"] = json!(["confirm"]),
            }
        }
        let read_only = is_read_only(command);
        json!({
            "name": command,
            "description": format!("kinekino `{command}` (see `kinekino schema`)"),
            "inputSchema": input,
            "annotations": {"readOnlyHint": read_only, "destructiveHint": !read_only, "openWorldHint": false},
        })
    }

    fn tools_call(&mut self, params: &Value) -> Result<Value, (i64, &'static str)> {
        let name = params["name"]
            .as_str()
            .ok_or((-32602, "tools/call needs name"))?;
        if !self.commands().iter().any(|c| c == name) {
            return Err((-32602, "Unknown or disallowed tool"));
        }
        let mut payload = params
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));
        if !payload.is_object() {
            return Err((-32602, "arguments must be an object"));
        }
        if self.needs_confirm(name) {
            if payload.get("confirm") != Some(&json!(true)) {
                return Err((-32602, "This tool writes; pass \"confirm\": true"));
            }
            payload.as_object_mut().map(|o| o.remove("confirm"));
        }
        let response = self.call(name, payload);
        let ok = response["ok"] == json!(true);
        Ok(json!({
            "content": [{"type": "text", "text": response.to_string()}],
            "structuredContent": response,
            "isError": !ok,
        }))
    }

    /// Handles one JSON-RPC message; notifications produce no reply.
    pub fn handle(&mut self, message: Value) -> Option<Value> {
        let id = message.get("id").cloned()?;
        let method = message["method"].as_str().unwrap_or_default().to_owned();
        let params = message.get("params").cloned().unwrap_or(Value::Null);
        let result = match method.as_str() {
            "initialize" => {
                let asked = params["protocolVersion"].as_str().unwrap_or_default();
                let version = PROTOCOL_VERSIONS
                    .iter()
                    .find(|v| **v == asked)
                    .unwrap_or(&PROTOCOL_VERSIONS[0]);
                Ok(json!({
                    "protocolVersion": version,
                    "capabilities": {"tools": {"listChanged": false}},
                    "serverInfo": {"name": "kinekino", "version": env!("CARGO_PKG_VERSION")},
                }))
            }
            "ping" => Ok(json!({})),
            "tools/list" => {
                let tools: Vec<Value> = self.commands().iter().map(|c| self.tool(c)).collect();
                Ok(json!({ "tools": tools }))
            }
            "tools/call" => self.tools_call(&params),
            "" => Err((-32600, "Invalid request")),
            _ => Err((-32601, "Method not found")),
        };
        Some(match result {
            Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
            Err((code, message)) => rpc_error(id, code, message),
        })
    }

    /// Reads newline-delimited messages until EOF.
    pub fn serve(&mut self, input: impl BufRead, mut output: impl Write) -> std::io::Result<()> {
        for line in input.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let reply = match serde_json::from_str::<Value>(&line) {
                Ok(message) if message.is_object() => self.handle(message),
                Ok(_) => Some(rpc_error(Value::Null, -32600, "Invalid request")),
                Err(_) => Some(rpc_error(Value::Null, -32700, "Parse error")),
            };
            if let Some(reply) = reply {
                writeln!(output, "{reply}")?;
                output.flush()?;
            }
        }
        Ok(())
    }
}

/// `mcp serve`: refuses unless `settings.mcp.enabled` is true.
pub fn run(args: &crate::Args, env: &crate::Env) -> PortResult<()> {
    if args.get(1) != Some("serve") || args.get(2).is_some() {
        return Err(PortError::new(
            ErrorCode::InvalidInput,
            "usage: kinekino mcp serve",
        ));
    }
    let settings = crate::settings::mcp(&crate::settings::load(env)?)?;
    if !settings.enabled {
        return Err(PortError::new(
            ErrorCode::InvalidInput,
            "MCP is disabled; run `kinekino settings set mcp.enabled true`",
        ));
    }
    let stdin = std::io::stdin();
    Server::new(settings, kinekino_host::execute_value)
        .serve(stdin.lock(), std::io::stdout().lock())
        .map_err(|e| PortError::new(ErrorCode::Io, e.to_string()))
}

#[cfg(test)]
#[path = "mcp_tests.rs"]
mod tests;
