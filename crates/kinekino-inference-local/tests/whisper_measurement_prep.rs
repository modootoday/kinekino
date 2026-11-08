//! M4-9: local whisper measurement prep. These tests lock fixtures, measurement
//! items, and the reproduction procedure. They must not download a model or
//! spawn a real whisper-cli.
use kinekino_inference::InferenceError;
use kinekino_inference_local::{
    FIXTURE_DURATION_SECS, FIXTURE_SAMPLE_RATE, MeasurementStatus, PROCEDURE_STEPS, WhisperConfig,
    assess_prep, blank_record, format_fixture_wav, inspect_wav, required_measurement_items,
    start_measurement,
};
use std::path::{Path, PathBuf};
use std::time::Duration;

struct Pack {
    dir: PathBuf,
    config: WhisperConfig,
}

impl Pack {
    fn new(name: &str, with_model: bool) -> Self {
        let dir =
            std::env::temp_dir().join(format!("kk-whisper-prep-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let program = dir.join("whisper-cli");
        std::fs::write(&program, "#!/bin/sh\ntouch \"$0.spawned\"\nexit 0\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let model = dir.join("ggml-large-v3-turbo.bin");
        if with_model {
            std::fs::write(&model, b"fake").unwrap();
        }
        let config = WhisperConfig {
            program,
            model,
            language: "ko".into(),
            threads: 1,
            timeout: Duration::from_secs(1),
            work_dir: dir.join("work"),
        };
        Self { dir, config }
    }

    fn spawned(&self) -> bool {
        self.dir.join("whisper-cli.spawned").exists()
    }
}

impl Drop for Pack {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn required_ids() -> &'static [&'static str] {
    &[
        "availability",
        "process_exit",
        "wall_time_median_s",
        "peak_memory_bytes",
        "real_time_factor",
        "transcript_text",
        "character_error_rate",
        "chunk_bounds",
        "provenance_backend",
        "provenance_model",
        "provenance_cost_usd",
        "language",
        "measured",
        "conditions",
    ]
}

#[test]
fn format_fixture_is_16khz_mono_pcm16_of_known_duration() {
    let wav = format_fixture_wav().expect("format fixture");
    let info = inspect_wav(&wav).expect("inspect fixture");
    assert_eq!(info.sample_rate, FIXTURE_SAMPLE_RATE);
    assert_eq!(info.sample_rate, 16_000);
    assert_eq!(info.channels, 1);
    assert_eq!(info.bits_per_sample, 16);
    assert!(
        (info.duration_secs - FIXTURE_DURATION_SECS).abs() < 0.001,
        "duration {}",
        info.duration_secs
    );
    assert_eq!(FIXTURE_DURATION_SECS, 2.0);
    assert_eq!(&wav[..4], b"RIFF");
    assert!(matches!(
        inspect_wav(b"not a wav"),
        Err(InferenceError::InvalidInput(_))
    ));
}

#[test]
fn measurement_items_include_required_fields_and_mark_cer() {
    let items = required_measurement_items();
    let ids: Vec<&str> = items.iter().map(|i| i.id).collect();
    for id in required_ids() {
        assert!(ids.contains(id), "missing item {id}");
    }
    let cer = items
        .iter()
        .find(|i| i.id == "character_error_rate")
        .expect("cer");
    assert!(
        cer.requires_speech_reference,
        "CER needs a licensed speech fixture after approval"
    );
    for item in items {
        if item.id != "character_error_rate" {
            assert!(
                !item.requires_speech_reference,
                "{} should not require a speech reference",
                item.id
            );
        }
    }
}

#[test]
fn procedure_covers_repro_without_download_or_quality_approval() {
    let text = PROCEDURE_STEPS.join("\n").to_ascii_lowercase();
    for needle in [
        "does not download",
        "whisper-cli",
        "16 khz",
        "approval",
        "three",
        "quality",
        "speech fixture",
    ] {
        assert!(
            text.contains(needle),
            "procedure missing {needle:?}\n{text}"
        );
    }
}

#[test]
fn blank_record_is_not_run_and_has_empty_values() {
    let record = blank_record();
    assert_eq!(record.status, MeasurementStatus::NotRun);
    let ids: Vec<&str> = record.items.iter().map(|i| i.id.as_str()).collect();
    for id in required_ids() {
        assert!(ids.contains(id), "blank record missing {id}");
    }
    assert!(record.items.iter().all(|i| i.value.is_none()));
}

#[test]
fn assess_prep_is_ready_without_measuring_or_creating_a_missing_model() {
    let pack = Pack::new("missing-model", false);
    assert!(!pack.config.model.exists());
    let report = assess_prep(&pack.config);
    assert!(report.fixture_ready);
    assert!(report.items_ready);
    assert!(report.procedure_ready);
    assert!(report.artifacts_ready);
    assert!(!report.model_present);
    assert!(!report.measured);
    assert_eq!(report.inference_invocations, 0);
    assert!(
        !pack.config.model.exists(),
        "assess_prep must not download or create the model"
    );
    assert!(!pack.spawned());
}

#[test]
fn assess_prep_and_start_measurement_do_not_spawn_whisper_cli() {
    let pack = Pack::new("present", true);
    let report = assess_prep(&pack.config);
    assert!(report.binary_present);
    assert!(report.model_present);
    assert!(report.artifacts_ready);
    assert!(
        !report.measured,
        "prep ready is not a completed measurement"
    );
    assert_eq!(report.inference_invocations, 0);
    assert!(!pack.spawned());

    let err = start_measurement(&pack.config).expect_err("measurement must be refused");
    match err {
        InferenceError::Unsupported(message) => {
            let lower = message.to_ascii_lowercase();
            assert!(
                lower.contains("approval") && lower.contains("prep"),
                "{message}"
            );
        }
        other => panic!("expected Unsupported, got {other:?}"),
    }
    assert!(
        !pack.spawned(),
        "start_measurement must not spawn whisper-cli"
    );
    assert!(
        !Path::new(&pack.dir).join("work").exists()
            || std::fs::read_dir(pack.dir.join("work"))
                .map(|it| it.count() == 0)
                .unwrap_or(true),
        "measurement must not write scratch audio"
    );
}
