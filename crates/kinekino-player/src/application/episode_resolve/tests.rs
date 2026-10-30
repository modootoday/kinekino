use super::*;
use std::sync::Arc;

struct Markup(EpisodeDocumentData);
impl EpisodeMarkupPort for Markup {
    fn read(&self, _: &[u8]) -> Result<EpisodeDocumentData, ApiError> {
        Ok(self.0.clone())
    }
}
fn fixture() -> (ResolveEpisode, BrowserDocument, Markup) {
    let input = ResolveEpisode {
        episode_url: "https://catalog.example.test/e/fixture".into(),
    };
    let document = BrowserDocument {
        final_url: input.episode_url.clone(),
        document_generation: DecimalU64(3),
        observed_unix_ms: DecimalU64(1),
        kind: CaptureKind::DomSnapshot,
        body: Arc::from(b"fixture DOM".as_slice()),
        response: None,
    };
    let markup = Markup(EpisodeDocumentData {
        page_url: "/e/fixture".into(),
        playing_slug: "fixture".into(),
        content_id: 1,
        body_type: "video".into(),
        iframe: true,
        target_url: "https://media.example.test/video/0123456789abcdef0123456789abcdef".into(),
    });
    (input, document, markup)
}
#[test]
fn binds_target_to_requested_observed_and_embedded_episode_identity() {
    let (input, document, markup) = fixture();
    let target = player_target(&input, &document, &markup).unwrap();
    assert_eq!(target.episode_id, "catalog:/e/fixture");
    assert_eq!(target.source_document_generation, DecimalU64(3));
    assert_eq!(target.source_pointer, "/props/playing/body/url");
    assert_eq!(
        target.source_body_sha256,
        format!("{:x}", Sha256::digest(&document.body))
    );
}
#[test]
fn observed_secondary_provider_target_keeps_original_episode_and_dom_evidence_binding() {
    let (input, document, mut markup) = fixture();
    markup.0.target_url = "https://michealcdn.com/video/0123456789abcdef0123456789abcdef".into();
    let target = player_target(&input, &document, &markup).unwrap();
    assert_eq!(target.document.as_str(), markup.0.target_url);
    assert_eq!(target.episode_id, "catalog:/e/fixture");
    assert_eq!(
        target.source_document_generation,
        document.document_generation
    );
    assert_eq!(
        target.source_body_sha256,
        format!("{:x}", Sha256::digest(&document.body))
    );
    assert_eq!(target.source_pointer, "/props/playing/body/url");
    markup.0.playing_slug = "other".into();
    assert!(player_target(&input, &document, &markup).is_err());
}
#[test]
fn foreign_identity_unobserved_schema_and_provider_are_not_admitted() {
    for mutation in 0..7 {
        let (input, mut document, mut markup) = fixture();
        match mutation {
            0 => document.final_url = "https://catalog.example.test/e/other".into(),
            1 => markup.0.page_url = "/e/other".into(),
            2 => markup.0.playing_slug = "other".into(),
            3 => markup.0.content_id = 0,
            4 => markup.0.iframe = false,
            5 => markup.0.body_type = "html".into(),
            _ => {
                markup.0.target_url =
                    "https://127.0.0.1/video/0123456789abcdef0123456789abcdef".into()
            }
        }
        assert!(player_target(&input, &document, &markup).is_err());
    }
}

#[test]
fn unicode_episode_slug_uses_the_same_canonical_location_and_requires_dom_generation() {
    let (mut input, mut document, mut markup) = fixture();
    input.episode_url = PageLocation::parse("https://catalog.example.test/e/가상 작품 1/2")
        .unwrap()
        .as_str()
        .into();
    document.final_url = input.episode_url.clone();
    markup.0.page_url = input.episode_url.clone();
    markup.0.playing_slug = "가상 작품 1/2".into();
    assert!(player_target(&input, &document, &markup).is_ok());
    document.document_generation = DecimalU64(0);
    assert!(player_target(&input, &document, &markup).is_err());
    document.document_generation = DecimalU64(1);
    document.kind = CaptureKind::NetworkResponse;
    assert!(player_target(&input, &document, &markup).is_err());
}
