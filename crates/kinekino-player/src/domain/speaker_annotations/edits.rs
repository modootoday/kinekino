use super::*;
fn known(state: &SpeakerState, id: &str) -> bool {
    state.speakers.iter().any(|s| s.id == id)
}
fn selections(ids: &[String], known: &[String], empty: bool) -> Result<(), &'static str> {
    if !unique_ids(ids, empty) || ids.iter().any(|id| !known.contains(id)) {
        Err("Unknown, duplicate or empty selection")
    } else {
        Ok(())
    }
}
fn replace(ids: &mut Vec<String>, selected: &[String], replacement: &[String]) {
    let mut next = Vec::new();
    for id in ids.iter() {
        if !selected.contains(id) && !next.contains(id) {
            next.push(id.clone());
        }
    }
    if ids.iter().any(|id| selected.contains(id)) {
        for id in replacement {
            if !next.contains(id) {
                next.push(id.clone());
            }
        }
    }
    *ids = next;
}
fn apply(state: &mut SpeakerState, edit: &SpeakerEdit) -> Result<(), &'static str> {
    let ids = state
        .speakers
        .iter()
        .map(|s| s.id.clone())
        .collect::<Vec<_>>();
    match edit {
        SpeakerEdit::Add { speaker } => {
            if !valid_id(&speaker.id) || !text(&speaker.label, 1024) || known(state, &speaker.id) {
                return Err("New unique speaker identity required");
            }
            state.speakers.push(speaker.clone());
        }
        SpeakerEdit::Rename { speaker_id, label } => {
            if !text(label, 1024) {
                return Err("Nonempty speaker label required");
            }
            state
                .speakers
                .iter_mut()
                .find(|s| &s.id == speaker_id)
                .ok_or("Unknown speaker")?
                .label = label.clone();
            for span in &mut state.spans {
                if span.speaker_ids.contains(speaker_id) {
                    span.review_state = Some(SpeakerReviewState::NeedsReview);
                }
            }
        }
        SpeakerEdit::Merge {
            speaker_ids,
            target,
        } => {
            selections(speaker_ids, &ids, false)?;
            if speaker_ids.len() < 2 || !speaker_ids.contains(target) {
                return Err("Merge requires two speakers and an explicit selected target");
            }
            for span in &mut state.spans {
                if span.speaker_ids.iter().any(|id| speaker_ids.contains(id)) {
                    replace(
                        &mut span.speaker_ids,
                        speaker_ids,
                        std::slice::from_ref(target),
                    );
                    span.origin = "user_edit".into();
                    span.review_state = Some(SpeakerReviewState::NeedsReview);
                }
            }
            for cue in &mut state.links {
                if cue.speaker_ids.iter().any(|id| speaker_ids.contains(id)) {
                    replace(
                        &mut cue.speaker_ids,
                        speaker_ids,
                        std::slice::from_ref(target),
                    );
                    cue.assignment_origin = "user_edit".into();
                }
            }
            state
                .speakers
                .retain(|s| !speaker_ids.contains(&s.id) || &s.id == target);
        }
        SpeakerEdit::Split {
            speaker_id,
            new_speaker,
            span_ids,
            cue_ids,
        } => {
            if !known(state, speaker_id)
                || known(state, &new_speaker.id)
                || !valid_id(&new_speaker.id)
                || !text(&new_speaker.label, 1024)
                || (span_ids.is_empty() && cue_ids.is_empty())
            {
                return Err("Explicit source/new speaker and selected spans/cues required");
            }
            selections(
                span_ids,
                &state
                    .spans
                    .iter()
                    .filter(|s| s.speaker_ids.contains(speaker_id))
                    .map(|s| s.id.clone())
                    .collect::<Vec<_>>(),
                true,
            )?;
            selections(
                cue_ids,
                &state
                    .links
                    .iter()
                    .filter(|s| s.speaker_ids.contains(speaker_id))
                    .map(|s| s.cue_id.clone())
                    .collect::<Vec<_>>(),
                true,
            )?;
            for span in &mut state.spans {
                if span_ids.contains(&span.id) {
                    replace(
                        &mut span.speaker_ids,
                        std::slice::from_ref(speaker_id),
                        std::slice::from_ref(&new_speaker.id),
                    );
                    span.origin = "user_edit".into();
                    span.review_state = Some(SpeakerReviewState::NeedsReview);
                }
            }
            for cue in &mut state.links {
                if cue_ids.contains(&cue.cue_id) {
                    replace(
                        &mut cue.speaker_ids,
                        std::slice::from_ref(speaker_id),
                        std::slice::from_ref(&new_speaker.id),
                    );
                    cue.assignment_origin = "user_edit".into();
                }
            }
            state.speakers.push(new_speaker.clone());
        }
        SpeakerEdit::AssignCues {
            cue_ids,
            speaker_ids,
        } => {
            selections(
                cue_ids,
                &state
                    .links
                    .iter()
                    .map(|s| s.cue_id.clone())
                    .collect::<Vec<_>>(),
                false,
            )?;
            selections(speaker_ids, &ids, true)?;
            for cue in &mut state.links {
                if cue_ids.contains(&cue.cue_id) {
                    cue.speaker_ids = speaker_ids.clone();
                    cue.assignment_origin = "user_edit".into();
                }
            }
        }
        SpeakerEdit::Delete {
            speaker_id,
            reassign,
        } => {
            if !known(state, speaker_id) {
                return Err("Unknown speaker");
            }
            let used = state
                .spans
                .iter()
                .any(|s| s.speaker_ids.contains(speaker_id))
                || state
                    .links
                    .iter()
                    .any(|s| s.speaker_ids.contains(speaker_id));
            if used && reassign.is_none() {
                return Err("Referenced speaker requires explicit reassignment or unknown");
            }
            let replacements = reassign.as_deref().unwrap_or(&[]);
            selections(replacements, &ids, true)?;
            if replacements.contains(speaker_id) {
                return Err("Deleted speaker cannot reassign to itself");
            }
            for span in &mut state.spans {
                if span.speaker_ids.contains(speaker_id) {
                    replace(
                        &mut span.speaker_ids,
                        std::slice::from_ref(speaker_id),
                        replacements,
                    );
                    span.origin = "user_edit".into();
                    span.review_state = Some(SpeakerReviewState::NeedsReview);
                }
            }
            for cue in &mut state.links {
                if cue.speaker_ids.contains(speaker_id) {
                    replace(
                        &mut cue.speaker_ids,
                        std::slice::from_ref(speaker_id),
                        replacements,
                    );
                    cue.assignment_origin = "user_edit".into();
                }
            }
            state.speakers.retain(|s| &s.id != speaker_id);
        }
        SpeakerEdit::Undo | SpeakerEdit::Redo => return Err("History operation outside replay"),
    }
    Ok(())
}
fn rebuild(document: &SpeakerDocument, active: &[usize]) -> Result<SpeakerState, &'static str> {
    let mut state = document.original.clone();
    for &index in active {
        apply(&mut state, &document.edits[index])?;
    }
    Ok(state)
}
pub(super) fn replay(document: &SpeakerDocument) -> Result<SpeakerState, &'static str> {
    let mut state = document.original.clone();
    let mut active = Vec::new();
    let mut redo = Vec::new();
    let mut dirty = false;
    let mut used = document
        .original
        .speakers
        .iter()
        .map(|s| s.id.clone())
        .collect::<BTreeSet<_>>();
    for (index, edit) in document.edits.iter().enumerate() {
        match edit {
            SpeakerEdit::Undo => {
                redo.push(active.pop().ok_or("Nothing to undo")?);
                dirty = true;
            }
            SpeakerEdit::Redo => {
                active.push(redo.pop().ok_or("Nothing to redo")?);
                dirty = true;
            }
            _ => {
                if dirty {
                    state = rebuild(document, &active)?;
                }
                let created = match edit {
                    SpeakerEdit::Add { speaker } => Some(&speaker.id),
                    SpeakerEdit::Split { new_speaker, .. } => Some(&new_speaker.id),
                    _ => None,
                };
                if created.is_some_and(|id| !used.insert(id.clone())) {
                    return Err("Historical speaker identities cannot be reused");
                }
                apply(&mut state, edit)?;
                state.validate(&document.source)?;
                active.push(index);
                redo.clear();
                dirty = false;
            }
        }
    }
    if dirty {
        state = rebuild(document, &active)?;
    }
    state.validate(&document.source)?;
    Ok(state)
}
