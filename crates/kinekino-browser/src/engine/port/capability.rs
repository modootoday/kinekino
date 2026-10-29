//! Capability table vocabulary: what one adapter declares it can do.

/// One engine capability. Media entries are playback support; API entries are port calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Capability {
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
}

impl Capability {
    pub const ALL: [Self; 10] = [
        Self::H264,
        Self::Aac,
        Self::Vp9,
        Self::Widevine,
        Self::Eval,
        Self::ScriptInjection,
        Self::RequestInterception,
        Self::FrameCapture,
        Self::AudioMute,
        Self::WindowEmbed,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::H264 => "h264",
            Self::Aac => "aac",
            Self::Vp9 => "vp9",
            Self::Widevine => "widevine",
            Self::Eval => "eval",
            Self::ScriptInjection => "script-injection",
            Self::RequestInterception => "request-interception",
            Self::FrameCapture => "frame-capture",
            Self::AudioMute => "audio-mute",
            Self::WindowEmbed => "window-embed",
        }
    }

    const fn bit(self) -> u16 {
        1 << (self as u16)
    }
}

/// A set of capabilities (bit set; cheap to copy and compare).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CapabilitySet(u16);

impl CapabilitySet {
    pub const EMPTY: Self = Self(0);

    pub const fn of(caps: &[Capability]) -> Self {
        let mut bits = 0;
        let mut i = 0;
        while i < caps.len() {
            bits |= caps[i].bit();
            i += 1;
        }
        Self(bits)
    }

    pub const fn contains(self, cap: Capability) -> bool {
        self.0 & cap.bit() != 0
    }

    pub const fn contains_all(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Capabilities in `required` that this set lacks.
    pub const fn missing(self, required: Self) -> Self {
        Self(required.0 & !self.0)
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn iter(self) -> impl Iterator<Item = Capability> {
        Capability::ALL
            .into_iter()
            .filter(move |c| self.contains(*c))
    }
}

impl std::fmt::Display for CapabilitySet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names: Vec<_> = self.iter().map(Capability::name).collect();
        write!(f, "[{}]", names.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::{Capability::*, CapabilitySet};

    #[test]
    fn set_algebra() {
        let s = CapabilitySet::of(&[H264, Eval]);
        assert!(s.contains(H264) && !s.contains(Vp9));
        assert!(s.contains_all(CapabilitySet::of(&[Eval])));
        assert_eq!(
            s.missing(CapabilitySet::of(&[Eval, Vp9])),
            CapabilitySet::of(&[Vp9])
        );
        assert_eq!(s.to_string(), "[h264, eval]");
    }
}
