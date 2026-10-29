//! Engine identities and their declared capability tables (single source of truth).
use super::capability::{Capability::*, CapabilitySet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineKind {
    /// Microsoft Edge WebView2 over wry (Windows only).
    WebView2,
    /// Official CEF build through cef-rs, created as a native child window.
    Cef,
    /// Installed Google Chrome in `--app` mode, reparented into the app window; CDP controls it.
    SystemChrome,
    /// Installed Google Chrome over CDP, headless or as a separate window (CLI and checks).
    ChromeCdp,
}

const CHROME_MEDIA: CapabilitySet = CapabilitySet::of(&[H264, Aac, Vp9, Widevine]);
const CDP_API: CapabilitySet = CapabilitySet::of(&[
    Eval,
    ScriptInjection,
    RequestInterception,
    FrameCapture,
    AudioMute,
]);

impl EngineKind {
    pub const ALL: [Self; 4] = [
        Self::WebView2,
        Self::Cef,
        Self::SystemChrome,
        Self::ChromeCdp,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::WebView2 => "webview2",
            Self::Cef => "cef",
            Self::SystemChrome => "system-chrome",
            Self::ChromeCdp => "chrome-cdp",
        }
    }

    /// What the adapter implements today (not the engine's potential). ADR 0002 table.
    pub const fn declared(self) -> CapabilitySet {
        match self {
            Self::WebView2 => CapabilitySet::of(&[H264, Aac, Vp9, Widevine, WindowEmbed]),
            // Official CEF: no proprietary codecs; CDM fetched at runtime (VP9 only).
            Self::Cef => CapabilitySet::of(&[Vp9, Widevine, Eval, AudioMute, WindowEmbed]),
            Self::SystemChrome => CapabilitySet::of(&[
                H264,
                Aac,
                Vp9,
                Widevine,
                Eval,
                ScriptInjection,
                RequestInterception,
                FrameCapture,
                AudioMute,
                WindowEmbed,
            ]),
            Self::ChromeCdp => CHROME_MEDIA.union(CDP_API),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_chrome_is_cdp_plus_embed() {
        let cdp = EngineKind::ChromeCdp.declared();
        let embed = EngineKind::SystemChrome.declared();
        assert!(embed.contains_all(cdp));
        assert_eq!(cdp.missing(embed), CapabilitySet::of(&[WindowEmbed]));
    }

    #[test]
    fn official_cef_lacks_proprietary_codecs() {
        let cef = EngineKind::Cef.declared();
        assert!(!cef.contains(H264) && !cef.contains(Aac));
        assert!(cef.contains(WindowEmbed) && cef.contains(Widevine));
    }
}
