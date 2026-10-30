//! `kinekino adblock rules` | `kinekino adblock check URL [--source URL] [--kind KIND]
//! [--method METHOD] [--extra FILE]` (plan P3, inventory #6). JSON on stdout.
use kinekino_core::port::{ErrorCode, PortError, PortResult};
use serde_json::{Value, json};

const KINDS: [&str; 11] = [
    "document",
    "subdocument",
    "script",
    "stylesheet",
    "image",
    "media",
    "font",
    "xmlhttprequest",
    "websocket",
    "ping",
    "other",
];

fn usage() -> PortError {
    PortError::new(
        ErrorCode::InvalidInput,
        "usage: kinekino adblock <rules | check URL [--source URL] [--kind KIND] [--method METHOD] [--extra FILE]>",
    )
}

fn check(args: &crate::Args) -> PortResult<Value> {
    let url = args
        .get(2)
        .filter(|u| !u.starts_with("--"))
        .ok_or_else(usage)?;
    let (mut source, mut kind, mut method, mut extra_file) = ("", "other", "GET", None);
    let mut index = 3;
    while let Some(flag) = args.get(index) {
        let value = args.get(index + 1).ok_or_else(usage)?;
        match flag {
            "--source" => source = value,
            "--kind" => kind = value,
            "--method" => method = value,
            "--extra" => extra_file = Some(value),
            _ => return Err(usage()),
        }
        index += 2;
    }
    if !KINDS.contains(&kind) {
        return Err(PortError::new(
            ErrorCode::InvalidInput,
            format!("--kind must be one of {}", KINDS.join("|")),
        ));
    }
    let extra = match extra_file {
        Some(path) => std::fs::read_to_string(path)
            .map_err(|e| PortError::new(ErrorCode::Io, format!("{path}: {e}")))?,
        None => String::new(),
    };
    let blocked = kinekino_host::adblock::check(&extra, url, source, kind, method);
    Ok(json!({"url": url, "source": source, "kind": kind, "method": method, "blocked": blocked}))
}

pub fn run(args: &crate::Args) -> (u8, String) {
    let result = match args.get(1) {
        Some("rules") if args.get(2).is_none() => {
            Ok(json!({"bundled_rules": kinekino_host::adblock::bundled_rule_count()}))
        }
        Some("check") => check(args),
        _ => Err(usage()),
    };
    match result {
        Ok(v) => (0, v.to_string()),
        Err(e) => (2, e.to_json()),
    }
}
