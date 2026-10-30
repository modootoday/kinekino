mod metadata;
mod movie;
use super::*;
use crate::infrastructure::{
    clip_working_file::{ClipWorkingFile, ClipWorkingFileRelease, WorkingFileFacts},
    frame_capture::tools,
    immutable_storage_paths as paths,
};
pub use metadata::ClipOutputMetadata;
use std::{
    fs::File,
    path::{Path, PathBuf},
};
#[derive(Serialize)]
pub struct ClipOutputVerification {
    pub(crate) working_input: WorkingFileFacts,
    pub input_transport: String,
    pub schema_version: String,
    pub output_sha256: String,
    pub probe_sha256: String,
    pub decoder_sha256: String,
    pub raw_probe_sha256: String,
    pub decode_stdout_sha256: String,
    pub decode_stderr_sha256: String,
    pub metadata: ClipOutputMetadata,
    pub decode_integrity_verified: bool,
    pub color_verified: bool,
    pub quality_approved: bool,
}
pub struct VerifiedClipCandidate<'a> {
    candidate: &'a EncodedClipCandidate,
    proof: ProofRef<'a>,
}
struct VerificationProof {
    record: ClipOutputVerification,
    raw_probe: Vec<u8>,
    working: Option<ClipWorkingFile>,
}
enum ProofRef<'a> {
    Owned(Box<VerificationProof>),
    Borrowed(&'a VerificationProof),
}
/// Owns both original encoded bytes and the original non-deserializable verification proof.
pub struct OwnedVerifiedClipCandidate {
    candidate: EncodedClipCandidate,
    proof: Box<VerificationProof>,
}
impl OwnedVerifiedClipCandidate {
    pub(crate) fn has_working(&self) -> bool {
        self.proof.working.is_some()
    }
    pub(crate) fn take_working_release(&mut self) -> Option<ClipWorkingFileRelease> {
        self.proof.working.take().map(ClipWorkingFile::into_release)
    }
    pub fn candidate(&self) -> &EncodedClipCandidate {
        &self.candidate
    }
    pub fn as_verified(&self) -> VerifiedClipCandidate<'_> {
        VerifiedClipCandidate {
            candidate: &self.candidate,
            proof: ProofRef::Borrowed(&self.proof),
        }
    }
}
impl VerifiedClipCandidate<'_> {
    fn proof(&self) -> &VerificationProof {
        match &self.proof {
            ProofRef::Owned(proof) => proof,
            ProofRef::Borrowed(proof) => proof,
        }
    }
    pub fn candidate(&self) -> &EncodedClipCandidate {
        self.candidate
    }
    pub fn verification(&self) -> &ClipOutputVerification {
        &self.proof().record
    }
    pub fn raw_probe(&self) -> &[u8] {
        &self.proof().raw_probe
    }
    pub fn spool_path(&self) -> &Path {
        self.proof()
            .working
            .as_ref()
            .expect("Original retained verification input")
            .path()
    }
}
pub struct ClipOutputVerifier {
    probe: PathBuf,
    probe_sha256: String,
    decoder: PathBuf,
    decoder_sha256: String,
    temporary: PathBuf,
    timeout: Duration,
    _guards: Vec<File>,
}
struct Control<'a> {
    outer: &'a dyn MutationControl,
    deadline: Instant,
}
impl MutationControl for Control<'_> {
    fn native_processes(&self) -> Option<&crate::application::review_source::NativeProcessLedger> {
        self.outer.native_processes()
    }
    fn checkpoint(&self) -> Result<(), ApiError> {
        self.outer.checkpoint()?;
        if Instant::now() >= self.deadline {
            Err(ApiError::new(
                ErrorCode::Timeout,
                "Clip output verification deadline elapsed",
            ))
        } else {
            Ok(())
        }
    }
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
impl ClipOutputVerifier {
    pub fn verify_owned(
        &self,
        candidate: EncodedClipCandidate,
        outer: &dyn MutationControl,
    ) -> Result<OwnedVerifiedClipCandidate, ApiError> {
        let VerifiedClipCandidate { proof, .. } = self.verify(&candidate, outer)?;
        let ProofRef::Owned(proof) = proof else {
            unreachable!("Verifier creates original owned proof")
        };
        Ok(OwnedVerifiedClipCandidate { candidate, proof })
    }
    pub fn open(
        probe: &Path,
        probe_sha256: &str,
        decoder: &Path,
        decoder_sha256: &str,
        temporary: &Path,
        timeout: Duration,
    ) -> Result<Self, ApiError> {
        if timeout.is_zero() || timeout > Duration::from_secs(120) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Bounded output verification timeout required",
            ));
        }
        let (probe, mut guards) = tools::open(probe, probe_sha256)?;
        let (decoder, held) = tools::open(decoder, decoder_sha256)?;
        guards.extend(held);
        let (_canonical_temporary, held) = paths::root(temporary)?;
        guards.extend(held);
        Ok(Self {
            probe,
            probe_sha256: probe_sha256.into(),
            decoder,
            decoder_sha256: decoder_sha256.into(),
            temporary: temporary.to_path_buf(),
            timeout,
            _guards: guards,
        })
    }
    pub fn verify<'a>(
        &self,
        candidate: &'a EncodedClipCandidate,
        outer: &dyn MutationControl,
    ) -> Result<VerifiedClipCandidate<'a>, ApiError> {
        let deadline = Instant::now() + self.timeout;
        let control = Control { outer, deadline };
        control.checkpoint()?;
        if candidate.bytes.len() != candidate.facts.output_bytes
            || hash(&candidate.bytes) != candidate.facts.output_sha256
            || self.decoder_sha256 != candidate.facts.encoder_tool_sha256
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Clip candidate bytes or decoder pin differ from their original identity",
            ));
        }
        let (scale, duration) = movie::header(&candidate.bytes)?;
        let mut working = ClipWorkingFile::create(&self.temporary, 128 * 1024 * 1024, &control)?;
        let working_input = working.seal(&candidate.bytes, &control)?;
        if working_input.bytes != candidate.facts.output_bytes as u64
            || working_input.sha256 != candidate.facts.output_sha256
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Retained verification input differs from candidate bytes",
            ));
        }
        let mut inspect = Command::new(&self.probe);
        inspect.args(["-v","error","-threads","1","-protocol_whitelist","file,fd","-f","mov","-enable_drefs","0","-count_frames","-show_entries","stream=codec_type,codec_name,width,height,pix_fmt,sample_aspect_ratio,color_range,color_space,color_transfer,color_primaries,nb_read_frames,r_frame_rate,start_pts,time_base,duration_ts,sample_rate,channels:format=start_time,duration","-of","json"]).args(["-fd", "0", "fd:"]);
        let raw = working.with_reader(|input| {
            process::run_with_input(&mut inspect, deadline, 262144, Some(&control), input)
        })?;
        control.checkpoint()?;
        let json = serde_json::from_slice(&raw.stdout)
            .map_err(|_| ApiError::new(ErrorCode::NeedsInput, "Invalid output probe JSON"))?;
        let metadata = metadata::validate(&candidate.facts.plan, &json, scale, duration)?;
        let mut decode = Command::new(&self.decoder);
        decode
            .args([
                "-v",
                "error",
                "-xerror",
                "-err_detect",
                "explode",
                "-threads",
                "2",
                "-protocol_whitelist",
                "file,fd",
                "-f",
                "mov",
                "-enable_drefs",
                "0",
                "-fd",
                "0",
                "-i",
            ])
            .arg("fd:")
            .args(["-map", "0", "-f", "null", "-"]);
        let decoded = working.with_reader(|input| {
            process::run_with_input(&mut decode, deadline, 65536, Some(&control), input)
        })?;
        control.checkpoint()?;
        let record = ClipOutputVerification {
            working_input,
            input_transport: "owned_regular_stdin".into(),
            schema_version: "clip-output-verification/1".into(),
            output_sha256: candidate.facts.output_sha256.clone(),
            probe_sha256: self.probe_sha256.clone(),
            decoder_sha256: self.decoder_sha256.clone(),
            raw_probe_sha256: hash(&raw.stdout),
            decode_stdout_sha256: hash(&decoded.stdout),
            decode_stderr_sha256: hash(&decoded.stderr),
            metadata,
            decode_integrity_verified: true,
            color_verified: false,
            quality_approved: false,
        };
        Ok(VerifiedClipCandidate {
            candidate,
            proof: ProofRef::Owned(Box::new(VerificationProof {
                record,
                raw_probe: raw.stdout,
                working: Some(working),
            })),
        })
    }
}
#[cfg(test)]
mod tests;
