use super::*;

struct Fake(SourceManifest);
impl SourceProvider for Fake {
    fn manifest(&self) -> &SourceManifest {
        &self.0
    }
    fn search(&self, query: &str, _page: u32) -> Result<SourcePage, SourceError> {
        Ok(SourcePage {
            items: vec![SourceItem {
                reference: SourceRef {
                    provider: self.0.id.clone(),
                    id: query.into(),
                },
                title: query.into(),
            }],
            next_page: None,
        })
    }
}

fn fake() -> Fake {
    Fake(SourceManifest {
        id: "com.example.source".into(),
        version: "0.1.0".into(),
        api: SOURCE_API.into(),
        capabilities: vec![SourceCapability::Search],
        media_hosts: vec!["media.example.test".into()],
    })
}

#[test]
fn undeclared_abilities_are_unsupported() {
    let p = fake();
    let r = SourceRef {
        provider: "com.example.source".into(),
        id: "x".into(),
    };
    assert_eq!(
        p.resolve(&r),
        Err(SourceError::Unsupported(SourceCapability::Resolve))
    );
    assert_eq!(
        p.browse(&r, 1),
        Err(SourceError::Unsupported(SourceCapability::Browse))
    );
    assert_eq!(
        p.search("q", 1).unwrap().items[0].reference.provider,
        "com.example.source"
    );
}

#[test]
fn manifest_gates_api_and_media_hosts() {
    let mut m = fake().0;
    assert!(m.is_compatible() && m.supports(SourceCapability::Search));
    assert!(m.allows_media_host("MEDIA.example.test"));
    assert!(!m.allows_media_host("media.example.test.evil.test"));
    m.api = "source/2".into();
    assert!(!m.is_compatible());
}
