use url::Url;

use super::admitted_hosts::active_hosts;

pub fn navigation_allowed(raw: &str) -> bool {
    if super::player_document::PlayerDocumentLocation::parse(raw).is_ok() {
        return true;
    }
    Url::parse(raw).is_ok_and(|u| {
        secure_https(&u) && u.host_str().is_some_and(|host| active_hosts().admits(host))
    })
}

pub fn media_candidate(raw: &str) -> bool {
    Url::parse(raw).is_ok_and(|u| {
        secure_https(&u)
            && [".m3u8", ".mpd", ".mp4", ".webm"]
                .iter()
                .any(|s| u.path().to_ascii_lowercase().ends_with(s))
    })
}

pub fn media_transport_allowed(raw: &str) -> bool {
    Url::parse(raw).is_ok_and(|url| secure_https(&url))
}
fn secure_https(url: &Url) -> bool {
    url.scheme() == "https"
        && url.port_or_known_default() == Some(443)
        && url.username().is_empty()
        && url.password().is_none()
}

/// Catalogue pages cannot activate a native player from late media callbacks.
///
/// Document-only origins (listed in `admits_player` but not `admits`) qualify solely via
/// [`PlayerDocumentLocation`]; general player hosts need both admission lists.
pub fn player_page(value: &str) -> bool {
    if super::player_document::PlayerDocumentLocation::parse(value).is_ok() {
        return true;
    }
    Url::parse(value).is_ok_and(|u| {
        secure_https(&u)
            && u.host_str().is_some_and(|host| {
                let hosts = active_hosts();
                hosts.admits(host) && hosts.admits_player(host)
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::admitted_hosts::{AdmittedHosts, with_admitted_hosts};

    #[test]
    fn secondary_cdn_is_rejected_until_hosts_are_installed() {
        let document = "https://michealcdn.com/video/0123456789abcdef0123456789abcdef";
        assert!(!navigation_allowed(document));
        assert!(!player_page(document));
        for raw in [
            "https://michealcdn.com/",
            "https://michealcdn.com/other",
            "https://michealcdn.com/video/short",
            "https://sub.michealcdn.com/video/0123456789abcdef0123456789abcdef",
            "https://michealcdn.com/video/0123456789abcdef0123456789abcdef?redirect=1",
        ] {
            assert!(!navigation_allowed(raw));
            assert!(!player_page(raw));
        }
    }

    #[test]
    fn installing_secondary_cdn_admits_validated_player_documents_only() {
        let document = "https://michealcdn.com/video/0123456789abcdef0123456789abcdef";
        // Secondary CDN is a player-document origin only (not general navigation).
        let hosts = AdmittedHosts::fixture().with_player_hosts([
            "frame.example.test",
            "media.example.test",
            "michealcdn.com",
        ]);
        with_admitted_hosts(hosts, || {
            assert!(navigation_allowed(document));
            assert!(player_page(document));
            for raw in [
                "https://michealcdn.com/",
                "https://michealcdn.com/other",
                "https://michealcdn.com/video/short",
                "https://sub.michealcdn.com/video/0123456789abcdef0123456789abcdef",
                "https://michealcdn.com/video/0123456789abcdef0123456789abcdef?redirect=1",
            ] {
                assert!(!navigation_allowed(raw), "lookalike still navigable: {raw}");
                assert!(!player_page(raw), "lookalike still player page: {raw}");
            }
            assert!(navigation_allowed(
                "https://frame.example.test/tv-show.php?v=test"
            ));
            assert!(player_page("https://frame.example.test/tv-show.php?v=test"));
            assert!(!player_page("https://catalog.example.test/e/fixture"));
        });
    }

    #[test]
    fn rejects_lookalikes_and_privileged_schemes() {
        for u in [
            "https://catalog.example.test.evil.test/",
            "file:///C:/Windows",
            "javascript:alert(1)",
            "http://catalog.example.test/",
        ] {
            assert!(!navigation_allowed(u));
        }
        assert!(navigation_allowed(
            "https://frame.example.test/tv-show.php?v=test"
        ));
        assert!(!media_candidate("blob:https://media.example.test/id"));
        assert!(media_candidate("https://cdn.example/movie.m3u8?token=test"));
    }
}
