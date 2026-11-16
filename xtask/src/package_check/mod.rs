//! package-check: the release archive holds exactly the payload list (ADR 20261109101726,
//! slice b). package.sh builds the archive from `.github/scripts/package-payload.txt`; this
//! check lists the finished archive and fails on a missing or an unexpected file, so the
//! archive, the list and a later installable package cannot drift apart.
mod archive;
mod payload;

use std::path::PathBuf;
use std::process::ExitCode;

const DEFAULT_PAYLOAD: &str = ".github/scripts/package-payload.txt";
const USAGE: &str = "usage: package-check --target TRIPLE --archive PATH [--payload FILE]";

pub fn run(args: &[String]) -> ExitCode {
    match parse(args).and_then(|o| check(&o)) {
        Ok(n) => {
            println!("package-check: {n} file(s) match the payload list");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("package-check: {e}");
            ExitCode::FAILURE
        }
    }
}

struct Opts {
    target: String,
    archive: PathBuf,
    payload: PathBuf,
}

fn parse(args: &[String]) -> Result<Opts, String> {
    let (mut target, mut archive) = (None, None);
    let mut payload = PathBuf::from(DEFAULT_PAYLOAD);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut value = || it.next().cloned().ok_or_else(|| USAGE.to_owned());
        match a.as_str() {
            "--target" => target = Some(value()?),
            "--archive" => archive = Some(PathBuf::from(value()?)),
            "--payload" => payload = PathBuf::from(value()?),
            _ => return Err(USAGE.into()),
        }
    }
    Ok(Opts {
        target: target.ok_or(USAGE)?,
        archive: archive.ok_or(USAGE)?,
        payload,
    })
}

fn check(o: &Opts) -> Result<usize, String> {
    let text = std::fs::read_to_string(&o.payload)
        .map_err(|e| format!("read {}: {e}", o.payload.display()))?;
    let entries = payload::parse(&text)?;
    let want = payload::expected(&entries, &o.target);
    let got = archive::list(&o.archive)?;
    compare(&want, &got, &payload::top_folder(&o.target))?;
    Ok(want.len())
}

/// Compares listed archive paths with the expected ones. Directory entries for the top
/// folder are ignored; anything else outside the list, or a missing file, is an error.
fn compare(want: &[String], listed: &[String], top: &str) -> Result<(), String> {
    let mut got: Vec<String> = listed
        .iter()
        .map(|p| p.trim_start_matches("./").trim_end_matches('/').to_owned())
        .filter(|p| !p.is_empty() && p != top)
        .collect();
    got.sort();
    got.dedup();
    let missing: Vec<&String> = want.iter().filter(|w| !got.contains(w)).collect();
    let extra: Vec<&String> = got.iter().filter(|g| !want.contains(g)).collect();
    if missing.is_empty() && extra.is_empty() {
        return Ok(());
    }
    let mut msg = String::from("the archive does not match the payload list");
    if !missing.is_empty() {
        msg += &format!("; missing: {missing:?}");
    }
    if !extra.is_empty() {
        msg += &format!("; not in the list: {extra:?}");
    }
    Err(msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| (*x).to_owned()).collect()
    }

    #[test]
    fn matching_listing_passes_with_folder_entries_and_dot_prefix() {
        let want = s(&["k/LICENSE", "k/player"]);
        assert!(compare(&want, &s(&["k/", "./k/player", "k/LICENSE"]), "k").is_ok());
    }

    #[test]
    fn missing_and_extra_files_fail() {
        let want = s(&["k/LICENSE", "k/player"]);
        let err = compare(&want, &s(&["k/", "k/player", "k/notes.txt"]), "k").unwrap_err();
        assert!(err.contains("missing: [\"k/LICENSE\"]"), "{err}");
        assert!(err.contains("not in the list: [\"k/notes.txt\"]"), "{err}");
    }

    #[test]
    fn zip_archive_end_to_end() {
        let dir = std::env::temp_dir().join(format!("kk-pkgcheck-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let list = dir.join("payload.txt");
        std::fs::write(&list, "bin player\nfile LICENSE\n").unwrap();
        let t = "x86_64-pc-windows-msvc";
        let zip = dir.join("k.zip");
        let top = payload::top_folder(t);
        let names = [
            format!("{top}/"),
            format!("{top}/player.exe"),
            format!("{top}/LICENSE"),
        ];
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        std::fs::write(&zip, archive::tests::central_only_zip(&refs)).unwrap();
        let opts = |archive: &std::path::Path| Opts {
            target: t.into(),
            archive: archive.to_owned(),
            payload: list.clone(),
        };
        assert_eq!(check(&opts(&zip)), Ok(2));
        let short = dir.join("short.zip");
        std::fs::write(&short, archive::tests::central_only_zip(&refs[..2])).unwrap();
        assert!(check(&opts(&short)).unwrap_err().contains("missing"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn options_need_target_and_archive() {
        assert!(parse(&s(&["--target", "x"])).is_err());
        assert!(parse(&s(&["--archive", "a.zip", "--bogus"])).is_err());
        let o = parse(&s(&["--target", "x", "--archive", "a.tar.gz"])).unwrap();
        assert_eq!(o.payload, PathBuf::from(DEFAULT_PAYLOAD));
    }
}
