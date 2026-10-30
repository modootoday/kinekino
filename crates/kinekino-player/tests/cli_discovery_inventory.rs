//! Read-only normal CLI discovery; never executes the advertised operations.
#![cfg(windows)]
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[path = "review_source_cli/fixture.rs"]
mod fixture;
#[path = "review_source_cli/process.rs"]
mod process;
#[path = "support/review_discovery_config.rs"]
mod review_discovery_config;
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

struct Owned(Child);
impl Drop for Owned {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
fn digest(path: &Path) -> String {
    kinekino_player::infrastructure::content_digest::sha256(
        &mut File::open(path).unwrap(),
        512 * 1024 * 1024,
    )
    .unwrap()
    .1
}
fn query(root: &Path, ordinal: usize, args: &[&str]) -> Value {
    let output = root.join(format!("{ordinal:03}.stdout.json"));
    let error = root.join(format!("{ordinal:03}.stderr"));
    let mut child = Owned(
        Command::new(crate::cli_exe::kinekino_cli_exe())
            .args(args)
            .stdin(Stdio::null())
            .stdout(File::create(&output).unwrap())
            .stderr(File::create(&error).unwrap())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(15);
    let status = loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "discovery timeout; retained evidence {}",
            root.display()
        );
        std::thread::sleep(Duration::from_millis(20));
    };
    fs::write(
        root.join(format!("{ordinal:03}.process.json")),
        serde_json::to_vec_pretty(
            &json!({"pid":child.0.id(),"args":args,"exit_code":status.code()}),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(fs::metadata(&output).unwrap().len() <= 1024 * 1024);
    let response: Value = serde_json::from_slice(&fs::read(output).unwrap()).unwrap();
    assert!(
        status.success() && response["ok"] == true,
        "{args:?}: {response}"
    );
    response["data"].clone()
}

#[test]
fn configured_project_proposal_job_stores_match_schemas_without_mutating_initialized_stores() {
    kinekino_platform::install();
    fn inventory(root: &Path) -> std::collections::BTreeMap<PathBuf, String> {
        fn visit(root: &Path, at: &Path, out: &mut std::collections::BTreeMap<PathBuf, String>) {
            for entry in fs::read_dir(at).unwrap() {
                let path = entry.unwrap().path();
                let metadata = fs::symlink_metadata(&path).unwrap();
                assert!(!metadata.file_type().is_symlink());
                out.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    if metadata.is_dir() {
                        "directory".into()
                    } else {
                        digest(&path)
                    },
                );
                if metadata.is_dir() {
                    visit(root, &path, out);
                }
            }
        }
        let mut out = std::collections::BTreeMap::new();
        visit(root, root, &mut out);
        out
    }
    let root = std::env::temp_dir().join(format!("cli-store-discovery-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    let mut stores = Vec::new();
    for name in ["projects", "proposals", "jobs"] {
        let path = root.join(name);
        fs::create_dir(&path).unwrap();
        stores.push(path);
    }
    let options = [
        "--project-root",
        stores[0].to_str().unwrap(),
        "--proposal-root",
        stores[1].to_str().unwrap(),
        "--job-root",
        stores[2].to_str().unwrap(),
    ];
    let invoke = |ordinal, command: &str, schema: bool| {
        let mut args = if schema {
            vec!["schema", command, "--json"]
        } else {
            vec![command, "--json"]
        };
        args.extend(options);
        query(&root, ordinal, &args)
    };
    let binary = Path::new(crate::cli_exe::kinekino_cli_exe());
    let producer = digest(binary);
    let capabilities = invoke(0, "capabilities", false);
    let initialized: Vec<_> = stores.iter().map(|store| inventory(store)).collect();
    let expected = BTreeSet::from([
        "projects.create",
        "projects.inspect",
        "projects.save_revision",
        "projects.reconnect_source",
        "proposals.create",
        "proposals.list",
        "proposals.inspect",
        "proposals.edit",
        "proposals.accept",
        "proposals.export_subtitles",
        "proposals.inspect_output_review_target",
        "proposals.record_output_decision",
        "proposals.output_decisions",
        "proposals.edit_capture",
        "proposals.accept_capture",
        "jobs.get",
        "jobs.list",
        "jobs.watch",
    ]);
    let commands = capabilities["commands"].as_array().unwrap();
    let mut seen = BTreeSet::new();
    for (index, item) in commands.iter().enumerate() {
        let command = item["command"].as_str().unwrap();
        assert!(seen.insert(command));
        let schema = invoke(index + 1, command, true);
        assert_eq!(schema["command"], command);
        assert_eq!(schema["status"], item["status"], "{command}");
        if ["projects.", "proposals.", "jobs."]
            .iter()
            .any(|prefix| command.starts_with(prefix))
        {
            assert_eq!(
                item["status"],
                if expected.contains(command) {
                    "supported"
                } else {
                    "unsupported"
                },
                "{command}"
            );
        }
    }
    for command in expected {
        assert!(seen.contains(command), "missing {command}");
    }
    for (store, original) in stores.iter().zip(&initialized) {
        assert_eq!(
            &inventory(store),
            original,
            "discovery mutated initialized store {}",
            store.display()
        );
    }
    assert_eq!(digest(binary), producer);
    fs::write(
        root.join("configured-store-inventory.json"),
        serde_json::to_vec_pretty(&json!({
            "cli_sha256":producer,"commands":commands,"command_count":seen.len(),
            "scope":"explicit_project_proposal_readonly_job_roots_discovery_only",
            "functional_acceptance":"not_proven_by_discovery","functional_completion_percent":null,
        "initialized_stores_unchanged":true
        }))
        .unwrap(),
    )
    .unwrap();
    println!("configured store discovery evidence {}", root.display());
}

#[test]
fn normal_cli_discovery_inventory_has_unique_commands_and_matching_schema_status() {
    kinekino_platform::install();
    let root = PathBuf::from(
        std::env::var_os("TMP").unwrap_or_else(|| std::env::temp_dir().into_os_string()),
    )
    .join(format!("cli-discovery-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    let binary = Path::new(crate::cli_exe::kinekino_cli_exe());
    let before = digest(binary);
    let capabilities = query(&root, 0, &["capabilities", "--json"]);
    let commands = capabilities["commands"].as_array().unwrap();
    assert!(!commands.is_empty());
    let mut unique = BTreeSet::new();
    let mut inventory = Vec::new();
    for (index, item) in commands.iter().enumerate() {
        let command = item["command"].as_str().unwrap();
        assert!(unique.insert(command), "duplicate command {command}");
        assert!(matches!(
            item["status"].as_str(),
            Some("supported" | "unsupported")
        ));
        let schema = query(&root, index + 1, &["schema", command, "--json"]);
        assert_eq!(schema["command"], command);
        assert_eq!(schema["status"], item["status"], "{command}");
        inventory.push(json!({"command":command,"default_in_process_status":item["status"],"schema_observed":true,"functional_acceptance":"not_proven_by_discovery","configured_hosts":"not_observed"}));
    }
    assert_eq!(digest(binary), before, "normal CLI producer changed");
    fs::write(root.join("inventory.json"), serde_json::to_vec_pretty(&json!({"cli_sha256":before,"commands":inventory,"command_count":unique.len(),"scope":"default_in_process_discovery_only","functional_completion_percent":null})).unwrap()).unwrap();
    println!("{} commands; evidence {}", unique.len(), root.display());
}

#[test]
fn configured_review_discovery_shard_0() {
    kinekino_platform::install();
    configured_review_discovery(0);
}
#[test]
fn configured_review_discovery_shard_1() {
    kinekino_platform::install();
    configured_review_discovery(1);
}
#[test]
fn configured_review_discovery_shard_2() {
    kinekino_platform::install();
    configured_review_discovery(2);
}
#[test]
fn configured_review_discovery_shard_3() {
    kinekino_platform::install();
    configured_review_discovery(3);
}
fn configured_review_discovery(shard: usize) {
    const SHARDS: usize = 4;
    assert!(shard < SHARDS);
    let binary = Path::new(crate::cli_exe::kinekino_cli_exe());
    let producer = digest(binary);
    let fixture = fixture::Fixture::new();
    let host_options = review_discovery_config::frame_options();
    let call = |label: &str, request: &Value| {
        if host_options.is_empty() {
            process::call(&fixture, label, request, true, None)
        } else {
            process::call_with_options(&fixture, label, request, true, None, &host_options)
        }
    };
    let storage_commands = BTreeSet::from([
        "reviews.inspect",
        "reviews.record_decision",
        "reviews.decisions",
        "reviews.read_text_asset",
        "reviews.association.inspect",
        "reviews.association.create",
        "reviews.association.revise",
        "reviews.range_draft.read",
        "reviews.range_draft.save",
    ]);
    let playback_commands = BTreeSet::from([
        "reviews.playback.open",
        "reviews.playback.status",
        "reviews.playback.close",
        "reviews.playback.control",
    ]);
    // These require separately pinned media-preparation settings, which this
    // fixture does not provide. Frame discovery options do not enable the port.
    let media_commands = BTreeSet::from(["reviews.media.prepare", "reviews.media.inspect"]);
    let request = fixture.request("capabilities", json!({}), None, None);
    let response = call("inventory-capabilities", &request);
    assert_eq!(
        response["data"]["review_media_preparation"]["read_available"],
        false
    );
    assert_eq!(
        response["data"]["review_media_preparation"]["write_available"],
        false
    );
    let commands = response["data"]["commands"].as_array().unwrap();
    assert_eq!(commands.len(), 119, "review discovery command set changed");
    let mut inventory = Vec::new();
    let mut unique = BTreeSet::new();
    let mut observed_schemas = Vec::new();
    for (index, item) in commands.iter().enumerate() {
        let command = item["command"].as_str().unwrap();
        assert!(unique.insert(command));
        if index % SHARDS == shard {
            let request = fixture.request("schema", json!({"command":command}), None, None);
            let schema = call(&format!("inventory-schema-{index:03}"), &request);
            assert_eq!(schema["data"]["command"], command);
            assert_eq!(schema["data"]["status"], item["status"], "{command}");
            observed_schemas.push(command);
        }
        if command.starts_with("reviews.") {
            assert!(
                storage_commands.contains(command)
                    || playback_commands.contains(command)
                    || media_commands.contains(command),
                "unclassified review command {command}"
            );
            assert_eq!(
                item["status"],
                if storage_commands.contains(command) {
                    "supported"
                } else {
                    "unsupported"
                },
                "{command}"
            );
        }
        inventory.push(item.clone());
    }
    let observed_reviews: BTreeSet<_> = unique
        .iter()
        .copied()
        .filter(|name| name.starts_with("reviews."))
        .collect();
    assert_eq!(
        observed_reviews,
        storage_commands
            .iter()
            .chain(&playback_commands)
            .chain(&media_commands)
            .copied()
            .collect()
    );
    assert_eq!(
        observed_schemas.len(),
        (commands.len() + SHARDS - 1 - shard) / SHARDS
    );
    assert_eq!(digest(binary), producer);
    fs::write(fixture.root.join("configured-review-inventory.json"), serde_json::to_vec_pretty(&json!({"cli_sha256":producer,"commands":inventory,"schema_commands":observed_schemas,"shard":shard,"shards":SHARDS,"host_options":host_options,"fixture_case":fixture.case,"fixture_media":fixture.media,"scope":"configured_review_roots_discovery_only","functional_acceptance":"not_proven_by_discovery"})).unwrap()).unwrap();
    fixture.assert_originals();
    println!(
        "configured review discovery evidence {}",
        fixture.root.display()
    );
}
