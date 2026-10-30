//! OBJ read-only props draft. Asset delivery, commands and Native slots are separate adapters.
use crate::application::workspace_read::{
    AssetReference, MaskSnapshot, ReadOrigin, ReadState, SubjectMask, WorkspaceId,
    WorkspaceSnapshot,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectProjectionError {
    WrongWorkspace,
}

#[derive(Debug)]
pub struct SubjectRowProjection<'a> {
    pub subject: &'a SubjectMask,
    pub selected: bool,
    /// The SDK currently has no user label or validated cropped thumbnail.
    pub user_label: Option<&'a str>,
    pub thumbnail: Option<&'a AssetReference>,
}

#[derive(Debug)]
pub enum SubjectContent<'a> {
    /// Absence is not a successful zero-track result.
    Missing,
    Mask {
        source: &'a MaskSnapshot,
        rows: Vec<SubjectRowProjection<'a>>,
    },
}

#[derive(Debug)]
pub struct SubjectWorkspaceProjection<'a> {
    pub workspace_origin: ReadOrigin,
    pub workspace_state: ReadState,
    pub content: SubjectContent<'a>,
}

impl<'a> SubjectWorkspaceProjection<'a> {
    pub fn from_workspace(
        snapshot: &'a WorkspaceSnapshot,
        selected_track_id: Option<&str>,
    ) -> Result<Self, SubjectProjectionError> {
        if snapshot.workspace_id != WorkspaceId::Obj {
            return Err(SubjectProjectionError::WrongWorkspace);
        }
        let content = match &snapshot.mask {
            None => SubjectContent::Missing,
            Some(source) => SubjectContent::Mask {
                source,
                rows: source
                    .subjects
                    .iter()
                    .map(|subject| SubjectRowProjection {
                        subject,
                        selected: selected_track_id == Some(subject.track_id.as_str()),
                        user_label: None,
                        thumbnail: None,
                    })
                    .collect(),
            },
        };
        Ok(Self {
            workspace_origin: snapshot.origin,
            workspace_state: snapshot.state,
            content,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::workspace_read::{
        LiveWorkspaceProvider, MaskReviewState, WorkspaceReadPort,
        preview::{PreviewScenario, PreviewWorkspaceProvider},
    };

    #[test]
    fn unsupported_absence_is_distinct_from_ready_zero_tracks() {
        let mut snapshot = LiveWorkspaceProvider.snapshot(WorkspaceId::Obj);
        let missing = SubjectWorkspaceProjection::from_workspace(&snapshot, None).unwrap();
        assert_eq!(missing.workspace_state, ReadState::Unsupported);
        assert!(matches!(missing.content, SubjectContent::Missing));
        let mut demo = PreviewWorkspaceProvider::new(PreviewScenario::Populated)
            .snapshot(WorkspaceId::Obj);
        let mut mask = demo.mask.take().unwrap();
        mask.subjects.clear();
        snapshot.mask = Some(mask);
        let projected = SubjectWorkspaceProjection::from_workspace(&snapshot, None).unwrap();
        assert_eq!(projected.workspace_origin, ReadOrigin::Live);
        assert_eq!(projected.workspace_state, ReadState::Unsupported);
        let SubjectContent::Mask { source, rows } = projected.content else {
            panic!("An existing mask must remain present");
        };
        assert_eq!(source.origin, ReadOrigin::DesignSample);
        assert_eq!(source.state, ReadState::Ready);
        assert!(rows.is_empty());
    }

    #[test]
    fn track_selection_never_invents_labels_thumbnails_or_confirmation() {
        let mut snapshot = PreviewWorkspaceProvider::new(PreviewScenario::Populated)
            .snapshot(WorkspaceId::Obj);
        let mask = snapshot.mask.as_mut().unwrap();
        let mut second = mask.subjects[0].clone();
        second.track_id = "another-track".into();
        second.review_state = MaskReviewState::NeedsReview;
        mask.subjects.push(second);
        let projected =
            SubjectWorkspaceProjection::from_workspace(&snapshot, Some("another-track")).unwrap();
        let SubjectContent::Mask { source, rows } = projected.content else {
            panic!("Expected source mask");
        };
        assert!(!rows[0].selected);
        assert!(rows[1].selected);
        assert!(rows.iter().all(|row| row.user_label.is_none() && row.thumbnail.is_none()));
        assert_eq!(rows[1].subject.review_state, MaskReviewState::NeedsReview);
        assert!(std::ptr::eq(rows[1].subject, &source.subjects[1]));
        let unselected = SubjectWorkspaceProjection::from_workspace(&snapshot, Some("stale"))
            .unwrap();
        let SubjectContent::Mask { rows, .. } = unselected.content else {
            panic!("Expected source mask");
        };
        assert!(rows.iter().all(|row| !row.selected));
    }
}
