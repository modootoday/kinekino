//! Fixed local selection catalog, using the existing package publisher and reader.
use super::*;
use crate::{
    application::review_source::{media_api::*, media_preparation},
    infrastructure::{
        model_review::ReviewFiles,
        timeline_evidence::publication::{
            TimelinePackageDestination,
            preparation::{LocalMediaSelection, prepare_local_media},
        },
    },
};
use std::collections::BTreeMap;
pub struct FixedMediaSelection {
    pub path: PathBuf,
    pub asset: crate::domain::model_review::EvidenceAsset,
}
pub struct MediaPreparationFiles {
    source: PathBuf,
    destination: PathBuf,
    selections: BTreeMap<String, FixedMediaSelection>,
    max_total_bytes: u64,
    writable: bool,
    _guards: Vec<File>,
    _settings:
        Option<crate::infrastructure::review_startup_settings::RetainedReviewStartupSettings>,
}
impl MediaPreparationFiles {
    pub fn from_pinned_settings(
        source: PathBuf,
        path: &Path,
        sha256: &str,
        writable: bool,
    ) -> Result<Self, ApiError> {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Selection {
            path: PathBuf,
            asset: crate::domain::model_review::EvidenceAsset,
        }
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Config {
            destination_root: PathBuf,
            max_total_bytes: u64,
            selections: BTreeMap<String, Selection>,
        }
        let settings =
            crate::infrastructure::review_startup_settings::RetainedReviewStartupSettings::open(
                path, sha256,
            )?;
        let config: Config = serde_json::from_slice(settings.bytes()).map_err(|_| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid pinned media preparation settings",
            )
        })?;
        let selections = config
            .selections
            .into_iter()
            .map(|(id, selection)| {
                (
                    id,
                    FixedMediaSelection {
                        path: selection.path,
                        asset: selection.asset,
                    },
                )
            })
            .collect();
        let mut port = Self::new(
            source,
            config.destination_root,
            selections,
            config.max_total_bytes,
            writable,
        )?;
        port._settings = Some(settings);
        Ok(port)
    }
    pub fn new(
        source: PathBuf,
        destination: PathBuf,
        selections: BTreeMap<String, FixedMediaSelection>,
        max_total_bytes: u64,
        writable: bool,
    ) -> Result<Self, ApiError> {
        if selections.is_empty()
            || selections.len() > 64
            || max_total_bytes == 0
            || max_total_bytes > 4_294_967_296
            || selections
                .keys()
                .any(|id| !crate::domain::review_source::valid_review_object_id(id))
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid fixed media preparation configuration",
            ));
        }
        let (a, mut guards) = rooted::root(&source)?;
        let (b, held) = rooted::root(&destination)?;
        guards.extend(held);
        let a = PathBuf::from(a.to_string_lossy().to_lowercase());
        let b = PathBuf::from(b.to_string_lossy().to_lowercase());
        if a.starts_with(&b) || b.starts_with(&a) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Media preparation roots must be disjoint",
            ));
        }
        ReviewFiles::open(&source)?;
        Ok(Self {
            source,
            destination,
            selections,
            max_total_bytes,
            writable,
            _guards: guards,
            _settings: None,
        })
    }
    fn selection(&self, input: &MediaSelectionRequest) -> Result<&FixedMediaSelection, ApiError> {
        input.validate()?;
        self.selections
            .get(&input.selection_id)
            .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Fixed media selection missing"))
    }
}
impl MediaPreparationPort for MediaPreparationFiles {
    fn writable(&self) -> bool {
        self.writable
    }
    fn prepare(
        &self,
        input: &MediaSelectionRequest,
        control: &dyn MutationControl,
    ) -> Result<MediaPreparationReceipt, ApiError> {
        if !self.writable {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Media preparation writes unavailable",
            ));
        }
        let selection = self.selection(input)?;
        let source = ReviewFiles::open(&self.source)?;
        let prepared = prepare_local_media(
            &source,
            &input.base_case,
            LocalMediaSelection {
                path: selection.path.clone(),
                asset: selection.asset.clone(),
            },
            TimelinePackageDestination {
                root: self.destination.clone(),
                max_total_bytes: self.max_total_bytes,
                excluded_roots: vec![self.source.clone()],
            },
            control,
        )?;
        let receipt = MediaPreparationReceipt {
            selection_id: input.selection_id.clone(),
            case: prepared.case().clone(),
            relation: prepared.relation().clone(),
            replayed: prepared.replayed(),
        };
        receipt.validate(input)?;
        Ok(receipt)
    }
    fn inspect(&self, input: &MediaSelectionRequest) -> Result<MediaPreparationReceipt, ApiError> {
        use std::io::{Read, Seek, SeekFrom};
        let selection = self.selection(input)?;
        let source = ReviewFiles::open(&self.source)?;
        let mut export = source.export_case(&input.base_case)?;
        export
            .package_file
            .seek(SeekFrom::Start(0))
            .map_err(io_error)?;
        let mut bytes = Vec::new();
        Read::by_ref(&mut export.package_file)
            .take(524289)
            .read_to_end(&mut bytes)
            .map_err(io_error)?;
        let prepared =
            media_preparation::prepare(&bytes, &input.base_case, selection.asset.clone())?;
        let store = ReviewFiles::open(&self.destination)?;
        let _session = crate::application::model_review::ReviewStore::open(
            &store,
            &prepared.case.package_sha256,
        )?;
        // ReviewFiles verifies the exact content-addressed package and all retained objects.
        // External selected path is not required to read an already committed package.
        let receipt = MediaPreparationReceipt {
            selection_id: input.selection_id.clone(),
            case: prepared.case,
            relation: prepared.relation,
            replayed: true,
        };
        receipt.validate(input)?;
        Ok(receipt)
    }
}
