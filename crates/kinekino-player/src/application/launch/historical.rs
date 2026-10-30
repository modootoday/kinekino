use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct HistoricalVerifierConfig {
    pub executable: PathBuf,
    pub executable_sha256: String,
    pub workspace: PathBuf,
    pub package: String,
}

pub fn historical_verifier(args: &[String]) -> anyhow::Result<Option<HistoricalVerifierConfig>> {
    let options = [
        "--historical-verifier",
        "--historical-verifier-sha256",
        "--historical-workspace",
        "--historical-package",
    ];
    let mut values = std::collections::BTreeMap::new();
    for (index, arg) in args.iter().enumerate() {
        for option in options {
            let value = if arg == option {
                Some(
                    args.get(index + 1)
                        .map(String::as_str)
                        .ok_or_else(|| anyhow::anyhow!("historical option requires a value"))?,
                )
            } else {
                arg.strip_prefix(&format!("{option}="))
            };
            if let Some(value) = value {
                anyhow::ensure!(
                    !value.is_empty()
                        && !value.starts_with("--")
                        && values.insert(option, value).is_none(),
                    "invalid or duplicate historical option"
                );
            }
        }
    }
    if values.is_empty() {
        return Ok(None);
    }
    anyhow::ensure!(
        values.len() == options.len(),
        "all historical verifier options are required"
    );
    anyhow::ensure!(
        super::proposal_root(args)?.is_some()
            && !args
                .iter()
                .any(|arg| arg == "--endpoint" || arg == "--ui-demo"),
        "historical verifier requires local proposal storage"
    );
    let executable = PathBuf::from(values[options[0]]);
    let executable_sha256 = values[options[1]].to_owned();
    let workspace = PathBuf::from(values[options[2]]);
    let package = values[options[3]].to_owned();
    anyhow::ensure!(
        executable.is_absolute()
            && workspace.is_absolute()
            && crate::domain::identifiers::valid_sha256(&executable_sha256),
        "invalid historical verifier path or pin"
    );
    anyhow::ensure!(
        !package.is_empty()
            && package.len() <= 80
            && package
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-'),
        "invalid historical package label"
    );
    Ok(Some(HistoricalVerifierConfig {
        executable,
        executable_sha256,
        workspace,
        package,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verifier_is_explicit_complete_local_and_not_preview_configuration() {
        let args = |values: &[&str]| values.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(historical_verifier(&[]).unwrap().is_none());
        assert!(historical_verifier(&args(&["--historical-package", "test"])).is_err());
        let root = std::env::temp_dir();
        let root = root.to_str().unwrap();
        let pin = "a".repeat(64);
        let mut input = args(&[
            "--proposal-root",
            root,
            "--historical-verifier",
            root,
            "--historical-verifier-sha256",
            &pin,
            "--historical-workspace",
            root,
            "--historical-package",
            "test",
        ]);
        assert!(historical_verifier(&input).unwrap().is_some());
        input.push("--ui-demo".into());
        assert!(historical_verifier(&input).is_err());
    }
}
