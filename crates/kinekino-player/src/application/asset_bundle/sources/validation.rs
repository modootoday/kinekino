use super::*;
use crate::{
    application::asset_projects::{AssetPublication, publication_sha256},
    domain::identifiers::valid_sha256,
};
impl AssetSourceManifest {
    /// Metadata consistency only; original file hashes/decodes remain host obligations.
    pub fn validate_project(
        &self,
        bytes: &[u8],
        manifest_bytes: u64,
        control: &dyn MutationControl,
    ) -> Result<AssetPublication, ApiError> {
        control.checkpoint()?;
        if bytes.len() as u64 > crate::application::asset_artifacts::MAX_ASSET_PUBLICATION_BYTES
            || manifest_bytes == 0
            || manifest_bytes > crate::application::asset_artifacts::MAX_ASSET_PUBLICATION_BYTES
            || self.schema_version != "asset-sources/1"
            || self.quality_approved
            || self.review_admitted
        {
            return Err(invalid());
        }
        self.project.validate()?;
        let project: AssetPublication = serde_json::from_slice(bytes).map_err(|_| invalid())?;
        project.document.validate().map_err(|_| invalid())?;
        if project.schema_version != "asset-project-publication/1"
            || project.publication_revision != self.project.publication_revision
            || project.document.document_id != self.project.document_id
            || publication_sha256(&project)? != self.project.publication_sha256
            || self.project_file != descriptor("project.json".into(), bytes, "application/json")
            || self.project_file.sha256 != self.project.publication_sha256
            || (project.publication_revision.0 == 1) != project.parent_publication_sha256.is_none()
            || project
                .parent_publication_sha256
                .as_ref()
                .is_some_and(|s| !valid_sha256(s))
        {
            return Err(invalid());
        }
        let mut bindings = Vec::new();
        let mut missing = Vec::new();
        let mut hashes = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for frame in &project.document.frames {
            control.checkpoint()?;
            match frame {
                AssetFrameEntry::Missing { .. } => missing.push(frame.frame_id().to_owned()),
                AssetFrameEntry::Available { frame_id, input } => {
                    for sha in [&input.image.blob.sha256, &input.mask.blob.sha256] {
                        if seen.insert(sha) {
                            hashes.push(sha);
                        }
                    }
                    bindings.push(AssetSourceBinding {
                        frame_id: frame_id.clone(),
                        image: input.image.clone(),
                        mask: input.mask.clone(),
                    });
                }
            }
        }
        if bindings.is_empty()
            || self.bindings != bindings
            || self.missing_frames != missing
            || self.originals.len() != hashes.len()
        {
            return Err(invalid());
        }
        let mut total = bytes.len() as u64;
        add(&mut total, manifest_bytes)?;
        for (file, sha) in self.originals.iter().zip(hashes) {
            control.checkpoint()?;
            if file.name != format!("source-{sha}.png")
                || file.sha256 != *sha
                || file.media_type != "image/png"
                || file.bytes.0 == 0
                || file.bytes.0 > crate::domain::frame_capture::MAX_FRAME_PNG_BYTES
            {
                return Err(invalid());
            }
            add(&mut total, file.bytes.0)?;
        }
        Ok(project)
    }
}
