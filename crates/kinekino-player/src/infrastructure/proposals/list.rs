//! One bounded page under the original writer; no persistent reader or authority.
use super::{
    ProposalFiles, corrupt, io_error, paths, reservation::WriterReservation, state_read::ReadBudget,
};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        proposals::list::*,
    },
    domain::identifiers::valid_id,
    infrastructure::storage_directory_identity::StorageDirectoryIdentity,
};
use std::fs;

const MAX_ROOT_ENTRIES: usize = 4096;
const MAX_REVISION_ENTRIES: usize = 8192;
const MAX_REVISIONS_PER_PROPOSAL: usize = 4096;
const MAX_TOTAL_STATE_BYTES: u64 = 32 * 1024 * 1024;

fn dataset(store: &ProposalFiles) -> Result<String, ApiError> {
    let root = store._guards.last().ok_or_else(corrupt)?;
    hash_witness(&(
        "proposal-list-dataset/1",
        StorageDirectoryIdentity::observe(root)?,
    ))
}
fn names(store: &ProposalFiles, writer: &WriterReservation<'_>) -> Result<Vec<String>, ApiError> {
    let mut ids = Vec::new();
    for (index, entry) in fs::read_dir(&store.path).map_err(io_error)?.enumerate() {
        if index >= MAX_ROOT_ENTRIES {
            return Err(exhausted());
        }
        let entry = entry.map_err(io_error)?;
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(corrupt)?;
        let path = entry.path();
        match name {
            // The same reservation already owns and validated this exclusive
            // file. Reopening it would conflict with our own share_mode(0).
            "writer.lock" => {
                writer.check(store)?;
            }
            ".proposal-store-origin.v1.json" => {
                let _guard = paths::file(&path)?;
            }
            "output-decisions" => {
                let _guard = paths::directory(&path)?;
            }
            _ => {
                let id = name
                    .strip_prefix("p-")
                    .filter(|id| valid_id(id))
                    .ok_or_else(corrupt)?;
                let _guard = paths::directory(&path)?;
                ids.push(id.to_owned());
            }
        }
    }
    ids.sort_unstable();
    if ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(corrupt());
    }
    Ok(ids)
}
pub(super) fn read(
    store: &ProposalFiles,
    input: &ListProposals,
) -> Result<ProposalListPage, ApiError> {
    let continuation = prepare_cursor(input)?; // invalid wire never acquires a writer
    let writer = WriterReservation::acquire(store)?;
    let dataset = dataset(store)?;
    if let Some(cursor) = &continuation {
        cursor.check_dataset(&dataset)?;
    }
    let ids = names(store, &writer)?;
    let mut budget = ReadBudget {
        revision_entries_left: MAX_REVISION_ENTRIES,
        per_proposal_revisions: MAX_REVISIONS_PER_PROPOSAL,
        state_bytes_left: MAX_TOTAL_STATE_BYTES,
    };
    let mut metadata = Vec::with_capacity(ids.len());
    let mut witnesses = Vec::with_capacity(ids.len());
    let mut metadata_bytes = 0usize;
    for id in ids {
        let (state, sha) = store.read_current_bounded(&writer, &id, &mut budget)?;
        let row = metadata_row(&state)?;
        metadata_bytes = metadata_bytes
            .checked_add(encoded_bytes(&row, MAX_ROW_BYTES)?)
            .filter(|v| *v <= MAX_METADATA_BYTES)
            .ok_or_else(exhausted)?;
        witnesses.push((id, state.document.draft_revision, sha));
        metadata.push(row);
    }
    // Recheck the original handle, never reopen a path to establish identity.
    if dataset != self::dataset(store)? {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Original list dataset changed",
        ));
    }
    project_page(input, continuation, dataset, metadata, witnesses)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::content_digest;
    use crate::{
        application::{
            ai_provenance::{HistoricalEvidenceReader, VerifiedHistoricalSample},
            proposals::ProposalStore,
        },
        domain::proposals::*,
    };
    use std::{
        io::Cursor,
        path::PathBuf,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::{SystemTime, UNIX_EPOCH},
    };
    fn root() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "proposal-list-reader-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        eprintln!("proposal_list_reader_fixture={}", path.display());
        path
    }
    #[test]
    fn bounded_reader_counts_revision_and_actual_bytes_without_changing_legacy_load() {
        let root = root();
        let store = ProposalFiles::open(&root).unwrap();
        let mut state = crate::domain::projects::tests::proposal();
        store.publish(None, &state).unwrap();
        let bytes = serde_json::to_vec(&state).unwrap().len() as u64;
        {
            let writer = WriterReservation::acquire(&store).unwrap();
            let mut exact = ReadBudget {
                revision_entries_left: 1,
                per_proposal_revisions: 1,
                state_bytes_left: bytes,
            };
            assert!(
                store
                    .read_current_bounded(&writer, "draft-a", &mut exact)
                    .is_ok()
            );
            assert_eq!(exact.state_bytes_left, 0);
            let mut short = ReadBudget {
                revision_entries_left: 1,
                per_proposal_revisions: 1,
                state_bytes_left: bytes - 1,
            };
            assert_eq!(
                store
                    .read_current_bounded(&writer, "draft-a", &mut short)
                    .unwrap_err()
                    .code,
                ErrorCode::ResourceExhausted
            );
        }
        state.document.draft_revision = crate::domain::api_values::DecimalU64(2);
        store.publish(Some(1), &state).unwrap();
        {
            let writer = WriterReservation::acquire(&store).unwrap();
            let mut budget = ReadBudget {
                revision_entries_left: 2,
                per_proposal_revisions: 1,
                state_bytes_left: MAX_TOTAL_STATE_BYTES,
            };
            assert_eq!(
                store
                    .read_current_bounded(&writer, "draft-a", &mut budget)
                    .unwrap_err()
                    .code,
                ErrorCode::ResourceExhausted
            );
            let mut total = ReadBudget {
                revision_entries_left: 1,
                per_proposal_revisions: 2,
                state_bytes_left: MAX_TOTAL_STATE_BYTES,
            };
            assert_eq!(
                store
                    .read_current_bounded(&writer, "draft-a", &mut total)
                    .unwrap_err()
                    .code,
                ErrorCode::ResourceExhausted
            );
        }
        assert_eq!(store.load("draft-a").unwrap().document.draft_revision.0, 2);
    }
    #[test]
    fn metadata_listing_does_not_invoke_historical_verifier_but_inspect_still_does() {
        struct Reader(AtomicUsize);
        impl HistoricalEvidenceReader for Reader {
            fn verify(&self, _: &CompanionReference) -> Result<VerifiedHistoricalSample, ApiError> {
                self.0.fetch_add(1, Ordering::AcqRel);
                Err(ApiError::new(
                    ErrorCode::Unsupported,
                    "Fixture evidence unavailable",
                ))
            }
        }
        let root = root();
        let mut state = crate::domain::projects::tests::proposal();
        state.document.schema_version = HISTORICAL_DOCUMENT_VERSION.into();
        state.historical = Some(HistoricalBinding::new(CompanionReference {
            sha256: "b".repeat(64),
            bytes: 20,
            item_id: "fixture".into(),
        }));
        state.validate().unwrap();
        // Raw structural fixture: this is not a trusted historical import.
        let raw = serde_json::to_vec(&state).unwrap();
        let sha = content_digest::sha256(&mut Cursor::new(&raw), super::super::MAX_STATE)
            .unwrap()
            .1;
        let revision = root.join("p-draft-a/r-1");
        fs::create_dir_all(&revision).unwrap();
        fs::write(revision.join(format!("{sha}.json")), raw).unwrap();
        let reader = Arc::new(Reader(AtomicUsize::new(0)));
        let store = ProposalFiles::open(&root)
            .unwrap()
            .with_historical_reader(reader.clone());
        assert_eq!(
            read(&store, &ListProposals::default()).unwrap().rows.len(),
            1
        );
        assert_eq!(reader.0.load(Ordering::Acquire), 0);
        assert_eq!(
            store.load("draft-a").unwrap_err().code,
            ErrorCode::Unsupported
        );
        assert_eq!(reader.0.load(Ordering::Acquire), 1);
    }
}
