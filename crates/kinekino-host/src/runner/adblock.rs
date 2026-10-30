//! Offline ad-block rule evaluation (plan P3, inventory #6): the same engine and bundled rules
//! the browser request filter uses, without a browser.
use kinekino_player::infrastructure::adblock;

/// Number of non-empty, non-comment lines in the bundled rule list.
pub fn bundled_rule_count() -> usize {
    adblock::RULES
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('!'))
        .count()
}

/// Whether a request would be blocked by the bundled rules plus `extra` rules.
pub fn check(extra: &str, url: &str, source: &str, kind: &str, method: &str) -> bool {
    adblock::blocked(&adblock::engine(extra), url, source, kind, method)
}
