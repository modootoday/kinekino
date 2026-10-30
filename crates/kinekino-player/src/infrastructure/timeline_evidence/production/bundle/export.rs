use super::*;
use crate::{application::review_playback::ReviewInputIdentity, infrastructure::content_digest};
use crate::{
    domain::{
        api_values::DecimalU64,
        review_source::{EvidenceReference, timeline_evidence::TimelineObservations},
    },
    infrastructure::timeline_evidence::live_contract::{
        LiveExecutionContract as Contract, ProducerPin, RoleContract, ToolPin,
    },
};
pub(crate) struct ProducedArtifact {
    pub role: String,
    pub name: String,
    pub file: File,
    pub bytes: u64,
    pub sha256: String,
    pub media_type: String,
}
impl ProducedTimelineBundle<'_, '_> {
    pub(crate) fn publication_budget(
        &self,
    ) -> crate::infrastructure::timeline_evidence::live_contract::ResourceBudget {
        crate::infrastructure::timeline_evidence::live_contract::ResourceBudget {
            memory_committed_bytes: self.session.budget.max_committed_bytes as u64,
            affinity_mask: self.session.budget.affinity_mask,
            max_output_bytes: self.session.max_output_bytes,
            cpu_threads: 1,
        }
    }
    pub(crate) fn publication_video_index(&self) -> u32 {
        self.inputs.video.frames().stream_index()
    }
    pub(crate) fn publication_producer(&self) -> ProducerPin {
        self.producer.clone()
    }
    pub(crate) fn source_path(&self) -> String {
        self.session.source.path().to_string_lossy().into()
    }
    pub(crate) fn publication_contracts(&self) -> Vec<RoleContract> {
        let mut contracts = vec![
            RoleContract {
                role: "video".into(),
                root: self.inputs.video.output_root().to_string_lossy().into(),
                contract: Contract::VideoFirstStreamWindow {
                    interval: self.inputs.video.interval.clone(),
                },
            },
            RoleContract {
                role: "original-pcm".into(),
                root: self
                    .inputs
                    .pcm
                    .original
                    .clock
                    .output_root()
                    .to_string_lossy()
                    .into(),
                contract: Contract::AudioSourceCopyTsMono16kS16,
            },
            RoleContract {
                role: "replay-pcm".into(),
                root: self.inputs.pcm.output_root().to_string_lossy().into(),
                contract: Contract::RawPcmReplayMono16kS16,
            },
        ];
        for (role, capture) in [
            ("origin", self.inputs.origin),
            ("start", self.inputs.start),
            ("end", self.inputs.end),
        ] {
            contracts.push(RoleContract {
                role: role.into(),
                root: capture.output_root().to_string_lossy().into(),
                contract: match capture.point {
                    TimelineCapturePoint::Origin => Contract::FirstDecodedOrigin,
                    TimelineCapturePoint::AtOrAfter { source_us } => {
                        Contract::CaptureAtOrAfter { source_us }
                    }
                },
            });
        }
        contracts
    }
    pub(crate) fn publication_tools(&self) -> Vec<ToolPin> {
        [
            ("probe", &self.inputs.video._tool.0),
            ("audio-engine", &self.inputs.pcm.original.clock._tool.0),
            ("capture-engine", &self.inputs.origin._tool.0),
        ]
        .into_iter()
        .map(|(role, tool)| ToolPin {
            role: role.into(),
            path: tool.path.to_string_lossy().into(),
            sha256: tool.sha256().into(),
            file_identity: tool.file_identity().into(),
        })
        .collect()
    }
    pub(crate) fn publication_snapshot(
        &self,
        case: crate::domain::review_source::ReviewCase,
        reference: EvidenceReference,
        revision: DecimalU64,
    ) -> Result<TimelineObservations, ApiError> {
        let source = &self.session.source.identity().case;
        if case.item_id != source.item_id
            || case.source_request_sha256 != source.source_request_sha256
        {
            return Err(invalid("Published case source relationship differs"));
        }
        let facts = TimelineObservations {
            video_cadence: self.inputs.video.frames().cadence(),
            case,
            media: self.session.source.identity().media.clone(),
            reference,
            mapping_revision: revision,
            time_base: self.inputs.video.frames().time_base(),
            origin: self.origin.clone(),
            video_frames: self.inputs.video.frames().frames().to_vec(),
            pcm: self.pcm.clone(),
            start_capture: self.inputs.start.observation().clone(),
            end_capture: self.inputs.end.observation().clone(),
        };
        if facts
            .validate()
            .map_err(|_| invalid("Published timeline snapshot invalid"))?
            != self.mapping
        {
            return Err(invalid("Published mapping differs"));
        }
        Ok(facts)
    }
    pub(crate) fn source_identity(&self) -> &ReviewInputIdentity {
        self.session.source.identity()
    }
    pub(crate) fn protected_roots(&self) -> Vec<PathBuf> {
        let mut roots = vec![self.session.root.clone()];
        if let Some(parent) = Path::new(&self.producer.path).parent() {
            roots.push(parent.to_path_buf());
        }
        for tool in [
            &self.inputs.video._tool.0,
            &self.inputs.pcm.original.clock._tool.0,
            &self.inputs.origin._tool.0,
        ] {
            if let Some(parent) = tool.path.parent() {
                roots.push(parent.to_path_buf());
            }
        }
        roots
    }
    pub(crate) fn check_publication(&self, control: &dyn MutationControl) -> Result<(), ApiError> {
        self.publication_checkpoint(control)?;
        self.check_current()?;
        self.publication_checkpoint(control)
    }
    pub(crate) fn publication_checkpoint(
        &self,
        control: &dyn MutationControl,
    ) -> Result<(), ApiError> {
        control.checkpoint()?;
        if self
            .session
            .deadline
            .operation_remaining(Instant::now())
            .is_zero()
        {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Timeline publication exceeded original operation deadline",
            ));
        }
        Ok(())
    }
    pub(crate) fn within_publication<T>(
        &self,
        control: &dyn MutationControl,
        operation: impl FnOnce(Vec<ProducedArtifact>) -> Result<T, ApiError>,
    ) -> Result<T, ApiError> {
        if self.session.active.get() != 0 {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Timeline component active or abandoned",
            ));
        }
        self.session.active.set(1);
        let _publication = Publication(&self.session.active);
        self.check_publication(control)?;
        let mut artifacts = Vec::new();
        for (role, root) in [
            ("video", self.inputs.video.output_root()),
            ("original-pcm", self.inputs.pcm.original.clock.output_root()),
            ("replay-pcm", self.inputs.pcm.output_root()),
            ("origin", self.inputs.origin.output_root()),
            ("start", self.inputs.start.output_root()),
            ("end", self.inputs.end.output_root()),
        ] {
            let mut names = vec![
                "INPUT.json",
                "STARTED.json",
                "PROCESS.json",
                "RETURN.json",
                "stdout.raw",
                "stderr.raw",
            ];
            if role == "replay-pcm" {
                names.extend(["DERIVED-INPUT.json", "clip.s16le"]);
            }
            if ["origin", "start", "end"].contains(&role) {
                names.push("VIDEO-INPUT.json");
            }
            for name in names {
                artifacts.push(export(role, name, &root.join(name), 64 * 1024 * 1024)?);
            }
            self.check_publication(control)?;
        }
        artifacts.push(export(
            "bundle",
            "BUNDLE.json",
            &self.root.join("BUNDLE.json"),
            64 * 1024 * 1024,
        )?);
        let producer = export(
            "producer",
            "engine.bin",
            Path::new(&self.producer.path),
            512 * 1024 * 1024,
        )?;
        if producer.sha256 != self.producer.sha256 {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Rust producer identity changed",
            ));
        }
        artifacts.push(producer);
        for (role, tool) in [
            ("probe", &self.inputs.video._tool.0),
            ("audio-engine", &self.inputs.pcm.original.clock._tool.0),
            ("capture-engine", &self.inputs.origin._tool.0),
        ] {
            let artifact = export(role, "engine.bin", &tool.path, 512 * 1024 * 1024)?;
            if artifact.sha256 != tool.sha256() {
                return Err(ApiError::new(
                    ErrorCode::SourceExpired,
                    "Producer tool changed during export",
                ));
            }
            artifacts.push(artifact);
            self.check_publication(control)?;
        }
        operation(artifacts)
    }
}
fn export(
    role: &str,
    name: &str,
    path: &Path,
    max_bytes: u64,
) -> Result<ProducedArtifact, ApiError> {
    let mut file = rooted::file(path)?;
    let (bytes, sha256) = content_digest::sha256(&mut file, max_bytes)?;
    Ok(ProducedArtifact {
        role: role.to_owned(),
        name: name.to_owned(),
        file,
        bytes,
        sha256,
        media_type: if name.ends_with(".json") {
            "application/json"
        } else {
            "application/octet-stream"
        }
        .to_owned(),
    })
}
