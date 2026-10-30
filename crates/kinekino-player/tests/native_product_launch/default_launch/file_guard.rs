//! Explicit ignored filesystem probes; all mutations stay inside one create-new case directory.
use super::{
    Failure, GateSpec, ImageIdentity, Mode, Pin, Stage, capture_phase, check_binding,
    content_digest, file_information, identity_from, open_ordinary_file, ordinary_executable,
    verify_pin,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::windows::fs::{OpenOptionsExt, symlink_dir},
    path::{Path, PathBuf},
    time::Duration,
};

const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureSpec {
    run_id: String,
    parent: PathBuf,
    image: PathBuf,
    image_sha256: String,
}
impl FixtureSpec {
    fn load() -> Result<Self, Failure> {
        let path = std::env::var_os("NATIVE_DEFAULT_FILE_GUARD_CONFIG")
            .map(PathBuf::from)
            .ok_or_else(|| Failure::new(Stage::Admission, "Explicit fixture config required"))?;
        let expected = std::env::var("NATIVE_DEFAULT_FILE_GUARD_CONFIG_SHA256")
            .map_err(|error| Failure::new(Stage::Admission, error))?;
        require(
            ordinary_executable(&path),
            "Ordinary absolute config path required",
        )?;
        let mut bytes = Vec::new();
        File::open(path)
            .map_err(admission)?
            .take(16385)
            .read_to_end(&mut bytes)
            .map_err(admission)?;
        require(bytes.len() <= 16384, "Fixture config exceeds 16KiB")?;
        let (_, sha256) =
            content_digest::sha256_with_checkpoint(&mut bytes.as_slice(), 16384, || Ok(()))
                .map_err(|error| Failure::debug(Stage::Admission, error))?;
        verify_pin(&expected, &sha256)?;
        let spec: Self = serde_json::from_slice(&bytes)
            .map_err(|error| Failure::new(Stage::Admission, error))?;
        require(
            !spec.run_id.is_empty()
                && spec.run_id.len() <= 64
                && spec
                    .run_id
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                && ordinary_executable(&spec.parent)
                && ordinary_executable(&spec.image)
                && spec
                    .image
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("exe")),
            "Explicit run ID, ordinary parent and pinned image.exe required",
        )?;
        Ok(spec)
    }
}
fn admission(error: io::Error) -> Failure {
    Failure::new(Stage::Admission, error)
}
fn require(condition: bool, message: &'static str) -> Result<(), Failure> {
    if condition {
        Ok(())
    } else {
        Err(Failure::new(Stage::Report, message))
    }
}
fn directory(path: &Path) -> Result<File, Failure> {
    let file = OpenOptions::new()
        .read(true)
        .share_mode(3)
        .custom_flags(0x02200000)
        .open(path)
        .map_err(admission)?;
    let info = file_information(&file)?;
    require(
        info.dwFileAttributes & 0x400 == 0 && info.dwFileAttributes & 0x10 != 0,
        "Ordinary non-reparse fixture ancestor required",
    )?;
    Ok(file)
}
fn write_new(path: &Path) -> Result<(), Failure> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(admission)?;
    file.write_all(b"abc").map_err(admission)?;
    file.sync_all().map_err(admission)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct Snapshot {
    volume: u32,
    file_id: u64,
    final_path: PathBuf,
    bytes: u64,
    sha256: String,
    mtime_filetime: u64,
}
impl Snapshot {
    fn read(path: &Path) -> Result<Self, Failure> {
        let mut file = open_ordinary_file(path)?;
        let identity = identity_from(&file)?;
        let info = file_information(&file)?;
        let (bytes, sha256) = content_digest::sha256_with_checkpoint(&mut file, 3, || Ok(()))
            .map_err(|error| Failure::debug(Stage::Pin, error))?;
        require(
            bytes == 3 && sha256 == ABC_SHA256 && identity.file_id != 0,
            "Create-new abc fixture bytes/hash and supported FileID required",
        )?;
        Ok(Self {
            volume: identity.volume,
            file_id: identity.file_id,
            final_path: identity.final_path,
            bytes,
            sha256,
            mtime_filetime: (u64::from(info.ftLastWriteTime.dwHighDateTime) << 32)
                | u64::from(info.ftLastWriteTime.dwLowDateTime),
        })
    }
    fn identity(&self) -> ImageIdentity {
        ImageIdentity {
            volume: self.volume,
            file_id: self.file_id,
            final_path: self.final_path.clone(),
        }
    }
    fn same_file(&self, other: &Self) -> bool {
        self.volume == other.volume && self.file_id == other.file_id
    }
}
struct Fixture {
    root: PathBuf,
    live: PathBuf,
    a: PathBuf,
    b: PathBuf,
    original: Option<Snapshot>,
    pin: Option<Pin>,
    image_pin: Option<Pin>,
    journal: File,
    root_identity: ImageIdentity,
    _directories: Vec<File>,
    denials: Vec<Failure>,
    link_created: bool,
    link_identity: Option<ImageIdentity>,
}
impl Fixture {
    fn create(spec: &FixtureSpec, case: &'static str) -> Result<Self, Failure> {
        let root = spec.parent.join(format!("{}-{case}", spec.run_id));
        require(
            !spec.image.starts_with(&root),
            "Image must be outside the new mutable case root",
        )?;
        let mut ancestors = spec.parent.ancestors().collect::<Vec<_>>();
        ancestors.reverse();
        let mut directories = ancestors
            .into_iter()
            .map(directory)
            .collect::<Result<Vec<_>, _>>()?;
        // No existing directory or file is adopted as a mutable fixture.
        fs::create_dir(&root).map_err(admission)?;
        let root_guard = directory(&root)?;
        let root_identity = identity_from(&root_guard)?;
        require(
            root_identity.file_id != 0,
            "Supported case-root FileID required",
        )?;
        directories.push(root_guard);
        fs::create_dir(root.join("records")).map_err(admission)?;
        let journal = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join("records/events.jsonl"))
            .map_err(admission)?;
        let live = root.join("workspace/live");
        let a = live.join("probe.bin");
        let b = root.join("workspace/other/probe.bin");
        let mut fixture = Self {
            root,
            live,
            a,
            b,
            original: None,
            pin: None,
            image_pin: None,
            journal,
            root_identity,
            _directories: directories,
            denials: Vec::new(),
            link_created: false,
            link_identity: None,
        };
        fixture.record(json!({"event":"manifest","run_id":spec.run_id,"case":case,
            "root":fixture.root,"root_identity":identity_value(&fixture.root_identity),
            "declared_mutable_relative_paths":["workspace","workspace/live","workspace/other",
                "workspace/live/probe.bin","workspace/other/probe.bin","workspace/live/probe-moved.bin",
                "workspace/live-moved","workspace/linked","records","records/events.jsonl"],
            "fixture_sha256":ABC_SHA256,"fixture_bytes":3,"programs_spawned":0}))?;
        Ok(fixture)
    }
    fn spec(&self, path: PathBuf, sha256: String) -> GateSpec {
        // Pin-only input: no Deadline or process is constructed by the filesystem fixtures.
        GateSpec {
            mode: Mode::NoArgs,
            binary: path,
            sha256,
            cwd: self.root.clone(),
            evidence: self.root.clone(),
            project: None,
            total: Duration::ZERO,
            cleanup: Duration::ZERO,
        }
    }
    fn setup(&mut self, spec: &FixtureSpec) -> Result<(), Failure> {
        self.record(json!({"event":"setup-started"}))?;
        for path in [
            self.root.join("workspace"),
            self.live.clone(),
            self.root.join("workspace/other"),
        ] {
            fs::create_dir(path).map_err(admission)?;
        }
        write_new(&self.a)?;
        write_new(&self.b)?;
        let a = Snapshot::read(&self.a)?;
        let b = Snapshot::read(&self.b)?;
        let image_pin = Pin::open(&self.spec(spec.image.clone(), spec.image_sha256.clone()))?;
        require(
            a.volume == b.volume
                && a.volume == image_pin.volume
                && a.volume == self.root_identity.volume
                && !a.same_file(&b),
            "A/B need distinct FileIDs on the pinned image and case-root volume",
        )?;
        self.image_pin = Some(image_pin);
        self.pin = Some(Pin::open(&self.spec(self.a.clone(), ABC_SHA256.into()))?);
        self.record(json!({"event":"fixture-admitted","a":a,"b":b,
            "pinned_image":self.image_pin.as_ref().map(Pin::evidence),
            "volume_scope_only":true,"fixture_files_are_not_executed":true}))?;
        self.original = Some(a);
        Ok(())
    }
    fn record(&mut self, event: Value) -> Result<(), Failure> {
        serde_json::to_writer(&mut self.journal, &event)
            .map_err(|error| Failure::new(Stage::Raw, error))?;
        self.journal
            .write_all(b"\n")
            .map_err(|error| Failure::new(Stage::Raw, error))?;
        self.journal
            .sync_all()
            .map_err(|error| Failure::new(Stage::Raw, error))
    }
    fn original(&self) -> Result<&Snapshot, Failure> {
        self.original
            .as_ref()
            .ok_or_else(|| Failure::new(Stage::Admission, "Fixture not admitted"))
    }
    fn pin(&self) -> Result<&Pin, Failure> {
        self.pin
            .as_ref()
            .ok_or_else(|| Failure::new(Stage::Pin, "Original fixture Pin is absent"))
    }
    fn check_original(&mut self, phase: &'static str) -> Result<(), Failure> {
        self.pin()?.verify_bound()?;
        let snapshot = Snapshot::read(&self.a)?;
        require(
            &snapshot == self.original()?,
            "Guarded A binding/bytes/mtime changed",
        )?;
        self.record(json!({"event":"guarded-a-unchanged","phase":phase,"a":snapshot}))
    }
    fn denied<T>(&mut self, operation: &'static str, result: io::Result<T>) -> Result<(), Failure> {
        match result {
            Ok(_) => Err(Failure::new(
                Stage::Report,
                format!("Unexpected success while Pin held: {operation}"),
            )),
            Err(original) => {
                let error = Failure::new(Stage::Pin, original);
                let evidence = error.evidence();
                self.denials.push(error);
                self.record(
                    json!({"event":"operation-denied","operation":operation,"error":evidence}),
                )
            }
        }
    }
    fn expected_pin_error(
        &mut self,
        operation: &'static str,
        result: Result<(), Failure>,
        expected_message: &'static str,
    ) -> Result<(), Failure> {
        match result {
            Ok(()) => Err(Failure::new(
                Stage::Report,
                format!("Missing Pin rejection: {operation}"),
            )),
            Err(original) => {
                if original.stage != Stage::Pin || original.message != expected_message {
                    return Err(original);
                }
                let evidence = original.evidence();
                self.denials.push(original);
                self.record(json!({"event":"pin-rejection","operation":operation,"error":evidence}))
            }
        }
    }
    fn rename_a(&mut self, target: PathBuf) -> Result<(), Failure> {
        let from = self.a.clone();
        self.record(json!({"event":"rename-a-start","from":from,"to":target}))?;
        fs::rename(&from, &target).map_err(admission)?;
        self.a = target;
        let snapshot = Snapshot::read(&self.a)?;
        require(
            snapshot.same_file(self.original()?),
            "Released rename changed A FileID",
        )?;
        self.record(json!({"event":"rename-a-complete","a":snapshot}))
    }
    fn rename_live(&mut self, target: PathBuf) -> Result<(), Failure> {
        self.record(json!({"event":"rename-parent-start","from":self.live,"to":target}))?;
        fs::rename(&self.live, &target).map_err(admission)?;
        self.live = target;
        self.a = self.live.join("probe.bin");
        let snapshot = Snapshot::read(&self.a)?;
        require(
            snapshot.same_file(self.original()?),
            "Released parent rename changed A FileID",
        )?;
        self.record(json!({"event":"rename-parent-complete","a":snapshot}))
    }
    fn finish(mut self, outcome: Result<(), Failure>) -> Result<(), FixtureFailure> {
        // capture_phase leaves these original guards in Fixture even after a body panic.
        drop(self.pin.take());
        drop(self.image_pin.take());
        let mut first = outcome.err();
        let mut secondary = Vec::new();
        let value = json!({"event":"case-result","body_succeeded":first.is_none(),
            "fixture_admitted":self.original.is_some(),
            "first_error":first.as_ref().map(Failure::evidence),"case_root":self.root,
            "current_a":self.a,"current_live":self.live,"retained_denial_objects":self.denials.len(),
            "link_created_and_remaining":self.link_created,
            "link_identity":self.link_identity.as_ref().map(identity_value),
            "probe_pin_released":self.pin.is_none(),"image_pin_released":self.image_pin.is_none(),
            "small_workspace_and_raw_preserved":true,"recursive_cleanup":false,"programs_spawned":0});
        if let Err(error) = self.record(value) {
            if first.is_none() {
                first = Some(error);
            } else {
                secondary.push(error);
            }
        }
        match first {
            None => Ok(()),
            Some(first) => Err(FixtureFailure {
                first,
                secondary,
                root: Some(self.root.clone()),
            }),
        }
    }
}
fn identity_value(identity: &ImageIdentity) -> Value {
    json!({"volume":identity.volume,"file_id":identity.file_id,"final_path":identity.final_path})
}
struct FixtureFailure {
    first: Failure,
    secondary: Vec<Failure>,
    root: Option<PathBuf>,
}
impl std::fmt::Debug for FixtureFailure {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.debug_struct("FixtureFailure")
            .field("case_root", &self.root)
            .field("first", &self.first.evidence())
            .field(
                "secondary",
                &self
                    .secondary
                    .iter()
                    .map(Failure::evidence)
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}
fn run_fixture(
    case: &'static str,
    body: impl FnOnce(&mut Fixture) -> Result<(), Failure>,
) -> Result<(), FixtureFailure> {
    let spec = FixtureSpec::load().map_err(|first| FixtureFailure {
        first,
        secondary: Vec::new(),
        root: None,
    })?;
    let mut fixture = Fixture::create(&spec, case).map_err(|first| FixtureFailure {
        first,
        secondary: Vec::new(),
        root: Some(spec.parent.join(format!("{}-{case}", spec.run_id))),
    })?;
    let outcome = capture_phase(|| {
        fixture.setup(&spec)?;
        body(&mut fixture)
    });
    fixture.finish(outcome)
}

#[test]
#[ignore = "Requires explicit filesystem lease and pinned create-new fixture config"]
fn ordinary_fixture_identity_matches_handle_and_final_path() -> Result<(), FixtureFailure> {
    kinekino_platform::install();
    run_fixture("ordinary", |fixture| {
        fixture.check_original("ordinary-admission")?;
        let expected = fixture.pin()?.identity();
        let original_handle = identity_from(&fixture.pin()?._guard)?;
        let reopened = Snapshot::read(&fixture.pin()?.path)?;
        check_binding(&expected, &original_handle)?;
        check_binding(&expected, &reopened.identity())?;
        let b = Snapshot::read(&fixture.b)?;
        let result = check_binding(&expected, &b.identity());
        fixture.expected_pin_error(
            "same-sha-distinct-file-id",
            result,
            "Original file identity/final path binding changed",
        )?;
        fixture.record(json!({"event":"ordinary-binding-complete","original_handle":identity_value(&original_handle),
            "reopened":reopened,"same_sha_distinct_b":b,"process_section_file_id_observed":false}))
    })
}

#[test]
#[ignore = "Requires explicit filesystem lease and pinned create-new fixture config"]
fn file_and_parent_replacement_stays_denied_until_pin_release() -> Result<(), FixtureFailure> {
    kinekino_platform::install();
    run_fixture("lifetime", |fixture| {
        let moved_file = fixture.live.join("probe-moved.bin");
        let rename = fs::rename(&fixture.a, &moved_file);
        if rename.is_ok() {
            fixture.a = moved_file.clone();
        }
        fixture.denied("rename-file", rename)?;
        fixture.check_original("after-file-rename-denial")?;
        let write = OpenOptions::new().write(true).open(&fixture.a);
        fixture.denied("open-file-for-write", write)?;
        fixture.check_original("after-write-open-denial")?;
        let moved_parent = fixture.root.join("workspace/live-moved");
        let rename = fs::rename(&fixture.live, &moved_parent);
        if rename.is_ok() {
            fixture.live = moved_parent.clone();
            fixture.a = fixture.live.join("probe.bin");
        }
        fixture.denied("rename-parent", rename)?;
        fixture.check_original("after-parent-rename-denial")?;
        drop(fixture.pin.take());
        fixture.record(
            json!({"event":"original-probe-pin-explicitly-released","probe_handles_remaining":0}),
        )?;
        fixture.rename_a(moved_file)?;
        fixture.rename_a(fixture.live.join("probe.bin"))?;
        fixture.rename_live(moved_parent)?;
        fixture.rename_live(fixture.root.join("workspace/live"))?;
        let mut file = OpenOptions::new()
            .write(true)
            .open(&fixture.a)
            .map_err(admission)?;
        file.write_all(b"abc").map_err(admission)?;
        file.sync_all().map_err(admission)?;
        drop(file);
        let restored = Snapshot::read(&fixture.a)?;
        require(
            restored.same_file(fixture.original()?),
            "Released write changed A FileID",
        )?;
        fixture.record(
            json!({"event":"released-recovery-complete","original":fixture.original()?,
            "restored":restored,"mtime_may_change_from_owned_write":true}),
        )
    })
}

#[test]
#[ignore = "Requires explicit filesystem lease; symlink capability errors are failures, never skips"]
fn ancestor_reparse_fixture_is_rejected_before_spawn() -> Result<(), FixtureFailure> {
    kinekino_platform::install();
    run_fixture("reparse", |fixture| {
        fixture.check_original("ordinary-target-before-link")?;
        let link = fixture.root.join("workspace/linked");
        fixture.record(json!({"event":"create-owned-directory-symlink-start","link":link,"target":fixture.live}))?;
        symlink_dir(&fixture.live, &link).map_err(admission)?;
        fixture.link_created = true;
        let info = link_identity(&link)?;
        fixture.link_identity = Some(info.clone());
        let via_link = Snapshot::read(&link.join("probe.bin"))?;
        require(
            via_link.same_file(fixture.original()?),
            "Symlink lookup did not reach original A",
        )?;
        fixture.record(json!({"event":"actual-reparse-observed","link":identity_value(&info),"target_lookup":via_link}))?;
        let result = Pin::open(&fixture.spec(link.join("probe.bin"), ABC_SHA256.into())).map(drop);
        fixture.expected_pin_error(
            "ancestor-reparse-before-any-spawn",
            result,
            "Reparse executable ancestor rejected",
        )?;
        check_binding(&info, &link_identity(&link)?)?;
        // Exact owned reparse entry only; never follow/remove its target or recurse.
        fs::remove_dir(&link).map_err(admission)?;
        fixture.link_created = false;
        fixture.check_original("ordinary-target-after-link-removal")?;
        fixture.record(
            json!({"event":"owned-link-removed-target-preserved","removed_link":link,
            "link_identity":identity_value(&info),"programs_spawned":0}),
        )
    })
}
fn link_identity(path: &Path) -> Result<ImageIdentity, Failure> {
    let file = OpenOptions::new()
        .read(true)
        .share_mode(3)
        .custom_flags(0x02200000)
        .open(path)
        .map_err(admission)?;
    let info = file_information(&file)?;
    require(
        info.dwFileAttributes & 0x400 != 0 && info.dwFileAttributes & 0x10 != 0,
        "Owned directory link must actually be a directory reparse entry",
    )?;
    identity_from(&file)
}
