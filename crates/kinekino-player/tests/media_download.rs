#![cfg(windows)]
//! Mount under root tests/media_download.rs only after central source admission.
//! Compile exact staged source and existing original storage guards; no registration.
pub use kinekino_player::domain;
#[path = "../src/application/download/mod.rs"]
pub mod download;
#[path = "../src/infrastructure/download/mod.rs"]
pub mod download_infrastructure;
#[path = "../src/infrastructure/immutable_storage_paths.rs"]
#[allow(
    dead_code,
    reason = "Mounts the unchanged shared guard; file/lock helpers are outside this transfer"
)]
pub(crate) mod immutable_storage_paths;
#[path = "../src/infrastructure/storage_directory_identity.rs"]
pub(crate) mod storage_directory_identity;

mod application {
    pub use crate::download;
    pub use kinekino_player::application::{api, browser_provider, jobs};
}
mod infrastructure {
    pub(crate) use crate::{immutable_storage_paths, storage_directory_identity};
    #[cfg(feature = "source-adapter-test-support")]
    pub use kinekino_player::infrastructure::jobs;
    pub(crate) use kinekino_player::infrastructure::os_policy;
}
