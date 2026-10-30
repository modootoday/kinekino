use crate::{
    application::{
        api::{ApiError, ErrorCode},
        browser_network::{ResponsePurpose, TimedResponseBody},
        browser_provider::{BrowserSnapshot, BrowserWorkState},
    },
    domain::{
        hls::{self, HlsError, VerifiedHlsManifest},
        media::VerifiedManifestSource,
    },
};

pub struct VerifiedObservedManifest {
    pub body: TimedResponseBody,
    pub manifest: VerifiedHlsManifest,
    pub source: VerifiedManifestSource,
}
pub fn observed_response_status(
    snapshot: &BrowserSnapshot,
    expected_document: &str,
    purpose: &ResponsePurpose,
) -> Option<u16> {
    if snapshot.state != BrowserWorkState::Running {
        return None;
    }
    snapshot
        .network
        .as_ref()?
        .bodies
        .iter()
        .find(|body| {
            body.current
                && body.session == body.body.capture.scope().session
                && body.owner_context.document_generation == snapshot.document_generation
                && Some(body.owner_context.native_navigation_id) == snapshot.native_navigation_id
                && body.body.capture.scope().url == expected_document
                && body.body.capture.purpose() == purpose
        })
        .map(|body| body.body.capture.response().status)
}

pub fn observed_manifest(
    snapshot: &BrowserSnapshot,
    expected_document: &str,
) -> Result<Option<VerifiedObservedManifest>, ApiError> {
    if snapshot.state != BrowserWorkState::Running || snapshot.navigation_pending {
        return Ok(None);
    }
    let Some(network) = &snapshot.network else {
        return Ok(None);
    };
    for body in &network.bodies {
        let capture = &body.body.capture;
        if !body.current
            || body.session != capture.scope().session
            || body.owner_context.document_generation != snapshot.document_generation
            || Some(body.owner_context.native_navigation_id) != snapshot.native_navigation_id
            || capture.scope().url != expected_document
            || !(200..300).contains(&capture.response().status)
            || !matches!(
                capture.purpose(),
                ResponsePurpose::ManifestCandidate | ResponsePurpose::MediaProbe
            )
        {
            continue;
        }
        let bytes = body.body.bytes.as_ref();
        let signature = bytes.starts_with(b"#EXTM3U") || bytes.starts_with(b"\xef\xbb\xbf#EXTM3U");
        if !signature && capture.purpose() != &ResponsePurpose::ManifestCandidate {
            continue;
        }
        let manifest = hls::verify(&capture.response().url, bytes).map_err(|reason| {
            let code = if reason == HlsError::Unsupported {
                ErrorCode::Unsupported
            } else {
                ErrorCode::NeedsInput
            };
            let mut error = ApiError::new(code, "Observed manifest failed bounded HLS validation");
            error.details =
                serde_json::json!({"stage":"manifest_validation_failed","manifest_verified":false});
            error
        })?;
        let Some(headers) = capture.request_headers() else {
            continue;
        };
        let source =
            VerifiedManifestSource::new(&manifest, Some(&headers.referer)).ok_or_else(|| {
                ApiError::new(
                    ErrorCode::PermissionDenied,
                    "Observed wire Referer is outside policy",
                )
            })?;
        return Ok(Some(VerifiedObservedManifest {
            body: body.clone(),
            manifest,
            source,
        }));
    }
    Ok(None)
}
