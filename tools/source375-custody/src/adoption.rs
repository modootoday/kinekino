//! Read-only candidate comparison. No canonical apply, Git stage or commit.
use crate::{infrastructure as io, json, paths};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

fn deltas(text: &str) -> Result<BTreeMap<String, (String, String)>, String> {
    let mut result = BTreeMap::new();
    for row in text.lines().skip(1) {
        let fields: Vec<_> = row.split('\t').collect();
        if fields.len() != 3
            || fields[1..]
                .iter()
                .any(|s| s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return Err("invalid delta row".into());
        }
        paths::relative(fields[0])?;
        if result
            .insert(
                fields[0].replace('\\', "/"),
                (fields[1].into(), fields[2].into()),
            )
            .is_some()
        {
            return Err("duplicate delta".into());
        }
    }
    if result.len() != 7 {
        return Err("r9 requires seven deltas".into());
    }
    Ok(result)
}
pub fn run(args: &[String]) -> Result<(), String> {
    run_mode(args, false)
}
pub fn canonical(args: &[String]) -> Result<(), String> {
    if args.len() != 4 {
        return Err("expected canonical-audit ROOT B0 DELTA_TSV NEW_OUT".into());
    }
    run_mode(
        &[
            args[0].clone(),
            args[0].clone(),
            args[1].clone(),
            args[2].clone(),
            args[3].clone(),
        ],
        true,
    )
}
fn run_mode(args: &[String], canonical: bool) -> Result<(), String> {
    if args.len() != 5 {
        return Err("expected adoption ROOT STAGE B0 DELTA_TSV NEW_OUT".into());
    }
    let root = io::ordinary(Path::new(&args[0]))?;
    let stage = io::ordinary(Path::new(&args[1]))?;
    let b0 = io::ordinary(Path::new(&args[2]))?;
    if !(stage.starts_with(root.join("runtime/development")) || canonical && stage == root)
        || !b0.starts_with(root.join("runtime/verification"))
    {
        return Err("private stage/B0 required".into());
    }
    let source = b0.join("historical-source375.json");
    let closure = b0.join("stage-source-closure.tsv");
    let receipt = root.join("runtime/development/source375-jobs-recovery-fix-r1/INPUT-R9.md");
    for (path, sha) in [
        (
            &source,
            "A09B51ACBC77645FD12D8488B23021173780851E267A8BA5103D34F827294D39",
        ),
        (
            &closure,
            "C940DFC7860CA6A43EFDA51EBD5C076827C69BA922E8B1365B4DD70C150F204E",
        ),
        (
            &receipt,
            "2E6BFA809A8554EBBCB82611997B16C075F28BD231558A76CA516F3341FB216B",
        ),
    ] {
        if io::hash(path)? != sha {
            return Err("parent/r9 pin changed".into());
        }
    }
    let delta = deltas(&fs::read_to_string(&args[3]).map_err(|e| e.to_string())?)?;
    let mut expected = BTreeMap::new();
    for row in fs::read_to_string(&closure)
        .map_err(|e| e.to_string())?
        .lines()
        .skip(1)
    {
        let f: Vec<_> = row.split('\t').collect();
        if f.len() != 8 {
            return Err("closure row shape".into());
        }
        let name = f[0].replace('\\', "/");
        let after = if let Some((before, after)) = delta.get(&name) {
            if before != f[3] {
                return Err(format!("delta preimage mismatch: {name}"));
            }
            after.clone()
        } else {
            f[3].to_string()
        };
        expected.insert(name, (f[3].to_string(), after));
    }
    if expected.len() != 2103 || delta.keys().any(|p| !expected.contains_key(p)) {
        return Err("closure/delta count".into());
    }
    let manifest = json::parse(&fs::read(&source).map_err(|e| e.to_string())?)?;
    let mut candidates = BTreeSet::new();
    for row in manifest.array()? {
        candidates.insert(row.field("relative")?.text()?.replace('\\', "/"));
    }
    candidates.extend(delta.keys().cloned());
    let mut canonical_before = BTreeMap::new();
    for row in fs::read_to_string(b0.join("custody-preimage.tsv"))
        .map_err(|e| e.to_string())?
        .lines()
        .skip(1)
    {
        let f: Vec<_> = row.split('\t').collect();
        if f.len() != 13 {
            return Err("B0 row shape".into());
        }
        canonical_before.insert(f[0].replace('\\', "/"), f[8].to_string());
    }
    let output = Path::new(&args[4]);
    if !io::ordinary(output.parent().ok_or("output parent")?)?
        .starts_with(root.join("runtime/verification"))
    {
        return Err("private output required".into());
    }
    fs::create_dir(output).map_err(|e| e.to_string())?;
    let mut verified = String::from("relative\tsha256\tfileid\tbytes\tcreation100ns\tmtime100ns\n");
    for (name, (_, sha)) in &expected {
        let path = io::ordinary(&stage.join(name))?;
        if !path.starts_with(&stage) || io::hash(&path)? != *sha {
            return Err(format!("stage drift: {name}"));
        }
        verified.push_str(&format!("{name}\t{sha}\t{}\n", io::identity(&path)?));
    }
    io::write_new(&output.join("verified-stage.tsv"), verified.as_bytes())?;
    let mut table = String::from(
        "relative\towner_boundary\tstatus\tcandidate_sha256\tcanonical_sha256\tcanonical_drift_from_B0\n",
    );
    let mut counts = BTreeMap::<String, usize>::new();
    for name in candidates {
        let (baseline, after) = expected.get(&name).ok_or("candidate outside closure")?;
        let path = root.join(paths::relative(&name)?);
        let current = match fs::symlink_metadata(&path) {
            Ok(_) => {
                io::ordinary(&path)?;
                io::hash(&path)?
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => "-".into(),
            Err(e) => return Err(e.to_string()),
        };
        let before = canonical_before.get(&name).unwrap_or(baseline);
        let status = if current == *after {
            "same_no_write"
        } else if current == "-" {
            "new_candidate"
        } else {
            "different_review_required"
        };
        let owner = if name.starts_with("vendor/meari/") {
            "independent_meari"
        } else {
            "root_pm_owner_review"
        };
        *counts.entry(format!("{owner}/{status}")).or_default() += 1;
        table.push_str(&format!(
            "{name}\t{owner}\t{status}\t{after}\t{current}\t{}\n",
            current != *before
        ));
        if status == "different_review_required" {
            io::copy_verified(&path, &output.join("preimage").join(&name), &current)?;
        }
    }
    io::write_new(&output.join("candidates.tsv"), table.as_bytes())?;
    io::copy_verified(
        Path::new(&args[3]),
        &output.join("deltas.tsv"),
        &io::hash(Path::new(&args[3]))?,
    )?;
    io::write_new(&output.join("RETURN.txt"),format!("r9_full_known_closure_verified=2103\nsource_root={}\nsource_is_canonical={canonical}\ncounts={counts:?}\ncanonical_writes=0\nindex_writes=0\napproval_or_ownership_not_implied=true\nextra_source_inventory_not_checked=true\n",stage.display()).as_bytes())?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn delta_input_refuses_partial_invalid_duplicate_and_escaping_rows() {
        let sha = "A".repeat(64);
        let valid = format!(
            "relative\tbefore\tafter\n{}",
            (0..7)
                .map(|i| format!("src/test{i}.rs\t{sha}\t{sha}\n"))
                .collect::<String>()
        );
        assert_eq!(deltas(&valid).unwrap().len(), 7);
        assert!(deltas("header\n").is_err());
        assert!(deltas(&valid.replace("src/test0.rs", "../test0.rs")).is_err());
        assert!(deltas(&valid.replace("src/test1.rs", "src/test0.rs")).is_err());
        assert!(deltas(&valid.replace(&sha, "xyz")).is_err());
    }
}
