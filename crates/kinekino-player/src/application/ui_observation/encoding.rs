//! Bounded serialized observations, never native/SDK authority or cleanup decisions.
use super::*;
use serde::{Serializer, ser::SerializeMap};
use serde_json::Value;
use std::io::{self, Write};
pub struct EncodedObservation(Vec<u8>);
impl EncodedObservation {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn identity(value: &str) -> io::Result<()> {
    if value.is_empty()
        || value.len() > IDENTITY_TEXT_MAX_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(invalid("Observation identity exceeds the text contract"));
    }
    Ok(())
}
struct BoundedBuffer {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for BoundedBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(invalid(
                "Full observation envelope exceeds its byte contract",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
#[derive(Serialize)]
struct Envelope<'a, T: Serialize + ?Sized> {
    kind: &'a str,
    elapsed_ms: u128,
    elapsed_scope: &'a str,
    data: &'a T,
}
struct FinalData<'a> {
    legacy: &'a serde_json::Map<String, Value>,
    native_routed: &'a NativeRoutedFinalV2,
}
impl Serialize for FinalData<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.legacy.len() + 1))?;
        for (key, value) in self.legacy {
            map.serialize_entry(key, value)?;
        }
        map.serialize_entry("native_routed", self.native_routed)?;
        map.end()
    }
}
fn envelope<T: Serialize + ?Sized>(
    kind: &str,
    data: &T,
    elapsed_ms: u128,
    scope: &str,
    limit: usize,
) -> io::Result<EncodedObservation> {
    identity(scope)?;
    let mut buffer = BoundedBuffer {
        bytes: Vec::with_capacity(limit),
        limit,
    };
    serde_json::to_writer(
        &mut buffer,
        &Envelope {
            kind,
            elapsed_ms,
            elapsed_scope: scope,
            data,
        },
    )
    .map_err(io::Error::from)?;
    Ok(EncodedObservation(buffer.bytes))
}
pub fn ready(
    record: &NativeRoutedFrameReadyV1,
    elapsed_ms: u128,
) -> io::Result<EncodedObservation> {
    if record.schema_version != 1 {
        return Err(invalid("Unsupported Ready observation version"));
    }
    identity(&record.document.instance_document)?;
    identity(&record.frame.context_id)?;
    envelope(
        NATIVE_READY_KIND,
        record,
        elapsed_ms,
        "native-viewer",
        READY_MAX_BYTES,
    )
}
pub fn final_outcome(
    legacy_report: &Value,
    record: &NativeRoutedFinalV2,
    elapsed_ms: u128,
) -> io::Result<EncodedObservation> {
    if record.schema_version != 2 {
        return Err(invalid("Unsupported final observation version"));
    }
    if record.provider.secondary_errors.len() > 8
        || record
            .provider
            .secondary_errors
            .iter()
            .any(|error| error.message.len() > 4096 || error.remediation.len() > 256)
    {
        return Err(invalid(
            "Original bounded provider diagnostics exceed their contract",
        ));
    }
    if let Some(id) = &record.transition.remaining_job_id {
        identity(id)?;
    }
    if let Some(error) = &record.first_error {
        identity(&error.phase)?;
    }
    let report = legacy_report
        .as_object()
        .ok_or_else(|| invalid("Original final report is not an object"))?;
    if report.contains_key("native_routed") {
        return Err(invalid(
            "Original final report already contains a typed outcome",
        ));
    }
    envelope(
        VIEWER_OUTCOME_KIND,
        &FinalData {
            legacy: report,
            native_routed: record,
        },
        elapsed_ms,
        "native-viewer",
        FINAL_MAX_BYTES,
    )
}
/// Encoded bytes are constructed and checked before output; an I/O failure is secondary.
pub fn write_line(encoded: &EncodedObservation, output: &mut dyn Write) -> io::Result<()> {
    output.write_all(encoded.as_bytes())?;
    output.write_all(b"\n")?;
    output.flush()
}

#[cfg(test)]
mod tests;
