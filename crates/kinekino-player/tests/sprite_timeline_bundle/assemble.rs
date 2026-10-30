use super::storage::{Bundle, sha, write_new};
use kinekino_player::domain::review_source::{EvidenceReference, MediaReference, ReviewCase};
use serde_json::json;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
pub struct Pins {
    pub audio: String,
    pub probe: String,
    pub capture: String,
    pub cli: String,
}
pub struct Prepared {
    pub bundle: Bundle,
    pub case: ReviewCase,
    pub media: MediaReference,
    pub evidence: EvidenceReference,
    pub pins: Pins,
}
fn field<'a>(raw: &'a str, key: &str) -> &'a str {
    let mut values = raw
        .lines()
        .filter_map(|line| line.strip_prefix(&format!("{key}=")));
    let value = values.next().unwrap();
    assert!(values.next().is_none());
    value
}
fn argv(raw: &str) -> Vec<String> {
    serde_json::Deserializer::from_str(raw.lines().next().unwrap())
        .into_iter::<String>()
        .collect::<Result<_, _>>()
        .unwrap()
}
fn option<'a>(args: &'a [String], key: &str) -> &'a str {
    let mut selected = args
        .iter()
        .enumerate()
        .filter(|(_, arg)| arg.as_str() == key);
    let index = selected.next().unwrap().0;
    assert!(selected.next().is_none());
    &args[index + 1]
}
fn accepted(raw: &str) -> &'static str {
    match (field(raw, "result"), field(raw, "terminal")) {
        ("Ok(())", "true") => "observer_accepted",
        value => panic!("unaccepted original completion {value:?}"),
    }
}
pub fn prepare_from_pinned_sprite_history() -> Prepared {
    let workspace = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let mut bundle = Bundle::new(workspace.join("runtime/verification/sprite-timeline-actual-r1"));
    let mut refs = BTreeMap::new();
    for (id, path, digest, mime) in [
        (
            "source-media",
            "runtime/ai-spike/source/sprite-fright-sprite-source-r1/source.mp4",
            "7cfc8b8c6eeb19685048a289aca208a01356f5082b470fb9f4d933fe4b2087ef",
            "video/mp4",
        ),
        (
            "full-wav",
            "runtime/ai-spike/assets/sprite-clips-r1/full-mono-16k.wav",
            "872987bc1c6296b33241e833f5a315e8bd656b2d3ed05d14034231cad4ae51a4",
            "audio/wav",
        ),
        (
            "clip-wav",
            "runtime/ai-spike/assets/sprite-clips-r1/latin-17.wav",
            "d12ec627660e5b3de88579296d45b6f3cf368c7fe6e2419ea3acbeb643c33270",
            "audio/wav",
        ),
        (
            "replay-pcm",
            "runtime/build-cache/tmp/sprite-audio-legacy-replay-datalab-r1/decoded.s16le",
            "74f30535a9ef581f3a048917941de6f41dad1c40101149c1ebe84352a1300d8b",
            "application/octet-stream",
        ),
        (
            "audio-engine",
            "runtime/toolchains/ffmpeg-6.1-replay-r1/ffmpeg-6.1-essentials_build/bin/ffmpeg.exe",
            "71414777922d05f2639a5cf4a0f9a405f169809cf459000a6f976c9add61f060",
            "application/octet-stream",
        ),
        (
            "audio-log",
            "runtime/verification/sprite-audio-legacy-replay-datalab-r1/decode.stderr",
            "ec1a5907637c54a4da4a5184fbb9f72b7f9e1d1dacca229d1b1d3a17a755345c",
            "text/plain",
        ),
        (
            "audio-command",
            "runtime/verification/sprite-audio-legacy-replay-datalab-r1/command.txt",
            "cbf1aa5d1251aea411b05ee93a4b25a4452f0f44bf5a0ded11d169137e31f822",
            "text/plain",
        ),
        (
            "audio-inputs",
            "runtime/verification/sprite-audio-legacy-replay-datalab-r1/INPUT.txt",
            "e24832b322bc0af67084870a7e877084ce3499cf4b8c2b2d0d93916f72f41a08",
            "text/plain",
        ),
        (
            "audio-process",
            "runtime/verification/sprite-audio-legacy-replay-datalab-r1/process.txt",
            "90cf6f699cbd2699a58b6836cbfbaa95017c96a446ed54dcb7c7196fe046e628",
            "text/plain",
        ),
        (
            "clip-lineage",
            "runtime/verification/sprite-pcm-lineage-datalab-r1/RETURN.txt",
            "4e99f27179538b8fa5c7223a735c3e551a2eb98cd597baa77bf364fd9dacaff8",
            "text/plain",
        ),
        (
            "video-frames",
            "runtime/verification/sprite-frame-pts-datalab-r1/frames.stdout.json",
            "ba26bd11b6bee0fef8fcb5a461f4b9601beee05abcf26e275bb935d2df954a34",
            "application/json",
        ),
        (
            "video-command",
            "runtime/verification/sprite-frame-pts-datalab-r1/command.txt",
            "daf80946f9b2bbe5db37d0c52a93b1bf700fb0b3c9627e03ab184714998ff5b1",
            "text/plain",
        ),
        (
            "video-inputs",
            "runtime/verification/sprite-frame-pts-datalab-r1/INPUT.txt",
            "54da2231adcf268a21cf195d46432caf4730f5c906794e6ce685685d72788ada",
            "text/plain",
        ),
        (
            "video-process",
            "runtime/verification/sprite-frame-pts-datalab-r1/process.txt",
            "5a35569a7f072d4a3ac33e158fac2968530d52460291205d0ca41aaa6911e5aa",
            "text/plain",
        ),
        (
            "capture-inputs",
            "runtime/build-cache/tmp/sprite-capture-cli-11384/INPUT.json",
            "2f0f6c895148cd365fa7fbc4bbc215da0a0745f9f49065a81949dfbf05b882d6",
            "application/json",
        ),
    ] {
        refs.insert(
            id.to_string(),
            bundle.import(id, &workspace.join(path), digest, mime),
        );
    }
    // This input was imported with a fixed SHA above; paths select read-only copies,
    // never executable commands or untrusted request payloads.
    let capture_inputs = bundle.json(&refs["capture-inputs"]);
    let capture_executable = PathBuf::from(
        capture_inputs["dependency"]["ffmpeg_path"]
            .as_str()
            .unwrap(),
    );
    let probe_executable = PathBuf::from(
        capture_inputs["dependency"]["ffprobe_path"]
            .as_str()
            .unwrap(),
    );
    refs.insert(
        "capture-engine".into(),
        bundle.import(
            "capture-engine",
            &capture_executable,
            "09948d4cdd0650da6ff5a87577469f2a218dc2615ae379f8f734d24c49de0f73",
            "application/octet-stream",
        ),
    );
    refs.insert(
        "video-engine".into(),
        bundle.import(
            "video-engine",
            &probe_executable,
            "a6618e99bb58869ded3c6f37b53aa1a8d701c3591dbb7b5b317d47369c112be2",
            "application/octet-stream",
        ),
    );
    let media = refs["source-media"].clone();
    let audio_args = argv(&bundle.text(&refs["audio-command"]));
    let filter = option(&audio_args, "-af");
    let filters: Vec<_> = filter.split(',').collect();
    assert_eq!(filters.len(), 3);
    assert_eq!(filters[2], "ashowinfo");
    let resample: BTreeMap<_, _> = filters[0]
        .split(':')
        .map(|entry| entry.split_once('=').unwrap())
        .collect();
    let rate = resample["aresample"].parse::<u32>().unwrap();
    let format = resample["osf"];
    let channels = match resample["ochl"] {
        "mono" => 1,
        value => panic!("unsupported channel layout {value}"),
    };
    let async_correction = resample["async"].parse::<u32>().unwrap() != 0;
    let time_base = filters[1].strip_prefix("asettb=expr=").unwrap();
    let audio_execution=bundle.generated("audio-execution",&json!({"schema_version":"1","source_sha256":media["sha256"],"engine_sha256":refs["audio-engine"]["sha256"],"log_sha256":refs["audio-log"]["sha256"],"replay_sha256":refs["replay-pcm"]["sha256"],"completion":accepted(&bundle.text(&refs["audio-process"])),"copy_timestamps":audio_args.iter().any(|arg|arg=="-copyts"),"sample_rate":rate,"channels":channels,"sample_format":format,"time_base":time_base,"async_correction":async_correction,"trace":{"command":refs["audio-command"],"inputs":refs["audio-inputs"],"process":refs["audio-process"],"contract":{"kind":"audio_copy_ts_mono16k_s16_first_stream"}}}),"application/json");
    let video_args = argv(&bundle.text(&refs["video-command"]));
    let raw_video = bundle.json(&refs["video-frames"]);
    assert_eq!(raw_video["streams"].as_array().unwrap().len(), 1);
    let video_execution=bundle.generated("video-execution",&json!({"schema_version":"1","source_sha256":media["sha256"],"engine_sha256":refs["video-engine"]["sha256"],"frames_sha256":refs["video-frames"]["sha256"],"stream_index":raw_video["streams"][0]["index"],"completion":accepted(&bundle.text(&refs["video-process"])),"trace":{"command":refs["video-command"],"inputs":refs["video-inputs"],"process":refs["video-process"],"contract":{"kind":"video_first_stream_frame_window","interval":option(&video_args,"-read_intervals")}}}),"application/json");
    let mut captures = Vec::new();
    let capture_folder = workspace.join("runtime/build-cache/tmp/sprite-capture-cli-11384");
    for (label, request_sha, response_sha, process_sha, png_sha) in [
        (
            "0s",
            "26e522acf499184862ab198be374d4e2ac364ff87b1258725fc0ca9662d51a16",
            "ebb57c402bb1d07c86e29a846797005915b8394905acda590d22bcb3adfeccfc",
            "862c5c47c71e8ef048f80491a0b3d7764a46b646b1f91265196a5b6d0da841b3",
            "6c2ac82030e7a1a584e85ef3053863287c376a00133f2e9462f480e693baa38f",
        ),
        (
            "17s",
            "a4aa2dbd7c3ced1bfd8dbd7252d03f402f1c3221e27001f6ce561c06dd783550",
            "b6244762d523e75705b990c51e9dee384c6db71b347167836b0db4b7019a3463",
            "fc997cd6cbe1bc41c4f6842b477c0b31feaa2576b2d923b5b02162db2dae1c28",
            "ac5d6d071c716a943852261bad441a695aa7cb53583340a8a8d6349ff108eed0",
        ),
        (
            "25-5s",
            "0605e5f5e4cef0cacc307f31eeff476aed0d2801370a659a5c97e1f494e27c7f",
            "d00367a456be8a9a0ab6cb18c1d17c4d5b80dbd31f48baefbc2bf8c84d2c8c54",
            "601ff39334f85bb13757c68d6a61135f57fa920920ac5835cedecee29c7afcea",
            "bf0e3d8ccfbec31fe03745fc45521bbaeff0d69253f594e324643e431b2e360f",
        ),
    ] {
        let request = bundle.import(
            &format!("capture-{label}-request"),
            &capture_folder.join(format!("{label}.request.json")),
            request_sha,
            "application/json",
        );
        let response = bundle.import(
            &format!("capture-{label}-response"),
            &capture_folder.join(format!("{label}.stdout.json")),
            response_sha,
            "application/json",
        );
        let process = bundle.import(
            &format!("capture-{label}-process"),
            &capture_folder.join(format!("{label}.process.json")),
            process_sha,
            "application/json",
        );
        let png = bundle.import(
            &format!("capture-{label}-png"),
            &capture_folder.join(format!("{label}.png")),
            png_sha,
            "image/png",
        );
        let req = bundle.json(&request);
        let resp = bundle.json(&response);
        let proc = bundle.json(&process);
        let exit = proc["exit_code"].as_i64().unwrap();
        let complete = exit == 0 && resp["ok"].as_bool().unwrap();
        let normalized = json!({"schema_version":"1","source_sha256":capture_inputs["original"]["sha256"],"engine":refs["capture-engine"],"png":png,"requested_source_us":req["payload"]["source_pts_us"],"actual_pts":resp["data"]["source_pts_ticks"].as_str().unwrap().parse::<i64>().unwrap(),"time_base":resp["data"]["source_time_base"],"exit_code":exit,"complete":complete,"trace":{"inputs":refs["capture-inputs"],"request":request,"response":response,"process":process,"contract":"sdk_first_source_frame_at_or_after"}});
        captures.push(bundle.generated(
            &format!("capture-{label}"),
            &normalized,
            "application/json",
        ));
    }
    let lineage = bundle.text(&refs["clip-lineage"]);
    let clip_start = field(&lineage, "start_sample").parse::<u64>().unwrap();
    let source_request=bundle.generated("source-request",&json!({"kind":"technical_timeline_revalidation","media":media,"clip_lineage":refs["clip-lineage"],"actual9_binding":false,"quality_approved":false,"new_decoder_execution":false}),"application/json");
    let manifest=bundle.generated("timeline-proof",&json!({"schema_version":"1","source_request_sha256":source_request["sha256"],"media":media,"mapping_revision":"1","audio":{"log":refs["audio-log"],"full_wav":refs["full-wav"],"clip_wav":refs["clip-wav"],"replay_pcm":refs["replay-pcm"],"clip_start_sample":clip_start.to_string(),"engine":refs["audio-engine"],"execution":audio_execution},"video":{"frames":refs["video-frames"],"engine":refs["video-engine"],"execution":video_execution},"origin_capture":captures[0],"start_capture":captures[1],"end_capture":captures[2]}),"application/json");
    let source = bundle
        .assets
        .iter()
        .find(|asset| asset["asset_id"] == "source-request")
        .unwrap()
        .clone();
    bundle
        .assets
        .retain(|asset| asset["asset_id"] != "source-request");
    let mut assets = bundle.assets.clone();
    let extra = assets.split_off(16.min(assets.len()));
    let checks=extra.chunks(8).enumerate().map(|(index,evidence)|json!({"check_id":format!("technical-{index}"),"outcome":"not_run","summary":"Technical observation; no model quality approval","evidence":evidence})).collect::<Vec<_>>();
    let package = json!({"schema_version":"1","package_id":"sprite-timeline-technical-r1","items":[{"item_id":"sprite-latin-17-technical","task":"media_timeline","source":source,"range":null,"baseline":null,"candidate":{"condition":{"model_id":"technical-timeline-observer","revision":"r1","precision":"pcm16","engine_id":"guarded-observer","engine_revision":"r1","input_conditions":{"source_sha256":media["sha256"]}},"assets":assets,"automated_checks":checks,"measured_resources":{"kind":"read_only_historical_revalidation"}},"criteria":[{"criterion_id":"technical","how_to_review":"Inspect original observations and guarded mapping","expected":"Exact source and derived-byte bindings; not AI quality approval"}],"known_issues":["No playback readiness or actual9 binding; source producer CLI is historical"]}]});
    let body = serde_json::to_vec(&package).unwrap();
    let package_sha = sha(&body);
    write_new(
        &bundle
            .root
            .join("packages")
            .join(format!("{package_sha}.json")),
        &body,
    );
    let case = ReviewCase {
        package_sha256: package_sha,
        item_id: "sprite-latin-17-technical".into(),
        source_request_sha256: source_request["sha256"].as_str().unwrap().into(),
    };
    let pins = Pins {
        audio: refs["audio-engine"]["sha256"].as_str().unwrap().into(),
        probe: refs["video-engine"]["sha256"].as_str().unwrap().into(),
        capture: refs["capture-engine"]["sha256"].as_str().unwrap().into(),
        cli: capture_inputs["cli"]["sha256"].as_str().unwrap().into(),
    };
    assert_eq!(
        pins.cli,
        "8d06f1919d05abedcdaa799c9a7b43fa42cf8f14b5c092fd6eb4d6d058cf99f9"
    );
    bundle.assert_all_unchanged();
    let originals=bundle.seal_file("ORIGINALS.json",&json!({"package_sha256":case.package_sha256,"records":bundle.receipts,"scope":"copied identities separate from historical originals"}));
    let nomination=bundle.seal_file("NOMINATION.json",&json!({"case":case,"media":media,"evidence":manifest,"audio_engine_sha256":pins.audio,"video_engine_sha256":pins.probe,"capture_engine_sha256":pins.capture,"capture_cli_sha256":pins.cli,"originals_sha256":originals,"authority":"explicit local PM nomination of pinned history; not caller input","actual9_binding":false,"quality_approved":false}));
    bundle.seal_file("SEALED.json",&json!({"package_sha256":case.package_sha256,"originals_sha256":originals,"nomination_sha256":nomination,"new_capture_execution":false,"new_decode_execution":false,"actual9_binding":false,"quality_approved":false}));
    Prepared {
        bundle,
        case,
        media: serde_json::from_value(media).unwrap(),
        evidence: serde_json::from_value(manifest).unwrap(),
        pins,
    }
}
