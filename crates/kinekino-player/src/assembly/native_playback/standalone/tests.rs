//! Pinned inert constructor fixtures; never playback.open, a child process or GUI acceptance.
use super::*;
use crate::{
    application::{
        api::{ErrorCode, Request},
        playback_session::EpisodePlaybackEngine,
    },
    infrastructure::{content_digest, projects::ProjectFiles},
};
use serde_json::{Value, json};
use std::{
    fs::{self, File, OpenOptions},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

struct NoMediaFixture {
    root: PathBuf,
    catalog: PathBuf,
    executable: PathBuf,
    sha256: String,
}
impl NoMediaFixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "native-assembly-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        eprintln!("native_assembly_fixture_root={}", root.display());
        let catalog = root.join("catalog");
        fs::create_dir(&catalog).unwrap();
        let executable = root.join("mpv.exe");
        fs::write(&executable, b"inert constructor artifact; never executed").unwrap();
        let (_, sha256) =
            content_digest::sha256(&mut File::open(&executable).unwrap(), 1024).unwrap();
        Self {
            root,
            catalog,
            executable,
            sha256,
        }
    }

    fn configuration(&self) -> StandaloneNativeConfiguration<'_> {
        StandaloneNativeConfiguration {
            catalog: &self.catalog,
            executable: &self.executable,
            sha256: &self.sha256,
        }
    }

    fn assemble(&self, base: PlayerSdk<SubtitleFiles>) -> StandaloneNativeAssembly {
        match assemble_standalone(|| base, self.configuration()) {
            Ok(assembly) => assembly,
            Err(failure) => {
                let (error, host) = failure.into_parts();
                if let Some(host) = host {
                    let _ = host.shutdown();
                }
                panic!("Unexpected assembly failure: {error:?}");
            }
        }
    }
}

fn capabilities(client: &PlayerSdk<SubtitleFiles>) -> Value {
    let response = client.execute(Request::new("capabilities"));
    assert!(response.ok);
    response.data.unwrap()
}

#[test]
fn trust_pin_failure_returns_no_native_owner() {
    let fixture = NoMediaFixture::new();
    let mut wrong = fixture.sha256.clone();
    wrong.replace_range(0..1, if wrong.starts_with('0') { "1" } else { "0" });
    let base_created = std::cell::Cell::new(false);
    let failure = match assemble_standalone(
        || {
            base_created.set(true);
            PlayerSdk::new(SubtitleFiles)
        },
        StandaloneNativeConfiguration {
            sha256: &wrong,
            ..fixture.configuration()
        },
    ) {
        Err(failure) => failure,
        Ok(_) => panic!("Mismatched executable pin must fail before native ownership"),
    };
    let (error, host) = failure.into_parts();
    assert_eq!(error.code, ErrorCode::SourceExpired);
    assert!(host.is_none());
    assert!(!base_created.get());
}

#[test]
fn one_original_scope_preserves_base_sdk_and_starts_no_media() {
    let fixture = NoMediaFixture::new();
    let projects = fixture.root.join("projects");
    fs::create_dir(&projects).unwrap();
    let base =
        PlayerSdk::new(SubtitleFiles).with_project_store(ProjectFiles::open(&projects).unwrap());
    let before = capabilities(&base)["projects"].clone();
    assert_eq!(before["backend_available"], true);
    let assembly = fixture.assemble(base);
    let issuer = assembly.host.engine().native_export_issuer().unwrap();
    assert_eq!(
        assembly.skip.export_capability().issuer_scope,
        Some(issuer.scope())
    );
    let caps = capabilities(&assembly.client);
    assert_eq!(caps["skip_exports"]["issuer_scope"], json!(issuer.scope()));
    assert_eq!(caps["skip_exports"]["available"], true);
    assert_eq!(caps["skip_sessions"]["seek_submission_available"], false);
    assert_eq!(caps["projects"], before);
    // This would execute an inert file if assembly implicitly opened media.
    assert!(assembly.host.shutdown().unwrap().is_none());
}

#[test]
fn capture_setup_failures_return_original_host_for_explicit_cleanup() {
    for generation_fails in [true, false] {
        let fixture = NoMediaFixture::new();
        let base_created = std::cell::Cell::new(false);
        let failure = match assemble_using(
            || {
                base_created.set(true);
                PlayerSdk::new(SubtitleFiles)
            },
            fixture.configuration(),
            || {
                if generation_fails {
                    let mut error =
                        ApiError::new(ErrorCode::EngineFailed, "original generation failure");
                    error.details = json!({"original":"retained"});
                    Err(error)
                } else {
                    Ok(String::new())
                }
            },
        ) {
            Err(failure) => failure,
            Ok(_) => panic!("Failed identity/registry setup cannot return an SDK assembly"),
        };
        let (error, host) = failure.into_parts();
        assert!(!base_created.get());
        if generation_fails {
            assert_eq!(error.code, ErrorCode::EngineFailed);
            assert_eq!(error.message, "original generation failure");
            assert_eq!(error.details, json!({"original":"retained"}));
        } else {
            assert_eq!(error.code, ErrorCode::InvalidArgument);
        }
        let host = host.expect("Original native guard must survive partial setup failure");
        assert!(
            OpenOptions::new()
                .write(true)
                .open(&fixture.executable)
                .is_err()
        );
        assert!(host.shutdown().unwrap().is_none());
        drop(host);
        assert!(
            OpenOptions::new()
                .write(true)
                .open(&fixture.executable)
                .is_ok()
        );
    }
}

#[test]
fn skip_owner_close_reaches_sdk_clones_while_native_guard_remains_owned() {
    let fixture = NoMediaFixture::new();
    let StandaloneNativeAssembly {
        host,
        skip,
        skip_owner,
        client,
    } = fixture.assemble(PlayerSdk::new(SubtitleFiles));
    let clone = client.clone();
    assert_eq!(capabilities(&clone)["skip_exports"]["available"], true);
    drop(skip_owner);
    assert!(!skip.export_capability().available);
    assert_eq!(capabilities(&clone)["skip_exports"]["available"], false);
    assert_eq!(
        capabilities(&clone)["skip_sessions"]["seek_submission_available"],
        false
    );
    assert!(
        OpenOptions::new()
            .write(true)
            .open(&fixture.executable)
            .is_err()
    );
    assert!(host.shutdown().unwrap().is_none());
}
