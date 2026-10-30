use std::fmt;
use url::Url;

/// Provider document targets observed in preserved 2026-10-19/25 episode DOM payloads.
/// This is a document target, not an admitted media stream or a successful playback.
#[derive(Clone, PartialEq, Eq)]
pub struct PlayerDocumentLocation(Url);

impl PlayerDocumentLocation {
    /// Origin shared by declared documents and already-observed provider asset responses.
    /// This alone grants no navigation, request, document or media authority.
    pub(crate) fn admitted_response_origin(url: &Url) -> bool {
        url.scheme() == "https"
            && matches!(
                url.host_str(),
                Some("media.example.test" | "michealcdn.com")
            )
            && url.port_or_known_default() == Some(443)
            && url.username().is_empty()
            && url.password().is_none()
    }
    pub fn parse(raw: &str) -> Result<Self, &'static str> {
        if raw.len() > 8192 {
            return Err("Player document URL exceeds limit");
        }
        let url = Url::parse(raw).map_err(|_| "Invalid player document URL")?;
        let identifier = url.path().strip_prefix("/video/").unwrap_or("");
        if !Self::admitted_response_origin(&url)
            || url.query().is_some()
            || url.fragment().is_some()
            || identifier.len() != 32
            || !identifier.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err("Unsupported player document location");
        }
        Ok(Self(url))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Debug for PlayerDocumentLocation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "PlayerDocumentLocation(https://{}/video/<opaque>)",
            self.0.host_str().unwrap_or("invalid")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observed_secondary_provider_retains_exact_origin_and_document_shape() {
        let identifier = "0123456789abcdef0123456789abcdef";
        let raw = format!("https://michealcdn.com/video/{identifier}");
        let location = PlayerDocumentLocation::parse(&raw).unwrap();
        assert_eq!(location.as_str(), raw);
        assert!(!format!("{location:?}").contains(identifier));
        assert!(format!("{location:?}").contains("michealcdn.com"));
        for candidate in [
            format!("http://michealcdn.com/video/{identifier}"),
            format!("https://michealcdn.com.evil.test/video/{identifier}"),
            format!("https://sub.michealcdn.com/video/{identifier}"),
            format!("https://michealcdn.com:444/video/{identifier}"),
            format!("https://user@michealcdn.com/video/{identifier}"),
            format!("https://michealcdn.com/video/{identifier}?token=fixture"),
            format!("https://michealcdn.com/video/{identifier}#fixture"),
            format!("https://michealcdn.com/video/{identifier}/"),
            "https://michealcdn.com/".into(),
            "https://michealcdn.com/video/short".into(),
            "https://michealcdn.com/video/zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz".into(),
        ] {
            assert!(
                PlayerDocumentLocation::parse(&candidate).is_err(),
                "Unexpected admission: {candidate}"
            );
        }
    }

    #[test]
    fn admits_only_the_observed_provider_document_shape() {
        let identifier = "0123456789abcdef0123456789abcdef";
        let raw = format!("https://media.example.test/video/{identifier}");
        let location = PlayerDocumentLocation::parse(&raw).unwrap();
        assert_eq!(location.as_str(), raw);
        assert!(!format!("{location:?}").contains(identifier));
        for raw in [
            format!("http://media.example.test/video/{identifier}"),
            format!("https://media.example.test.evil.test/video/{identifier}"),
            format!("https://media.example.test:444/video/{identifier}"),
            format!("https://user@media.example.test/video/{identifier}"),
            format!("https://media.example.test/video/{identifier}?token=fixture"),
            format!("https://media.example.test/video/{identifier}#fixture"),
            format!("https://media.example.test/video/{identifier}/"),
            "https://media.example.test/video/short".into(),
            "https://media.example.test/video/zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz".into(),
            format!("https://127.0.0.1/video/{identifier}"),
            format!("https://frame.example.test/video/{identifier}"),
        ] {
            assert!(PlayerDocumentLocation::parse(&raw).is_err());
        }
    }
}
