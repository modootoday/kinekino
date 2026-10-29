//! `cargo xtask install`: copy an already-built `kinekino` binary into a bin dir and add the
//! `kk` alias as a symbolic link. Never builds, downloads or edits PATH.
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const BIN: &str = "kinekino";
const ALIAS: &str = "kk";

pub fn run(args: &[String]) -> ExitCode {
    let mut from = None;
    let mut bin_dir = None;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--from" => from = it.next().map(PathBuf::from),
            "--bin-dir" => bin_dir = it.next().map(PathBuf::from),
            other => {
                eprintln!(
                    "unknown argument: {other}\nusage: cargo xtask install [--from BINARY] [--bin-dir DIR]"
                );
                return ExitCode::from(2);
            }
        }
    }
    let from = from.unwrap_or_else(default_source);
    let Some(bin_dir) = bin_dir.or_else(default_bin_dir) else {
        eprintln!("cannot determine a bin dir; pass --bin-dir DIR");
        return ExitCode::from(2);
    };
    match install(&from, &bin_dir) {
        Ok((bin, alias)) => {
            println!(
                "installed {} (alias {} -> {})",
                bin.display(),
                alias.display(),
                bin.display()
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("install failed: {error}");
            ExitCode::from(1)
        }
    }
}

fn exe(name: &str) -> String {
    format!("{name}{}", std::env::consts::EXE_SUFFIX)
}

fn default_source() -> PathBuf {
    let target = crate::workspace::target_dir(
        &crate::workspace::root().unwrap_or_else(|_| PathBuf::from(".")),
    );
    target.join("release").join(exe(BIN))
}

fn default_bin_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(PathBuf::from(home).join(".local").join("bin"))
}

/// Copies `from` to `<bin_dir>/kinekino` and (re)creates `<bin_dir>/kk` as a symlink to it.
pub fn install(from: &Path, bin_dir: &Path) -> std::io::Result<(PathBuf, PathBuf)> {
    if !from.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "{} not built; run `cargo build --release -p kinekino-cli` first",
                from.display()
            ),
        ));
    }
    std::fs::create_dir_all(bin_dir)?;
    let bin = bin_dir.join(exe(BIN));
    std::fs::copy(from, &bin)?;
    let alias = bin_dir.join(exe(ALIAS));
    match std::fs::symlink_metadata(&alias) {
        Ok(meta) if meta.file_type().is_symlink() => std::fs::remove_file(&alias)?,
        Ok(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!(
                    "{} exists and is not a symlink; refusing to replace",
                    alias.display()
                ),
            ));
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }
    symlink(Path::new(&exe(BIN)), &alias)?;
    Ok((bin, alias))
}

#[cfg(unix)]
fn symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}
// Windows symlinks need Developer Mode or elevation; unverified until CI.
#[cfg(windows)]
fn symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installs_binary_and_relative_kk_symlink_idempotently() {
        let root = std::env::temp_dir().join(format!("xtask-install-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let src = root.join("built");
        std::fs::write(&src, b"bin").unwrap();
        let dir = root.join("bin");
        for _ in 0..2 {
            let (bin, alias) = install(&src, &dir).unwrap();
            assert_eq!(std::fs::read(&bin).unwrap(), b"bin");
            assert!(
                std::fs::symlink_metadata(&alias)
                    .unwrap()
                    .file_type()
                    .is_symlink()
            );
            assert_eq!(std::fs::read_link(&alias).unwrap(), PathBuf::from(exe(BIN)));
            assert_eq!(std::fs::read(&alias).unwrap(), b"bin");
        }
        std::fs::remove_file(dir.join(exe(ALIAS))).unwrap();
        std::fs::write(dir.join(exe(ALIAS)), b"user file").unwrap();
        assert!(
            install(&src, &dir).is_err(),
            "must not clobber a non-symlink kk"
        );
        std::fs::remove_dir_all(&root).unwrap();
    }
}
