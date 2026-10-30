use super::fixtures::{FixtureInspection, StubRasterizer, request};
use crate::{
    application::{
        PngInspectionStage, RenderError, RenderFailure,
        ports::{PngEncoding, PngInspection, RenderPorts},
        render,
    },
    domain::{RenderLimits, Size},
    infrastructure::png_encoding::BoundedPngEncoder,
};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use std::cell::{Cell, RefCell};

struct Inspector {
    cause: RefCell<Option<ApiError>>,
    fail_at: PngInspectionStage,
    calls: Cell<usize>,
}

impl PngInspection for Inspector {
    type Error = ApiError;

    fn inspect_static_png(&self, bytes: &[u8], limits: RenderLimits) -> Result<Size, ApiError> {
        self.calls.set(self.calls.get() + 1);
        let phase = if bytes == b"frame" {
            PngInspectionStage::Frame
        } else {
            PngInspectionStage::Output
        };
        if phase == self.fail_at {
            return Err(self
                .cause
                .borrow_mut()
                .take()
                .expect("cause moved exactly once"));
        }
        Ok(FixtureInspection::default()
            .inspect_static_png(bytes, limits)
            .expect("non-failing synthetic phase must remain valid"))
    }
}

struct Encoder {
    calls: Cell<usize>,
}

impl PngEncoding for Encoder {
    fn encode_rgba(&self, size: Size, rgba: &[u8], limit: usize) -> Result<Vec<u8>, RenderError> {
        self.calls.set(self.calls.get() + 1);
        BoundedPngEncoder.encode_rgba(size, rgba, limit)
    }
}

fn check_owned_cause(code: ErrorCode, phase: PngInspectionStage) {
    let mut original = ApiError::new(code, String::from("original strict PNG failure"));
    original.retryable = true;
    original.details = serde_json::json!({
        "first_cause": {"identity": "original-parser-cause", "offset": 29},
        "parser": "existing-strict-parser-injection-only"
    });
    original.remediation = String::from("retain original source and inspect the recorded cause");
    original.request_id = String::from("original-request-id");
    let message_ptr = original.message.as_ptr();
    let remediation_ptr = original.remediation.as_ptr();
    let request_ptr = original.request_id.as_ptr();
    let detail_ptr = original.details["first_cause"]["identity"]
        .as_str()
        .unwrap()
        .as_ptr();
    let inspector = Inspector {
        cause: RefCell::new(Some(original)),
        fail_at: phase,
        calls: Cell::new(0),
    };
    let encoder = Encoder {
        calls: Cell::new(0),
    };
    let mut rasterizer = StubRasterizer::default();
    let failure = render(
        request(),
        RenderPorts {
            png: &inspector,
            encoder: &encoder,
            rasterizer: &mut rasterizer,
        },
    )
    .err()
    .expect("injected inspector cause must fail the render");
    let RenderFailure::PngInspection { stage, cause } = failure else {
        panic!("original PNG cause was replaced by a renderer error");
    };
    assert_eq!(stage, phase);
    assert_eq!(cause.code, code);
    assert_eq!(cause.message, "original strict PNG failure");
    assert_eq!(cause.message.as_ptr(), message_ptr);
    assert!(cause.retryable);
    assert_eq!(
        cause.details,
        serde_json::json!({
            "first_cause": {"identity": "original-parser-cause", "offset": 29},
            "parser": "existing-strict-parser-injection-only"
        })
    );
    assert_eq!(
        cause.details["first_cause"]["identity"]
            .as_str()
            .unwrap()
            .as_ptr(),
        detail_ptr
    );
    assert_eq!(
        cause.remediation,
        "retain original source and inspect the recorded cause"
    );
    assert_eq!(cause.remediation.as_ptr(), remediation_ptr);
    assert_eq!(cause.request_id, "original-request-id");
    assert_eq!(cause.request_id.as_ptr(), request_ptr);
    assert!(inspector.cause.borrow().is_none());
    match phase {
        PngInspectionStage::Frame => {
            assert_eq!(inspector.calls.get(), 1);
            assert_eq!(rasterizer.calls, 0);
            assert_eq!(encoder.calls.get(), 0);
        }
        PngInspectionStage::Output => {
            assert_eq!(inspector.calls.get(), 2);
            assert_eq!(rasterizer.calls, 1);
            assert_eq!(encoder.calls.get(), 1);
        }
    }
}

#[test]
fn frame_resource_exhausted_retains_original_owned_api_error_and_stops() {
    check_owned_cause(ErrorCode::ResourceExhausted, PngInspectionStage::Frame);
}

#[test]
fn frame_invalid_argument_retains_original_owned_api_error_and_stops() {
    check_owned_cause(ErrorCode::InvalidArgument, PngInspectionStage::Frame);
}

#[test]
fn frame_apng_unsupported_retains_original_owned_api_error_and_stops() {
    check_owned_cause(ErrorCode::Unsupported, PngInspectionStage::Frame);
}

#[test]
fn output_inspection_retains_original_owned_api_error_and_phase() {
    check_owned_cause(ErrorCode::ResourceExhausted, PngInspectionStage::Output);
}
