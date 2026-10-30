//! Pinned encoder tool, observed inventory and original guard lifetime. No encode yet.
mod inventory;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        clip_render::{ClipEncoderAdmission, ClipEncoderInventory, ClipEncoderLease},
        review_source::MutationControl,
    },
    infrastructure::frame_capture::{process, tools},
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

#[derive(Serialize)]
pub struct EncoderToolObservation {
    pub stage: String,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
pub struct PinnedClipEncoder {
    path: PathBuf,
    inventory: ClipEncoderInventory,
    observations: Vec<EncoderToolObservation>,
    _guards: Vec<File>,
}
impl ClipEncoderLease for PinnedClipEncoder {
    fn inventory(&self) -> &ClipEncoderInventory {
        &self.inventory
    }
}
impl PinnedClipEncoder {
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn observations(&self) -> &[EncoderToolObservation] {
        &self.observations
    }
    pub fn open(
        path: &Path,
        sha256: &str,
        timeout: Duration,
        control: &dyn MutationControl,
    ) -> Result<Self, ApiError> {
        if timeout.is_zero() || timeout > Duration::from_secs(30) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Bounded encoder inventory timeout required",
            ));
        }
        let deadline = Instant::now() + timeout;
        control.checkpoint()?;
        let (path, guards) = tools::open(path, sha256)?;
        let mut observations = Vec::new();
        for (stage, option) in [
            ("version", "-version"),
            ("build_configuration", "-buildconf"),
            ("license", "-L"),
            ("encoders", "-encoders"),
            ("filters", "-filters"),
            ("muxers", "-muxers"),
        ] {
            control.checkpoint()?;
            let mut command = Command::new(&path);
            command.args(["-hide_banner", option]);
            let output =
                process::run_with_recovery(&mut command, deadline, 524288, Some(control), None)?;
            observations.push(EncoderToolObservation {
                stage: stage.into(),
                stdout: output.stdout,
                stderr: output.stderr,
            });
        }
        control.checkpoint()?;
        let version = std::str::from_utf8(&observations[0].stdout)
            .ok()
            .and_then(|text| text.lines().next())
            .filter(|line| line.starts_with("ffmpeg version "))
            .ok_or_else(|| {
                ApiError::new(
                    ErrorCode::NeedsInput,
                    "Pinned tool did not report an FFmpeg version",
                )
            })?
            .to_owned();
        if observations[1].stdout.is_empty() && observations[1].stderr.is_empty()
            || observations[2].stdout.is_empty() && observations[2].stderr.is_empty()
        {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Encoder build/license observation is empty",
            ));
        }
        let observation_hash = |index: usize| -> Result<String, ApiError> {
            Ok(format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(&observations[index]).map_err(|_| {
                    ApiError::new(ErrorCode::Internal, "Invalid encoder observation")
                })?)
            ))
        };
        #[derive(Serialize)]
        struct Identity<'a> {
            contract: &'static str,
            tool_sha256: &'a str,
            observations: &'a [EncoderToolObservation],
        }
        let inventory_sha256 = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&Identity {
                    contract: "clip-encoder-inventory/1",
                    tool_sha256: sha256,
                    observations: &observations
                })
                .map_err(|_| ApiError::new(ErrorCode::Internal, "Invalid encoder inventory"))?
            )
        );
        let feature_names = |index: usize, table| {
            inventory::names(&observations[index].stdout, table).map_err(|mut error| {
                error.details["stage"] = serde_json::json!(observations[index].stage);
                error.details["tool_sha256"] = serde_json::json!(sha256);
                error
            })
        };
        let inventory = ClipEncoderInventory {
            tool_sha256: sha256.into(),
            inventory_sha256,
            version_line: version,
            build_configuration_sha256: observation_hash(1)?,
            license_sha256: observation_hash(2)?,
            encoders: feature_names(3, inventory::Table::Encoders)?,
            filters: feature_names(4, inventory::Table::Filters)?,
            muxers: feature_names(5, inventory::Table::Muxers)?,
        };
        control.checkpoint()?;
        if Instant::now() >= deadline {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Clip encoder inventory deadline elapsed",
            ));
        }
        Ok(Self {
            path,
            inventory,
            observations,
            _guards: guards,
        })
    }
}
/// Trusted assembly only. No automatic tool discovery, download or fallback.
pub struct FixedClipEncoder {
    pub path: PathBuf,
    pub sha256: String,
    pub inventory_timeout: Duration,
}
impl ClipEncoderAdmission for FixedClipEncoder {
    type Lease = PinnedClipEncoder;
    fn open_encoder(&self, control: &dyn MutationControl) -> Result<Self::Lease, ApiError> {
        PinnedClipEncoder::open(&self.path, &self.sha256, self.inventory_timeout, control)
    }
}
#[cfg(test)]
mod tests;
