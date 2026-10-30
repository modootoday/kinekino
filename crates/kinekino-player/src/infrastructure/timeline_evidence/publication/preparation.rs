//! Trusted local preparation into the existing immutable review package namespace.
use super::*;
use crate::application::review_source::media_preparation::{self, MediaSelectionRelation};

/// Rust assembly only. Never deserialize paths from an SDK request.
pub struct LocalMediaSelection {
    pub path: PathBuf,
    pub asset: EvidenceAsset,
}
/// Retained custody of a prepared selection, not a timeline or playback grant.
pub struct PreparedLocalMedia {
    root: PathBuf,
    case: ReviewCase,
    relation: MediaSelectionRelation,
    replayed: bool,
    _guards: Vec<File>,
}
impl PreparedLocalMedia {
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn case(&self) -> &ReviewCase {
        &self.case
    }
    pub fn relation(&self) -> &MediaSelectionRelation {
        &self.relation
    }
    pub fn replayed(&self) -> bool {
        self.replayed
    }
}
pub fn prepare_local_media(
    source: &ReviewFiles,
    base_case: &ReviewCase,
    selection: LocalMediaSelection,
    destination: TimelinePackageDestination,
    control: &dyn MutationControl,
) -> Result<PreparedLocalMedia, ApiError> {
    control.checkpoint()?;
    if destination.max_total_bytes == 0 || destination.max_total_bytes > 4_294_967_296 {
        return Err(invalid("Explicit bounded package byte budget required"));
    }
    let parent = selection
        .path
        .parent()
        .ok_or_else(|| invalid("Selected media parent missing"))?;
    let (source_parent, mut guards) = rooted::root(parent)?;
    let mut selected_file = rooted::file(&selection.path)?;
    let (canonical, destination_guards) = rooted::root(&destination.root)?;
    guards.extend(destination_guards);
    if overlaps(&canonical, source.canonical_export_root()) || overlaps(&canonical, &source_parent)
    {
        return Err(invalid(
            "Preparation overlaps original evidence or selected media root",
        ));
    }
    for excluded in &destination.excluded_roots {
        let (root, held) = rooted::root(excluded)?;
        guards.extend(held);
        if overlaps(&canonical, &root) {
            return Err(invalid("Preparation overlaps protected root"));
        }
    }
    let mut exported = source.export_case(base_case)?;
    exported
        .package_file
        .seek(SeekFrom::Start(0))
        .map_err(io_error)?;
    let mut base_bytes = Vec::new();
    Read::by_ref(&mut exported.package_file)
        .take(524289)
        .read_to_end(&mut base_bytes)
        .map_err(io_error)?;
    // Validate metadata/case/size before scanning or publishing the selected media.
    let prepared = media_preparation::prepare(&base_bytes, base_case, selection.asset)?;
    let checkpoint = || control.checkpoint();
    let observed = content_digest::sha256_with_checkpoint(
        &mut selected_file,
        prepared.relation.media.bytes.0,
        &checkpoint,
    )
    .map_err(|error| error.into_api_error())?;
    if observed
        != (
            prepared.relation.media.bytes.0,
            prepared.relation.media.sha256.clone(),
        )
    {
        return Err(conflict(
            "Selected media bytes differ from explicit identity",
        ));
    }
    let mut seen = BTreeSet::new();
    let mut total = 0;
    for (asset, _) in &exported.objects {
        reserve_bytes(&mut seen, &mut total, asset, destination.max_total_bytes)?;
    }
    for asset in [
        &prepared.relation.media,
        &prepared.relation_asset,
        &prepared.base_asset,
    ] {
        reserve_bytes(&mut seen, &mut total, asset, destination.max_total_bytes)?;
    }
    control.checkpoint()?;
    // Same writer as timeline publication; exact package link is the sole commit.
    let _writer = rooted::lock(&canonical.join(".timeline-package-publish.lock"))?;
    for name in ["objects", "packages", "decisions"] {
        let path = canonical.join(name);
        match std::fs::create_dir(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(io_error(error)),
        }
        guards.push(rooted::directory(&path)?);
    }
    for (asset, file) in exported.objects {
        guards.push(objects::retain_or_copy(
            &canonical.join("objects"),
            &asset,
            file,
            &checkpoint,
        )?);
    }
    guards.push(objects::retain_or_copy(
        &canonical.join("objects"),
        &prepared.relation.media,
        selected_file,
        &checkpoint,
    )?);
    for (asset, bytes) in [
        (&prepared.base_asset, &base_bytes),
        (&prepared.relation_asset, &prepared.relation_bytes),
    ] {
        guards.push(objects::retain_or_publish(
            &canonical.join("objects"),
            asset,
            bytes,
            &checkpoint,
        )?);
    }
    let path = canonical
        .join("packages")
        .join(format!("{}.json", prepared.case.package_sha256));
    let replayed = if path.try_exists().map_err(io_error)? {
        true
    } else {
        crate::infrastructure::new_file::publish_guarded(
            path.to_str()
                .ok_or_else(|| invalid("Unicode package path required"))?,
            &prepared.body,
            checkpoint,
        )?;
        false
    };
    let mut package_guard = rooted::file(&path)?;
    if content_digest::sha256(&mut package_guard, 524288)?
        != (
            prepared.body.len() as u64,
            prepared.case.package_sha256.clone(),
        )
    {
        return Err(conflict("Prepared package identity differs"));
    }
    guards.push(package_guard);
    Ok(PreparedLocalMedia {
        root: destination.root,
        case: prepared.case,
        relation: prepared.relation,
        replayed,
        _guards: guards,
    })
}
