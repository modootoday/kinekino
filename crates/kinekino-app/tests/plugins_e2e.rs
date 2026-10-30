//! ADR 0011 M3: discover a fake plugin, list it, and browse through the process boundary.
use kinekino_app::{Args, Env, run_plugins};
use kinekino_core::source::{SourceError, SourceProvider, SourceRef};
use kinekino_source_host::plugin_host::ProcessSource;

fn setup() -> (std::path::PathBuf, Env) {
    let root = std::env::temp_dir().join(format!("kinekino-plugins-{}", std::process::id()));
    let dir = root.join("fixture");
    std::fs::create_dir_all(&dir).unwrap();
    let exe = env!("CARGO_BIN_EXE_fake-source-plugin");
    let target = dir.join(format!("plugin{}", std::env::consts::EXE_SUFFIX));
    std::fs::copy(exe, &target).unwrap();
    let manifest = serde_json::json!({
        "id": "com.example.fixture", "version": "0.1.0", "api": "source/1",
        "capabilities": ["browse", "resolve"], "media_hosts": ["media.example.test"],
        "executable": target.file_name().unwrap().to_str().unwrap(),
    });
    std::fs::write(dir.join("plugin.json"), manifest.to_string()).unwrap();
    std::fs::create_dir_all(root.join("broken")).unwrap();
    std::fs::write(root.join("broken/plugin.json"), "{").unwrap();
    let env = Env::from_pairs([("KINEKINO_PLUGIN_DIR", root.to_str().unwrap())]);
    (root, env)
}

#[test]
fn list_browse_and_capability_gate_over_process() {
    let (root, env) = setup();
    let (code, out) = run_plugins(&Args::new(["plugins", "list"]), &env);
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["plugins"][0]["id"], "com.example.fixture");
    assert_eq!(v["plugins"][0]["media_hosts"][0], "media.example.test");
    assert_eq!(v["rejected"].as_array().unwrap().len(), 1);

    let plugin = kinekino_app::plugins::discovered(&env).remove(0);
    let source = ProcessSource::new(plugin);
    let cat = SourceRef {
        provider: "com.example.fixture".into(),
        id: "airing".into(),
    };
    let page = source.browse(&cat, 3).unwrap();
    assert_eq!(page.items[0].reference.id, "airing-3");
    assert_eq!(page.next_page, Some(4));
    assert!(matches!(
        source.search("x", 1),
        Err(SourceError::Unsupported(_))
    ));
    // Declared but the fixture refuses: error kind crosses the boundary.
    assert!(matches!(
        source.resolve(&cat),
        Err(SourceError::Unsupported(_))
    ));
    std::fs::remove_dir_all(root).ok();
}
