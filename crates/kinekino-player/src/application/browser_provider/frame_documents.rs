use super::*;
use crate::application::api::ErrorCode;

const MAX_FRAME_DOCUMENTS: usize = 16;
const MAX_FRAMES: usize = 8;

/// A direct child observed through native FrameCreated, not inferred from an iframe URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameDocumentIdentity {
    pub owner_document_generation: DecimalU64,
    pub owner_native_navigation_id: DecimalU64,
    pub frame_id: DecimalU64,
    pub document_generation: DecimalU64,
    pub native_navigation_id: DecimalU64,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameNavigationDiagnostic {
    pub observed_unix_ms: DecimalU64,
    pub http_status_code: Option<i32>,
    pub is_success: bool,
    pub web_error_status: i32,
}

#[derive(Clone)]
pub struct FrameDocumentState {
    pub runtime: FrameRuntimeState,
    pub identity: FrameDocumentIdentity,
    pub completion: Option<FrameNavigationDiagnostic>,
    pub document: Option<Arc<BrowserDocument>>,
    pub destroyed: bool,
    pub superseded: bool,
}

pub enum FrameDocumentEvent {
    Started(FrameDocumentIdentity),
    Completed(FrameDocumentIdentity, FrameNavigationDiagnostic),
    Document(FrameDocumentIdentity, BrowserDocument),
    Destroyed(FrameDocumentIdentity),
    Superseded(FrameDocumentIdentity),
}
impl FrameDocumentEvent {
    pub fn identity(&self) -> &FrameDocumentIdentity {
        match self {
            Self::Started(identity)
            | Self::Completed(identity, _)
            | Self::Document(identity, _)
            | Self::Destroyed(identity)
            | Self::Superseded(identity) => identity,
        }
    }
}

pub(super) fn apply(
    frames: &mut Vec<FrameDocumentState>,
    event: FrameDocumentEvent,
) -> Result<EventDisposition, ApiError> {
    let identity = event.identity();
    let previous = frames
        .iter()
        .rposition(|frame| frame.identity.frame_id == identity.frame_id);
    if let FrameDocumentEvent::Started(identity) = event {
        if identity.document_generation.0 == 0
            || identity.url.len() > 8192
            || !crate::domain::navigation::player_page(&identity.url)
        {
            return Err(invalid("Invalid direct child document identity"));
        }
        if let Some(index) = previous {
            if frames[index].destroyed
                || identity.document_generation <= frames[index].identity.document_generation
            {
                return Ok(EventDisposition::Stale);
            }
        } else {
            let distinct = frames
                .iter()
                .map(|frame| frame.identity.frame_id.0)
                .collect::<std::collections::HashSet<_>>()
                .len();
            if distinct >= MAX_FRAMES {
                return Err(limit());
            }
        }
        if frames.len() >= MAX_FRAME_DOCUMENTS {
            return Err(limit());
        }
        if let Some(index) = previous {
            frames[index].superseded = true;
        }
        frames.push(FrameDocumentState {
            runtime: FrameRuntimeState::default(),
            identity,
            completion: None,
            document: None,
            destroyed: false,
            superseded: false,
        });
        return Ok(EventDisposition::Applied);
    }
    let Some(index) = previous else {
        return Ok(EventDisposition::Stale);
    };
    let current = &mut frames[index];
    if current.destroyed
        || current.identity != *identity
        || current.superseded && !matches!(&event, FrameDocumentEvent::Destroyed(_))
    {
        return Ok(EventDisposition::Stale);
    }
    match event {
        FrameDocumentEvent::Completed(_, completion) => {
            if current.completion.is_some() {
                return Ok(EventDisposition::Stale);
            }
            if completion
                .http_status_code
                .is_some_and(|status| status != 0 && !(100..=599).contains(&status))
            {
                return Err(invalid("Invalid direct child navigation status"));
            }
            current.completion = Some(completion);
        }
        FrameDocumentEvent::Document(_, document) => {
            if current.completion.is_none() || current.document.is_some() {
                return Ok(EventDisposition::Stale);
            }
            if document.document_generation != current.identity.document_generation
                || document.final_url != current.identity.url
                || document.kind != CaptureKind::DomSnapshot
                || document.response.is_some()
                || document.body.len() > MAX_CAPTURE_BYTES
            {
                return Err(invalid("Direct child DOM does not match its navigation"));
            }
            current.document = Some(Arc::new(document));
        }
        FrameDocumentEvent::Destroyed(_) => current.destroyed = true,
        FrameDocumentEvent::Superseded(_) => current.superseded = true,
        FrameDocumentEvent::Started(_) => {
            unreachable!("Handled before selecting the current frame")
        }
    }
    Ok(EventDisposition::Applied)
}
fn invalid(message: &'static str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn limit() -> ApiError {
    ApiError::new(
        ErrorCode::ResourceExhausted,
        "Direct child document observation limit exceeded",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(generation: u64) -> FrameDocumentIdentity {
        FrameDocumentIdentity {
            owner_document_generation: DecimalU64(2),
            owner_native_navigation_id: DecimalU64(5),
            frame_id: DecimalU64(0),
            document_generation: DecimalU64(generation),
            native_navigation_id: DecimalU64(generation + 10),
            url: "https://media.example.test/video/0123456789abcdef0123456789abcdef".into(),
        }
    }
    fn completion() -> FrameNavigationDiagnostic {
        FrameNavigationDiagnostic {
            observed_unix_ms: DecimalU64(3),
            http_status_code: Some(0),
            is_success: false,
            web_error_status: 1,
        }
    }
    fn document(identity: &FrameDocumentIdentity) -> BrowserDocument {
        BrowserDocument {
            kind: CaptureKind::DomSnapshot,
            final_url: identity.url.clone(),
            document_generation: identity.document_generation,
            observed_unix_ms: DecimalU64(4),
            body: Arc::from(b"synthetic frame error document".as_slice()),
            response: None,
        }
    }
    #[test]
    fn reload_rejects_old_completion_and_requires_current_completion_before_dom() {
        let mut frames = Vec::new();
        let first = identity(1);
        let second = identity(2);
        apply(&mut frames, FrameDocumentEvent::Started(first.clone())).unwrap();
        apply(&mut frames, FrameDocumentEvent::Started(second.clone())).unwrap();
        assert_eq!(
            apply(
                &mut frames,
                FrameDocumentEvent::Completed(first, completion())
            )
            .unwrap(),
            EventDisposition::Stale
        );
        assert_eq!(
            apply(
                &mut frames,
                FrameDocumentEvent::Document(second.clone(), document(&second))
            )
            .unwrap(),
            EventDisposition::Stale
        );
        apply(
            &mut frames,
            FrameDocumentEvent::Completed(second.clone(), completion()),
        )
        .unwrap();
        apply(
            &mut frames,
            FrameDocumentEvent::Document(second.clone(), document(&second)),
        )
        .unwrap();
        assert!(frames[0].completion.is_none());
        assert!(frames[0].document.is_none());
        assert_eq!(frames[1].identity.frame_id, DecimalU64(0));
        assert_eq!(
            frames[1].completion.as_ref().unwrap().http_status_code,
            Some(0)
        );
        assert!(frames[1].document.is_some());
        apply(&mut frames, FrameDocumentEvent::Destroyed(second.clone())).unwrap();
        assert_eq!(
            apply(
                &mut frames,
                FrameDocumentEvent::Completed(second, completion())
            )
            .unwrap(),
            EventDisposition::Stale
        );
    }
    #[test]
    fn distinct_frames_and_reloads_have_separate_limits_without_eviction() {
        let mut frames = Vec::new();
        for id in 0..8 {
            let mut next = identity(1);
            next.frame_id = DecimalU64(id);
            apply(&mut frames, FrameDocumentEvent::Started(next)).unwrap();
        }
        let mut ninth = identity(1);
        ninth.frame_id = DecimalU64(8);
        assert_eq!(
            apply(&mut frames, FrameDocumentEvent::Started(ninth))
                .unwrap_err()
                .code,
            ErrorCode::ResourceExhausted
        );
        assert_eq!(frames.len(), 8);
        let mut reloads = Vec::new();
        for generation in 1..=16 {
            apply(
                &mut reloads,
                FrameDocumentEvent::Started(identity(generation)),
            )
            .unwrap();
        }
        assert_eq!(
            apply(&mut reloads, FrameDocumentEvent::Started(identity(17)))
                .unwrap_err()
                .code,
            ErrorCode::ResourceExhausted
        );
        assert_eq!(reloads.len(), 16);
    }
    #[test]
    fn parent_identity_and_cancellation_gate_the_frame_policy() {
        let (ticket, mut work) = super::super::activation_tests::episode();
        work.reserve_episode_activation(1).unwrap();
        let child = identity(1);
        assert_eq!(
            work.apply(
                &ticket,
                BrowserEvent::FrameDocument(FrameDocumentEvent::Started(child.clone()))
            ),
            EventDisposition::Stale
        );
        work.apply(
            &ticket,
            BrowserEvent::Navigated {
                url: "https://frame.example.test/tv-show.php?v=fixture".into(),
                document_generation: 2,
            },
        );
        work.apply(
            &ticket,
            BrowserEvent::NavigationIdentity {
                document_generation: 2,
                native_navigation_id: 5,
            },
        );
        assert_eq!(
            work.apply(
                &ticket,
                BrowserEvent::FrameDocument(FrameDocumentEvent::Started(child.clone()))
            ),
            EventDisposition::Applied
        );
        assert_eq!(work.snapshot().state, BrowserWorkState::Running);
        assert!(work.snapshot().result.is_none());
        work.cancel();
        assert_eq!(
            work.apply(
                &ticket,
                BrowserEvent::FrameDocument(FrameDocumentEvent::Completed(child, completion()))
            ),
            EventDisposition::Stale
        );
        assert!(work.snapshot().frame_documents[0].completion.is_none());
    }
    #[test]
    fn previous_dom_is_preserved_but_never_ready_after_another_frame_navigation_starts() {
        let mut frames = Vec::new();
        let first = identity(1);
        apply(&mut frames, FrameDocumentEvent::Started(first.clone())).unwrap();
        apply(
            &mut frames,
            FrameDocumentEvent::Completed(first.clone(), completion()),
        )
        .unwrap();
        apply(
            &mut frames,
            FrameDocumentEvent::Document(first.clone(), document(&first)),
        )
        .unwrap();
        let (_, work) = super::super::activation_tests::episode();
        let mut snapshot = work.snapshot();
        snapshot.document_generation = first.owner_document_generation;
        snapshot.native_navigation_id = Some(first.owner_native_navigation_id);
        snapshot.frame_documents = frames;
        assert!(super::super::episode_jobs::child_document_ready(
            &snapshot, &first.url
        ));
        apply(
            &mut snapshot.frame_documents,
            FrameDocumentEvent::Superseded(first.clone()),
        )
        .unwrap();
        assert!(snapshot.frame_documents[0].document.is_some());
        assert!(!snapshot.frame_documents[0].destroyed);
        assert!(!super::super::episode_jobs::child_document_ready(
            &snapshot, &first.url
        ));
        apply(
            &mut snapshot.frame_documents,
            FrameDocumentEvent::Started(identity(2)),
        )
        .unwrap();
        assert!(!super::super::episode_jobs::child_document_ready(
            &snapshot, &first.url
        ));
        assert_eq!(
            apply(
                &mut snapshot.frame_documents,
                FrameDocumentEvent::Completed(first, completion())
            )
            .unwrap(),
            EventDisposition::Stale
        );
    }
}
