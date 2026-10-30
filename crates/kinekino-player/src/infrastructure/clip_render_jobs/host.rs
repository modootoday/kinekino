//! Pinned local administrative configuration, never supplied by SDK request JSON.
use super::*;
use crate::infrastructure::{
    clip_projects::ClipProjectFiles,
    clip_sources::{ClipContainer, FixedClipSource},
    immutable_storage_paths as paths,
};
use std::{collections::BTreeMap, io::Read, path::Path};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HostDocument {
    schema_version: String,
    project_root: PathBuf,
    render: ClipRenderHostConfig,
    sources: BTreeMap<String, Source>,
    #[serde(default)]
    subtitle_assets: Option<SubtitleCatalog>,
    #[serde(default)]
    preset_files: BTreeMap<String, PathBuf>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SubtitleCatalog {
    documents: BTreeMap<String, PathBuf>,
    styles: BTreeMap<String, PathBuf>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    path: PathBuf,
    container: Container,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Container {
    Mp4,
    Matroska,
}
pub struct OpenedClipRenderHost {
    pub projects: Arc<dyn ClipProjectStore>,
    pub executor: Arc<dyn JobExecutor>,
    pub sources: Arc<dyn crate::application::clip_render::ClipSourceInspector>,
    pub config_sha256: String,
    pub presets: Arc<dyn crate::application::clip_render::presets::ClipPresetCatalogPort>,
}
impl OpenedClipRenderHost {
    pub fn open(path: &Path, expected_sha256: &str) -> Result<Self, ApiError> {
        if !valid_sha256(expected_sha256) {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Pinned clip host configuration SHA256 required",
            ));
        }
        let name = path.file_name().ok_or_else(|| {
            error(
                ErrorCode::InvalidArgument,
                "Clip host configuration filename required",
            )
        })?;
        let (parent, mut guards) = paths::root(path.parent().ok_or_else(|| {
            error(
                ErrorCode::InvalidArgument,
                "Absolute clip host configuration required",
            )
        })?)?;
        let file = paths::file(&parent.join(name))?;
        if file
            .metadata()
            .map_err(kinekino_core::file_publication::io_error)?
            .len()
            > 1_048_576
        {
            return Err(error(
                ErrorCode::ResourceExhausted,
                "Clip host configuration exceeds byte limit",
            ));
        }
        let mut bytes = Vec::new();
        (&file)
            .take(1_048_577)
            .read_to_end(&mut bytes)
            .map_err(kinekino_core::file_publication::io_error)?;
        if bytes.len() > 1_048_576 || hash(&bytes) != expected_sha256 {
            return Err(error(
                ErrorCode::SourceExpired,
                "Clip host configuration differs from its pin",
            ));
        }
        let document: HostDocument = serde_json::from_slice(&bytes).map_err(|_| {
            error(
                ErrorCode::InvalidArgument,
                "Expected strict clip-render-host/1 configuration",
            )
        })?;
        if document.schema_version != "clip-render-host/1"
            || document.sources.is_empty()
            || document.sources.len() > 256
        {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Invalid clip host schema or source catalog count",
            ));
        }
        if document
            .sources
            .values()
            .any(|source| !source.path.is_absolute())
        {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Clip source catalog requires absolute local paths",
            ));
        }
        let catalog = document
            .sources
            .into_iter()
            .map(|(id, source)| {
                (
                    id,
                    FixedClipSource {
                        path: source.path,
                        container: match source.container {
                            Container::Mp4 => ClipContainer::Mp4,
                            Container::Matroska => ClipContainer::Matroska,
                        },
                    },
                )
            })
            .collect();
        let sources = Arc::new(FfmpegClipSources::new(
            catalog,
            &document.render.probe,
            &document.render.probe_sha256,
            crate::domain::frame_capture::MAX_MEDIA_BYTES,
            Duration::from_secs(120),
        )?);
        let projects: Arc<dyn ClipProjectStore> =
            Arc::new(ClipProjectFiles::open(&document.project_root)?);
        let mut executor =
            ClipRenderJobExecutor::open(projects.clone(), sources.clone(), document.render)?;
        let presets = Arc::new(FixedClipPresets::open(document.preset_files)?);
        executor = executor.with_preset_catalog(presets.clone())?;
        if let Some(assets) = document.subtitle_assets {
            executor = executor
                .with_subtitle_assets(SubtitleAssetFiles::open(assets.documents, assets.styles)?)?;
        }
        // Bind catalog paths and project root as well as runtime settings into capture.
        executor.host_sha256 = hash(
            &serde_json::to_vec(&[executor.host_sha256.as_str(), expected_sha256])
                .map_err(|_| error(ErrorCode::InvalidArgument, "Invalid clip host binding"))?,
        );
        guards.push(file);
        executor._host_guards = Arc::new(guards);
        Ok(Self {
            projects,
            executor: Arc::new(executor),
            sources,
            config_sha256: expected_sha256.into(),
            presets,
        })
    }
}
