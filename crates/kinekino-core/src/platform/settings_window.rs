//! Settings window port (plan P6): a window adapter renders the settings page model and
//! relays its JSON messages. The page model and message handling stay OS-neutral in
//! `kinekino-app`; the shipped adapter is Meari-native (`kinekino-settings-ui`).
use std::io;

/// What the host wants after one page message.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WindowReply {
    /// JSON reply for the window to show (`page`/`apply`/`dialog` results), if any.
    pub json: Option<String>,
    /// Capture the rendered window to this PNG path before acting on `close` (smoke runs only).
    pub capture_png: Option<std::path::PathBuf>,
    /// Close the window and return from [`SettingsWindowPort::run`].
    pub close: bool,
}

/// Opens one modal settings window and runs its event loop until it closes.
pub trait SettingsWindowPort {
    /// The adapter first sends `{"op":"page"}` and renders the returned page model;
    /// `smoke` makes it apply a fixed invalid batch, report `ready` and close.
    fn run(
        &self,
        title: &str,
        smoke: bool,
        on_message: &mut dyn FnMut(&str) -> WindowReply,
    ) -> io::Result<()>;
}

/// Adapter for OSes without a settings window yet.
#[derive(Debug, Default, Clone, Copy)]
pub struct UnsupportedSettingsWindow;

impl SettingsWindowPort for UnsupportedSettingsWindow {
    fn run(
        &self,
        _title: &str,
        _smoke: bool,
        _on_message: &mut dyn FnMut(&str) -> WindowReply,
    ) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "no settings window adapter; use `kinekino settings page|apply`",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_adapter_reports_unsupported() {
        let err = UnsupportedSettingsWindow
            .run("t", false, &mut |_| WindowReply::default())
            .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Unsupported);
    }
}
