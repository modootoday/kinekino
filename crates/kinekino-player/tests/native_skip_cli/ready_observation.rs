//! Explicit ready/EOF-only observation. Never a media or endpoint command checkpoint.
use super::{
    fixture::Fixture,
    kinekino_process::{self, CliFailure, OwnedHost, ScenarioFailure},
};
use ::kinekino_process::domain::{
    deadline::Deadline,
    observation::{FamilyObservation, ProcessReceipt},
};
use kinekino_player::domain::identifiers::valid_sha256;
use serde_json::Value;
use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
    time::{Duration, Instant},
};

fn invalid(message: &str) -> io::Error {
    io::Error::other(message)
}
struct Inputs {
    catalog: PathBuf,
    mpv: PathBuf,
    sha256: String,
}
impl Inputs {
    fn environment() -> io::Result<Self> {
        let path = |name| {
            std::env::var_os(name)
                .map(PathBuf::from)
                .ok_or_else(|| invalid("Explicit ready-only input is required"))
        };
        let input = Self {
            catalog: path("NATIVE_EXPORT_CATALOG_ROOT")?,
            mpv: path("NATIVE_MPV_EXE")?,
            sha256: std::env::var("NATIVE_MPV_SHA256")
                .map_err(|_| invalid("Explicit native SHA256 is required"))?,
        };
        if !input.catalog.is_absolute()
            || !input.catalog.is_dir()
            || !input.mpv.is_absolute()
            || input.mpv.file_name().and_then(|name| name.to_str()) != Some("mpv.exe")
            || !valid_sha256(&input.sha256)
        {
            return Err(invalid(
                "Ready-only inputs must name original absolute catalog/mpv and SHA256",
            ));
        }
        Ok(input)
    }
}
fn validate_original_receipt(receipt: &ProcessReceipt) -> io::Result<()> {
    if receipt.child_exit != Some(0)
        || receipt.family != FamilyObservation::Empty
        || receipt.forced
        || !receipt.stdin_eof_requested
    {
        return Err(invalid(
            "Original ready-only owner did not observe graceful EOF",
        ));
    }
    Ok(())
}
fn validate_parent_summary(bytes: &[u8]) -> io::Result<()> {
    if bytes.len() > 2048 || !bytes.ends_with(b"\n") {
        return Err(invalid("Parent summary is incomplete or exceeds budget"));
    }
    let value: Value = serde_json::from_slice(bytes).map_err(io::Error::other)?;
    if value["schema_version"] != 1
        || value["exit"] != "Record"
        || value["last_record"] != "Record"
        || value.get("has_unknown") != Some(&Value::Bool(false))
        || value.get("host_cause_established") != Some(&Value::Bool(false))
        || value["elapsed_us"].as_u64().is_none()
        || !value["iterations"].as_u64().is_some_and(|count| count > 0)
    {
        return Err(invalid("Completed parent record observation is missing"));
    }
    let before = value["offset_before"]
        .as_u64()
        .ok_or_else(|| invalid("Missing original offset"))?;
    let after = value["offset_after"]
        .as_u64()
        .ok_or_else(|| invalid("Missing final offset"))?;
    if after <= before {
        return Err(invalid("Original ready cursor did not advance"));
    }
    for phase in ["limit_check", "record_poll", "child_poll"] {
        for field in ["calls", "total_us", "max_us"] {
            if value[phase][field].as_u64().is_none() {
                return Err(invalid("Parent phase observation is unknown"));
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "Separately allocated original normal native host ready/EOF observation; no media or endpoint requests"]
fn actual_original_native_ready_and_eof_observation() -> Result<(), ScenarioFailure> {
    kinekino_platform::install();
    let mut fixture = Fixture::named("native-ready-record-observation");
    fixture.keep_evidence();
    let before =
        |stage, error| ScenarioFailure::host(CliFailure::not_spawned(stage, error, &fixture.root));
    let input = Inputs::environment().map_err(|error| before("ready-only inputs", error))?;
    let original = Deadline::new(
        Instant::now(),
        Duration::from_secs(90),
        Duration::from_secs(10),
    )
    .map_err(|error| before("ready-only original deadline", error))?;
    let ready = kinekino_process::phase_deadline(original, Duration::from_secs(10))
        .map_err(|error| before("ready-only deadline", error))?;
    let host = OwnedHost::launch(
        fixture.cli.clone(),
        vec![
            "--serve-native".into(),
            "--catalog-root".into(),
            input.catalog.into_os_string(),
            "--mpv-executable".into(),
            input.mpv.into_os_string(),
            "--mpv-sha256".into(),
            input.sha256.clone().into(),
        ],
        &fixture.root,
        original,
    )
    .map_err(ScenarioFailure::host)?;
    let (host, ()) = host
        .ready_observed(ready, &fixture.root.join("logs/ready.stdout"), |bytes| {
            let value: Value = serde_json::from_slice(bytes).map_err(io::Error::other)?;
            if value["event"] != "native_host_ready"
                || value["engine_sha256"] != input.sha256
                || value["scope"] != "current_user_same_privilege"
                || value.get("playback_verified") != Some(&Value::Bool(false))
                || !value["endpoint"]
                    .as_str()
                    .is_some_and(|endpoint| !endpoint.is_empty())
            {
                return Err(invalid("Original normal native ready record differs"));
            }
            Ok(())
        })
        .map_err(ScenarioFailure::host)?;
    // No request, observation or media operation: close original stdin immediately.
    let stopped = host
        .finish(
            Duration::from_secs(10),
            &fixture.root.join("logs/stop.stdout"),
            &fixture.root.join("logs/host.stderr"),
        )
        .and_then(|terminal| {
            let failed = |error| {
                CliFailure::terminal(
                    "ready-only EOF proof",
                    error,
                    &fixture.root,
                    terminal.receipt.clone(),
                )
            };
            let value: Value = serde_json::from_slice(&terminal.tail)
                .map_err(|error| failed(io::Error::other(error)))?;
            super::validate_no_media_stop(&value).map_err(failed)?;
            validate_original_receipt(&terminal.receipt).map_err(failed)?;
            Ok(terminal.receipt)
        });
    let evidence_error = (|| {
        let mut bytes = Vec::new();
        fs::File::open(fixture.root.join("owned/ready-observation.json"))?
            .take(2049)
            .read_to_end(&mut bytes)?;
        validate_parent_summary(&bytes)
    })()
    .err();
    ScenarioFailure::conclude(Ok(Ok(())), stopped, evidence_error)
}

#[test]
fn ready_only_receipt_requires_original_exit_empty_unforced_eof() {
    kinekino_platform::install();
    use ::kinekino_process::domain::identity::ProcessIdentity;
    let original = ProcessReceipt {
        identity: ProcessIdentity {
            pid: 7,
            creation_filetime: Some(9),
            executable: "pure-original".into(),
        },
        child_exit: Some(0),
        family: FamilyObservation::Empty,
        forced: false,
        stdin_eof_requested: true,
    };
    assert!(validate_original_receipt(&original).is_ok());
    for field in ["exit", "family", "forced", "eof"] {
        let mut changed = original.clone();
        match field {
            "exit" => changed.child_exit = None,
            "family" => changed.family = FamilyObservation::Unknown,
            "forced" => changed.forced = true,
            "eof" => changed.stdin_eof_requested = false,
            _ => unreachable!(),
        };
        assert!(validate_original_receipt(&changed).is_err(), "{field}");
    }
}
#[test]
fn ready_only_summary_rejects_unknown_cutoff_partial_and_over_budget() {
    kinekino_platform::install();
    use serde_json::json;
    let phase = json!({"calls":1,"total_us":0,"max_us":0});
    let complete = json!({"schema_version":1,"iterations":1,"exit":"Record","last_record":"Record","offset_before":0,"offset_after":6,"elapsed_us":2,"has_unknown":false,"host_cause_established":false,"limit_check":phase,"record_poll":phase,"child_poll":phase});
    let encode = |value: &Value| {
        let mut bytes = serde_json::to_vec(value).unwrap();
        bytes.push(b'\n');
        bytes
    };
    assert!(validate_parent_summary(&encode(&complete)).is_ok());
    for (field, value) in [
        ("has_unknown", json!(true)),
        ("exit", json!("PostCutoff")),
        ("offset_after", json!(0)),
        ("elapsed_us", Value::Null),
    ] {
        let mut changed = complete.clone();
        changed[field] = value;
        assert!(
            validate_parent_summary(&encode(&changed)).is_err(),
            "{field}"
        );
    }
    let mut missing = complete.clone();
    missing["limit_check"]["calls"] = Value::Null;
    assert!(validate_parent_summary(&encode(&missing)).is_err());
    assert!(validate_parent_summary(&serde_json::to_vec(&complete).unwrap()).is_err());
    assert!(validate_parent_summary(&vec![b'x'; 2049]).is_err());
}
