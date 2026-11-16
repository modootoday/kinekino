use super::*;
use crate::{
    application::{
        ai_provenance,
        proposals::{CreateProposal, source_import},
    },
    domain::{api_values::DecimalU64, proposals::GeneratedResult},
};

#[test]
fn malformed_configuration_is_rejected_before_any_process() {
    assert_eq!(
        HistoricalVerifier::open(
            Path::new("relative.exe"),
            "a".repeat(64),
            Path::new("relative"),
            "valid".into()
        )
        .err()
        .unwrap()
        .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        HistoricalVerifier::open(
            Path::new("C:/absent.exe"),
            "A".repeat(64),
            Path::new("C:/"),
            "valid".into()
        )
        .err()
        .unwrap()
        .code,
        ErrorCode::InvalidArgument
    );
}

#[test]
#[ignore = "requires explicitly pinned installed verifier; offline actual-nine evidence"]
fn actual_pinned_verifier_reconstructs_nine_originals_and_rejects_swapped_reference() {
    let executable =
        std::env::var_os("HISTORICAL_VERIFIER_EXE").expect("explicit verifier path required");
    let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let sha = "e00c1726603ef7d0167c6d58deb79e694071044c20a2723de8abe8a504f45e41";
    assert_eq!(
        HistoricalVerifier::open(
            Path::new(&executable),
            "0".repeat(64),
            root,
            "sdk-text-results-r3".into()
        )
        .err()
        .unwrap()
        .code,
        ErrorCode::PermissionDenied
    );
    let reader = HistoricalVerifier::open(
        Path::new(&executable),
        sha.into(),
        root,
        "sdk-text-results-r3".into(),
    )
    .unwrap();
    let companion_path = std::env::var_os("HISTORICAL_COMPANION_PATH");
    let companion = crate::test_support::historical_fixture::read_companion(
        companion_path.as_deref().map(Path::new),
        "7a7cd05eafa6294e846c8080c976600ea135aa91e9da315ac9f770e45f624b10",
        16566,
    )
    .expect("explicit preserved companion required");
    let rows = companion["items"].as_array().unwrap();
    assert_eq!(rows.len(), 9);
    for row in rows {
        let reference = CompanionReference {
            sha256: "7a7cd05eafa6294e846c8080c976600ea135aa91e9da315ac9f770e45f624b10".into(),
            bytes: 16566,
            item_id: row["item_id"].as_str().unwrap().into(),
        };
        let sample = reader.verify(&reference).unwrap();
        assert_eq!(
            sample.source_sha256,
            row["source"]["sha256"].as_str().unwrap()
        );
        assert_eq!(
            sample.candidate_sha256,
            row["candidate"]["sha256"].as_str().unwrap()
        );
        let source = source_import::from_request_asset(
            "imported-source".into(),
            DecimalU64(1),
            sample.source_sha256.clone(),
            sample.source_request_utf8.clone(),
            sample.task,
        )
        .unwrap();
        let prepared = ai_provenance::prepare(
            &reader,
            reference.clone(),
            CreateProposal {
                proposal_id: format!("import-{}", reference.item_id),
                source,
                generated: GeneratedResult {
                    task: sample.task,
                    raw_text: sample.candidate_raw_utf8.clone(),
                },
            },
        )
        .unwrap();
        assert_eq!(prepared.into_parts().1, reference);
        let mut swapped = reference;
        swapped.sha256 = "0".repeat(64);
        assert_eq!(
            reader.verify(&swapped).err().unwrap().code,
            ErrorCode::NeedsInput
        );
    }
}

#[test]
fn pinned_verifier_runs_its_verified_program() {
    let _ = crate::infrastructure::os_policy::host();
    let name = format!("true{}", std::env::consts::EXE_SUFFIX);
    let Some(native) = std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join(&name))
            .find(|path| path.is_file())
    }) else {
        eprintln!("no native `true` on PATH; pinned verifier check skipped");
        return;
    };
    let scratch = std::env::temp_dir().join(format!("kk-historical-pin-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir(&scratch).unwrap();
    let executable = scratch.join(&name);
    std::fs::copy(&native, &executable).unwrap();
    let sha = content_digest::sha256(&mut File::open(&executable).unwrap(), 134_217_728)
        .unwrap()
        .1;
    let reader = HistoricalVerifier::open(&executable, sha, &scratch, "pin-check".into()).unwrap();
    let reference = CompanionReference {
        sha256: "a".repeat(64),
        bytes: 1,
        item_id: "item-1".into(),
    };
    // `true` prints nothing: the run reaches the verified program and its empty output is
    // reported as missing evidence, not as a spawn or pin failure.
    let error = reader.verify(&reference).err().unwrap();
    assert_eq!(error.code, ErrorCode::NeedsInput, "{error:?}");
    drop(reader);
    let _ = std::fs::remove_dir_all(&scratch);
}

#[test]
fn script_verifier_is_refused_where_sharing_is_advisory() {
    let _ = crate::infrastructure::os_policy::host();
    if crate::infrastructure::os_policy::has_mandatory_share_modes() {
        return;
    }
    let scratch = std::env::temp_dir().join(format!("kk-historical-script-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir(&scratch).unwrap();
    let executable = scratch.join("verifier");
    std::fs::write(&executable, b"#!/bin/sh\nexit 0\n").unwrap();
    let sha = content_digest::sha256(&mut File::open(&executable).unwrap(), 1024)
        .unwrap()
        .1;
    let reader = HistoricalVerifier::open(&executable, sha, &scratch, "pin-check".into()).unwrap();
    let reference = CompanionReference {
        sha256: "a".repeat(64),
        bytes: 1,
        item_id: "item-1".into(),
    };
    // A script's interpreter would reopen the path instead of the verified file.
    let error = reader.verify(&reference).err().unwrap();
    assert_eq!(error.code, ErrorCode::InvalidArgument, "{error:?}");
    drop(reader);
    let _ = std::fs::remove_dir_all(&scratch);
}
