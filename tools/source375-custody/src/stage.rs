//! Fresh private source closure; never overlay or restore the shared checkout.
use crate::{infrastructure as io, json, paths};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

fn excluded(name: &std::ffi::OsStr) -> bool {
    [
        ".git",
        ".spec",
        ".agents",
        "runtime",
        "target",
        "node_modules",
        ".cache",
    ]
    .iter()
    .any(|v| name == std::ffi::OsStr::new(v))
}
pub(super) fn walk(
    root: &Path,
    directory: &Path,
    files: &mut BTreeMap<PathBuf, PathBuf>,
) -> Result<(), String> {
    io::ordinary(directory)?;
    for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if excluded(&entry.file_name()) {
            continue;
        }
        let path = entry.path();
        io::ordinary(&path)?;
        let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if metadata.is_dir() {
            walk(root, &path, files)?;
        } else if metadata.is_file() {
            files.insert(
                path.strip_prefix(root).map_err(|e| e.to_string())?.into(),
                path,
            );
        } else {
            return Err("nonordinary source closure entry".into());
        }
    }
    Ok(())
}
pub fn run(args: &[String]) -> Result<(), String> {
    if args.len() != 3 {
        return Err("expected stage ROOT B0_RECEIPT_DIR NEW_STAGE_DIR".into());
    }
    let root = io::ordinary(Path::new(&args[0]))?;
    let receipt = io::ordinary(Path::new(&args[1]))?;
    if !receipt.starts_with(root.join("runtime/verification"))
        || !receipt.join("B0-INPUT-RETURN.txt").is_file()
    {
        return Err("completed B0 receipt required".into());
    }
    let historical = receipt.join("historical-source375.json");
    if io::hash(&historical)? != "A09B51ACBC77645FD12D8488B23021173780851E267A8BA5103D34F827294D39"
    {
        return Err("B0 source pin differs".into());
    }
    let manifest = json::parse(&fs::read(historical).map_err(|e| e.to_string())?)?;
    let mut selected = BTreeMap::new();
    for row in manifest.array()? {
        let relative = paths::relative(row.field("relative")?.text()?)?;
        let input = receipt.join("selected").join(&relative);
        let sha = row.field("sha256")?.text()?.to_ascii_uppercase();
        if io::hash(&input)? != sha {
            return Err("frozen selected copy changed".into());
        }
        if selected.insert(relative, (input, sha)).is_some() {
            return Err("duplicate source".into());
        }
    }
    if selected.len() != 375 {
        return Err("375 inputs required".into());
    }
    let destination = PathBuf::from(&args[2]);
    let parent = io::ordinary(destination.parent().ok_or("stage parent missing")?)?;
    if !parent.starts_with(root.join("runtime/development")) {
        return Err("private development stage required".into());
    }
    let mut baseline = BTreeMap::new();
    for directory in ["src", "tests", "assets", "packages"] {
        walk(&root, &root.join(directory), &mut baseline)?;
    }
    for name in ["Cargo.toml", "Cargo.lock", "build.rs", "README.md"] {
        let path = root.join(name);
        if path.is_file() {
            baseline.insert(name.into(), path);
        }
    }
    fs::create_dir(&destination).map_err(|e| e.to_string())?;
    let mut inventory = String::from(
        "relative\trole\tinput_path\tsha256\tinput_fileid\tbytes\tcreation100ns\tmtime100ns\n",
    );
    // Select before copying: no intermediate overwrite even inside the new stage.
    let mut combined = BTreeMap::new();
    for (relative, input) in baseline {
        combined.insert(relative, ("baseline", input, None));
    }
    for (relative, (input, sha)) in selected {
        combined.insert(relative, ("source375", input, Some(sha)));
    }
    let count = combined.len();
    for (index, (relative, (role, input, pin))) in combined.into_iter().enumerate() {
        let sha = match pin {
            Some(pin) => pin,
            None => io::hash(&input)?,
        };
        let identity = io::identity(&input)?;
        io::copy_verified(&input, &destination.join(&relative), &sha)?;
        if io::identity(&input)? != identity {
            return Err(format!("source identity changed: {}", input.display()));
        }
        inventory.push_str(&format!(
            "{}\t{role}\t{}\t{sha}\t{identity}\n",
            relative.display(),
            input.display()
        ));
        if (index + 1) % 250 == 0 {
            println!("source_closure_copied={}/{count}", index + 1);
            std::io::stdout().flush().map_err(|e| e.to_string())?;
        }
    }
    io::write_new(
        &receipt.join("stage-source-closure.tsv"),
        inventory.as_bytes(),
    )?;
    io::write_new(&receipt.join("STAGE-RETURN.txt"),format!("stage={}\nfiles={count}\nselected=375\ncanonical_writes=0\nsource_config_omitted=true\nexplicit_target_env_required=true\n",destination.display()).as_bytes())?;
    println!("stage_complete files={count} selected=375 canonical_writes=0");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_and_cache_boundaries_are_excluded() {
        for name in [
            ".git",
            ".spec",
            ".agents",
            "runtime",
            "target",
            "node_modules",
            ".cache",
        ] {
            assert!(excluded(std::ffi::OsStr::new(name)));
        }
        for name in ["src", "fixtures", "assets", "Cargo.lock", "NOTICE"] {
            assert!(!excluded(std::ffi::OsStr::new(name)));
        }
    }
}
