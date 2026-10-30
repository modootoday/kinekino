use kinekino_player::application::api::ApplicationClient;
use kinekino_player::application::api::{ApiError, ErrorCode, PlayerSdk, Request, Response};
use std::io::{Read, Write};

const MAX_INPUT_BYTES: u64 = kinekino_player::application::local_host::MAX_REQUEST_BYTES as u64;
#[path = "runner/adblock.rs"]
pub mod adblock;
#[path = "runner/in_process.rs"]
mod in_process;
#[path = "runner/transport_timeout.rs"]
mod transport_timeout;
pub use in_process::execute_value;

#[derive(serde::Serialize)]
struct HostFailure {
    #[serde(flatten)]
    response: Response,
    #[serde(skip_serializing_if = "Option::is_none")]
    local_host_shutdown: Option<crate::host_shutdown::LocalHostShutdownView>,
}

fn finish_host<D>(outcome: crate::host_shutdown::HostRunOutcome<D>, host: &str) -> u8 {
    match outcome {
        crate::host_shutdown::HostRunOutcome::Complete { result: Ok(()), .. } => 0,
        crate::host_shutdown::HostRunOutcome::Complete {
            result: Err(error),
            local_host_shutdown,
        } => {
            let code = error.exit_code();
            let failure = HostFailure {
                response: Response::failure(host.into(), error),
                local_host_shutdown: local_host_shutdown
                    .map(crate::host_shutdown::LocalHostShutdownView::from),
            };
            match serde_json::to_string(&failure) {
                Ok(json) => {
                    if writeln!(std::io::stdout().lock(), "{json}").is_err() {
                        let _ = writeln!(std::io::stderr().lock(), "{json}");
                    }
                }
                Err(report_error) => {
                    // Keep the original failure object and code; never replace it with a fallback success.
                    report_host_serialization_failure(
                        &mut std::io::stderr().lock(),
                        &report_error,
                        &failure,
                    );
                }
            }
            code
        }
        crate::host_shutdown::HostRunOutcome::DependencyIncomplete(owner) => {
            let code = owner.exit_code();
            let diagnostic = owner.diagnostic(host);
            if writeln!(std::io::stdout().lock(), "{diagnostic}").is_err() {
                let _ = writeln!(std::io::stderr().lock(), "{diagnostic}");
            }
            // Keep original dependency/lease alive at abnormal process exit.
            // This is not cleanup success or SDK cancellation acknowledgment.
            std::process::exit(i32::from(code));
        }
        crate::host_shutdown::HostRunOutcome::Incomplete(owner) => {
            let code = owner.exit_code();
            let diagnostic = owner.diagnostic(host);
            if writeln!(std::io::stdout().lock(), "{diagnostic}").is_err() {
                let _ = writeln!(std::io::stderr().lock(), "{diagnostic}");
            }
            // The original Pending and dependency are alive here. This terminates the
            // process abnormally; it does not run Drop or certify SDK cancellation.
            std::process::exit(i32::from(code));
        }
    }
}

fn report_host_serialization_failure<W: Write>(
    diagnostic: &mut W,
    report_error: &dyn std::fmt::Display,
    failure: &HostFailure,
) {
    let _ = writeln!(
        diagnostic,
        "Host failure serialization failed: {report_error}; original_error={:?}",
        failure.response.error
    );
}

#[cfg(all(test, windows))]
#[path = "runner/isolated_shutdown_tests.rs"]
mod isolated_shutdown_tests;

pub fn run() -> u8 {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--clip-project-root" | "--asset-project-root"))
        && args.iter().any(|arg| arg.starts_with("--serve-"))
        && !args
            .iter()
            .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        let error = ApiError::new(
            ErrorCode::InvalidArgument,
            "Project roots are supported by local commands only",
        );
        let _ = writeln!(
            std::io::stdout().lock(),
            "{}",
            serde_json::to_string(&Response::failure("clip-project-host".into(), error))
                .unwrap_or_default()
        );
        return 2;
    }
    if args.iter().any(|arg| arg == "--serve-execution")
        && !args
            .iter()
            .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        #[cfg(windows)]
        {
            return crate::execution_host::run(&args);
        }
        #[cfg(not(windows))]
        {
            return 3;
        }
    }
    if args.iter().any(|arg| arg == "--serve-review")
        && !args
            .iter()
            .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        #[cfg(windows)]
        {
            return crate::review_host::run(&args);
        }
        #[cfg(not(windows))]
        {
            return 3;
        }
    }
    if args.iter().any(|arg| arg == "--serve-playback")
        && !args
            .iter()
            .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        #[cfg(windows)]
        {
            return finish_host(crate::playback_host::run(&args), "playback-host");
        }
        #[cfg(not(windows))]
        {
            println!(
                "{}",
                serde_json::to_string(&Response::failure(
                    "playback-host".into(),
                    ApiError::new(
                        ErrorCode::Unsupported,
                        "Combined playback host requires Windows"
                    )
                ))
                .unwrap_or_default()
            );
            return 3;
        }
    }
    if args.iter().any(|arg| arg == "--serve-native")
        && !args
            .iter()
            .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        #[cfg(windows)]
        {
            return finish_host(crate::native_host::run(&args), "native-host");
        }
        #[cfg(not(windows))]
        {
            println!(
                "{}",
                serde_json::to_string(&Response::failure(
                    "native-host".into(),
                    ApiError::new(
                        ErrorCode::Unsupported,
                        "Explicit native host requires Windows"
                    )
                ))
                .unwrap_or_default()
            );
            return 3;
        }
    }
    if args.iter().any(|arg| arg == "--serve-provider")
        && !args
            .iter()
            .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        #[cfg(windows)]
        {
            return finish_host(crate::provider_host::run(&args), "provider-host");
        }
        #[cfg(not(windows))]
        {
            println!(
                "{}",
                serde_json::to_string(&Response::failure(
                    "provider-host".into(),
                    ApiError::new(ErrorCode::Unsupported, "Browser provider requires Windows")
                ))
                .unwrap_or_default()
            );
            return 3;
        }
    }
    if args.iter().any(|arg| arg == "--serve-jobs")
        && !args
            .iter()
            .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        return finish_host(crate::job_host::run(&args), "job-host");
    }
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "Asset build jobs: --serve-jobs --job-root EXISTING_ABSOLUTE_DIRECTORY --asset-build-config ABSOLUTE_FILE --asset-build-config-sha256 SHA256; pinned asset-build-host/1 with current producer SHA, project/artifact roots and fixed PNG/mask catalogs. Submit production.build_assets through jobs.submit using the returned endpoint; direct production command remains unsupported. Review artifacts only, no inference/video admission/user quality approval."
        );
        println!(
            "Asset projects: asset_projects.create/inspect/save_revision require --asset-project-root EXISTING_ABSOLUTE_DIRECTORY; local storage only, no source/review admission, rendering, timeout/idempotency or quality approval.\nClip projects: clip_projects.create/inspect/save_revision require --clip-project-root EXISTING_ABSOLUTE_DIRECTORY and strict Request JSON. Save requires expected_revision of the publication head. Local commands only; do not combine with --endpoint or --serve-* hosts. No media provenance verification, rendering, timeout/idempotency, automatic retry or quality approval."
        );
        println!(
            "Manual speakers: subtitles.speakers reads speaker-annotations/1 JSON; subtitles.edit_speakers requires input SHA, expected_revision, new output and explicit add/rename/merge/split/assign_cues/delete/undo/redo edits. Original spans/history are preserved. No overwrite, diarization, media binding, global latest CAS or quality approval. Use --input and inspect schema."
        );
        println!(
            "Models: --model-root ABS_DIR enables read-only models.list/inspect/verify. Inspect requires exact revision; verify additionally pins catalog_sha256 and maximum_bytes with optional timeout_ms (1..120000, default30000). Hash matching is point-in-time file integrity, never engine compatibility, license permission, quality approval or persisted admission. install/remove remain unavailable. No downloads or model execution."
        );
        println!(
            "Review host: --serve-review --review-host-config ABS_JSON --review-host-config-sha256 SHA256. Explicit local producer inputs/tools/limits; creates fresh source-bound evidence before ready. No imported authority or automatic restart resume. stdin EOF closes original transport and media owners. GUI/model/network are not started."
        );
        println!(
            "Combined playback: --serve-playback --job-root ABS_DIR --catalog-root ABS_DIR --browser-profile-root ABS_DIR --mpv-executable ABS_MPV_EXE --mpv-sha256 SHA256. One owned browser jobs/native host, stdin EOF shutdown. playback.previous/next submit a verified-list adjacent resolution; observe then explicitly commit the owned transition. Resolve completion and process creation do not mean media playback. No automatic retry or host restart resumption."
        );
        println!(
            "Native playback: --serve-native --catalog-root ABS_DIR --mpv-executable ABS_MPV_EXE --mpv-sha256 SHA256 creates an explicit owned host and stops on stdin EOF. playback.open accepts a stored resolution artifact ID, paused and volume_percent. Process start is not media load/decode. playback.engine reports instance-bound observations. Controls and playback.close require engine_instance_id. No automatic host startup, source reconstruction, command retry or instance replacement."
        );
        println!(
            "{} CLI\nUsage: kinekino <version|capabilities|schema COMMAND|GROUP ACTION> [--json|--human] [--request-id ID] [--timeout-ms MS] [--expected-revision N] [--idempotency-key KEY] [--endpoint ENDPOINT]\n       kinekino --input FILE|- [--json|--human] [--endpoint ENDPOINT]\n--input reads a full Request JSON with context, command and payload. Do not combine it with a positional command or context flags.\nDiscovery, subtitles.convert (UTF-8 SRT/WebVTT via full Request JSON) and subtitles.from_transcript (ASR transcript JSON chunks to SRT/WebVTT/styled ASS) are supported. production.capture_frame accepts local MP4/MKV/WebM and writes PNG when FFmpeg/FFprobe are available. production.export_snapshot accepts a fixed-SHA local PNG and explicit pixel crop, writes a new PNG without FFmpeg, and does not preserve ancillary metadata. Query capabilities for current dependency availability. --endpoint forwards one request to an explicit current-user local host (Windows named pipe or Unix domain socket under XDG_RUNTIME_DIR), with a 5-second transport deadline and no automatic retry or fallback. Without --endpoint the CLI uses its in-process SDK. Host capabilities depend on the SDK adapters actually injected. Transport timeout or disconnect after transmission means execution may have occurred; accepted/unobserved control receipts do not mean mpv applied the command.",
            kinekino_player::domain::product::PRODUCT.display_name
        );
        println!(
            "Endpoint transport: --transport-timeout-ms 10..30000 explicitly overrides the default 5000 ms deadline and requires --endpoint. It also works with --input and does not change Request.context.timeout_ms. Expiry after transmission remains execution_unknown; no automatic retry."
        );
        println!(
            "Review commands reviews.inspect/reviews.record_decision/reviews.decisions/reviews.read_text_asset require --review-root ABSOLUTE_DIRECTORY with prepared packages/objects/decisions directories. Text reads require package-referenced strict UTF-8 text/JSON at most 64 KiB. Association inspect/create/revise and range_draft read/save additionally require --user-review-root EXISTING_ABSOLUTE_DIRECTORY disjoint from the evidence root. Use full Request JSON and query schema. Read commands open the user root readonly; other commands enable its user transaction namespace. Only package-custodied declared media is admitted; decode/PTS/readiness are not verified. Mutations require an idempotency key; revise requires expected_revision. The default review backend is unavailable. Do not combine local roots with --endpoint. Decisions never enable automatic application or approve an entire model."
        );
        println!(
            "Proposal commands proposals.create/inspect/edit/accept require --proposal-root ABSOLUTE_DIRECTORY (existing empty local directory initially). Typed output revisions preserve source and generated originals; rendering/export is separate. proposals.export_subtitles explicitly consumes a fixed accepted OutputReviewTarget and writes a new SRT/VTT file; inspect schema, review losses and explicitly acknowledge allow_loss. It does not infer cue timing, Accept, overwrite or apply a project. Malformed imports remain needs_repair. Edit/accept require expected_revision; acceptance is explicit and independent of model quality reviews. Do not combine local roots with --endpoint."
        );
        println!(
            "Project commands projects.create/inspect/save_revision require --project-root ABSOLUTE_DIRECTORY. Live proposal captures also require --proposal-root; saved captures can be reopened and saved without it. Nonempty source/work context is mandatory. Save requires expected_revision and does not automatically retry conflicts. Stored locators never open media or URLs. Project-current acceptance and media binding remain unverified. Do not combine local roots with --endpoint."
        );
        println!(
            "Capture edits: proposals.edit_capture / proposals.accept_capture require both --project-root and --proposal-root. Payload anchor selects an immutable project revision/hash and proposal capture/hash. Omit expected for the original capture, then explicitly reuse returned continuation for later mutations; it is a CAS expectation, not an authority token. Exact accept retry repeats the original request. Legacy captures without origin stay read-only. Context expected_revision/idempotency_key/timeout_ms are unsupported. No automatic refresh, project save or media application."
        );
        println!(
            "Card projects: card_projects.create/inspect/save_revision require --card-project-root EXISTING_ABSOLUTE_DIRECTORY and full Request JSON. Save requires expected_revision of the publication head; edit revision is separate. No timeout/idempotency, automatic retry, media opening or quality approval. Do not combine a local root with --endpoint. A card-review-export build can explicitly start --serve-jobs --job-root ABS_DIR --card-project-root ABS_DIR to add production.export_cards to jobs.submit alongside subtitle conversion/range export. This is unapproved review-artifact output. The direct production.export_cards RPC remains unsupported; use durable Jobs. Combined VAD/card host configuration is currently rejected."
        );
        println!(
            "Accepted output review: proposals.inspect_output_review_target / proposals.record_output_decision / proposals.output_decisions use --proposal-root and the common SDK. Only record requires expected_revision=current output revision; decision_id is the exact-request retry identity. Transport idempotency_key/timeout_ms are unsupported. Bound inspect/record require the explicit historical verifier; history reads stored records with current_evidence=not_checked. Technical acceptance, declared review verdict, local record time and actual media application are distinct. No automatic application or model-sample success is granted."
        );
        println!(
            "Historical imports use proposals.import_historical with explicit proposal/source IDs, source_revision and companion reference. Configure --proposal-root plus --historical-verifier ABS_EXE --historical-verifier-sha256 SHA256 --historical-workspace ABS_DIR --historical-package LABEL. No verifier is discovered automatically. Bound proposal/project reads, saved captures and acceptance retries require this configuration and reverify original evidence. Verification is synchronous and does not approve model quality or apply output."
        );
        println!(
            "Jobs: --job-root ABSOLUTE_DIRECTORY provides get/list/watch only. Start an explicit worker with kinekino --serve-jobs --job-root ABSOLUTE_DIRECTORY; it prints a current-user endpoint and stops on stdin EOF. Use --endpoint for jobs.submit/cancel/retry. Submit requires idempotency_key; the default host supports subtitles.convert, subtitles.export_range and subtitles.from_transcript. Product VAD opt-in adds paired --vad-root ABS_RELEASE_BUNDLE --vad-workspace ABS_DATA_ROOT and requires an independently configured Rust product release/native deployment; absent pins return NeedsInput before ready. Workspace audio/audio-ID.wav must be strict PCM16 mono16k WAV, destination IDs map under outputs, and source SHA is the original WAV SHA. No caller flag grants model rights. Source snapshots, no-overwrite output, worker cancel acknowledgement and restart Interrupted are shared Rust policies. watch returns one bounded durable event page, not an implicit stream. retry requires expected_revision; cancel requires payload.expected_attempt. No automatic host startup, restart execution or frame jobs."
        );
        println!(
            "Experimental catalog reads require --catalog-root ABSOLUTE_DIRECTORY. catalog.search accepts category (airing/completed/movies) and page only; episodes.list accepts series_url. Actual site selectors remain unverified. Complete HTTP bodies are captured before parsing; metadata contains relative artifact references, UTC capture time and SHA. Text search/genres/sort/media resolution are unsupported. No automatic retry. Do not combine local roots with --endpoint."
        );
        println!(
            "Pinned frame tools (Windows local commands or --serve-jobs): provide --frame-ffmpeg ABS_EXE --frame-ffmpeg-sha256 SHA256 --frame-ffprobe ABS_EXE --frame-ffprobe-sha256 SHA256 together. Verified files remain guarded while the adapter lives; a rejected pin never falls back to PATH. Decoder feature/version checks still apply. Job host pin checks precede ready and job store startup; capture is a direct SDK command, not a job executor. Do not combine local flags with --endpoint. Without these options local commands retain automatic discovery and job hosts retain no frame adapter. Other serve hosts and GUI are not configured by these flags; visual/color equivalence remains unapproved."
        );
        println!(
            "Experimental browser jobs: --serve-provider --job-root ABS_DIR --catalog-root ABS_DIR --browser-profile-root ABS_DIR. Submit catalog.search or episodes.list through jobs.submit at its endpoint; read completed SHA with the same catalog command and result_artifact_sha256. DOM has null HTTP status. First use requires an empty dedicated profile. Roots must be separate. Stops on stdin EOF; popup/resolve/media/native support is separate."
        );
        println!(
            "Explicit execution host: --serve-execution requires complete pinned --frame-ffmpeg/--frame-ffprobe paths and SHA256 flags. Ready lists private execution RPCs; use full Request JSON through --endpoint. Submit only admits an operation. Observe its original result and explicitly acknowledge before stdin EOF. Unresolved work or unacknowledged results make shutdown incomplete; no automatic retry or result ACK."
        );
        return 0;
    }
    let human = args.iter().any(|arg| arg == "--human");
    let mut command = String::new();
    let response = match parse(&args) {
        Ok(request) => {
            command.clone_from(&request.command);
            if let Some(endpoint) = args
                .windows(2)
                .find(|pair| pair[0] == "--endpoint")
                .map(|pair| pair[1].as_str())
            {
                match transport_timeout::from_args(&args) {
                    Ok(timeout) => execute_endpoint(endpoint, request, timeout),
                    Err(error) => Response::failure(request.context.request_id, error),
                }
            } else {
                execute_local(&args, request)
            }
        }
        Err(error) => Response::failure(error.request_id.clone(), error),
    };
    if kinekino_player::application::skip_segments::host::is_export_command(&command) {
        return write_export_response(&mut std::io::stdout().lock(), &command, &response, human);
    }
    let code = response.error.as_ref().map_or(0, ApiError::exit_code);
    let output = if human {
        serde_json::to_string_pretty(&response)
    } else {
        serde_json::to_string(&response)
    };
    match output {
        Ok(output) if writeln!(std::io::stdout().lock(), "{output}").is_ok() => code,
        _ => 1,
    }
}

fn write_export_response<W: Write>(
    writer: &mut W,
    command: &str,
    response: &Response,
    human: bool,
) -> u8 {
    use kinekino_player::application::local_host::{ResponseFormat, encode_command_response};
    let format = if human {
        ResponseFormat::Pretty
    } else {
        ResponseFormat::Compact
    };
    match encode_command_response(command, response, format) {
        Ok(encoded) => {
            if writer
                .write_all(&encoded.json)
                .and_then(|_| writer.write_all(b"\n"))
                .is_ok()
            {
                encoded.exit_code
            } else {
                1
            }
        }
        Err(_) => 1,
    }
}

fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}

fn parse(args: &[String]) -> Result<Request, ApiError> {
    let mut words = Vec::new();
    let mut options = std::collections::BTreeMap::new();
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if matches!(arg.as_str(), "--json" | "--human") {
            if options.insert(arg.as_str(), "").is_some() {
                return Err(invalid("Duplicate output option"));
            }
        } else if arg.starts_with('-') {
            if !matches!(
                arg.as_str(),
                "--input"
                    | "--frame-ffmpeg"
                    | "--frame-ffmpeg-sha256"
                    | "--frame-ffprobe"
                    | "--frame-ffprobe-sha256"
                    | "--request-id"
                    | "--timeout-ms"
                    | "--expected-revision"
                    | "--idempotency-key"
                    | "--endpoint"
                    | "--transport-timeout-ms"
                    | "--review-root"
                    | "--user-review-root"
                    | "--review-media-config"
                    | "--review-media-config-sha256"
                    | "--proposal-root"
                    | "--historical-verifier"
                    | "--historical-verifier-sha256"
                    | "--historical-workspace"
                    | "--historical-package"
                    | "--project-root"
                    | "--card-project-root"
                    | "--clip-project-root"
                    | "--asset-project-root"
                    | "--job-root"
                    | "--catalog-root"
                    | "--model-root"
            ) {
                return Err(invalid("Unknown option"));
            }
            index += 1;
            let value = args
                .get(index)
                .filter(|v| !v.starts_with("--"))
                .ok_or_else(|| invalid("Option requires a value"))?;
            if options.insert(arg.as_str(), value.as_str()).is_some() {
                return Err(invalid("Duplicate option"));
            }
        } else {
            words.push(arg.as_str());
        }
        index += 1;
    }
    if options.contains_key("--human") && options.contains_key("--json") {
        return Err(invalid("Conflicting output options"));
    }
    transport_timeout::from_args(args)?;
    let mut request = if let Some(path) = options.get("--input") {
        if options.keys().any(|key| {
            !matches!(
                *key,
                "--input"
                    | "--frame-ffmpeg"
                    | "--frame-ffmpeg-sha256"
                    | "--frame-ffprobe"
                    | "--frame-ffprobe-sha256"
                    | "--human"
                    | "--json"
                    | "--endpoint"
                    | "--transport-timeout-ms"
                    | "--review-root"
                    | "--user-review-root"
                    | "--review-media-config"
                    | "--review-media-config-sha256"
                    | "--proposal-root"
                    | "--historical-verifier"
                    | "--historical-verifier-sha256"
                    | "--historical-workspace"
                    | "--historical-package"
                    | "--project-root"
                    | "--card-project-root"
                    | "--clip-project-root"
                    | "--asset-project-root"
                    | "--job-root"
                    | "--catalog-root"
                    | "--model-root"
            )
        }) {
            return Err(invalid("JSON context and context flags cannot be combined"));
        }
        read_request(path)?
    } else {
        let mut request = Request::new("");
        if let Some(id) = options.get("--request-id") {
            request.context.request_id = (*id).into();
        }
        if let Some(key) = options.get("--idempotency-key") {
            request.context.idempotency_key = Some((*key).into());
        }
        if let Some(revision) = options.get("--expected-revision") {
            request.context.expected_revision = Some((*revision).into());
        }
        if let Some(timeout) = options.get("--timeout-ms") {
            request.context.timeout_ms =
                Some(timeout.parse().map_err(|_| invalid("Invalid timeout"))?);
        }
        request
    };
    if !words.is_empty() {
        if !request.command.is_empty() {
            return Err(invalid("Command duplicated in JSON and arguments"));
        }
        request.command = match words.as_slice() {
            ["schema", command] => {
                request.payload = serde_json::json!({"command":command.replace(' ',".")});
                "schema".into()
            }
            ["schema", group, action] => {
                request.payload = serde_json::json!({"command":format!("{group}.{action}")});
                "schema".into()
            }
            [command] => (*command).into(),
            [group, action] => format!("{group}.{action}"),
            _ => return Err(invalid("Invalid command arguments")),
        };
    }
    request.validate()?;
    crate::frame_tools::options(args)?;
    kinekino_player::application::launch::historical_verifier(args)
        .map_err(|_| invalid("Invalid historical verifier configuration"))?;
    if options.contains_key("--endpoint")
        && (options.contains_key("--review-root")
            || options.contains_key("--frame-ffmpeg")
            || options.contains_key("--user-review-root")
            || options.contains_key("--proposal-root")
            || options.contains_key("--project-root")
            || options.contains_key("--card-project-root")
            || options.contains_key("--clip-project-root")
            || options.contains_key("--asset-project-root")
            || options.contains_key("--job-root")
            || options.contains_key("--catalog-root")
            || options.contains_key("--model-root"))
    {
        let mut error = invalid("Local review root and remote endpoint cannot be combined");
        error.request_id = request.context.request_id;
        return Err(error);
    }
    if options.contains_key("--user-review-root") && !options.contains_key("--review-root") {
        return Err(invalid("--user-review-root requires --review-root"));
    }
    if options.contains_key("--review-media-config")
        != options.contains_key("--review-media-config-sha256")
        || options.contains_key("--review-media-config")
            && (!options.contains_key("--review-root")
                || !options.contains_key("--user-review-root")
                || options.contains_key("--endpoint"))
    {
        return Err(invalid(
            "Pinned media configuration requires both settings flags and local review/user roots",
        ));
    }
    Ok(request)
}
fn execute_local(args: &[String], request: Request) -> Response {
    if request.command == kinekino_player::application::proposals::card_export::COMMAND
        || request.command == "schema"
            && request.payload["command"]
                == kinekino_player::application::proposals::card_export::COMMAND
    {
        #[cfg(windows)]
        {
            return execute_card_review(args, request);
        }
        #[cfg(not(windows))]
        {
            return PlayerSdk::default().execute(request);
        }
    }
    // Pure PNG operations must not discover or start a media decoder.
    if request.command == "production.export_snapshot"
        || request.command == "schema" && request.payload["command"] == "production.export_snapshot"
    {
        let client = PlayerSdk::default();
        let client = client.with_snapshot_export(
            kinekino_player::infrastructure::snapshot_export::PngSnapshotExport,
        );
        return client.execute(request);
    }
    let historical = match kinekino_player::application::launch::historical_verifier(args) {
        Ok(config) => config,
        Err(_) => {
            return Response::failure(
                request.context.request_id,
                invalid("Invalid historical verifier configuration"),
            );
        }
    };
    let frame = match crate::frame_tools::adapter(args) {
        Ok(frame) => frame,
        Err(error) => return Response::failure(request.context.request_id, error),
    };
    let client = PlayerSdk::new(kinekino_player::infrastructure::subtitle_files::SubtitleFiles)
        .with_frame_capture(frame);
    #[cfg(windows)]
    let client = client.with_speaker_documents(
        kinekino_player::infrastructure::speaker_annotation_files::LocalSpeakerFiles,
    );
    #[cfg(windows)]
    let client = if let Some(root) = args
        .windows(2)
        .find(|pair| pair[0] == "--model-root")
        .map(|pair| pair[1].as_str())
    {
        match kinekino_player::infrastructure::model_catalog::LocalModelCatalog::open(
            std::path::Path::new(root),
        ) {
            Ok(catalog) => client.with_model_catalog(catalog),
            Err(error) => return Response::failure(request.context.request_id, error),
        }
    } else {
        client
    };
    #[cfg(not(windows))]
    if args.iter().any(|arg| arg == "--model-root") {
        return Response::failure(
            request.context.request_id,
            ApiError::new(
                ErrorCode::Unsupported,
                "Local model catalog requires Windows path guards",
            ),
        );
    }
    #[cfg(all(windows, feature = "card-review-export"))]
    let client = client
        .with_card_image_export(kinekino_player::infrastructure::card_export::PngCardReviewExport);
    let client = client
        .with_snapshot_export(kinekino_player::infrastructure::snapshot_export::PngSnapshotExport);
    #[cfg(windows)]
    let client = if let Some(root) = args
        .windows(2)
        .find(|pair| pair[0] == "--catalog-root")
        .map(|pair| pair[1].as_str())
    {
        if matches!(
            request.command.as_str(),
            "catalog.read_raw" | "episodes.inspect_resolution"
        ) || request.payload.get("result_artifact_sha256").is_some()
        {
            match kinekino_player::infrastructure::catalog::archive(std::path::Path::new(root)) {
                Ok(catalog) => client.with_catalog(catalog),
                Err(error) => return Response::failure(request.context.request_id, error),
            }
        } else {
            match kinekino_player::infrastructure::catalog::open(std::path::Path::new(root)) {
                Ok(catalog) => client.with_catalog(catalog),
                Err(error) => return Response::failure(request.context.request_id, error),
            }
        }
    } else {
        client
    };
    #[cfg(not(windows))]
    if args.iter().any(|arg| arg == "--catalog-root") {
        return Response::failure(
            request.context.request_id,
            ApiError::new(
                ErrorCode::Unsupported,
                "Catalog artifacts require the Windows path guard adapter",
            ),
        );
    }
    let client = if let Some(root) = args
        .windows(2)
        .find(|pair| pair[0] == "--job-root")
        .map(|pair| pair[1].as_str())
    {
        match kinekino_player::infrastructure::jobs::read_only(std::path::Path::new(root)) {
            Ok(jobs) => client.with_jobs(jobs),
            Err(error) => return Response::failure(request.context.request_id, error),
        }
    } else {
        client
    };
    #[cfg(windows)]
    let client = if let Some(root) = args
        .windows(2)
        .find(|p| p[0] == "--proposal-root")
        .map(|p| p[1].as_str())
    {
        match kinekino_player::infrastructure::proposals::open_configured(
            std::path::Path::new(root),
            historical.as_ref(),
        ) {
            Ok(store) => client.with_proposal_store(store),
            Err(error) => return Response::failure(request.context.request_id, error),
        }
    } else {
        client
    };
    #[cfg(not(windows))]
    if args.iter().any(|a| a == "--proposal-root") {
        return Response::failure(
            request.context.request_id,
            ApiError::new(
                ErrorCode::Unsupported,
                "Rooted proposal storage requires Windows",
            ),
        );
    }
    #[cfg(windows)]
    let client = if let Some(root) = args
        .windows(2)
        .find(|p| p[0] == "--project-root")
        .map(|p| p[1].as_str())
    {
        match kinekino_player::infrastructure::projects::ProjectFiles::open(std::path::Path::new(
            root,
        )) {
            Ok(store) => client.with_project_store(store),
            Err(error) => return Response::failure(request.context.request_id, error),
        }
    } else {
        client
    };
    #[cfg(windows)]
    let client = if let Some(root) = args
        .windows(2)
        .find(|p| p[0] == "--card-project-root")
        .map(|p| p[1].as_str())
    {
        match kinekino_player::infrastructure::card_projects::CardProjectFiles::open(
            std::path::Path::new(root),
        ) {
            Ok(store) => client.with_card_project_store(store),
            Err(error) => return Response::failure(request.context.request_id, error),
        }
    } else {
        client
    };
    #[cfg(not(windows))]
    if args.iter().any(|a| a == "--card-project-root") {
        return Response::failure(
            request.context.request_id,
            ApiError::new(
                ErrorCode::Unsupported,
                "Rooted card project storage requires Windows",
            ),
        );
    }
    #[cfg(windows)]
    let client = if let Some(root) = args
        .windows(2)
        .find(|pair| pair[0] == "--asset-project-root")
        .map(|pair| pair[1].as_str())
    {
        match kinekino_player::infrastructure::asset_projects::AssetProjectFiles::open(
            std::path::Path::new(root),
        ) {
            Ok(store) => client.with_asset_project_store(store),
            Err(error) => return Response::failure(request.context.request_id, error),
        }
    } else {
        client
    };
    #[cfg(not(windows))]
    if args.iter().any(|arg| arg == "--asset-project-root") {
        return Response::failure(
            request.context.request_id,
            ApiError::new(
                ErrorCode::Unsupported,
                "Rooted asset project storage requires Windows",
            ),
        );
    }
    #[cfg(windows)]
    let client = if let Some(root) = args
        .windows(2)
        .find(|pair| pair[0] == "--clip-project-root")
        .map(|pair| pair[1].as_str())
    {
        match kinekino_player::infrastructure::clip_projects::ClipProjectFiles::open(
            std::path::Path::new(root),
        ) {
            Ok(store) => client.with_clip_project_store(store),
            Err(error) => return Response::failure(request.context.request_id, error),
        }
    } else {
        client
    };
    #[cfg(not(windows))]
    if args.iter().any(|arg| arg == "--clip-project-root") {
        return Response::failure(
            request.context.request_id,
            ApiError::new(
                ErrorCode::Unsupported,
                "Rooted clip project storage requires Windows",
            ),
        );
    }
    #[cfg(not(windows))]
    if args.iter().any(|a| a == "--project-root") {
        return Response::failure(
            request.context.request_id,
            ApiError::new(
                ErrorCode::Unsupported,
                "Rooted project storage requires Windows",
            ),
        );
    }
    if let Some(root) = args
        .windows(2)
        .find(|pair| pair[0] == "--review-root")
        .map(|pair| pair[1].as_str())
    {
        #[cfg(windows)]
        {
            match kinekino_player::infrastructure::model_review::ReviewFiles::open(
                std::path::Path::new(root),
            ) {
                Ok(store) => {
                    let client = client.with_review_store(store);
                    if let Some(user_root) = args
                        .windows(2)
                        .find(|pair| pair[0] == "--user-review-root")
                        .map(|pair| pair[1].as_str())
                    {
                        let writable = !kinekino_player::application::review_source::api::is_read(
                            &request.command,
                        ) && request.command
                            != kinekino_player::application::review_source::media_api::INSPECT;
                        match kinekino_player::infrastructure::review_source::open_api(
                            std::path::Path::new(root),
                            std::path::Path::new(user_root),
                            writable,
                        ) {
                            Ok(api) => {
                                let api = if let Some(path) = args
                                    .windows(2)
                                    .find(|pair| pair[0] == "--review-media-config")
                                    .map(|pair| pair[1].as_str())
                                {
                                    let sha = args
                                        .windows(2)
                                        .find(|pair| pair[0] == "--review-media-config-sha256")
                                        .map(|pair| pair[1].as_str())
                                        .unwrap_or("");
                                    match kinekino_player::infrastructure::review_source::media_preparation::MediaPreparationFiles::from_pinned_settings(std::path::PathBuf::from(root), std::path::Path::new(path), sha, writable) {
                                        Ok(port) => api.with_media_preparation(port),
                                        Err(error) => return Response::failure(request.context.request_id, error),
                                    }
                                } else {
                                    api
                                };
                                client.with_review_source(api).execute(request)
                            }
                            Err(error) => Response::failure(request.context.request_id, error),
                        }
                    } else {
                        client.execute(request)
                    }
                }
                Err(error) => Response::failure(request.context.request_id, error),
            }
        }
        #[cfg(not(windows))]
        {
            let _ = root;
            Response::failure(
                request.context.request_id,
                ApiError::new(
                    ErrorCode::Unsupported,
                    "Rooted review storage requires the Windows adapter",
                ),
            )
        }
    } else {
        client.execute(request)
    }
}
#[cfg(windows)]
fn execute_card_review(args: &[String], request: Request) -> Response {
    let historical = match kinekino_player::application::launch::historical_verifier(args) {
        Ok(config) => config,
        Err(_) => {
            return Response::failure(
                request.context.request_id,
                invalid("Invalid historical verifier configuration"),
            );
        }
    };
    let client = PlayerSdk::default();
    #[cfg(feature = "card-review-export")]
    let client = client
        .with_card_image_export(kinekino_player::infrastructure::card_export::PngCardReviewExport);
    let client = if let Some(root) = args
        .windows(2)
        .find(|p| p[0] == "--proposal-root")
        .map(|p| p[1].as_str())
    {
        match kinekino_player::infrastructure::proposals::open_configured(
            std::path::Path::new(root),
            historical.as_ref(),
        ) {
            Ok(store) => client.with_proposal_store(store),
            Err(error) => return Response::failure(request.context.request_id, error),
        }
    } else {
        client
    };
    client.execute(request)
}

fn execute_endpoint(endpoint: &str, request: Request, timeout: std::time::Duration) -> Response {
    use kinekino_player::infrastructure::local_host::{EndpointClient, TransportOptions};
    match EndpointClient::new(endpoint, TransportOptions { timeout }) {
        Ok(client) => client.execute(request),
        Err(error) => Response::failure(request.context.request_id, error),
    }
}

fn read_request(path: &str) -> Result<Request, ApiError> {
    let reader: Box<dyn Read> = if path == "-" {
        Box::new(std::io::stdin())
    } else {
        Box::new(std::fs::File::open(path).map_err(|_| invalid("Cannot open input file"))?)
    };
    let mut bytes = Vec::new();
    reader
        .take(MAX_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| invalid("Cannot read input"))?;
    if bytes.len() as u64 > MAX_INPUT_BYTES {
        return Err(invalid("Input exceeds 1 MiB limit"));
    }
    serde_json::from_slice(&bytes).map_err(|_| invalid("Invalid request JSON or unknown field"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| (*v).into()).collect()
    }
    #[test]
    fn normalizes_subcommands_and_schema() {
        assert_eq!(
            parse(&args(&["playback", "status"])).unwrap().command,
            "playback.status"
        );
        assert_eq!(
            parse(&args(&["schema", "playback", "status"]))
                .unwrap()
                .payload["command"],
            "playback.status"
        );
    }
    #[test]
    fn user_review_root_requires_local_evidence_and_rejects_remote_or_duplicate_roots() {
        let local = [
            "capabilities",
            "--review-root",
            "C:/review",
            "--user-review-root",
            "C:/user-review",
        ];
        assert_eq!(parse(&args(&local)).unwrap().command, "capabilities");
        assert!(
            parse(&args(&[
                "capabilities",
                "--user-review-root",
                "C:/user-review"
            ]))
            .is_err()
        );
        let mut remote = args(&local);
        remote.extend(args(&["--endpoint", "explicit-host"]));
        assert!(parse(&remote).is_err());
        let mut duplicate = args(&local);
        duplicate.extend(args(&["--user-review-root", "C:/other"]));
        assert!(parse(&duplicate).is_err());
    }
    #[test]
    fn rejects_unknown_duplicate_and_conflicting_flags() {
        for values in [
            vec!["version", "--unsafe"],
            vec!["version", "--json", "--human"],
            vec!["version", "--request-id", "a", "--request-id", "b"],
            vec!["version", "--timeout-ms", "0"],
        ] {
            assert!(parse(&args(&values)).is_err());
        }
    }
    #[test]
    fn historical_configuration_is_not_a_request_context_or_remote_option() {
        assert!(parse(&args(&["capabilities", "--historical-package", "test"])).is_err());
        let root = std::env::temp_dir();
        let root = root.to_str().unwrap();
        let pin = "a".repeat(64);
        let mut local = args(&[
            "capabilities",
            "--proposal-root",
            root,
            "--historical-verifier",
            root,
            "--historical-verifier-sha256",
            &pin,
            "--historical-workspace",
            root,
            "--historical-package",
            "test",
        ]);
        assert_eq!(parse(&local).unwrap().command, "capabilities");
        local.extend(args(&["--endpoint", "explicit-endpoint"]));
        assert!(parse(&local).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn host_failure_wrapper_preserves_response_and_adds_only_original_transport_view() {
        use kinekino_player::infrastructure::local_host::{LocalHostShutdownStatus, WorkerJoin};
        let mut error = ApiError::new(ErrorCode::PermissionDenied, "original ready failure");
        error.details = serde_json::json!({"original":true});
        let response = Response::failure("test-host".into(), error);
        let original = serde_json::to_value(&response).unwrap();
        let wrapper = HostFailure {
            response,
            local_host_shutdown: Some(crate::host_shutdown::LocalHostShutdownView::from(
                LocalHostShutdownStatus {
                    pipe: WorkerJoin::Joined,
                    sdk: WorkerJoin::Joined,
                    dispatch: None,
                },
            )),
        };
        let mut wire = serde_json::to_value(wrapper).unwrap();
        let joined = wire
            .as_object_mut()
            .unwrap()
            .remove("local_host_shutdown")
            .unwrap();
        assert_eq!(wire, original);
        assert_eq!(
            joined,
            serde_json::json!({"pipe":"joined","sdk":"joined","dispatch":null,"dispatch_poisoned":true})
        );
        let setup = HostFailure {
            response: Response::failure(
                "test-host".into(),
                ApiError::new(ErrorCode::InvalidArgument, "setup"),
            ),
            local_host_shutdown: None,
        };
        assert!(
            serde_json::to_value(setup)
                .unwrap()
                .get("local_host_shutdown")
                .is_none()
        );
    }

    #[cfg(windows)]
    #[test]
    fn broken_serialization_diagnostic_preserves_original_failure_and_exit_code() {
        struct Broken {
            attempts: usize,
            offered: Vec<u8>,
        }
        impl Write for Broken {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.attempts += 1;
                self.offered.extend_from_slice(bytes);
                Err(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "fixture stderr failure",
                ))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut error = ApiError::new(ErrorCode::PermissionDenied, "original failure");
        error.retryable = true;
        error.details = serde_json::json!({"original":true});
        error.remediation = "keep original remediation".into();
        let failure = HostFailure {
            response: Response::failure("original-host".into(), error),
            local_host_shutdown: None,
        };
        let expected = serde_json::to_value(&failure.response.error).unwrap();
        let exit_code = failure.response.error.as_ref().unwrap().exit_code();
        let mut broken = Broken {
            attempts: 0,
            offered: Vec::new(),
        };
        report_host_serialization_failure(&mut broken, &"serialization fixture", &failure);
        assert!(broken.attempts > 0);
        assert_eq!(
            serde_json::to_value(&failure.response.error).unwrap(),
            expected
        );
        assert_eq!(
            failure.response.error.as_ref().unwrap().exit_code(),
            exit_code
        );
        assert_eq!(exit_code, 6);
        let offered = String::from_utf8(broken.offered).unwrap();
        assert!(!offered.contains("stopped"));
        assert!(!offered.contains("host_lifetime_ended"));
    }
}

#[cfg(test)]
mod export_output_tests {
    use super::*;
    use kinekino_player::application::{local_host, skip_segments::host};
    #[test]
    fn compact_and_pretty_fallback_use_reported_failure_exit_and_exactly_one_newline() {
        let response = Response {
            api_version: "1",
            request_id: "😀".repeat(128),
            ok: true,
            data: Some(
                serde_json::json!({"large":"x".repeat(local_host::response_limit(host::EXPORT_LIST))}),
            ),
            error: None,
        };
        for human in [false, true] {
            let mut output = Vec::new();
            let code = write_export_response(&mut output, host::EXPORT_LIST, &response, human);
            assert_eq!(output.last(), Some(&b'\n'));
            assert!(
                output.len() + 4
                    <= kinekino_player::application::playback_session::v1::NATIVE_EXPORT_WIRE_BYTES
            );
            let value: serde_json::Value =
                serde_json::from_slice(&output[..output.len() - 1]).unwrap();
            let error: ApiError = serde_json::from_value(value["error"].clone()).unwrap();
            assert_eq!(error.code, ErrorCode::ResourceExhausted);
            assert_eq!(code, error.exit_code());
            assert!(!value["ok"].as_bool().unwrap());
            if !human {
                assert_eq!(output.iter().filter(|byte| **byte == b'\n').count(), 1);
            }
        }
    }
    #[test]
    fn broken_output_stops_after_one_failure_without_second_response_or_retry() {
        struct Broken(usize);
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                self.0 += 1;
                Err(std::io::ErrorKind::BrokenPipe.into())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let response = Response::failure(
            "original".into(),
            ApiError::new(ErrorCode::Unsupported, "original unavailable"),
        );
        let mut output = Broken(0);
        assert_eq!(
            write_export_response(&mut output, host::EXPORT_PEEK, &response, false),
            1
        );
        assert_eq!(output.0, 1);
        assert_eq!(response.error.unwrap().code, ErrorCode::Unsupported);
    }
}
