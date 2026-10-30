use super::*;
use crate::infrastructure::timeline_evidence::live_contract::LiveExecutionContract as Contract;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    case: ReviewCase,
    media: MediaReference,
    bytes: u64,
    declared_media_type: String,
    classification: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    affinity_mask: usize,
    arguments: Vec<String>,
    cpu_threads: u8,
    max_output_bytes: u64,
    memory_committed_bytes: u64,
    source: Source,
    source_path: String,
    tool_file_identity: String,
    tool_path: String,
    tool_sha256: String,
    verified_timeline: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Started {
    pid: u32,
    creation_filetime: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Process {
    child_exit: i32,
    creation_filetime: u64,
    family_empty: bool,
    forced: bool,
    pid: u32,
    stdin_eof_requested: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Derived {
    path: String,
    sha256: String,
    samples: u64,
    first_pts: i64,
    end_pts: i128,
    format: String,
    authority_granted: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VideoInput {
    root: String,
    sha256: String,
    time_base: String,
    stream_index: u32,
    authority_granted: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bundle {
    schema_version: String,
    producer: ProducerPin,
    source: Source,
    time_base: String,
    video: serde_json::Value,
    pcm: serde_json::Value,
    origin: serde_json::Value,
    start_capture: serde_json::Value,
    end_capture: serde_json::Value,
    mapping: serde_json::Value,
    content_validated: bool,
    custody_registered: bool,
    authority_granted: bool,
    verified_timeline: bool,
    quality_approved: bool,
}
fn source_matches(source: &Source, token: &PublishedTimelinePackage) -> bool {
    serde_json::to_value(source).ok() == serde_json::to_value(&token.source_identity).ok()
}
pub(super) fn verify(context: &Context<'_>) -> Result<(), ApiError> {
    let c = context;
    let t = c.token;
    let snapshot = &t.snapshot;
    if c.manifest.contracts.len() != 6
        || c.manifest.tools.len() != 3
        || c.manifest.producer.pid == 0
        || c.manifest.producer.file_identity.is_empty()
    {
        return Err(invalid("Incomplete live execution declarations"));
    }
    let mut expected = BTreeSet::new();
    for role in [
        "video",
        "original-pcm",
        "replay-pcm",
        "origin",
        "start",
        "end",
    ] {
        for name in [
            "INPUT.json",
            "STARTED.json",
            "PROCESS.json",
            "RETURN.json",
            "stdout.raw",
            "stderr.raw",
        ] {
            expected.insert((role.to_owned(), name.to_owned()));
        }
        if role == "replay-pcm" {
            for name in ["DERIVED-INPUT.json", "clip.s16le"] {
                expected.insert((role.into(), name.into()));
            }
        }
        if ["origin", "start", "end"].contains(&role) {
            expected.insert((role.into(), "VIDEO-INPUT.json".into()));
        }
    }
    for role in ["probe", "audio-engine", "capture-engine", "producer"] {
        expected.insert((role.into(), "engine.bin".into()));
    }
    expected.insert(("bundle".into(), "BUNDLE.json".into()));
    if expected != c.entries.keys().cloned().collect() {
        return Err(invalid("Missing, duplicate or extra live artifact"));
    }
    for tool in &c.manifest.tools {
        if c.asset(&tool.role, "engine.bin")?.sha256 != tool.sha256 || tool.file_identity.is_empty()
        {
            return Err(conflict("Live engine asset differs"));
        }
    }
    let producer = c.asset("producer", "engine.bin")?;
    if producer.sha256 != t.producer.sha256 || producer.bytes.0 != t.producer.bytes {
        return Err(conflict("Actual Rust producer binary differs"));
    }
    let base = snapshot.time_base;
    let (n, d) = base.ratio();
    let video_base = format!("{n}/{d}");
    let original = c.contract("original-pcm")?;
    let derived = Path::new(&original.root).join("stdout.raw");
    let mut roles = BTreeSet::new();
    for contract in &c.manifest.contracts {
        if !roles.insert(contract.role.clone()) {
            return Err(invalid("Duplicate execution role"));
        }
        let tool_role = match (&*contract.role, &contract.contract) {
            ("video", Contract::VideoFirstStreamWindow { .. }) => "probe",
            ("original-pcm", Contract::AudioSourceCopyTsMono16kS16)
            | ("replay-pcm", Contract::RawPcmReplayMono16kS16) => "audio-engine",
            ("origin", Contract::FirstDecodedOrigin) => "capture-engine",
            ("start", Contract::CaptureAtOrAfter { source_us })
                if *source_us == snapshot.start_capture.requested_source_us =>
            {
                "capture-engine"
            }
            ("end", Contract::CaptureAtOrAfter { source_us })
                if *source_us == snapshot.end_capture.requested_source_us =>
            {
                "capture-engine"
            }
            _ => return Err(invalid("Wrong explicit contract for live role")),
        };
        let input: Input = json(&c.bytes(&contract.role, "INPUT.json", 65536)?)?;
        let tool = c.tool(tool_role)?;
        let expected = contract
            .contract
            .arguments(
                Path::new(&input.source_path),
                Some(&derived),
                base,
                &t.source_identity.classification,
            )?
            .into_iter()
            .map(|arg| {
                arg.into_string()
                    .map_err(|_| invalid("Unicode argv required"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if !source_matches(&input.source, t)
            || input.source_path != t.source_path
            || input.tool_sha256 != tool.sha256
            || input.tool_path != tool.path
            || input.tool_file_identity != tool.file_identity
            || input.arguments != expected
            || input.verified_timeline
            || input.affinity_mask != t.budget.affinity_mask
            || input.memory_committed_bytes != t.budget.memory_committed_bytes
            || input.max_output_bytes != t.budget.max_output_bytes
            || input.cpu_threads != 1
            || input.cpu_threads != t.budget.cpu_threads
        {
            return Err(conflict(
                "Original live argv/source/tool/budget differs from contract",
            ));
        }
        let started: Started = json(&c.bytes(&contract.role, "STARTED.json", 65536)?)?;
        let process: Process = json(&c.bytes(&contract.role, "PROCESS.json", 65536)?)?;
        if process.pid == 0
            || process.creation_filetime == 0
            || started.pid != process.pid
            || started.creation_filetime != process.creation_filetime
            || process.child_exit != 0
            || !process.family_empty
            || process.forced
            || !process.stdin_eof_requested
        {
            return Err(conflict("Original child/Job did not complete normally"));
        }
        c.asset(&contract.role, "RETURN.json")?; // diagnostic only, never authority
        if ["origin", "start", "end"].contains(&contract.role.as_str()) {
            let input: VideoInput = json(&c.bytes(&contract.role, "VIDEO-INPUT.json", 65536)?)?;
            if input.root != c.contract("video")?.root
                || input.sha256 != c.asset("video", "stdout.raw")?.sha256
                || input.time_base != video_base
                || input.stream_index != c.manifest.video_stream_index
                || input.authority_granted
            {
                return Err(conflict("Capture video binding differs"));
            }
        }
    }
    let input: Derived = json(&c.bytes("replay-pcm", "DERIVED-INPUT.json", 65536)?)?;
    let frames = &snapshot.pcm.frames;
    let samples = frames
        .iter()
        .map(|frame| u64::from(frame.samples))
        .sum::<u64>();
    if input.path != derived.to_string_lossy()
        || input.sha256 != snapshot.pcm.original_data_sha256
        || input.samples != samples
        || input.first_pts != frames[0].pts
        || input.end_pts != i128::from(frames[0].pts) + i128::from(samples)
        || input.format != "mono16k_s16le"
        || input.authority_granted
    {
        return Err(conflict("PCM replay input lineage differs"));
    }
    if c.asset("bundle", "BUNDLE.json")?.sha256 != t.bundle_sha256 {
        return Err(conflict("Bundle receipt differs"));
    }
    let bundle: Bundle = json(&c.bytes("bundle", "BUNDLE.json", 64 * 1024 * 1024)?)?;
    let mapping = snapshot
        .validate()
        .map_err(|_| invalid("Live mapping invalid"))?;
    let expected_mapping = serde_json::json!({"source_start_us":mapping.source_start_us,"source_end_us":mapping.source_end_us,"start_frame_pts":mapping.start_frame_pts,"end_frame_pts":mapping.end_frame_pts});
    if bundle.schema_version != "live_observations_2"
        || bundle.producer != t.producer
        || !source_matches(&bundle.source, t)
        || bundle.time_base != video_base
        || bundle.mapping != expected_mapping
        || !bundle.content_validated
        || bundle.custody_registered
        || bundle.authority_granted
        || bundle.verified_timeline
        || bundle.quality_approved
        || bundle.video["frames_sha256"] != c.asset("video", "stdout.raw")?.sha256
        || bundle.video["cadence"] != serde_json::json!(snapshot.video_cadence)
        || bundle.pcm["original_data_sha256"] != snapshot.pcm.original_data_sha256
        || bundle.origin["observation_sha256"] != snapshot.origin.observation_sha256
        || bundle.start_capture["png_sha256"] != snapshot.start_capture.png_sha256
        || bundle.end_capture["png_sha256"] != snapshot.end_capture.png_sha256
    {
        return Err(conflict("Bundle diagnostic/source/producer differs"));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_creation_or_completion_cannot_default_to_success() {
        assert!(json::<Process>(br#"{"pid":1,"child_exit":0,"family_empty":true,"forced":false,"stdin_eof_requested":true}"#).is_err());
        assert!(json::<Started>(br#"{"pid":1}"#).is_err());
        assert!(
            json::<crate::infrastructure::timeline_evidence::live_contract::RoleContract>(
                br#"{"role":"video","root":"private"}"#
            )
            .is_err()
        );
    }
}
