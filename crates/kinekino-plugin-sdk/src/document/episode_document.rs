/// Untrusted fields decoded from site markup. Application policy must admit the target.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EpisodeDocumentData {
    pub page_url: String,
    pub playing_slug: String,
    pub content_id: u64,
    pub body_type: String,
    pub iframe: bool,
    pub target_url: String,
}
