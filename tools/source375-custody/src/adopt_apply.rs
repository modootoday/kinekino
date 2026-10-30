//! Apply one reviewed ownership batch with hash/absence and dirty-file guards.
use crate::{infrastructure as io, paths};
use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process::Command,
};

fn dirty(repo: &Path) -> Result<BTreeSet<String>, String> {
    let run = |args: &[&str]| -> Result<Vec<u8>, String> {
        let out = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .output()
            .map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err("Git inventory failed".into());
        }
        Ok(out.stdout)
    };
    if !run(&["diff", "--cached", "--name-only", "-z"])?.is_empty() {
        return Err("existing index entries: stop".into());
    }
    let mut result = BTreeSet::new();
    for args in [
        vec!["diff", "--name-only", "-z"],
        vec!["ls-files", "--others", "--exclude-standard", "-z"],
    ] {
        for p in run(&args)?.split(|b| *b == 0).filter(|p| !p.is_empty()) {
            result.insert(String::from_utf8(p.to_vec()).map_err(|e| e.to_string())?);
        }
    }
    Ok(result)
}
fn current(path: &Path) -> Result<String, String> {
    match fs::symlink_metadata(path) {
        Ok(_) => {
            io::ordinary(path)?;
            io::hash(path)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok("-".into()),
        Err(e) => Err(e.to_string()),
    }
}
pub fn run(args: &[String]) -> Result<(), String> {
    if args.len() != 5 {
        return Err("expected apply-reviewed ROOT STAGE REVIEW OWNER NEW_LOG".into());
    }
    let root = io::ordinary(Path::new(&args[0]))?;
    let stage = io::ordinary(Path::new(&args[1]))?;
    let review = io::ordinary(Path::new(&args[2]))?;
    if !stage.starts_with(root.join("runtime/development"))
        || !review.starts_with(root.join("runtime/verification"))
    {
        return Err("private reviewed inputs required".into());
    }
    let table = review.join("candidates.tsv");
    if io::hash(&table)? != "047982414769F28D041F386344DF95B8D6A6CA901B612A3E8F8CCA7C0BA700ED" {
        return Err("review table changed".into());
    }
    let owner = match args[3].as_str() {
        "root" => "root_pm_owner_review",
        "meari" => "independent_meari",
        _ => return Err("owner must be root or meari".into()),
    };
    let repo = if args[3] == "meari" {
        root.join("vendor/meari")
    } else {
        root.clone()
    };
    let protected = dirty(&repo)?;
    let mut rows = Vec::new();
    for row in fs::read_to_string(&table)
        .map_err(|e| e.to_string())?
        .lines()
        .skip(1)
    {
        let f: Vec<_> = row.split('\t').collect();
        if f.len() != 6 {
            return Err("review row shape".into());
        }
        if f[1] != owner || f[2] == "same_no_write" || f[0] == "src/application/mod.rs" {
            continue;
        }
        let path = paths::relative(f[0])?;
        if (args[3] == "meari") != path.starts_with("vendor/meari") {
            return Err("owner boundary mismatch".into());
        }
        let repo_path = if args[3] == "meari" {
            f[0].strip_prefix("vendor/meari/").ok_or("owner path")?
        } else {
            f[0]
        };
        if protected.contains(repo_path) || f[5] != "false" {
            return Err(format!("protected/drifted canonical: {}", f[0]));
        }
        if current(&root.join(&path))? != f[4]
            || io::hash(&io::ordinary(&stage.join(&path))?)? != f[3]
        {
            return Err(format!("preflight hash conflict: {}", f[0]));
        }
        rows.push((path, f[3].to_string(), f[4].to_string()));
    }
    let expected = if args[3] == "meari" { 56 } else { 113 };
    if rows.len() != expected {
        return Err("reviewed batch count differs".into());
    }
    let output = Path::new(&args[4]);
    if !io::ordinary(output.parent().ok_or("log parent")?)?
        .starts_with(root.join("runtime/verification"))
    {
        return Err("private log required".into());
    }
    fs::create_dir(output).map_err(|e| e.to_string())?;
    let temp = root.join(format!(
        "runtime/build-cache/tmp/source375-apply-{}-{}",
        args[3],
        std::process::id()
    ));
    io::ordinary(temp.parent().ok_or("temp parent")?)?;
    fs::create_dir(&temp).map_err(|e| e.to_string())?;
    // Freeze all selected bytes and preimages before the first canonical write.
    for (path, after, before) in &rows {
        io::copy_verified(
            &stage.join(path),
            &output.join("selected").join(path),
            after,
        )?;
        if before != "-" {
            io::copy_verified(
                &root.join(path),
                &output.join("preimage").join(path),
                before,
            )?;
        }
    }
    let mut journal = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output.join("journal.tsv"))
        .map_err(|e| e.to_string())?;
    for (index, (path, after, before)) in rows.iter().enumerate() {
        let target = root.join(path);
        if current(&target)? != *before {
            return Err(format!("late canonical conflict: {}", path.display()));
        }
        writeln!(journal, "prepare\t{}\t{before}\t{after}", path.display())
            .and_then(|()| journal.sync_all())
            .map_err(|e| e.to_string())?;
        let staged = temp.join(format!("{index}.source"));
        io::copy_verified(&output.join("selected").join(path), &staged, after)?;
        fs::create_dir_all(target.parent().ok_or("target parent")?).map_err(|e| e.to_string())?;
        io::ordinary(target.parent().ok_or("target parent")?)?;
        if before == "-" {
            fs::hard_link(&staged, &target).map_err(|e| e.to_string())?;
            fs::remove_file(&staged).map_err(|e| e.to_string())?;
        } else {
            fs::rename(&staged, &target).map_err(|e| e.to_string())?;
        }
        if io::hash(&target)? != *after {
            return Err("published hash mismatch".into());
        }
        writeln!(
            journal,
            "applied\t{}\t{after}\t{}",
            path.display(),
            io::identity(&target)?
        )
        .and_then(|()| journal.sync_all())
        .map_err(|e| e.to_string())?;
    }
    io::write_new(&output.join("RETURN.txt"),format!("owner={owner}\napplied={}\nindex_writes=0\nappmod_skipped_for_minimal_patch=true\nno_automatic_rollback=true\nnew_canonical_validation_required=true\n",rows.len()).as_bytes())?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dirty_guard_reports_untracked_and_refuses_existing_index_entries() {
        let repo = std::env::temp_dir().join(format!("adoption-git-guard-{}", std::process::id()));
        fs::create_dir(&repo).unwrap();
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .arg(&repo)
                .status()
                .unwrap()
                .success()
        );
        fs::write(repo.join("owned.rs"), "preserved WIP").unwrap();
        assert!(dirty(&repo).unwrap().contains("owned.rs"));
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(&repo)
                .args(["add", "--", "owned.rs"])
                .status()
                .unwrap()
                .success()
        );
        assert!(dirty(&repo).is_err());
        assert_eq!(
            fs::read_to_string(repo.join("owned.rs")).unwrap(),
            "preserved WIP"
        );
    }
}
