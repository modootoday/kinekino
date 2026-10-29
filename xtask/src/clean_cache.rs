//! clean-cache: keeps the Cargo target dir under a size cap (guide:kinekino/build-cache).
//! Stages, stopping as soon as the cap holds: 1) drop `incremental/`, 2) drop superseded
//! `deps/` artifacts (older hashes of the same crate+kind), 3) drop whole profile dirs.
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::SystemTime;

const DEFAULT_CAP: u64 = 8 << 30;
const PROFILES: [&str; 2] = ["debug", "release"];

pub fn run(args: &[String]) -> ExitCode {
    match parse(args).and_then(|o| clean(&o)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("clean-cache: {e}");
            ExitCode::FAILURE
        }
    }
}

struct Opts {
    dir: PathBuf,
    cap: u64,
    dry_run: bool,
}

fn parse(args: &[String]) -> Result<Opts, String> {
    let mut o = Opts {
        dir: default_dir(),
        cap: DEFAULT_CAP,
        dry_run: false,
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--max-size" => o.cap = parse_size(it.next().ok_or("--max-size needs a value")?)?,
            "--target-dir" => o.dir = PathBuf::from(it.next().ok_or("--target-dir needs a value")?),
            "--dry-run" => o.dry_run = true,
            _ => {
                return Err(format!(
                    "unknown argument {a}; usage: clean-cache [--max-size 8G] [--target-dir DIR] [--dry-run]"
                ));
            }
        }
    }
    Ok(o)
}

fn default_dir() -> PathBuf {
    let root = crate::workspace::root().unwrap_or_else(|_| PathBuf::from("."));
    crate::workspace::target_dir(&root)
}

pub fn parse_size(s: &str) -> Result<u64, String> {
    let (num, unit) = s.split_at(s.find(|c: char| c.is_ascii_alphabetic()).unwrap_or(s.len()));
    let n: f64 = num.trim().parse().map_err(|_| format!("bad size {s}"))?;
    let mul = match unit.to_ascii_uppercase().as_str() {
        "" | "B" => 1u64,
        "K" | "KB" => 1 << 10,
        "M" | "MB" => 1 << 20,
        "G" | "GB" => 1 << 30,
        _ => return Err(format!("bad size unit in {s}")),
    };
    Ok((n * mul as f64) as u64)
}

fn clean(o: &Opts) -> Result<(), String> {
    if !o.dir.is_dir() {
        println!(
            "clean-cache: {} does not exist; nothing to do",
            o.dir.display()
        );
        return Ok(());
    }
    let start = size_of(&o.dir);
    println!(
        "clean-cache: {} = {} (cap {})",
        o.dir.display(),
        human(start),
        human(o.cap)
    );
    let mut now = start;
    let profiles: Vec<PathBuf> = PROFILES
        .iter()
        .map(|p| o.dir.join(p))
        .filter(|p| p.is_dir())
        .collect();
    for p in &profiles {
        if now <= o.cap {
            break;
        }
        now = now.saturating_sub(remove(&p.join("incremental"), o.dry_run));
    }
    for p in &profiles {
        if now <= o.cap {
            break;
        }
        for f in superseded(&p.join("deps")) {
            now = now.saturating_sub(remove(&f, o.dry_run));
        }
    }
    for p in oldest_first(profiles) {
        if now <= o.cap {
            break;
        }
        now = now.saturating_sub(remove(&p, o.dry_run));
    }
    let verb = if o.dry_run { "would free" } else { "freed" };
    println!(
        "clean-cache: {verb} {}; now {}",
        human(start.saturating_sub(now)),
        human(now)
    );
    Ok(())
}

/// Older-hash artifacts of each crate (`lib<name>-<hash>.<ext>`): every artifact kind keeps
/// only its newest hash. Cargo rebuilds anything it still needs.
fn superseded(deps: &Path) -> Vec<PathBuf> {
    let mut groups: HashMap<(String, String), Vec<(SystemTime, PathBuf)>> = HashMap::new();
    for e in fs::read_dir(deps).into_iter().flatten().flatten() {
        let path = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        let (stem, ext) = name.split_once('.').unwrap_or((&name, ""));
        let Some((krate, hash)) = stem.rsplit_once('-') else {
            continue;
        };
        if hash.len() != 16 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            continue;
        }
        let mtime = e
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        groups
            .entry((krate.to_owned(), ext.to_owned()))
            .or_default()
            .push((mtime, path));
    }
    let mut out = Vec::new();
    for mut v in groups.into_values() {
        v.sort();
        v.pop();
        out.extend(v.into_iter().map(|(_, p)| p));
    }
    out
}

fn oldest_first(mut dirs: Vec<PathBuf>) -> Vec<PathBuf> {
    dirs.sort_by_key(|d| {
        fs::metadata(d)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH)
    });
    dirs
}

fn remove(p: &Path, dry_run: bool) -> u64 {
    if !p.exists() {
        return 0;
    }
    let n = size_of(p);
    if !dry_run {
        let r = if p.is_dir() {
            fs::remove_dir_all(p)
        } else {
            fs::remove_file(p)
        };
        if let Err(e) = r {
            eprintln!("clean-cache: cannot remove {}: {e}", p.display());
            return 0;
        }
    }
    n
}

fn size_of(p: &Path) -> u64 {
    let Ok(m) = fs::symlink_metadata(p) else {
        return 0;
    };
    if !m.is_dir() {
        return m.len();
    }
    fs::read_dir(p)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| size_of(&e.path()))
        .sum()
}

fn human(n: u64) -> String {
    format!("{:.1}G", n as f64 / (1u64 << 30) as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_parse_with_units() {
        assert_eq!(parse_size("8G").unwrap(), 8 << 30);
        assert_eq!(parse_size("500M").unwrap(), 500 << 20);
        assert_eq!(parse_size("1.5g").unwrap(), 3 << 29);
        assert!(parse_size("x").is_err());
    }

    #[test]
    fn superseded_keeps_newest_hash_per_kind() {
        let d = std::env::temp_dir().join(format!("kk-clean-cache-{}", std::process::id()));
        fs::create_dir_all(&d).unwrap();
        for n in [
            "libfoo-0000000000000001.rlib",
            "libfoo-0000000000000001.rmeta",
        ] {
            fs::write(d.join(n), b"x").unwrap();
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
        for n in [
            "libfoo-0000000000000002.rlib",
            "foo-0000000000000002.d",
            "build-script",
        ] {
            fs::write(d.join(n), b"x").unwrap();
        }
        let mut got: Vec<_> = superseded(&d)
            .iter()
            .map(|p| p.file_name().unwrap().to_owned())
            .collect();
        got.sort();
        fs::remove_dir_all(&d).unwrap();
        assert_eq!(got, ["libfoo-0000000000000001.rlib"]);
    }
}
