use super::*;
use crate::application::api::AssetClient;
use crate::infrastructure::asset_build_jobs::AssetArtifactFiles;

#[test]
#[ignore = "Pinned original asset artifact and fresh evidence directory required"]
fn pinned_artifact_readback_retains_files_and_rejects_tampering() {
    let source = PathBuf::from(std::env::var_os("ASSET_INSPECT_SOURCE").unwrap());
    let root = PathBuf::from(std::env::var_os("ASSET_INSPECT_EVIDENCE").unwrap());
    let id = std::env::var("ASSET_INSPECT_ID").unwrap();
    let sha = std::env::var("ASSET_INSPECT_SHA256").unwrap();
    fs::create_dir(&root).unwrap();
    let destination = root.join(format!("asset-{id}"));
    fs::create_dir(&destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        assert!(entry.file_type().unwrap().is_file());
        fs::copy(entry.path(), destination.join(entry.file_name())).unwrap();
    }
    let reader = AssetArtifactFiles::open(&root).unwrap();
    assert!(reader.inspect(&id, &"0".repeat(64), &Observe).is_err());
    let retained = reader.inspect(&id, &sha, &Observe).unwrap();
    assert_eq!(retained.sha256(), sha);
    let publication = retained.publication().clone();
    assert_eq!(publication.files.len(), 8);
    use crate::application::api::{PlayerSdk, Request, RequestContext};
    let input = crate::application::asset_artifacts::InspectAssetArtifact {
        artifact_id: id.clone(),
        publication_sha256: sha.clone(),
    };
    assert_eq!(
        PlayerSdk::default()
            .inspect_asset_artifact(RequestContext::default(), input.clone())
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    let sdk = PlayerSdk::default()
        .with_asset_artifact_inspector(Arc::new(AssetArtifactFiles::open(&root).unwrap()));
    let observed = sdk
        .inspect_asset_artifact(RequestContext::default(), input.clone())
        .unwrap();
    let mut schema_request = Request::new("schema");
    schema_request.payload = serde_json::json!({"command":"asset_artifacts.inspect"});
    let schema = sdk.execute(schema_request).data.unwrap();
    let data = &schema["response"]["properties"]["data"]["anyOf"][0];
    let actual = serde_json::to_value(&observed).unwrap();
    for name in data["required"].as_array().unwrap() {
        assert!(actual.get(name.as_str().unwrap()).is_some());
    }
    assert_eq!(data["additionalProperties"], false);
    assert_eq!(
        data["properties"]["publication"]["properties"]["quality_approved"]["const"],
        false
    );
    let layers = &data["properties"]["manifest"]["properties"]["layers"];
    assert_eq!(layers["maxItems"], 4096);
    assert_eq!(
        layers["items"]["properties"]["geometry"]["properties"]["pivot_x"]["properties"]["numerator"]
            ["type"],
        "string"
    );
    assert_eq!(
        layers["items"]["properties"]["request"]["properties"]["schema_version"]["const"],
        "asset-frame-build/1"
    );
    assert_eq!(observed.publication_sha256, sha);
    assert_eq!(observed.manifest.layers.len(), 2);
    let caps = sdk.execute(Request::new("capabilities")).data.unwrap();
    assert_eq!(caps["asset_artifacts"]["backend_available"], true);
    assert_eq!(caps["asset_artifacts"]["jobs_completion_authority"], false);
    let mut injected = Request::new("asset_artifacts.inspect");
    injected.payload = serde_json::to_value(&input).unwrap();
    injected.payload["path"] = serde_json::json!("C:/foreign");
    assert_eq!(
        sdk.execute(injected).error.unwrap().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        sdk.inspect_asset_artifact(
            RequestContext {
                timeout_ms: Some(1),
                ..RequestContext::default()
            },
            input
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    let project_bytes = fs::read(destination.join("project.json")).unwrap();
    let manifest_bytes = fs::read(destination.join("manifest.json")).unwrap();
    publication
        .validate_contents(&project_bytes, &manifest_bytes, &Observe)
        .unwrap();
    let manifest: serde_json::Value = serde_json::from_slice(&manifest_bytes).unwrap();
    for pointer in ["/quality_approved", "/colorimetry_verified"] {
        let mut changed = manifest.clone();
        *changed.pointer_mut(pointer).unwrap() = serde_json::json!(true);
        assert!(
            publication
                .validate_contents(
                    &project_bytes,
                    &serde_json::to_vec(&changed).unwrap(),
                    &Observe
                )
                .is_err()
        );
    }
    for pointer in [
        "/plan/units/0/frame_id",
        "/layers/0/unit/layer_id",
        "/layers/0/rgba8/name",
        "/plan/project/publication_sha256",
    ] {
        let mut changed = manifest.clone();
        *changed.pointer_mut(pointer).unwrap() = serde_json::json!("foreign");
        assert!(
            publication
                .validate_contents(
                    &project_bytes,
                    &serde_json::to_vec(&changed).unwrap(),
                    &Observe
                )
                .is_err()
        );
    }
    let mut changed = manifest.clone();
    changed["layers"][0]["geometry"]["output_width"] = serde_json::json!(1);
    assert!(
        publication
            .validate_contents(
                &project_bytes,
                &serde_json::to_vec(&changed).unwrap(),
                &Observe
            )
            .is_err()
    );
    let data = destination.join(&publication.files[0].name);
    drop(reader);
    assert!(OpenOptions::new().write(true).open(&data).is_err());
    assert!(fs::rename(&data, destination.join("moved.json")).is_err());
    drop(retained);
    let reader = AssetArtifactFiles::open(&root).unwrap();
    fs::write(destination.join("extra"), b"unexpected").unwrap();
    assert!(reader.inspect(&id, &sha, &Observe).is_err());
    fs::remove_file(destination.join("extra")).unwrap();
    let mut bytes = fs::read(&data).unwrap();
    bytes[0] ^= 1;
    fs::write(&data, bytes).unwrap();
    assert!(reader.inspect(&id, &sha, &Observe).is_err());
    for invalid in ["../escape", "C:/escape", ""] {
        assert!(reader.inspect(invalid, &sha, &Observe).is_err());
    }
    let mut invalid = publication.clone();
    invalid.files[1].name = "../escape".into();
    assert!(invalid.validate(&id, 1024).is_err());
    invalid = publication.clone();
    invalid.files[2] = invalid.files[1].clone();
    assert!(invalid.validate(&id, 1024).is_err());
    invalid = publication;
    invalid.quality_approved = true;
    assert!(invalid.validate(&id, 1024).is_err());
}
