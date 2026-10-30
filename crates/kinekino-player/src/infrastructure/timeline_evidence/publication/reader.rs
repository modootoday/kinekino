//! Only an actual publisher-created token supplies authority; JSON summaries do not.
mod data;
mod trace;
use super::*;
use crate::{
    application::review_source::{
        RetainedReviewEvidence, ReviewCaseEvidence,
        timeline_evidence::{TimelineEvidenceLease, TimelineEvidenceResolver},
    },
    domain::review_source::{MediaReference, timeline_evidence::TimelineObservations},
};
use std::collections::BTreeMap;
pub struct PublishedTimelineResolver {
    publication: PublishedTimelinePackage,
    reader: ReviewCaseEvidence<ReviewFiles>,
}
impl PublishedTimelineResolver {
    pub fn new(publication: PublishedTimelinePackage) -> Result<Self, ApiError> {
        publication.check_current()?;
        let reader = ReviewCaseEvidence::new(ReviewFiles::open(publication.root())?);
        Ok(Self {
            publication,
            reader,
        })
    }
}
struct Lease {
    input: RetainedReviewEvidence,
    facts: TimelineObservations,
}
impl TimelineEvidenceLease for Lease {
    fn observations(&self) -> &TimelineObservations {
        &self.facts
    }
    fn check_current(&self) -> Result<(), ApiError> {
        self.input.check_current()
    }
}
struct Context<'a> {
    input: &'a RetainedReviewEvidence,
    manifest: &'a Manifest,
    token: &'a PublishedTimelinePackage,
    entries: BTreeMap<(String, String), &'a Artifact>,
}
fn reference(asset: &EvidenceAsset) -> EvidenceReference {
    EvidenceReference {
        asset_id: asset.asset_id.clone(),
        sha256: asset.sha256.clone(),
    }
}
fn json<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, ApiError> {
    serde_json::from_slice(bytes)
        .map_err(|_| invalid("Malformed or missing mandatory live trace fields"))
}
impl Context<'_> {
    fn asset(&self, role: &str, name: &str) -> Result<&EvidenceAsset, ApiError> {
        match &self
            .entries
            .get(&(role.into(), name.into()))
            .ok_or_else(|| invalid("Required live artifact missing"))?
            .content
        {
            ArtifactContent::Asset { asset } => Ok(asset),
            _ => Err(invalid("Nonempty live artifact required")),
        }
    }
    fn bytes(&self, role: &str, name: &str, max: u64) -> Result<Vec<u8>, ApiError> {
        match &self
            .entries
            .get(&(role.into(), name.into()))
            .ok_or_else(|| invalid("Required live artifact missing"))?
            .content
        {
            ArtifactContent::Asset { asset } => self.input.read(&reference(asset), max),
            ArtifactContent::Empty { bytes, sha256 }
                if *bytes == 0 && *sha256 == format!("{:x}", Sha256::digest([])) =>
            {
                Ok(Vec::new())
            }
            _ => Err(invalid("Invalid empty live artifact")),
        }
    }
    fn contract(&self, role: &str) -> Result<&RoleContract, ApiError> {
        self.manifest
            .contracts
            .iter()
            .find(|entry| entry.role == role)
            .ok_or_else(|| invalid("Required role contract missing"))
    }
    fn tool(&self, role: &str) -> Result<&ToolPin, ApiError> {
        self.manifest
            .tools
            .iter()
            .find(|entry| entry.role == role)
            .ok_or_else(|| invalid("Required tool pin missing"))
    }
}
impl TimelineEvidenceResolver for PublishedTimelineResolver {
    fn retain(
        &self,
        case: &ReviewCase,
        media: &MediaReference,
        evidence: &EvidenceReference,
    ) -> Result<Box<dyn TimelineEvidenceLease + '_>, ApiError> {
        let token = &self.publication;
        if case != &token.case || media != &token.snapshot.media || evidence != &token.reference {
            return Err(conflict("Foreign live timeline binding"));
        }
        token.check_current()?;
        let input = self.reader.retain(case)?;
        let manifest: Manifest = json(&input.read(evidence, 65536)?)?;
        if manifest.schema_version != "live_2_1"
            || manifest.base_case != token.base_case
            || manifest.media != *media
            || manifest.bundle_sha256 != token.bundle_sha256
            || manifest.mapping_revision != token.snapshot.mapping_revision
            || manifest.contracts != token.contracts
            || manifest.tools != token.tools
            || manifest.producer != token.producer
            || manifest.budget != token.budget
        {
            return Err(conflict(
                "Live manifest differs from original publisher token",
            ));
        }
        let mut entries = BTreeMap::new();
        for entry in &manifest.artifacts {
            if entries
                .insert((entry.role.clone(), entry.name.clone()), entry)
                .is_some()
            {
                return Err(invalid("Duplicate live artifact role"));
            }
        }
        let context = Context {
            input: &input,
            manifest: &manifest,
            token,
            entries,
        };
        trace::verify(&context)?;
        data::verify(&context)?;
        let base_bytes = input.read(&reference(&manifest.base_package), 524288)?;
        if manifest.base_package.sha256 != token.base_case.package_sha256 {
            return Err(conflict("Base package reference differs"));
        }
        let base: crate::domain::model_review::ReviewPackage = json(&base_bytes)?;
        base.validate()
            .map_err(|_| invalid("Base review package invalid"))?;
        let mut added = vec![manifest.base_package.clone()];
        for entry in &manifest.artifacts {
            if let ArtifactContent::Asset { asset } = &entry.content {
                if input.inspect(&reference(asset))?.bytes != asset.bytes {
                    return Err(conflict("Live artifact metadata differs"));
                }
                added.push(asset.clone());
            }
        }
        added.push(input.inspect(evidence)?);
        let body = package::derive(
            &base,
            &token.base_case,
            &format!("live-{}", &token.bundle_sha256[..12]),
            added,
        )?;
        if format!("{:x}", Sha256::digest(&body)) != case.package_sha256
            || serde_json::to_value(json::<crate::domain::model_review::ReviewPackage>(&body)?).ok()
                != serde_json::to_value(input.package_snapshot()).ok()
        {
            return Err(conflict("Derived package changed original model evidence"));
        }
        token
            .snapshot
            .validate()
            .map_err(|_| invalid("Live domain observations inconsistent"))?;
        input.check_current()?;
        Ok(Box::new(Lease {
            input,
            facts: token.snapshot.clone(),
        }))
    }
}
