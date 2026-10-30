//! Layered app config: defaults → config file → `KINEKINO_*` env → CLI flags.
//! Secrets are never config: the file schema denies unknown keys, and API keys go
//! through the `CredentialStore` port.
mod layer;
#[cfg(test)]
mod tests;

pub use layer::config_path;

use crate::{Args, Env};
use kinekino_core::port::{ErrorCode, PortError, PortResult};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Default,
    File,
    Env,
    Cli,
}

impl Source {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::File => "file",
            Self::Env => "env",
            Self::Cli => "cli",
        }
    }
}

/// One resolved setting and the layer that set it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setting<T> {
    pub value: T,
    pub source: Source,
}

impl<T> Setting<T> {
    fn default(value: T) -> Self {
        Self {
            value,
            source: Source::Default,
        }
    }
    fn set(&mut self, value: Option<T>, source: Source) {
        if let Some(value) = value {
            *self = Self { value, source };
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    /// `auto` or an engine kind name (`cef`, `system-chrome`, `chrome-cdp`, `webview2`).
    pub browser_engine: Setting<String>,
    pub browser_headless: Setting<bool>,
    /// `local` | `remote` | `off`.
    pub inference_mode: Setting<String>,
    /// `json` | `text`.
    pub output_format: Setting<String>,
    /// Config file consulted (may not exist).
    pub file: Option<PathBuf>,
}

/// (key, env var, CLI flag, allowed values; empty = any).
pub const KEYS: [(&str, &str, &str, &[&str]); 4] = [
    (
        "browser.engine",
        "KINEKINO_BROWSER_ENGINE",
        "--browser-engine",
        &["auto", "cef", "system-chrome", "chrome-cdp", "webview2"],
    ),
    (
        "browser.headless",
        "KINEKINO_BROWSER_HEADLESS",
        "--browser-headless",
        &["true", "false"],
    ),
    (
        "inference.mode",
        "KINEKINO_INFERENCE_MODE",
        "--inference-mode",
        &["local", "remote", "off"],
    ),
    (
        "output.format",
        "KINEKINO_OUTPUT",
        "--output",
        &["json", "text"],
    ),
];

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            browser_engine: Setting::default("auto".into()),
            browser_headless: Setting::default(true),
            inference_mode: Setting::default("local".into()),
            output_format: Setting::default("json".into()),
            file: None,
        }
    }
}

impl AppConfig {
    pub fn load(args: &Args, env: &Env) -> PortResult<Self> {
        let mut c = Self {
            file: config_path(env),
            ..Self::default()
        };
        if let Some(f) = c
            .file
            .as_deref()
            .map(layer::read_file)
            .transpose()?
            .flatten()
        {
            c.apply(Source::File, |k| f.get(k))?;
        }
        c.apply(Source::Env, |k| {
            key_env(k).and_then(|e| env.get(e)).map(str::to_owned)
        })?;
        c.apply(Source::Cli, |k| {
            key_flag(k).and_then(|f| args.flag(f)).map(str::to_owned)
        })?;
        Ok(c)
    }

    fn apply(&mut self, source: Source, get: impl Fn(&str) -> Option<String>) -> PortResult<()> {
        let v = |k: &str| get(k).map(|s| validate(k, s)).transpose();
        self.browser_engine.set(v("browser.engine")?, source);
        self.browser_headless
            .set(v("browser.headless")?.map(|s| s == "true"), source);
        self.inference_mode.set(v("inference.mode")?, source);
        self.output_format.set(v("output.format")?, source);
        Ok(())
    }

    /// `(key, value, source)` rows in [`KEYS`] order.
    pub fn rows(&self) -> Vec<(&'static str, String, Source)> {
        vec![
            (
                KEYS[0].0,
                self.browser_engine.value.clone(),
                self.browser_engine.source,
            ),
            (
                KEYS[1].0,
                self.browser_headless.value.to_string(),
                self.browser_headless.source,
            ),
            (
                KEYS[2].0,
                self.inference_mode.value.clone(),
                self.inference_mode.source,
            ),
            (
                KEYS[3].0,
                self.output_format.value.clone(),
                self.output_format.source,
            ),
        ]
    }
}

fn key_env(key: &str) -> Option<&'static str> {
    KEYS.iter().find(|k| k.0 == key).map(|k| k.1)
}

fn key_flag(key: &str) -> Option<&'static str> {
    KEYS.iter().find(|k| k.0 == key).map(|k| k.2)
}

fn validate(key: &str, value: String) -> PortResult<String> {
    let allowed = KEYS.iter().find(|k| k.0 == key).map_or(&[][..], |k| k.3);
    if allowed.is_empty() || allowed.contains(&value.as_str()) {
        Ok(value)
    } else {
        Err(PortError::new(
            ErrorCode::InvalidInput,
            format!("{key}: `{value}` is not one of {}", allowed.join("|")),
        ))
    }
}
