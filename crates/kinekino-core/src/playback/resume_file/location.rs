//! Default resume file location from an injected environment lookup (never reads `std::env` here).
use std::path::PathBuf;

/// `KINEKINO_RESUME_FILE`, else `XDG_STATE_HOME/kinekino/resume.json`, else
/// `HOME/.local/state/kinekino/resume.json`, else `LOCALAPPDATA/kinekino/resume.json`.
/// Empty values count as unset.
pub fn default_resume_file(get: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    let var = |name: &str| get(name).filter(|v| !v.is_empty()).map(PathBuf::from);
    if let Some(explicit) = var("KINEKINO_RESUME_FILE") {
        return Some(explicit);
    }
    let base = var("XDG_STATE_HOME")
        .or_else(|| var("HOME").map(|h| h.join(".local").join("state")))
        .or_else(|| var("LOCALAPPDATA"))?;
    Some(base.join("kinekino").join("resume.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn lookup<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |k| {
            pairs
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| (*v).to_owned())
        }
    }

    const ALL: [(&str, &str); 4] = [
        ("KINEKINO_RESUME_FILE", "/o/custom.json"),
        ("XDG_STATE_HOME", "/x/state"),
        ("HOME", "/h/user"),
        ("LOCALAPPDATA", "/l/app"),
    ];

    #[test]
    fn explicit_override_wins() {
        assert_eq!(
            default_resume_file(lookup(&ALL)).as_deref(),
            Some(Path::new("/o/custom.json"))
        );
    }

    #[test]
    fn xdg_state_home_beats_home() {
        assert_eq!(
            default_resume_file(lookup(&ALL[1..])),
            Some(Path::new("/x/state").join("kinekino").join("resume.json"))
        );
    }

    #[test]
    fn home_falls_back_to_local_state() {
        assert_eq!(
            default_resume_file(lookup(&ALL[2..])),
            Some(
                Path::new("/h/user")
                    .join(".local")
                    .join("state")
                    .join("kinekino")
                    .join("resume.json")
            )
        );
    }

    #[test]
    fn local_app_data_is_last_resort() {
        assert_eq!(
            default_resume_file(lookup(&ALL[3..])),
            Some(Path::new("/l/app").join("kinekino").join("resume.json"))
        );
    }

    #[test]
    fn nothing_set_gives_none() {
        assert_eq!(default_resume_file(lookup(&[])), None);
    }
}
