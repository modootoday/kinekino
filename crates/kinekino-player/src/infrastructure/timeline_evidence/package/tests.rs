//! Synthetic producer bundle in real ReviewFiles; never a real decoder/quality claim.
use super::*;
use crate::{
    application::review_source::{ReviewCases, ReviewPackageMedia},
    infrastructure::model_review::ReviewFiles,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture {
    root: PathBuf,
    case: ReviewCase,
    media: MediaReference,
    reference: EvidenceReference,
    engine: String,
    mode: String,
}
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn asset(root: &Path, assets: &mut Vec<Value>, id: &str, bytes: &[u8], mime: &str) -> Value {
    let digest = sha(bytes);
    fs::write(root.join("objects").join(&digest), bytes).unwrap();
    assets.push(
        json!({"asset_id":id,"sha256":digest,"bytes":bytes.len().to_string(),"media_type":mime}),
    );
    json!({"asset_id":id,"sha256":digest})
}
fn encoded(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).unwrap()
}
fn trace(
    root: &Path,
    assets: &mut Vec<Value>,
    prefix: &str,
    command: &str,
    inputs: &str,
    contract: Value,
    process: &str,
) -> Value {
    let command = asset(
        root,
        assets,
        &format!("{prefix}-command"),
        command.as_bytes(),
        "text/plain",
    );
    let inputs = asset(
        root,
        assets,
        &format!("{prefix}-inputs"),
        inputs.as_bytes(),
        "text/plain",
    );
    let process = asset(
        root,
        assets,
        &format!("{prefix}-process"),
        process.as_bytes(),
        "text/plain",
    );
    json!({"command":command,"inputs":inputs,"process":process,"contract":contract})
}
fn wav(data: &[u8]) -> Vec<u8> {
    let mut bytes = b"RIFF".to_vec();
    bytes.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&[1, 0, 1, 0, 128, 62, 0, 0, 0, 125, 0, 0, 2, 0, 16, 0]);
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
    bytes.extend_from_slice(data);
    bytes
}
impl Fixture {
    fn new(mode: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "timeline-package-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir(&root).unwrap();
        for dir in ["packages", "objects", "decisions"] {
            fs::create_dir(root.join(dir)).unwrap();
        }
        let mut assets = Vec::new();
        let request_bytes = br#"{"source":"synthetic; no path lookup"}"#;
        let source_sha = sha(request_bytes);
        fs::write(root.join("objects").join(&source_sha), request_bytes).unwrap();
        let media = asset(
            &root,
            &mut assets,
            "source-media",
            b"synthetic media; no decode",
            "video/mp4",
        );
        let engine = asset(
            &root,
            &mut assets,
            "engine",
            b"synthetic engine; never executed",
            "application/octet-stream",
        );
        let log = if mode == "clock-gap" {
            "[Parsed_ashowinfo_2] n:0 pts:0 fmt:s16 channels:1 rate:16000 nb_samples:240000\n[Parsed_ashowinfo_2] n:1 pts:240001 fmt:s16 channels:1 rate:16000 nb_samples:240000"
        } else {
            "[Parsed_ashowinfo_2] n:0 pts:0 fmt:s16 channels:1 rate:16000 nb_samples:480000"
        };
        let log = asset(
            &root,
            &mut assets,
            "audio-log",
            log.as_bytes(),
            "text/plain",
        );
        let full_data = vec![0u8; 960000];
        let mut clip_data = full_data[544000..816000].to_vec();
        let mut replay_data = full_data.clone();
        if mode == "clip-mismatch" {
            clip_data[0] = 1;
        }
        if mode == "replay-mismatch" {
            replay_data[0] = 1;
        }
        let full = asset(
            &root,
            &mut assets,
            "full-wav",
            &wav(&full_data),
            "audio/wav",
        );
        let clip = asset(
            &root,
            &mut assets,
            "clip-wav",
            &wav(&clip_data),
            "audio/wav",
        );
        let replay = asset(
            &root,
            &mut assets,
            "replay-pcm",
            &replay_data,
            "application/octet-stream",
        );
        let video = json!({"frames":(384..624).map(|pts|json!({"pts":pts,"best_effort_timestamp":pts,"duration":1,"key_frame":0})).collect::<Vec<_>>(),"streams":[{"index":0,"time_base":"1/24"}]});
        let video = asset(
            &root,
            &mut assets,
            "video-frames",
            &encoded(&video),
            "application/json",
        );
        let mut audio_exec = json!({"schema_version":"1","source_sha256":media["sha256"],"engine_sha256":engine["sha256"],"log_sha256":log["sha256"],"replay_sha256":replay["sha256"],"completion":"observer_accepted","copy_timestamps":true,"sample_rate":16000,"channels":1,"sample_format":"s16","time_base":"1/16000","async_correction":false});
        let command = r#""fixture-engine" "-hide_banner" "-nostdin" "-nostats" "-loglevel" "info" "-threads" "2" "-filter_threads" "1" "-protocol_whitelist" "file" "-f" "mov" "-enable_drefs" "0" "-copyts" "-i" "fixture-source" "-map" "0:a:0" "-vn" "-sn" "-dn" "-af" "aresample=16000:osf=s16:ochl=mono:async=0,asettb=expr=1/16000,ashowinfo" "-c:a" "pcm_s16le" "-f" "s16le" "pipe:1""#;
        let command = if mode == "command-mismatch" {
            command.replace("async=0", "async=1")
        } else if mode == "selected-stream-mismatch" {
            command.replace("0:a:0", "0:a:1")
        } else {
            command.into()
        };
        let inputs = format!(
            "path=fixture-source\nsha256={}\npath=fixture-full\nsha256={}\npath=fixture-engine\nsha256={}\n",
            media["sha256"].as_str().unwrap(),
            full["sha256"].as_str().unwrap(),
            engine["sha256"].as_str().unwrap()
        );
        let process = if mode == "unfinished-original" || mode == "incomplete" {
            "pid=1\nresult=Ok(())\nelapsed_ms=0\nterminal=false\n"
        } else if mode == "nonzero-exit" {
            "pid=1\nresult=Err(\"decode exit=Some(1)\")\nelapsed_ms=0\nterminal=true\n"
        } else {
            "pid=1\nresult=Ok(())\nelapsed_ms=0\nterminal=true\n"
        };
        let contract = if mode == "wrong-contract" {
            json!({"kind":"video_first_stream_frame_window","interval":"16%26"})
        } else {
            json!({"kind":"audio_copy_ts_mono16k_s16_first_stream"})
        };
        audio_exec["trace"] = trace(
            &root,
            &mut assets,
            "audio",
            &command,
            &inputs,
            contract,
            process,
        );
        if mode == "missing-contract" {
            audio_exec["trace"]
                .as_object_mut()
                .unwrap()
                .remove("contract");
        }
        match mode {
            "wrong-source" => audio_exec["source_sha256"] = "a".repeat(64).into(),
            "async-correction" => audio_exec["async_correction"] = true.into(),
            _ => {}
        }
        let audio_exec = asset(
            &root,
            &mut assets,
            "audio-execution",
            &encoded(&audio_exec),
            "application/json",
        );
        let mut video_exec = json!({"schema_version":"1","source_sha256":media["sha256"],"engine_sha256":engine["sha256"],"frames_sha256":video["sha256"],"stream_index":if mode=="wrong-stream"{1}else{0},"completion":"observer_accepted"});
        let video_command = r#""fixture-engine" "-v" "error" "-protocol_whitelist" "file" "-f" "mov" "-enable_drefs" "0" "-probesize" "1048576" "-analyzeduration" "1000000" "-threads" "1" "-select_streams" "v:0" "-read_intervals" "16%26" "-show_frames" "-show_streams" "-show_entries" "frame=pts,best_effort_timestamp,duration,key_frame:stream=index,time_base" "-of" "json" "fixture-source""#;
        let video_inputs = format!(
            "source=fixture-source\nsource_sha256={}\nffprobe=fixture-engine\nffprobe_sha256={}\n",
            media["sha256"].as_str().unwrap(),
            engine["sha256"].as_str().unwrap()
        );
        video_exec["trace"] = trace(
            &root,
            &mut assets,
            "video",
            video_command,
            &video_inputs,
            json!({"kind":"video_first_stream_frame_window","interval":if mode=="window-mismatch"{"17%26"}else{"16%26"}}),
            "pid=2\nresult=Ok(())\nelapsed_ms=0\nterminal=true\n",
        );
        let video_exec = asset(
            &root,
            &mut assets,
            "video-execution",
            &encoded(&video_exec),
            "application/json",
        );
        let mut png = Vec::new();
        {
            let encoder = png::Encoder::new(&mut png, 1, 1);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&[0]).unwrap();
        }
        if mode == "bad-png" {
            png = b"not png".to_vec();
        }
        let png_bytes = png.len();
        let png = asset(&root, &mut assets, "png", &png, "image/png");
        let dependency = json!({"available":true,"adapter":"external_ffmpeg_cpu","ffmpeg_path":absolute("fixture/ffmpeg.exe"),"ffprobe_path":absolute("fixture/ffprobe.exe")});
        let pin = json!({"sha256":engine["sha256"],"file_identity":"fixture-engine-identity"});
        let input_json = json!({"source":absolute("fixture-source.mp4"),"original":{"sha256":media["sha256"],"file_identity":"fixture-source-identity"},"cli":pin,"ffmpeg":pin,"ffprobe":pin,"dependency":dependency,"model_output":false,"actual9_binding":false,"quality_approved":false});
        let capture_inputs = asset(
            &root,
            &mut assets,
            "capture-inputs",
            &encoded(&input_json),
            "application/json",
        );
        let mut captures = Vec::new();
        for (id, us, pts) in [
            ("origin", 0, 0),
            ("start", 17000000, 408),
            ("end", 25500000, 612),
        ] {
            let pts = if mode == "wrong-capture" && id == "start" {
                pts + 1
            } else {
                pts
            };
            let mut request = json!({"context":{"request_id":id,"api_version":"1","expected_revision":null,"idempotency_key":null,"timeout_ms":30000},"command":"production.capture_frame","payload":{"source_path":absolute("fixture-source.mp4"),"output_path":absolute(&format!("fixture-{id}.png")),"source_pts_us":us.to_string()}});
            let actual_us = crate::domain::media_timeline::TimelineTransform::new(
                TimeBase::new(1, 24).unwrap(),
                0,
            )
            .pts_to_us(pts, crate::domain::media_timeline::Rounding::Down)
            .unwrap()
            .value;
            let mut response = json!({"api_version":"1","request_id":id,"ok":true,"error":null,"data":{"output_path":absolute(&format!("fixture-{id}.png")),"requested_source_pts_us":us.to_string(),"actual_source_pts_us":actual_us.to_string(),"source_pts_ticks":pts.to_string(),"source_time_base":"1/24","width":1,"height":1,"output_bytes":png_bytes,"output_sha256":png["sha256"],"dependency":dependency}});
            let mut process = json!({"cli":pin,"pid":3,"exit_code":0,"elapsed_ms":1});
            if id == "start" {
                match mode {
                    "capture-request-mismatch" => {
                        request["payload"]["source_pts_us"] = (us + 1).to_string().into()
                    }
                    "capture-response-mismatch" => {
                        response["data"]["source_pts_ticks"] = (pts + 1).to_string().into()
                    }
                    "capture-producer-mismatch" => process["cli"]["sha256"] = "a".repeat(64).into(),
                    "capture-output-mismatch" => {
                        response["data"]["output_sha256"] = "a".repeat(64).into()
                    }
                    "capture-envelope-mismatch" => response["request_id"] = "foreign".into(),
                    "capture-original-error" => process["exit_code"] = 1.into(),
                    _ => {}
                }
            }
            let request = asset(
                &root,
                &mut assets,
                &format!("{id}-request"),
                &encoded(&request),
                "application/json",
            );
            let response = asset(
                &root,
                &mut assets,
                &format!("{id}-response"),
                &encoded(&response),
                "application/json",
            );
            let process = asset(
                &root,
                &mut assets,
                &format!("{id}-process"),
                &encoded(&process),
                "application/json",
            );
            let mut value = json!({"schema_version":"1","source_sha256":media["sha256"],"engine":engine,"png":png,"requested_source_us":us.to_string(),"actual_pts":pts,"time_base":"1/24","exit_code":0,"complete":true,"trace":{"inputs":capture_inputs,"request":request,"response":response,"process":process,"contract":"sdk_first_source_frame_at_or_after"}});
            if mode == "capture-missing-contract" && id == "start" {
                value["trace"].as_object_mut().unwrap().remove("contract");
            }
            captures.push(asset(
                &root,
                &mut assets,
                id,
                &encoded(&value),
                "application/json",
            ));
        }
        let manifest = json!({"schema_version":"1","source_request_sha256":source_sha,"media":media,"mapping_revision":"1","audio":{"log":log,"full_wav":full,"clip_wav":clip,"replay_pcm":replay,"clip_start_sample":"272000","engine":engine,"execution":audio_exec},"video":{"frames":video,"engine":engine,"execution":video_exec},"origin_capture":captures[0],"start_capture":captures[1],"end_capture":captures[2]});
        let reference = asset(
            &root,
            &mut assets,
            "timeline-proof",
            &encoded(&manifest),
            "application/json",
        );
        let extra = assets.split_off(16.min(assets.len()));
        let checks=extra.chunks(8).enumerate().map(|(index,evidence)|json!({"check_id":format!("quality-{index}"),"outcome":"not_run","summary":"Synthetic only","evidence":evidence})).collect::<Vec<_>>();
        let package = json!({"schema_version":"1","package_id":"timeline-fixture","items":[{"item_id":"item","task":"asr","source":{"asset_id":"request","sha256":source_sha,"bytes":request_bytes.len().to_string(),"media_type":"application/json"},"range":null,"baseline":null,"candidate":{"condition":{"model_id":"fixture","revision":"r1","precision":"fp32","engine_id":"fixture","engine_revision":"r1","input_conditions":{"sample":"synthetic"}},"assets":assets,"automated_checks":checks,"measured_resources":{"kind":"synthetic"}},"criteria":[{"criterion_id":"compare","how_to_review":"Compare source","expected":"Faithful result"}],"known_issues":[]}]});
        let bytes = encoded(&package);
        let package_sha = sha(&bytes);
        fs::write(
            root.join("packages").join(format!("{package_sha}.json")),
            bytes,
        )
        .unwrap();
        Self {
            root,
            case: ReviewCase {
                package_sha256: package_sha,
                item_id: "item".into(),
                source_request_sha256: source_sha,
            },
            media: serde_json::from_value(media).unwrap(),
            reference: serde_json::from_value(reference).unwrap(),
            engine: engine["sha256"].as_str().unwrap().into(),
            mode: mode.into(),
        }
    }
    fn resolver(&self) -> ReviewTimelineEvidence<ReviewFiles> {
        ReviewTimelineEvidence::new(
            ReviewFiles::open(&self.root).unwrap(),
            TrustedTimelineAdmission {
                case: self.case.clone(),
                media: self.media.clone(),
                evidence: self.reference.clone(),
                audio_engine_sha256: if self.mode == "wrong-tool" {
                    "a".repeat(64)
                } else {
                    self.engine.clone()
                },
                video_engine_sha256: self.engine.clone(),
                capture_engine_sha256: self.engine.clone(),
                capture_cli_sha256: self.engine.clone(),
            },
        )
        .unwrap()
    }
}
#[test]
fn nominated_package_builds_observations_and_holds_original_case_until_release() {
    let f = Fixture::new("");
    let resolver = f.resolver();
    let cases = ReviewCases::new(ReviewFiles::open(&f.root).unwrap());
    let media = ReviewPackageMedia::new(ReviewFiles::open(&f.root).unwrap());
    let retained = crate::application::review_source::timeline_evidence::retain(
        &cases,
        &media,
        &resolver,
        &f.case,
        &f.media,
        &f.reference,
    )
    .unwrap();
    assert_eq!(retained.mapping().source_start_us, 17000000);
    assert_eq!(retained.mapping().source_end_us, 25500000);
    assert!(!retained.playback_ready());
    retained.check_current().unwrap();
    let path = f.root.join("objects").join(&f.media.sha256);
    assert_eq!(
        fs::OpenOptions::new().write(true).open(&path).is_err(),
        crate::infrastructure::os_policy::has_mandatory_share_modes()
    );
    drop(retained);
    drop(fs::OpenOptions::new().write(true).open(path).unwrap());
}
#[test]
fn mismatched_source_tool_clock_output_capture_and_execution_are_rejected() {
    for mode in [
        "wrong-source",
        "incomplete",
        "nonzero-exit",
        "async-correction",
        "wrong-stream",
        "wrong-tool",
        "clock-gap",
        "clip-mismatch",
        "replay-mismatch",
        "bad-png",
        "wrong-capture",
        "command-mismatch",
        "selected-stream-mismatch",
        "unfinished-original",
        "wrong-contract",
        "missing-contract",
        "window-mismatch",
        "capture-request-mismatch",
        "capture-response-mismatch",
        "capture-producer-mismatch",
        "capture-output-mismatch",
        "capture-missing-contract",
        "capture-envelope-mismatch",
        "capture-original-error",
    ] {
        let f = Fixture::new(mode);
        let resolver = f.resolver();
        let result = resolver.retain(&f.case, &f.media, &f.reference);
        assert!(result.is_err(), "accepted {mode}");
        drop(result);
        drop(
            fs::OpenOptions::new()
                .write(true)
                .open(f.root.join("objects").join(&f.media.sha256))
                .unwrap(),
        );
    }
}
#[test]
fn foreign_nomination_is_rejected_before_package_lookup() {
    let f = Fixture::new("");
    let resolver = f.resolver();
    let mut case = f.case.clone();
    case.package_sha256 = "a".repeat(64);
    assert!(
        matches!(resolver.retain(&case,&f.media,&f.reference),Err(e) if e.code==ErrorCode::RevisionConflict)
    );
    let mut reference = f.reference.clone();
    reference.sha256 = "a".repeat(64);
    assert!(
        matches!(resolver.retain(&f.case,&f.media,&reference),Err(e) if e.code==ErrorCode::RevisionConflict)
    );
}

/// An absolute fixture path on the host OS (never opened; only absolute-path validation).
fn absolute(name: &str) -> String {
    std::env::temp_dir()
        .join("kinekino-timeline-fixture")
        .join(name)
        .to_string_lossy()
        .into_owned()
}
