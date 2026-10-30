use crate::{infrastructure as io, json, paths};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
};
const SOURCE_SHA: &str = "A09B51ACBC77645FD12D8488B23021173780851E267A8BA5103D34F827294D39";
const PLAN_SHA: &str = "082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464";
pub fn run() -> Result<(), String> {
    let args: Vec<_> = env::args().skip(1).collect();
    if args
        .first()
        .is_some_and(|v| v == "sprite-audio-legacy-replay")
    {
        return crate::sprite_audio_replay::run_legacy(&args[1..]);
    }
    if args
        .first()
        .is_some_and(|v| v == "sprite-audio-default-replay")
    {
        return crate::sprite_audio_replay::run_default(&args[1..]);
    }
    if args.first().is_some_and(|v| v == "sprite-audio-replay") {
        return crate::sprite_audio_replay::run(&args[1..]);
    }
    if args.first().is_some_and(|v| v == "sprite-pcm-lineage") {
        return crate::sprite_pcm::run(&args[1..]);
    }
    if args.first().is_some_and(|v| v == "sprite-frames") {
        return crate::sprite_metadata::frames(&args[1..]);
    }
    if args.first().is_some_and(|v| v == "sprite-metadata") {
        return crate::sprite_metadata::run(&args[1..]);
    }
    if args.first().is_some_and(|v| v == "snapshot-current") {
        return crate::snapshot::capture(&args[1..]);
    }
    if args.first().is_some_and(|v| v == "verify-current") {
        return crate::snapshot::verify(&args[1..]);
    }
    if args.first().is_some_and(|v| v == "canonical-audit") {
        return crate::adoption::canonical(&args[1..]);
    }
    if args.first().is_some_and(|v| v == "apply-reviewed") {
        return crate::adopt_apply::run(&args[1..]);
    }
    if args.first().is_some_and(|v| v == "adoption") {
        return crate::adoption::run(&args[1..]);
    }
    if args.first().is_some_and(|v| v == "stage") {
        return crate::stage::run(&args[1..]);
    }
    if args.first().is_some_and(|v| v == "exact") {
        return crate::exact::run(&args[1..]);
    }
    if args.len() != 4 {
        return Err("expected ROOT SOURCE_MANIFEST EXACT_PLAN NEW_RECEIPT_DIR".into());
    }
    let root = io::ordinary(Path::new(&args[0]))?;
    let manifest = io::ordinary(Path::new(&args[1]))?;
    let plan = io::ordinary(Path::new(&args[2]))?;
    if io::hash(&manifest)? != SOURCE_SHA || io::hash(&plan)? != PLAN_SHA {
        return Err("frozen manifest pin mismatch".into());
    }
    let source_bytes = fs::read(&manifest).map_err(|e| e.to_string())?;
    let source = json::parse(&source_bytes)?;
    let rows = source.array()?;
    if rows.len() != 375 {
        return Err("source375 count differs".into());
    }
    let plan_bytes = fs::read(&plan).map_err(|e| e.to_string())?;
    let exact = json::parse(&plan_bytes)?;
    let mut tests = BTreeSet::new();
    let mut groups = BTreeMap::<String, usize>::new();
    for row in exact.array()? {
        let group = row.field("group")?.text()?;
        let identity = format!(
            "{}\t{}\t{}",
            row.field("package")?.text()?,
            row.field("target")?.text()?,
            row.field("name")?.text()?
        );
        if !tests.insert(identity) {
            return Err("duplicate exact test identity".into());
        }
        *groups.entry(group.into()).or_default() += 1;
    }
    if tests.len() != 190 {
        return Err("exact190 count differs".into());
    }
    let mut seen = BTreeSet::new();
    let mut inputs = Vec::new();
    for row in rows {
        let raw = row.field("relative")?.text()?;
        let relative = paths::relative(raw)?;
        if !seen.insert(raw.replace('\\', "/").to_ascii_lowercase()) {
            return Err("case-insensitive duplicate source path".into());
        }
        let selected = io::ordinary(&paths::source(
            &root,
            row.field("source")?.text()?,
            &relative,
        )?)?;
        if !selected.starts_with(&root) {
            return Err("selected source outside root".into());
        }
        let sha = row.field("sha256")?.text()?.to_ascii_uppercase();
        if io::hash(&selected)? != sha {
            return Err(format!("selected SHA mismatch: {raw}"));
        }
        inputs.push((relative, selected, sha));
    }
    let output = PathBuf::from(&args[3]);
    let parent = io::ordinary(output.parent().ok_or("receipt parent absent")?)?;
    if !parent.starts_with(root.join("runtime/verification")) {
        return Err("receipt must be new child of private verification".into());
    }
    fs::create_dir(&output).map_err(|e| e.to_string())?;
    io::write_new(&output.join("historical-source375.json"), &source_bytes)?;
    io::write_new(&output.join("historical-exact190.json"), &plan_bytes)?;
    let mut inventory = String::from(
        "relative\tselected_path\tselected_sha256\tselected_fileid\tselected_bytes\tselected_creation100ns\tselected_mtime100ns\tcanonical_state\tcanonical_sha256\tcanonical_fileid\tcanonical_bytes\tcanonical_creation100ns\tcanonical_mtime100ns\n",
    );
    let mut existing = 0;
    let mut absent = 0;
    let mut different = 0;
    for (relative, selected, sha) in inputs {
        io::copy_verified(&selected, &output.join("selected").join(&relative), &sha)?;
        let canonical = root.join(&relative);
        let before = match fs::symlink_metadata(&canonical) {
            Ok(metadata) => {
                if !metadata.is_file() {
                    return Err("canonical not ordinary file".into());
                }
                io::ordinary(&canonical)?;
                let hash = io::hash(&canonical)?;
                io::copy_verified(&canonical, &output.join("preimage").join(&relative), &hash)?;
                existing += 1;
                if hash != sha {
                    different += 1;
                }
                format!("exists\t{hash}\t{}", io::identity(&canonical)?)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                absent += 1;
                "absent\t-\t-\t-\t-\t-".into()
            }
            Err(e) => return Err(e.to_string()),
        };
        inventory.push_str(&format!(
            "{}\t{}\t{sha}\t{}\t{before}\n",
            relative.display(),
            selected.display(),
            io::identity(&selected)?
        ));
    }
    io::write_new(&output.join("custody-preimage.tsv"), inventory.as_bytes())?;
    let summary = format!(
        "host={}\nsource_manifest_sha256={SOURCE_SHA}\nexact_plan_sha256={PLAN_SHA}\nselected_verified=375\ncanonical_existing={existing}\ncanonical_absent={absent}\ncanonical_existing_different={different}\nexact_unique=190\ngroups={groups:?}\ncanonical_writes=0\nstage_applied=false\nproduct_cargo_run=false\n",
        env::var("COMPUTERNAME").unwrap_or_default()
    );
    io::write_new(&output.join("B0-INPUT-RETURN.txt"), summary.as_bytes())?;
    print!("{summary}");
    Ok(())
}
