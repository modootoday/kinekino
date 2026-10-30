mod rename;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        card_bundle::{
            BundleFile, CardBundleAssets, CardBundlePublication, CardBundleStaging, check_control,
        },
        jobs::{ExecutionControl, VerifiedJobOutput},
    },
    domain::{api_values::DecimalU64, frame_capture::MAX_FRAME_PNG_BYTES, identifiers::valid_id},
    infrastructure::{
        card_export::CardBundleFiles, content_digest, immutable_storage_paths,
        storage_directory_identity::StorageDirectoryIdentity,
    },
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{Seek, SeekFrom, Write},
    os::windows::fs::{MetadataExt, OpenOptionsExt},
    path::PathBuf,
};
const MAX_TOTAL_BYTES: u64 = 512 * 1024 * 1024;
pub(super) struct NativeBundlePublication;
struct Record {
    descriptor: BundleFile,
    handle: Option<File>,
    identity: StorageDirectoryIdentity,
}
struct Staging {
    stage: PathBuf,
    target: PathBuf,
    display_target: PathBuf,
    directory: File,
    identity: StorageDirectoryIdentity,
    _parent_guards: Vec<File>,
    records: Vec<Record>,
    total: u64,
    verified: bool,
    published: bool,
}
fn error(code: ErrorCode, message: &str) -> ApiError {
    ApiError::new(code, message)
}
fn io(error: std::io::Error) -> ApiError {
    kinekino_core::file_publication::io_error(error)
}
impl CardBundlePublication for NativeBundlePublication {
    fn begin(&self, output: &str) -> Result<Box<dyn CardBundleStaging>, ApiError> {
        CardBundleFiles.verify_new_output_directory(output)?;
        let display_target = PathBuf::from(output);
        let name = display_target
            .file_name()
            .ok_or_else(|| error(ErrorCode::InvalidArgument, "Output name absent"))?;
        let (parent, guards) = immutable_storage_paths::root(
            display_target
                .parent()
                .ok_or_else(|| error(ErrorCode::InvalidArgument, "Output parent absent"))?,
        )?;
        let target = parent.join(name);
        let id = unsafe { windows::Win32::System::Com::CoCreateGuid() }
            .map_err(|e| error(ErrorCode::EngineFailed, &e.to_string()))?;
        let stage = parent.join(format!(
            ".card-bundle-{:08x}{:04x}{:04x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            id.data1,
            id.data2,
            id.data3,
            id.data4[0],
            id.data4[1],
            id.data4[2],
            id.data4[3],
            id.data4[4],
            id.data4[5],
            id.data4[6],
            id.data4[7]
        ));
        fs::create_dir(&stage).map_err(io)?;
        // GENERIC_READ | DELETE. Original handle denies competing rename/delete.
        let directory = OpenOptions::new()
            .access_mode(0x80010000)
            .share_mode(3)
            .custom_flags(0x02200000)
            .open(&stage)
            .map_err(io)?;
        if !directory.metadata().map_err(io)?.is_dir()
            || directory.metadata().map_err(io)?.file_attributes() & 0x400 != 0
        {
            return Err(error(
                ErrorCode::PermissionDenied,
                "Staging directory is not ordinary",
            ));
        }
        let identity = StorageDirectoryIdentity::observe(&directory)?;
        Ok(Box::new(Staging {
            stage,
            target,
            display_target,
            directory,
            identity,
            _parent_guards: guards,
            records: Vec::new(),
            total: 0,
            verified: false,
            published: false,
        }))
    }
}
impl Staging {
    fn check_identity(&self) -> Result<(), ApiError> {
        if StorageDirectoryIdentity::observe(&self.directory)? != self.identity {
            return Err(error(
                ErrorCode::SourceExpired,
                "Original staging directory identity changed",
            ));
        }
        Ok(())
    }
    fn verify_record(file: &mut File, record: &Record) -> Result<(), ApiError> {
        let metadata = file.metadata().map_err(io)?;
        if !metadata.is_file()
            || metadata.file_attributes() & 0x400 != 0
            || metadata.len() != record.descriptor.bytes.0
            || StorageDirectoryIdentity::observe(file)? != record.identity
        {
            return Err(error(
                ErrorCode::SourceExpired,
                "Bundle file identity or size changed",
            ));
        }
        file.seek(SeekFrom::Start(0)).map_err(io)?;
        let (bytes, sha) = content_digest::sha256(file, record.descriptor.bytes.0)?;
        if bytes != record.descriptor.bytes.0 || sha != record.descriptor.sha256 {
            return Err(error(ErrorCode::SourceExpired, "Bundle file bytes changed"));
        }
        Ok(())
    }
    fn verify_names(&self, path: &std::path::Path) -> Result<(), ApiError> {
        let mut actual = BTreeSet::new();
        for entry in fs::read_dir(path).map_err(io)? {
            actual.insert(
                entry
                    .map_err(io)?
                    .file_name()
                    .to_str()
                    .ok_or_else(|| error(ErrorCode::SourceExpired, "Non-UTF8 bundle filename"))?
                    .to_owned(),
            );
        }
        let expected: BTreeSet<_> = self
            .records
            .iter()
            .map(|r| r.descriptor.name.clone())
            .collect();
        if actual != expected {
            return Err(error(
                ErrorCode::SourceExpired,
                "Bundle directory entries changed",
            ));
        }
        Ok(())
    }
}
impl CardBundleStaging for Staging {
    fn write_new(
        &mut self,
        name: &str,
        bytes: &[u8],
        media_type: &str,
    ) -> Result<BundleFile, ApiError> {
        if self.published
            || !valid_id(name)
            || self.records.iter().any(|r| r.descriptor.name == name)
        {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Bundle filename must be new and portable",
            ));
        }
        if bytes.len() as u64 > MAX_FRAME_PNG_BYTES
            || self
                .total
                .checked_add(bytes.len() as u64)
                .is_none_or(|n| n > MAX_TOTAL_BYTES)
        {
            return Err(error(
                ErrorCode::ResourceExhausted,
                "Bundle file or total byte budget exceeded",
            ));
        }
        self.check_identity()?;
        self.verified = false;
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .share_mode(1)
            .custom_flags(0x00200000)
            .open(self.stage.join(name))
            .map_err(io)?;
        file.write_all(bytes).map_err(io)?;
        file.sync_all().map_err(io)?;
        file.seek(SeekFrom::Start(0)).map_err(io)?;
        let (length, sha) = content_digest::sha256(&mut file, MAX_FRAME_PNG_BYTES)?;
        let descriptor = BundleFile {
            name: name.into(),
            sha256: sha,
            bytes: DecimalU64(length),
            media_type: media_type.into(),
        };
        let identity = StorageDirectoryIdentity::observe(&file)?;
        self.total += length;
        self.records.push(Record {
            descriptor: descriptor.clone(),
            handle: Some(file),
            identity,
        });
        Ok(descriptor)
    }
    fn verify_all(&mut self, control: &dyn ExecutionControl) -> Result<(), ApiError> {
        check_control(control)?;
        self.check_identity()?;
        self.verify_names(&self.stage)?;
        for record in &mut self.records {
            check_control(control)?;
            let mut file = record
                .handle
                .take()
                .ok_or_else(|| error(ErrorCode::Internal, "Original bundle writer absent"))?;
            let result = Self::verify_record(&mut file, record);
            record.handle = Some(file);
            result?;
        }
        check_control(control)?;
        self.verified = true;
        Ok(())
    }
    fn publish_new(
        &mut self,
        manifest: &BundleFile,
        control: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError> {
        if !self.verified
            || self.published
            || manifest.name != "manifest.json"
            || !self.records.iter().any(|r| {
                r.descriptor.name == manifest.name
                    && r.descriptor.sha256 == manifest.sha256
                    && r.descriptor.bytes == manifest.bytes
            })
        {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Verified manifest is required for bundle publication",
            ));
        }
        check_control(control)?;
        self.check_identity()?;
        if fs::symlink_metadata(&self.target).is_ok() {
            return Err(error(
                ErrorCode::RevisionConflict,
                "Output already exists; bundle will not be replaced",
            ));
        }
        let result = VerifiedJobOutput {
            path: self
                .display_target
                .join("manifest.json")
                .to_str()
                .ok_or_else(|| error(ErrorCode::InvalidArgument, "Non-UTF8 output"))?
                .into(),
            sha256: manifest.sha256.clone(),
            bytes: manifest.bytes,
            receipt: json!({"schema_version":"card-bundle-receipt/1","manifest_sha256":manifest.sha256,"file_count":self.records.len(),"directory_identity":self.identity,"state":"review_artifact","quality_approved":false,"template_approved":false,"partial_cleanup_supported":false}),
        };
        result
            .validate()
            .map_err(|s| error(ErrorCode::InvalidArgument, s))?;
        // Close child writers for the directory rename; original directory handle remains alive.
        for record in &mut self.records {
            record.handle.take();
        }
        control.reserve_publication()?;
        rename::rename_new(&self.directory, &self.target)?;
        self.published = true;
        // The original Jobs Verifying state is post-publication, not staging inspection.
        control.begin_verification()?;
        let witness = OpenOptions::new()
            .read(true)
            .share_mode(7)
            .custom_flags(0x02200000)
            .open(&self.target)
            .map_err(io)?;
        if witness.metadata().map_err(io)?.file_attributes() & 0x400 != 0
            || StorageDirectoryIdentity::observe(&witness)? != self.identity
        {
            return Err(error(
                ErrorCode::SourceExpired,
                "Published bundle is not the original directory",
            ));
        }
        self.verify_names(&self.target)?;
        for record in &self.records {
            if std::time::Instant::now() >= control.deadline() {
                return Err(error(
                    ErrorCode::Timeout,
                    "Deadline expired after directory publication; preserve output for reconciliation",
                ));
            }
            let mut file = OpenOptions::new()
                .read(true)
                .share_mode(1)
                .custom_flags(0x00200000)
                .open(self.target.join(&record.descriptor.name))
                .map_err(io)?;
            Self::verify_record(&mut file, record)?;
        }
        Ok(result)
    }
    fn recovery_details(&self) -> Value {
        json!({"staging_directory":self.stage,"output_directory":self.target,"published":self.published,"directory_identity":self.identity,"partial_cleanup_supported":false,"automatic_retry":false})
    }
}
