use super::*;
use crate::application::jobs::{ExecutionControl, VerifiedJobOutput};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleFile {
    pub name: String,
    pub sha256: String,
    pub bytes: DecimalU64,
    pub media_type: String,
}
pub trait CardBundlePublication: Send + Sync {
    fn begin(&self, output_directory: &str) -> Result<Box<dyn CardBundleStaging>, ApiError>;
}
/// Owns original writer handles until failure/drop or a verified create-only directory rename.
pub trait CardBundleStaging {
    fn write_new(
        &mut self,
        name: &str,
        bytes: &[u8],
        media_type: &str,
    ) -> Result<BundleFile, ApiError>;
    fn verify_all(&mut self, control: &dyn ExecutionControl) -> Result<(), ApiError>;
    fn publish_new(
        &mut self,
        manifest: &BundleFile,
        control: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError>;
    fn recovery_details(&self) -> Value;
}
pub fn check_control(control: &dyn ExecutionControl) -> Result<(), ApiError> {
    if control.cancellation_requested() {
        return Err(error(
            ErrorCode::Cancelled,
            "Card bundle cancellation requested",
        ));
    }
    if std::time::Instant::now() >= control.deadline() {
        return Err(error(ErrorCode::Timeout, "Card bundle deadline expired"));
    }
    Ok(())
}

/// Manifest and raw frames keep the exported editable project resolvable without a fake acceptance.
pub fn execute_bundle(
    bundle: &PreparedCardBundle,
    raster: &dyn CardBundleRasterizer,
    output: &dyn CardBundlePublication,
    control: &dyn ExecutionControl,
) -> Result<VerifiedJobOutput, ApiError> {
    check_control(control)?;
    let identity = control.execution_identity().ok_or_else(|| {
        error(
            ErrorCode::Unsupported,
            "Original Jobs attempt identity is required",
        )
    })?;
    let attempt = json!({"schema_version":"card-bundle-attempt/1","job_id":identity.job_id().as_str(),"attempt":DecimalU64(identity.attempt()),"input_sha256":identity.input_sha256(),"source_blob_sha256":identity.source_blob_sha256(),"submission_request_sha256":identity.submission_request_sha256(),"quality_approved":false});
    let mut staging = output.begin(&bundle.output_directory)?;
    let result = (|| {
        let mut files = Vec::new();
        files.push(staging.write_new(
            "attempt.json",
            &serde_json::to_vec(&attempt).map_err(internal)?,
            "application/json",
        )?);
        let mut frames = BTreeMap::new();
        let mut frame_records = Vec::new();
        for prepared in &bundle.cards {
            check_control(control)?;
            let (frame, observation) = match (&prepared.frame, &prepared.observation) {
                (None, None) => continue,
                (Some(frame), Some(observation)) => (frame, observation),
                _ => {
                    return Err(error(
                        ErrorCode::InvalidArgument,
                        "Incomplete prepared frame identity",
                    ));
                }
            };
            let name = format!("frame-{}.png", frame.png_sha256);
            if !frames.contains_key(&frame.png_sha256) {
                let bytes = raster.read_frame_png(frame)?;
                let entry = staging.write_new(&name, &bytes, "image/png")?;
                if entry.sha256 != frame.png_sha256 {
                    return Err(error(
                        ErrorCode::SourceExpired,
                        "Bundled source frame differs from its pin",
                    ));
                }
                frames.insert(frame.png_sha256.clone(), name.clone());
                files.push(entry);
            }
            frame_records.push(json!({"artifact_id":frame.artifact_id,"path":name,"sha256":frame.png_sha256,"width":observation.width,"height":observation.height}));
        }
        let mut alt = Vec::new();
        for prepared in &bundle.cards {
            check_control(control)?;
            let rendered = raster.render_card(bundle, prepared.position)?;
            check_control(control)?;
            files.push(staging.write_new(&prepared.output_name, &rendered.png, "image/png")?);
            let stem = prepared
                .output_name
                .strip_suffix(".png")
                .ok_or_else(|| error(ErrorCode::InvalidArgument, "Invalid planned PNG filename"))?;
            files.push(staging.write_new(
                &format!("{stem}.provenance.json"),
                &serde_json::to_vec(&rendered.provenance).map_err(internal)?,
                "application/json",
            )?);
            let index = usize::try_from(prepared.position.0)
                .map_err(|_| error(ErrorCode::InvalidArgument, "Invalid card position"))?;
            let card = bundle
                .publication
                .document
                .layout
                .cards
                .get(index)
                .ok_or_else(|| error(ErrorCode::InvalidArgument, "Card position absent"))?;
            alt.push(json!({"card_id":card.card_id,"output_file":prepared.output_name,"text":card.alt_text_draft}));
        }
        check_control(control)?;
        files.push(staging.write_new(
            "caption.txt",
            bundle.publication.document.layout.caption_draft.as_bytes(),
            "text/plain; charset=utf-8",
        )?);
        files.push(staging.write_new(
            "alt-text.json",
            &serde_json::to_vec(&alt).map_err(internal)?,
            "application/json",
        )?);
        files.push(staging.write_new(
            "project.json",
            &serde_json::to_vec(&bundle.publication).map_err(internal)?,
            "application/json",
        )?);
        let manifest = json!({"schema_version":"card-bundle/1","project":bundle.project,"document_edit_revision":bundle.publication.document.revision,"template":bundle.template,"source_frames":frame_records,"files":files,"state":"review_artifact","quality_approved":false,"template_approved":false,"media_provenance_verified":false,"execution":attempt});
        let manifest_file = staging.write_new(
            "manifest.json",
            &serde_json::to_vec(&manifest).map_err(internal)?,
            "application/json",
        )?;
        check_control(control)?;
        staging.verify_all(control)?;
        check_control(control)?;
        staging.publish_new(&manifest_file, control)
    })();
    result.map_err(|mut failure: ApiError| {
        failure.details = json!({"primary_details":failure.details,"bundle_recovery":staging.recovery_details(),"completed":false});
        failure
    })
}
fn internal(error: impl std::fmt::Display) -> ApiError {
    ApiError::new(ErrorCode::Internal, error.to_string())
}
