//! Read-only bounded current source inventory; does not adopt the old overlay.
use crate::{infrastructure as io, stage};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
fn inventory(root: &Path) -> Result<String, String> {
    let mut files = BTreeMap::new();
    for name in ["src", "tests", "assets", "packages"] {
        stage::walk(root, &root.join(name), &mut files)?;
    }
    for name in [
        "Cargo.toml",
        "Cargo.lock",
        "build.rs",
        "README.md",
        "AGENTS.md",
        ".cargo/config.toml",
        "rust-toolchain",
        "rust-toolchain.toml",
    ] {
        let path = root.join(name);
        if path.is_file() {
            io::ordinary(&path)?;
            files.insert(PathBuf::from(name), path);
        }
    }
    let mut result = String::from("relative\tsha256\tfileid\tbytes\tcreation100ns\tmtime100ns\n");
    let total = files.len();
    for (index, (relative, path)) in files.into_iter().enumerate() {
        let identity = io::identity(&path)?;
        let hash = io::hash(&path)?;
        if io::identity(&path)? != identity {
            return Err(format!("source changed during hash: {}", path.display()));
        }
        let relative = relative.to_string_lossy().replace('\\', "/");
        if relative.contains(['\t', '\r', '\n']) {
            return Err("unrepresentable source path".into());
        }
        result.push_str(&format!("{relative}\t{hash}\t{identity}\n"));
        if (index + 1) % 250 == 0 {
            println!("current source {}/{total}", index + 1);
        }
    }
    Ok(result)
}
pub fn capture(args: &[String]) -> Result<(), String> {
    if args.len() != 2 {
        return Err("expected snapshot-current ROOT NEW_RECEIPT_DIR".into());
    }
    let root = io::ordinary(Path::new(&args[0]))?;
    let output = PathBuf::from(&args[1]);
    let parent = io::ordinary(output.parent().ok_or("receipt parent missing")?)?;
    if !parent.starts_with(root.join("runtime/verification")) {
        return Err("private verification receipt required".into());
    }
    fs::create_dir(&output).map_err(|e| e.to_string())?;
    let source = inventory(&root)?;
    io::write_new(&output.join("source-current.tsv"), source.as_bytes())?;
    let hash = io::hash(&output.join("source-current.tsv"))?;
    io::write_new(&output.join("SOURCE.md"),format!("# Current canonical source snapshot\n\nroot={}\nsource-current.tsv SHA256={}\nfiles={}\n\nBounded src/tests/assets/packages plus root Cargo/lock/build/README/AGENTS/toolchain/config. Private runtime/media/model evidence is not copied or inferred. Original source375 and prior canonical baseline remain distinct.\n",root.display(),hash,source.lines().count()-1).as_bytes())?;
    println!(
        "captured current source files={} SHA256={hash}",
        source.lines().count() - 1
    );
    Ok(())
}
pub fn verify(args: &[String]) -> Result<(), String> {
    if args.len() != 3 {
        return Err(
            "expected verify-current ROOT RECEIPT_DIR EXPECTED_SOURCE_RECEIPT_SHA256".into(),
        );
    }
    let root = io::ordinary(Path::new(&args[0]))?;
    let receipt = io::ordinary(Path::new(&args[1]))?;
    if !receipt.starts_with(root.join("runtime/verification")) {
        return Err("private verification receipt required".into());
    }
    if args[2].len() != 64
        || !args[2].bytes().all(|b| b.is_ascii_hexdigit())
        || io::hash(&receipt.join("SOURCE.md"))? != args[2].to_ascii_uppercase()
    {
        return Err("source receipt pin mismatch".into());
    }
    let source_receipt =
        fs::read_to_string(receipt.join("SOURCE.md")).map_err(|e| e.to_string())?;
    let inventory_pin = io::hash(&receipt.join("source-current.tsv"))?;
    if !source_receipt
        .lines()
        .any(|line| line == format!("source-current.tsv SHA256={inventory_pin}"))
    {
        return Err("source inventory pin mismatch".into());
    }
    let before =
        fs::read_to_string(receipt.join("source-current.tsv")).map_err(|e| e.to_string())?;
    let current = inventory(&root)?;
    if before != current {
        return Err("current source path/content/identity differs from captured input".into());
    }
    println!(
        "current source content/identity unchanged files={}",
        current.lines().count() - 1
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_receipt_pins_inventory_and_detects_content_or_path_drift() {
        let root =
            std::env::temp_dir().join(format!("current-source-snapshot-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        for name in ["src", "tests", "assets", "packages", "runtime/verification"] {
            fs::create_dir_all(root.join(name)).unwrap();
        }
        fs::write(root.join("src/input.rs"), b"original source").unwrap();
        let receipt = root.join("runtime/verification/capture");
        let args = vec![
            root.to_str().unwrap().into(),
            receipt.to_str().unwrap().into(),
        ];
        capture(&args).unwrap();
        assert!(capture(&args).is_err());
        let mut verify_args = args.clone();
        verify_args.push(io::hash(&receipt.join("SOURCE.md")).unwrap());
        verify(&verify_args).unwrap();
        fs::write(root.join("src/new.rs"), b"new source").unwrap();
        assert!(verify(&verify_args).unwrap_err().contains("differs"));
        fs::write(receipt.join("source-current.tsv"), b"tampered inventory").unwrap();
        assert!(
            verify(&verify_args)
                .unwrap_err()
                .contains("inventory pin mismatch")
        );
        verify_args[2] = "0".repeat(64);
        assert!(
            verify(&verify_args)
                .unwrap_err()
                .contains("receipt pin mismatch")
        );
    }
}
