use crate::domain::navigation::{media_candidate, media_transport_allowed, navigation_allowed};

#[derive(Debug, Clone)]
pub struct MediaSource {
    pub url: String,
    pub referer: String,
    pub hls: bool,
}

impl MediaSource {
    pub fn from_request(url: String, referer: String) -> Option<Self> {
        if !media_candidate(&url) || !navigation_allowed(&referer) {
            return None;
        }
        let hls = url::Url::parse(&url).ok()?.path().ends_with(".m3u8");
        Some(Self { url, referer, hls })
    }

    pub fn from_manifest(url: String, referer: String, prefix: &[u8]) -> Option<Self> {
        if !media_transport_allowed(&url) || !navigation_allowed(&referer) {
            return None;
        }
        let text = std::str::from_utf8(prefix)
            .ok()?
            .trim_start_matches('\u{feff}')
            .trim_start();
        (text.starts_with("#EXTM3U") && text.lines().any(|line| line.starts_with("#EXT-X-")))
            .then_some(Self {
                url,
                referer,
                hls: true,
            })
    }
}
#[derive(Debug, Clone)]
pub struct VerifiedManifestSource {
    pub url: String,
    pub referer: crate::domain::request_referer::RequestReferer,
}
impl VerifiedManifestSource {
    pub fn new(
        manifest: &crate::domain::hls::VerifiedHlsManifest,
        observed: Option<&crate::domain::request_referer::RequestReferer>,
    ) -> Option<Self> {
        let referer = observed?;
        referer.allowed().then(|| Self {
            url: manifest.source_url().into(),
            referer: referer.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verified_source_requires_observation_while_legacy_empty_referer_remains_rejected() {
        use crate::domain::request_referer::RequestReferer;
        let manifest = crate::domain::hls::verify(
            "https://cdn.example/master.txt",
            b"#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1\nvariant.txt\n",
        )
        .unwrap();
        assert!(VerifiedManifestSource::new(&manifest, None).is_none());
        assert!(VerifiedManifestSource::new(&manifest, Some(&RequestReferer::Absent)).is_some());
        assert!(VerifiedManifestSource::new(&manifest, Some(&RequestReferer::Empty)).is_some());
        assert!(
            VerifiedManifestSource::new(
                &manifest,
                Some(&RequestReferer::Url("file:///private".into()))
            )
            .is_none()
        );
        assert!(
            MediaSource::from_request("https://cdn.example/a.m3u8".into(), String::new()).is_none()
        );
        assert!(
            MediaSource::from_manifest(
                "https://cdn.example/a.txt".into(),
                String::new(),
                b"#EXTM3U\n#EXT-X-VERSION:3"
            )
            .is_none()
        );
    }
    #[test]
    fn recognizes_hls_content_despite_misleading_extension_and_mime() {
        assert!(
            MediaSource::from_manifest(
                "https://cdn.example/a.jpg".into(),
                "https://media.example.test/video/test".into(),
                b"#EXTM3U\n#EXT-X-VERSION:3"
            )
            .is_some()
        );
        assert!(
            MediaSource::from_manifest(
                "https://cdn.example/a.jpg".into(),
                "https://media.example.test/".into(),
                b"\xff\xd8\xff"
            )
            .is_none()
        );
        assert!(
            MediaSource::from_manifest(
                "file:///a".into(),
                "https://media.example.test/".into(),
                b"#EXTM3U"
            )
            .is_none()
        );
    }
}
