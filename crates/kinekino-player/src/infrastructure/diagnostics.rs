use serde::Serialize;
use serde_json::json;
use std::sync::OnceLock;
use std::time::Instant;

static ENABLED: OnceLock<bool> = OnceLock::new();
static STARTED: OnceLock<Instant> = OnceLock::new();

pub fn configure(enabled: bool) {
    let _ = STARTED.set(Instant::now());
    let _ = ENABLED.set(enabled);
}

pub fn record(kind: &str, payload: impl Serialize) {
    if *ENABLED.get().unwrap_or(&false) {
        let elapsed_ms = STARTED.get_or_init(Instant::now).elapsed().as_millis();
        println!(
            "{}",
            json!({"kind":kind,"elapsed_ms":elapsed_ms,"data":payload})
        );
    }
}

pub fn enabled() -> bool {
    *ENABLED.get().unwrap_or(&false)
}
