//! Identity rules: Co-authored-by trailers (shared rules C10.2/C4.8) and commit author and
//! committer (C10.5) use `@users.noreply.github.com` addresses.

const ALLOWED_SUFFIX: &str = "@users.noreply.github.com";
/// GitHub's own committer for merges and web edits made on github.com.
const GITHUB_WEB_COMMITTER: &str = "noreply@github.com";

/// Whether a commit author address is allowed.
pub fn author_ok(addr: &str) -> bool {
    addr.trim().to_ascii_lowercase().ends_with(ALLOWED_SUFFIX)
}

/// Whether a committer address is allowed: an author address, or GitHub's web committer.
pub fn committer_ok(addr: &str) -> bool {
    author_ok(addr) || addr.trim().eq_ignore_ascii_case(GITHUB_WEB_COMMITTER)
}

/// Number of `Co-authored-by` lines in `message` whose address is not allowed.
/// A line without an `<address>` counts as not allowed.
pub fn bad_coauthors(message: &str) -> usize {
    message
        .lines()
        .filter_map(|l| {
            let (key, value) = l.split_once(':')?;
            key.trim()
                .eq_ignore_ascii_case("co-authored-by")
                .then_some(value)
        })
        .filter(|value| !allowed(value))
        .count()
}

fn allowed(value: &str) -> bool {
    let Some((_, rest)) = value.split_once('<') else {
        return false;
    };
    let Some((addr, _)) = rest.split_once('>') else {
        return false;
    };
    addr.trim().to_ascii_lowercase().ends_with(ALLOWED_SUFFIX)
}

/// Unix seconds of a UTC timestamp `YYYY-MM-DDTHH:MM:SSZ`.
pub fn parse_utc(s: &str) -> Result<i64, String> {
    let bad = || format!("bad time {s}; expected YYYY-MM-DDTHH:MM:SSZ");
    let s2 = s.strip_suffix('Z').ok_or_else(bad)?;
    let (date, time) = s2.split_once('T').ok_or_else(bad)?;
    let num = |p: &str| p.parse::<i64>().map_err(|_| bad());
    let d: Vec<&str> = date.split('-').collect();
    let t: Vec<&str> = time.split(':').collect();
    if d.len() != 3 || t.len() != 3 {
        return Err(bad());
    }
    let (y, m, day) = (num(d[0])?, num(d[1])?, num(d[2])?);
    if !(1..=12).contains(&m) || !(1..=31).contains(&day) {
        return Err(bad());
    }
    // Days from civil date (Howard Hinnant's algorithm).
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Ok(days * 86_400 + num(t[0])? * 3600 + num(t[1])? * 60 + num(t[2])?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_github_noreply_coauthors_pass() {
        let ok = "x\n\nCo-authored-by: A <1+a@users.noreply.github.com>\n";
        assert_eq!(bad_coauthors(ok), 0);
        let bad = "x\n\nCo-Authored-By: Tool <noreply@example.com>\nco-authored-by: B\n";
        assert_eq!(bad_coauthors(bad), 2);
        assert_eq!(bad_coauthors("subject only"), 0);
    }

    #[test]
    fn author_and_committer_addresses() {
        assert!(author_ok("1+a@users.noreply.github.com"));
        assert!(!author_ok("noreply@github.com"));
        assert!(!author_ok("someone@example.com"));
        assert!(committer_ok("noreply@github.com"));
        assert!(committer_ok("1+a@Users.Noreply.GitHub.com"));
        assert!(!committer_ok("noreply@example.com"));
    }

    #[test]
    fn parses_utc_timestamps() {
        assert_eq!(parse_utc("1970-01-01T00:00:00Z"), Ok(0));
        assert_eq!(parse_utc("2026-11-16T06:57:52Z"), Ok(1_794_812_272));
        assert!(parse_utc("2026-11-16 06:57:52").is_err());
        assert!(parse_utc("2026-13-01T00:00:00Z").is_err());
    }
}
