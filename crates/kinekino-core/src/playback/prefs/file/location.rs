//! Default player prefs file location from an injected environment lookup (never reads `std::env` here).
use std::path::PathBuf;

/// `KINEKINO_PLAYER_PREFS_FILE`, else `XDG_CONFIG_HOME/kinekino/player-prefs.json`, else
/// `HOME/.config/kinekino/player-prefs.json`, else `APPDATA/kinekino/player-prefs.json`.
/// Empty values count as unset.
pub fn default_player_prefs_file(get: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    let var = |name: &str| get(name).filter(|v| !v.is_empty()).map(PathBuf::from);
    if let Some(explicit) = var("KINEKINO_PLAYER_PREFS_FILE") {
        return Some(explicit);
    }
    let base = var("XDG_CONFIG_HOME")
        .or_else(|| var("HOME").map(|h| h.join(".config")))
        .or_else(|| var("APPDATA"))?;
    Some(base.join("kinekino").join("player-prefs.json"))
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
        ("KINEKINO_PLAYER_PREFS_FILE", "/o/custom.json"),
        ("XDG_CONFIG_HOME", "/x/config"),
        ("HOME", "/h/user"),
        ("APPDATA", "/a/app"),
    ];

    #[test]
    fn explicit_override_wins() {
        assert_eq!(
            default_player_prefs_file(lookup(&ALL)).as_deref(),
            Some(Path::new("/o/custom.json"))
        );
    }

    #[test]
    fn xdg_config_beats_home() {
        assert_eq!(
            default_player_prefs_file(lookup(&ALL[1..])),
            Some(
                Path::new("/x/config")
                    .join("kinekino")
                    .join("player-prefs.json")
            )
        );
    }

    #[test]
    fn home_falls_back_to_config() {
        assert_eq!(
            default_player_prefs_file(lookup(&ALL[2..])),
            Some(
                Path::new("/h/user")
                    .join(".config")
                    .join("kinekino")
                    .join("player-prefs.json")
            )
        );
    }

    #[test]
    fn app_data_is_last_resort() {
        assert_eq!(
            default_player_prefs_file(lookup(&ALL[3..])),
            Some(
                Path::new("/a/app")
                    .join("kinekino")
                    .join("player-prefs.json")
            )
        );
    }

    #[test]
    fn nothing_set_gives_none() {
        assert_eq!(default_player_prefs_file(lookup(&[])), None);
    }
}
