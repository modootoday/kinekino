//! The release payload list: which binaries and repository files each archive carries.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// A release binary; gets `.exe` on Windows targets.
    Bin(String),
    /// A file copied from the repository root.
    File(String),
}

/// Parses `bin NAME` / `file NAME` lines; blank lines and `#` comments are skipped.
/// Names are single path components, and no name may appear twice.
pub fn parse(text: &str) -> Result<Vec<Entry>, String> {
    let mut out: Vec<Entry> = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let at = i + 1;
        let (kind, name) = line
            .split_once(char::is_whitespace)
            .map(|(k, n)| (k, n.trim()))
            .ok_or_else(|| format!("line {at}: expected `bin NAME` or `file NAME`"))?;
        if name.is_empty() || name.contains(['/', '\\', ' ', '\t']) || name == "." || name == ".." {
            return Err(format!("line {at}: `{name}` must be one plain file name"));
        }
        let entry = match kind {
            "bin" => Entry::Bin(name.to_owned()),
            "file" => Entry::File(name.to_owned()),
            other => return Err(format!("line {at}: unknown kind `{other}`")),
        };
        if out.iter().any(|e| e.name() == name) {
            return Err(format!("line {at}: `{name}` is listed twice"));
        }
        out.push(entry);
    }
    if out.is_empty() {
        return Err("the payload list is empty".into());
    }
    Ok(out)
}

impl Entry {
    fn name(&self) -> &str {
        match self {
            Self::Bin(n) | Self::File(n) => n,
        }
    }

    /// File name inside the archive for `target`.
    pub fn archived_name(&self, target: &str) -> String {
        match self {
            Self::Bin(n) if target.contains("windows") => format!("{n}.exe"),
            other => other.name().to_owned(),
        }
    }
}

/// Top folder of the archive for `target`, as package.sh names it.
pub fn top_folder(target: &str) -> String {
    format!("kinekino-{target}")
}

/// Every path the archive must hold besides its top folder, as `top/name`.
pub fn expected(entries: &[Entry], target: &str) -> Vec<String> {
    let top = top_folder(target);
    let mut v: Vec<String> = entries
        .iter()
        .map(|e| format!("{top}/{}", e.archived_name(target)))
        .collect();
    v.sort();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bins_and_files_and_skips_comments() {
        let got = parse("# c\n\nbin kinekino\nfile LICENSE\n").unwrap();
        assert_eq!(
            got,
            vec![Entry::Bin("kinekino".into()), Entry::File("LICENSE".into())]
        );
    }

    #[test]
    fn rejects_bad_lines() {
        for bad in [
            "bin",
            "lib x",
            "file docs/x.md",
            "file ..",
            "bin a\nfile a",
            "# only comments",
        ] {
            assert!(parse(bad).is_err(), "{bad:?} should fail");
        }
    }

    #[test]
    fn windows_binaries_get_exe() {
        let e = parse("bin player\nfile README.md").unwrap();
        assert_eq!(
            expected(&e, "x86_64-pc-windows-msvc"),
            [
                "kinekino-x86_64-pc-windows-msvc/README.md",
                "kinekino-x86_64-pc-windows-msvc/player.exe"
            ]
        );
        assert_eq!(
            expected(&e, "aarch64-apple-darwin"),
            [
                "kinekino-aarch64-apple-darwin/README.md",
                "kinekino-aarch64-apple-darwin/player"
            ]
        );
    }

    #[test]
    fn the_repository_list_parses() {
        let text = include_str!("../../../.github/scripts/package-payload.txt");
        let e = parse(text).unwrap();
        for bin in ["kinekino", "player", "kinekino-vad-worker"] {
            assert!(e.contains(&Entry::Bin(bin.into())), "{bin}");
        }
        assert!(e.contains(&Entry::File("LICENSE".into())));
    }
}
