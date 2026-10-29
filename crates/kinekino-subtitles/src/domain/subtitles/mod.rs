pub mod ass;
pub mod ass_input;
pub mod format;
pub mod parser;
pub mod range;
pub mod render;
#[cfg(test)]
mod tests;
pub mod timed_text;

use kinekino_media_contracts::api_values::DecimalU64;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubtitleFormat {
    Srt,
    Vtt,
    /// ASS v4+ (input: `Dialogue` events as plain cues; output: styled).
    Ass,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpeakerPolicy {
    #[default]
    Omit,
    Prefix,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Cue {
    pub in_us: DecimalU64,
    pub out_us: DecimalU64,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Loss {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct CueDocument {
    pub cues: Vec<Cue>,
    pub losses: Vec<Loss>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubtitleError {
    Invalid(&'static str),
    Unsupported(&'static str),
}

impl CueDocument {
    pub fn add_loss(&mut self, code: &'static str, message: &'static str) {
        if !self.losses.iter().any(|loss| loss.code == code) {
            self.losses.push(Loss {
                code: code.into(),
                message: message.into(),
            });
        }
    }
}
