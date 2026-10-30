#[path = "support/cli_exe.rs"]
mod cli_exe;
use kinekino_player::application::api::{ErrorCode, PlayerSdk, Request};
use serde_json::json;

#[cfg(windows)]
#[path = "support/snapshot_cli_evidence.rs"]
pub mod snapshot_cli_evidence;

const COMMAND: &str = "production.export_snapshot";

#[test]
fn unconfigured_sdk_discovery_schema_and_dispatch_agree() {
    kinekino_platform::install();
    let sdk = PlayerSdk::default();
    let capabilities = sdk.execute(Request::new("capabilities")).data.unwrap();
    assert_eq!(capabilities["snapshot_export"]["available"], false);
    assert_eq!(
        capabilities["commands"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["command"] == COMMAND)
            .unwrap()["status"],
        "unsupported"
    );
    let mut schema = Request::new("schema");
    schema.payload = json!({"command":COMMAND});
    let schema = sdk.execute(schema).data.unwrap();
    assert_eq!(schema["status"], "unsupported");
    assert_eq!(
        schema["request"]["properties"]["context"]["properties"]["timeout_ms"]["type"],
        "null"
    );
    let root = std::env::temp_dir();
    let mut request = Request::new(COMMAND);
    request.payload = json!({"source_path":root.join("source.png"),"expected_source_sha256":"a".repeat(64),"output_path":root.join("new.png"),"crop":{"x":0,"y":0,"width":1,"height":1}});
    assert_eq!(
        sdk.execute(request).error.unwrap().code,
        ErrorCode::Unsupported
    );
}

#[cfg(windows)]
mod windows {
    use super::snapshot_cli_evidence::{CliFailure, HostObservation, cli, cli_observed, hash};
    use super::*;
    use kinekino_player::{
        application::{
            api::{PlaybackClient, RequestContext},
            snapshot_export::{ExportSnapshot, ExportedSnapshot, PixelCrop, SnapshotExportPort},
        },
        infrastructure::{
            local_host::{LocalHost, TransportOptions},
            snapshot_export::PngSnapshotExport,
        },
    };
    use serde_json::Value;
    use std::{
        fs,
        io::Cursor,
        path::PathBuf,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::{SystemTime, UNIX_EPOCH},
    };

    struct Fixture {
        root: PathBuf,
        bytes: Vec<u8>,
        request: ExportSnapshot,
    }
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "snapshot-api-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(&root).unwrap();
            let mut bytes = Vec::new();
            {
                let mut encoder = png::Encoder::new(&mut bytes, 3, 2);
                encoder.set_color(png::ColorType::Grayscale);
                encoder.set_depth(png::BitDepth::Eight);
                let mut writer = encoder.write_header().unwrap();
                writer.write_image_data(&[0, 1, 2, 3, 4, 5]).unwrap();
                writer.finish().unwrap();
            }
            fs::write(root.join("source.png"), &bytes).unwrap();
            let request = ExportSnapshot {
                source_path: root.join("source.png").to_string_lossy().into(),
                expected_source_sha256: hash(&bytes),
                output_path: root.join("new.png").to_string_lossy().into(),
                crop: PixelCrop {
                    x: 1,
                    y: 0,
                    width: 2,
                    height: 2,
                },
            };
            Self {
                root,
                bytes,
                request,
            }
        }
        fn wire(&self) -> Request {
            let mut request = Request::new(COMMAND);
            request.context.request_id = "snapshot-explicit-request".into();
            request.payload = serde_json::to_value(&self.request).unwrap();
            request
        }
        fn assert_crop(&self, response: &Value) {
            assert_eq!(response["ok"], true, "{response}");
            assert_eq!(response["request_id"], "snapshot-explicit-request");
            let output = fs::read(&self.request.output_path).unwrap();
            assert_eq!(response["data"]["output_sha256"], hash(&output));
            assert_eq!(response["data"]["source_sha256"], hash(&self.bytes));
            assert_eq!(response["data"]["metadata_preserved"], false);
            assert!(response["data"].get("source_pts_us").is_none());
            let mut reader = png::Decoder::new(Cursor::new(&output)).read_info().unwrap();
            let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
            let info = reader.next_frame(&mut pixels).unwrap();
            reader.finish().unwrap();
            assert_eq!((info.width, info.height), (2, 2));
            assert_eq!(&pixels[..info.buffer_size()], &[1, 2, 4, 5]);
            assert_eq!(fs::read(&self.request.source_path).unwrap(), self.bytes);
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
    struct Counted(Arc<AtomicUsize>);
    impl SnapshotExportPort for Counted {
        fn export(
            &self,
            request: &ExportSnapshot,
        ) -> Result<ExportedSnapshot, kinekino_player::application::api::ApiError> {
            self.0.fetch_add(1, Ordering::SeqCst);
            PngSnapshotExport.export(request)
        }
    }
    #[test]
    fn typed_client_preserves_port_through_all_reconstructing_builders() {
        kinekino_platform::install();
        let fixture = Fixture::new();
        let count = Arc::new(AtomicUsize::new(0));
        let sdk = PlayerSdk::default()
            .with_snapshot_export(Counted(count.clone()))
            .with_frame_capture(
                kinekino_player::application::frame_capture::UnavailableFrameCapture,
            )
            .with_playback_status(
                kinekino_player::application::playback_status::UnavailablePlaybackStatus,
            )
            .with_playback_control(
                kinekino_player::application::playback_control::UnavailablePlaybackControl,
            )
            .with_review_store(kinekino_player::application::model_review::UnavailableReviewStore)
            .with_proposal_store(kinekino_player::application::proposals::UnavailableProposalStore)
            .with_project_store(kinekino_player::application::projects::UnavailableProjectStore)
            .with_catalog(kinekino_player::application::catalog::UnavailableCatalog)
            .with_jobs(Arc::new(
                kinekino_player::application::jobs::UnavailableJobs,
            ));
        let result = sdk
            .export_snapshot(RequestContext::default(), fixture.request.clone())
            .unwrap();
        assert_eq!(result.source_sha256, fixture.request.expected_source_sha256);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn configured_discovery_schema_and_context_rejection_match_actual_port() {
        kinekino_platform::install();
        let fixture = Fixture::new();
        let count = Arc::new(AtomicUsize::new(0));
        let sdk = PlayerSdk::default().with_snapshot_export(Counted(count.clone()));
        let cap = sdk.execute(Request::new("capabilities")).data.unwrap();
        assert_eq!(cap["snapshot_export"]["available"], true);
        assert_eq!(cap["snapshot_export"]["ffmpeg_required"], false);
        assert_eq!(
            cap["commands"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["command"] == COMMAND)
                .unwrap()["status"],
            "supported"
        );
        let mut schema = Request::new("schema");
        schema.payload = json!({"command":COMMAND});
        let schema = sdk.execute(schema).data.unwrap();
        assert_eq!(schema["status"], "supported");
        assert_eq!(
            schema["response"]["properties"]["data"]["anyOf"][0]["properties"]["metadata_preserved"]
                ["const"],
            false
        );
        for field in ["expected_revision", "idempotency_key", "timeout_ms"] {
            let mut request = fixture.wire();
            match field {
                "expected_revision" => request.context.expected_revision = Some("1".into()),
                "idempotency_key" => request.context.idempotency_key = Some("key".into()),
                _ => request.context.timeout_ms = Some(1000),
            }
            assert_eq!(
                sdk.execute(request).error.unwrap().code,
                ErrorCode::Unsupported
            );
        }
        assert_eq!(count.load(Ordering::SeqCst), 0);
        assert!(!std::path::Path::new(&fixture.request.output_path).exists());
    }
    #[test]
    fn fresh_cli_exports_exact_pixels_without_media_decoder_configuration()
    -> Result<(), Box<CliFailure>> {
        kinekino_platform::install();
        let fixture = Fixture::new();
        let (code, response) = cli(&fixture.wire(), None).map_err(|failure| failure.closeout())?;
        assert_eq!(code, 0);
        fixture.assert_crop(&response);
        let output_before = fs::read(&fixture.request.output_path).unwrap();
        let (code, response) = cli(&fixture.wire(), None).map_err(|failure| failure.closeout())?;
        assert_ne!(code, 0);
        assert_eq!(response["error"]["code"], "revision_conflict");
        assert_eq!(
            fs::read(&fixture.request.output_path).unwrap(),
            output_before
        );
        assert_eq!(
            fs::read(&fixture.request.source_path).unwrap(),
            fixture.bytes
        );
        Ok(())
    }
    #[test]
    fn fresh_cli_schema_and_invalid_payloads_do_not_publish() -> Result<(), Box<CliFailure>> {
        kinekino_platform::install();
        let fixture = Fixture::new();
        let mut schema = Request::new("schema");
        schema.payload = json!({"command":COMMAND});
        let (code, response) = cli(&schema, None).map_err(|failure| failure.closeout())?;
        assert_eq!(code, 0);
        assert_eq!(response["data"]["status"], "supported");
        let (code, response) =
            cli(&Request::new("capabilities"), None).map_err(|failure| failure.closeout())?;
        assert_eq!(code, 0);
        assert_eq!(response["data"]["snapshot_export"]["available"], true);
        assert_eq!(
            response["data"]["snapshot_export"]["ffmpeg_required"],
            false
        );
        for (field, value, expected) in [
            (
                "expected_source_sha256",
                json!("0".repeat(64)),
                "revision_conflict",
            ),
            (
                "crop",
                json!({"x":u32::MAX,"y":0,"width":2,"height":1}),
                "invalid_argument",
            ),
            ("unknown", json!(true), "invalid_argument"),
        ] {
            let mut request = fixture.wire();
            request.payload[field] = value;
            let (code, response) = cli(&request, None).map_err(|failure| failure.closeout())?;
            assert_ne!(code, 0);
            assert_eq!(response["error"]["code"], expected);
            assert!(!std::path::Path::new(&fixture.request.output_path).exists());
        }
        assert_eq!(
            fs::read(&fixture.request.source_path).unwrap(),
            fixture.bytes
        );
        Ok(())
    }
    #[test]
    fn fresh_cli_rejects_crc_damage_and_unsupported_context_before_publication()
    -> Result<(), Box<CliFailure>> {
        kinekino_platform::install();
        let fixture = Fixture::new();
        let mut timed = fixture.wire();
        timed.context.timeout_ms = Some(1000);
        let (code, response) = cli(&timed, None).map_err(|failure| failure.closeout())?;
        assert_ne!(code, 0);
        assert_eq!(response["error"]["code"], "unsupported");
        let mut damaged = fixture.bytes.clone();
        damaged[29] ^= 1;
        fs::write(&fixture.request.source_path, &damaged).unwrap();
        let mut request = fixture.wire();
        request.payload["expected_source_sha256"] = json!(hash(&damaged));
        let (code, response) = cli(&request, None).map_err(|failure| failure.closeout())?;
        assert_ne!(code, 0);
        assert_eq!(response["error"]["code"], "invalid_argument");
        assert!(!std::path::Path::new(&fixture.request.output_path).exists());
        assert_eq!(fs::read(&fixture.request.source_path).unwrap(), damaged);
        Ok(())
    }
    #[test]
    fn endpoint_uses_only_injected_host_port_and_never_local_fallback()
    -> Result<(), Box<CliFailure>> {
        kinekino_platform::install();
        let fixture = Fixture::new();
        let unsupported =
            LocalHost::start(Arc::new(PlayerSdk::default()), TransportOptions::default()).unwrap();
        let (code, response) = cli_observed(
            &fixture.wire(),
            Some(unsupported.endpoint()),
            Some(HostObservation {
                injected: false,
                calls: None,
            }),
        )
        .map_err(|failure| failure.closeout())?;
        assert_ne!(code, 0);
        assert_eq!(response["error"]["code"], "unsupported");
        assert!(!std::path::Path::new(&fixture.request.output_path).exists());
        drop(unsupported);
        let count = Arc::new(AtomicUsize::new(0));
        let host = LocalHost::start(
            Arc::new(PlayerSdk::default().with_snapshot_export(Counted(count.clone()))),
            TransportOptions::default(),
        )
        .unwrap();
        let (code, response) = cli_observed(
            &fixture.wire(),
            Some(host.endpoint()),
            Some(HostObservation {
                injected: true,
                calls: Some(&count),
            }),
        )
        .map_err(|failure| failure.closeout())?;
        assert_eq!(code, 0);
        fixture.assert_crop(&response);
        assert_eq!(count.load(Ordering::SeqCst), 1);
        drop(host);
        Ok(())
    }
}
