use super::*;
use crate::domain::{
    identifiers::valid_sha256,
    review_source::{AssociationRevision, RangeDraft},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Read, Seek, SeekFrom},
};

pub(super) fn read(path: &Path, case: &ReviewCase) -> Result<(MutationReceipt, File), ApiError> {
    let name = path
        .file_stem()
        .and_then(|n| n.to_str())
        .ok_or_else(|| incomplete("record_name"))?;
    if !valid_sha256(name) || path.extension().and_then(|x| x.to_str()) != Some("json") {
        return Err(incomplete("record_name"));
    }
    let mut file = rooted::file(path)?;
    let (_, sha) = content_digest::sha256(&mut file, MAX_TRANSACTION_BYTES)?;
    if sha != name {
        return Err(incomplete("record_hash"));
    }
    file.seek(SeekFrom::Start(0)).map_err(io_error)?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(MAX_TRANSACTION_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    let body: TransactionBody =
        serde_json::from_slice(&bytes).map_err(|_| incomplete("record_decode"))?;
    body.validate().map_err(|_| incomplete("record_contract"))?;
    if body.request.case != *case
        || serde_json::to_vec(&body).map_err(|_| incomplete("canonical_encoding"))? != bytes
    {
        return Err(incomplete("record_identity"));
    }
    Ok((
        MutationReceipt {
            envelope_sha256: sha,
            body,
        },
        file,
    ))
}
pub(super) fn load(
    directory: &Path,
    case: &ReviewCase,
) -> Result<(Vec<MutationReceipt>, Vec<File>), ApiError> {
    let mut records = Vec::new();
    let mut files = Vec::new();
    let mut pending = Vec::new();
    let mut count = 0;
    for entry in fs::read_dir(directory).map_err(io_error)? {
        let entry = entry.map_err(io_error)?;
        count += 1;
        if count > MAX_CASE_TRANSACTIONS * 2 {
            return Err(incomplete("history_capacity"));
        }
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(|| incomplete("record_name"))?;
        if name.starts_with(".export-") && name.ends_with(".tmp") {
            pending.push(entry.path());
            continue;
        }
        let (record, file) = read(&entry.path(), case)?;
        records.push(record);
        files.push(file);
    }
    if records.len() > MAX_CASE_TRANSACTIONS {
        return Err(incomplete("history_capacity"));
    }
    validate(&records)?;
    // A crash after link can leave a non-authoritative alias. Keep it, but
    // accept only complete bytes matching an already committed envelope.
    let committed: BTreeSet<_> = records.iter().map(|r| r.envelope_sha256.as_str()).collect();
    for path in pending {
        let mut file = rooted::file(&path)?;
        let (_, sha) = content_digest::sha256(&mut file, MAX_TRANSACTION_BYTES)?;
        if !committed.contains(sha.as_str()) {
            return Err(incomplete("unpublished_partial"));
        }
        files.push(file);
    }
    Ok((records, files))
}
pub(super) fn validate(records: &[MutationReceipt]) -> Result<(), ApiError> {
    let mut associations = BTreeMap::new();
    let mut snapshots = BTreeMap::new();
    let mut drafts = BTreeMap::new();
    let mut keys = BTreeSet::new();
    for record in records {
        let body = &record.body;
        let request = &body.request;
        if !keys.insert((
            request.scope().0,
            request.scope().1,
            &request.idempotency_key,
        )) {
            return Err(incomplete("duplicate_key"));
        }
        let a = &body.association;
        let key = (&a.association_id, a.revision);
        if snapshots.insert(key, a).is_some_and(|old| old != a) {
            return Err(incomplete("association_snapshot_conflict"));
        }
        let author = matches!(
            &request.mutation,
            Mutation::CreateAssociation(_)
                | Mutation::ReviseAssociation(_)
                | Mutation::VerifyAssociation { .. }
        ) || matches!(
            &request.mutation,
            Mutation::SaveDraft {
                create_association: Some(_),
                ..
            }
        );
        if author && associations.insert(key, a).is_some() {
            return Err(incomplete("duplicate_association_revision"));
        }
        if let Some(d) = &body.draft
            && drafts.insert((&d.draft_id, d.draft_revision), d).is_some()
        {
            return Err(incomplete("duplicate_draft_revision"));
        }
    }
    for (key, snapshot) in snapshots {
        if associations.get(&key).copied() != Some(snapshot) {
            return Err(incomplete("uncommitted_association"));
        }
    }
    let mut previous: Option<&AssociationRevision> = None;
    for a in associations.values() {
        if let Some(old) = previous.filter(|old| old.association_id == a.association_id) {
            a.validate_successor(old)
                .map_err(|_| incomplete("association_chain"))?;
        } else if a.revision.0 != 1 {
            return Err(incomplete("association_first_revision"));
        }
        previous = Some(a);
    }
    let mut previous: Option<&RangeDraft> = None;
    for d in drafts.values() {
        let a = associations
            .get(&(&d.association_id, d.association_revision))
            .ok_or_else(|| incomplete("draft_association"))?;
        if let Some(old) = previous.filter(|old| old.draft_id == d.draft_id) {
            d.validate_successor(old, a, None)
                .map_err(|_| incomplete("draft_chain"))?;
        } else if d.draft_revision.0 != 1 {
            return Err(incomplete("draft_first_revision"));
        }
        previous = Some(d);
    }
    Ok(())
}
