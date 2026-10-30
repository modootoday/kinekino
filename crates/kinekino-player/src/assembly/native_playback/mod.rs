//! Trusted standalone composition only; transport and cleanup drivers remain with the caller.
#[cfg(feature = "native-proposal-window")]
pub mod routed;
mod standalone;
pub use standalone::assemble_standalone;

use crate::{
    application::{
        api::{ApiError, PlayerSdk},
        skip_segments::host::{SkipHostOwner, SkipHostRegistry},
    },
    infrastructure::{mpv::NativePlaybackHost, subtitle_files::SubtitleFiles},
};
use std::{path::Path, sync::Arc};

pub struct StandaloneNativeConfiguration<'a> {
    pub catalog: &'a Path,
    pub executable: &'a Path,
    pub sha256: &'a str,
}

/// One original owner set. Client clones do not replace the native or skip owner.
#[must_use = "Retain the original native and skip owners through explicit transport cleanup"]
pub struct StandaloneNativeAssembly {
    pub host: NativePlaybackHost,
    pub skip: Arc<SkipHostRegistry>,
    pub skip_owner: SkipHostOwner,
    pub client: Arc<PlayerSdk<SubtitleFiles>>,
}

/// An error after native construction returns that original guard for caller-owned cleanup.
#[must_use = "Preserve the first error and explicitly finish any returned original host"]
pub struct NativeAssemblyFailure {
    error: ApiError,
    host: Option<NativePlaybackHost>,
}
impl NativeAssemblyFailure {
    fn before_native(error: ApiError) -> Box<Self> {
        Box::new(Self { error, host: None })
    }

    pub fn into_parts(self) -> (ApiError, Option<NativePlaybackHost>) {
        let Self { error, host } = self;
        (error, host)
    }
}
