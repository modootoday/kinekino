use super::*;
use crate::infrastructure::os_policy::{ShareMode, apply_share_mode};
use std::io::{Read, Seek, SeekFrom, Write};
pub(super) fn retain_or_copy(
    directory: &Path,
    expected: &EvidenceAsset,
    mut source: File,
    checkpoint: &impl Fn() -> Result<(), ApiError>,
) -> Result<File, ApiError> {
    let path = directory.join(&expected.sha256);
    if path.try_exists().map_err(io_error)? {
        let mut guard = rooted::file(&path)?;
        if content_digest::sha256(&mut guard, expected.bytes.0)?
            != (expected.bytes.0, expected.sha256.clone())
        {
            return Err(conflict(
                "Existing destination object differs; retained orphan requires explicit recovery",
            ));
        }
        return Ok(guard);
    }
    // An incomplete object is an orphan; no package references it until final commit.
    // Keep it on failure as evidence rather than overwrite/delete it implicitly.
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    apply_share_mode(&mut options, ShareMode::ReadOnly);
    let mut output = options.open(&path).map_err(io_error)?;
    source.seek(SeekFrom::Start(0)).map_err(io_error)?;
    let observed = content_digest::sha256_with_checkpoint(
        &mut CopyReader {
            source: &mut source,
            output: &mut output,
        },
        expected.bytes.0,
        checkpoint,
    )
    .map_err(|error| match error {
        content_digest::DigestFailure::Read(error) => io_error(error),
        other => other.into_api_error(),
    })?;
    if observed != (expected.bytes.0, expected.sha256.clone()) {
        return Err(conflict("Copied object differs from retained source"));
    }
    output.sync_all().map_err(io_error)?;
    drop(output);
    let mut guard = rooted::file(&path)?;
    if content_digest::sha256(&mut guard, expected.bytes.0)?
        != (expected.bytes.0, expected.sha256.clone())
    {
        return Err(conflict("Destination object verification differs"));
    }
    Ok(guard)
}
pub(super) fn retain_or_publish(
    directory: &Path,
    asset: &EvidenceAsset,
    bytes: &[u8],
    checkpoint: &impl Fn() -> Result<(), ApiError>,
) -> Result<File, ApiError> {
    let path = directory.join(&asset.sha256);
    if !path.try_exists().map_err(io_error)? {
        crate::infrastructure::new_file::publish_guarded(
            path.to_str()
                .ok_or_else(|| invalid("Unicode destination required"))?,
            bytes,
            checkpoint,
        )?;
    }
    let mut guard = rooted::file(&path)?;
    if content_digest::sha256(&mut guard, asset.bytes.0)? != (asset.bytes.0, asset.sha256.clone()) {
        return Err(conflict("Published object identity differs"));
    }
    Ok(guard)
}

struct CopyReader<'a> {
    source: &'a mut File,
    output: &'a mut File,
}
impl Read for CopyReader<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let count = self.source.read(buffer)?;
        self.output.write_all(&buffer[..count])?;
        Ok(count)
    }
}
