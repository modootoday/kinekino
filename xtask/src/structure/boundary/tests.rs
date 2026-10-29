use super::manifest::{DepKind, parse};
use super::source::*;
use super::*;

#[test]
fn cfg_counting_ignores_test_and_features() {
    let t = "#[cfg(test)]\n#[cfg(windows)]\nfn a(){ if cfg!(target_os = \"linux\") {} }\n#[cfg(feature = \"windows-x\")]\n// #[cfg(unix)]\n#[cfg_attr(not(windows), allow(x))]";
    assert_eq!(platform_cfg_count(t), 3);
}

#[test]
fn entry_shape() {
    assert!(entry_is_thin(
        "fn main() -> std::process::ExitCode {\n    kinekino_app::entry(a(), b(), runner::run)\n}\n"
    ));
    assert!(!entry_is_thin(
        "#[cfg(windows)]\nmod x;\nfn main() { kinekino_app::entry() }"
    ));
    assert!(!entry_is_thin(
        "fn main() { let a = 1; kinekino_app::entry(a); }"
    ));
    assert!(!entry_is_thin("fn main() { runner::run() }"));
}

#[test]
fn manifest_deps() {
    let m = parse(
        "crates/x",
        "[package]\nname = \"x\"\n[dependencies]\nserde.workspace = true\ntime = { version = \"1\" }\nkinekino-core = { workspace = true, features = [\"a\"] }\n[target.'cfg(windows)'.dev-dependencies]\nwry = \"1\"\n",
    );
    assert_eq!(m.name, "x");
    let d: Vec<(&str, DepKind, bool)> = m
        .deps
        .iter()
        .map(|d| (d.name.as_str(), d.kind, d.workspace))
        .collect();
    assert_eq!(
        d,
        vec![
            ("serde", DepKind::Normal, true),
            ("time", DepKind::Normal, false),
            ("kinekino-core", DepKind::Normal, true),
            ("wry", DepKind::Dev, false)
        ]
    );
}

#[test]
fn dependency_direction() {
    assert!(dep_violation(Class::Domain, "kinekino-browser").is_some());
    assert!(dep_violation(Class::Domain, "kinekino-core").is_none());
    assert!(dep_violation(Class::Bin, "kinekino-app").is_none());
    assert!(dep_violation(Class::Bin, "serde").is_some());
    assert!(dep_violation(Class::Composition, "kinekino-browser").is_none());
    assert!(dep_violation(Class::Contract, "kinekino-player").is_some());
}

#[test]
fn port_locations_and_traits() {
    assert!(is_port_location("src/application/ports.rs"));
    assert!(is_port_location("src/engine/port/mod.rs"));
    assert!(!is_port_location("src/engine/select/availability.rs"));
    assert_eq!(
        port_like_traits("pub trait JobStore {\npub trait Clock {\n  pub trait AsyncSdkPort<I> {"),
        vec!["JobStore", "AsyncSdkPort"]
    );
}

#[test]
fn every_workspace_crate_is_classified() {
    let root = crate::workspace::root().unwrap();
    for m in manifest::workspace(&root) {
        assert!(class_of(&m.name).is_some(), "{} unclassified", m.name);
    }
}
