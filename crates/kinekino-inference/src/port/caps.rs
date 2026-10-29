use kinekino_core::port::{AdapterKind, CapabilityTable};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Task {
    Transcribe,
    Diarize,
    Ocr,
    SegmentSubject,
    Generate,
    Embed,
}

impl Task {
    pub const ALL: [Self; 6] = [
        Self::Transcribe,
        Self::Diarize,
        Self::Ocr,
        Self::SegmentSubject,
        Self::Generate,
        Self::Embed,
    ];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Transcribe => "transcribe",
            Self::Diarize => "diarize",
            Self::Ocr => "ocr",
            Self::SegmentSubject => "segment-subject",
            Self::Generate => "generate",
            Self::Embed => "embed",
        }
    }
    const fn bit(self) -> u8 {
        1 << (self as u8)
    }
}

/// Bit set of supported [`Task`]s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct InferenceCaps(u8);

impl InferenceCaps {
    pub const fn of(tasks: &[Task]) -> Self {
        let mut bits = 0;
        let mut i = 0;
        while i < tasks.len() {
            bits |= tasks[i].bit();
            i += 1;
        }
        Self(bits)
    }
    pub const fn all() -> Self {
        Self::of(&Task::ALL)
    }
    pub const fn contains(self, task: Task) -> bool {
        self.0 & task.bit() != 0
    }
}

impl fmt::Display for InferenceCaps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.names().join(", "))
    }
}

impl CapabilityTable for InferenceCaps {
    fn empty() -> Self {
        Self(0)
    }
    fn contains_all(self, required: Self) -> bool {
        self.0 & required.0 == required.0
    }
    fn missing(self, required: Self) -> Self {
        Self(required.0 & !self.0)
    }
    fn is_empty(self) -> bool {
        self.0 == 0
    }
    fn names(self) -> Vec<&'static str> {
        Task::ALL
            .iter()
            .filter(|t| self.contains(**t))
            .map(|t| t.name())
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InferenceKind {
    HfCloud,
    LocalPack,
    Mock,
}

impl AdapterKind for InferenceKind {
    fn name(self) -> &'static str {
        match self {
            Self::HfCloud => "hf-cloud",
            Self::LocalPack => "local-pack",
            Self::Mock => "mock",
        }
    }
}
