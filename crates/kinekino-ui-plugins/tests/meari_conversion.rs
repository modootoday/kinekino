use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_subtitles::application::{
    convert,
    storage::{SubtitleStorage, UnavailableStorage},
};
use kinekino_ui_plugins::infrastructure::meari_conversion::{
    CONVERT_CAPABILITY, ConversionPins, MeariSubtitleConversion,
};
use meari::{
    application::{Language, plugins::*, state_program::StateProgram},
    domain::build::{ContentHash, TargetKind},
};
use serde_json::json;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeSet,
    rc::Rc,
};

struct MemoryStorage(RefCell<Option<String>>);
impl SubtitleStorage for MemoryStorage {
    fn read_utf8(&self, path: &str) -> Result<String, ApiError> {
        assert_eq!(path, "source.srt");
        Ok("1\n00:00:00,000 --> 00:00:01,000\n안녕하세요\n".into())
    }
    fn publish_new(&self, path: &str, text: &str) -> Result<(), ApiError> {
        assert_eq!(path, "new.vtt");
        assert!(self.0.borrow().is_none());
        *self.0.borrow_mut() = Some(text.into());
        Ok(())
    }
}

fn hash(value: &str) -> ContentHash {
    ContentHash::of(value.as_bytes())
}

fn pins() -> ConversionPins {
    // Test identifiers only; production assembly must supply pinned schema/provider hashes.
    ConversionPins {
        contract: hash("test-conversion-contract-v1"),
        provider: hash("test-provider"),
        request_schema: hash("test-request"),
        reply_schema: hash("test-reply"),
        error_schema: hash("test-error"),
    }
}

fn registry(target: TargetKind) -> PluginRegistry {
    let component = meari::parse(
        "type Props={};export function Scope(props:Props){return <main/>;}",
        Language::Tsx,
    )
    .unwrap();
    let program = StateProgram::mount(component, "product-plugin-contract".into(), 1).unwrap();
    PluginRegistry::new(
        hash("test-registry"),
        target,
        PluginScope {
            instance: program.snapshot().unwrap().instance,
            source_generation: 1,
        },
        8,
    )
    .unwrap()
}

fn payload() -> PluginPayload {
    PluginPayload {
        schema: pins().request_schema,
        value: json!({"input_path":"source.srt","output_path":"new.vtt",
            "input_format":"srt","output_format":"vtt"}),
    }
}

#[test]
fn native_and_web_registry_preserve_canonical_unavailable_error_and_lifetime() {
    for target in [TargetKind::Native, TargetKind::Web] {
        let mut registry = registry(target);
        let adapter = MeariSubtitleConversion::new(
            pins(),
            BTreeSet::from([TargetKind::Native, TargetKind::Web]),
            |ticket, request| {
                let mut error =
                    convert::execute(&UnavailableStorage, &serde_json::to_value(request).unwrap())
                        .unwrap_err();
                error.request_id = format!("host-conversion-{}", ticket.request_id());
                Err(error)
            },
        )
        .unwrap();
        let handle = registry
            .register(
                Box::new(adapter),
                BTreeSet::from([CONVERT_CAPABILITY.into()]),
            )
            .unwrap();
        let receipt = registry
            .dispatch(&handle, CONVERT_CAPABILITY, payload())
            .unwrap();
        let PluginDispatch::Immediate(Err(error)) = receipt.outcome else {
            panic!("Expected canonical SDK error");
        };
        assert_eq!(error.code, PluginErrorCode::Adapter);
        let sdk_error: ApiError = error.details.unwrap().decode(&pins().error_schema).unwrap();
        assert_eq!(sdk_error.code, ErrorCode::Unsupported);
        assert_eq!(
            sdk_error.request_id,
            format!("host-conversion-{}", receipt.ticket.request_id())
        );
        assert_eq!(
            registry.cancel(&receipt.ticket).unwrap(),
            CancelReceipt::AlreadyTerminal
        );
        registry.unregister(&handle).unwrap();
        assert_eq!(
            registry
                .dispatch(&handle, CONVERT_CAPABILITY, payload())
                .unwrap_err()
                .code,
            PluginErrorCode::Stale
        );
    }
}

#[test]
fn registry_grant_and_schema_rejections_never_reach_product_sdk() {
    let calls = Rc::new(Cell::new(0));
    for granted in [false, true] {
        let calls = calls.clone();
        let mut registry = registry(TargetKind::Native);
        let adapter = MeariSubtitleConversion::new(
            pins(),
            BTreeSet::from([TargetKind::Native]),
            move |_, _| {
                calls.set(calls.get() + 1);
                panic!("Rejected invocation reached SDK");
            },
        )
        .unwrap();
        let grants = if granted {
            BTreeSet::from([CONVERT_CAPABILITY.into()])
        } else {
            BTreeSet::new()
        };
        let handle = registry.register(Box::new(adapter), grants).unwrap();
        let mut input = payload();
        if granted {
            input.schema = hash("wrong-schema");
        }
        let error = registry
            .dispatch(&handle, CONVERT_CAPABILITY, input)
            .unwrap_err();
        assert_eq!(
            error.code,
            if granted {
                PluginErrorCode::InvalidPayload
            } else {
                PluginErrorCode::PermissionDenied
            }
        );
    }
    assert_eq!(calls.get(), 0);
}

#[test]
fn registry_returns_canonical_success_for_native_and_web_targets() {
    for target in [TargetKind::Native, TargetKind::Web] {
        let storage = Rc::new(MemoryStorage(RefCell::new(None)));
        let host_storage = storage.clone();
        let mut registry = registry(target);
        let adapter = MeariSubtitleConversion::new(
            pins(),
            BTreeSet::from([TargetKind::Native, TargetKind::Web]),
            move |_, request| {
                let value = convert::execute(
                    host_storage.as_ref(),
                    &serde_json::to_value(request).unwrap(),
                )?;
                Ok(serde_json::from_value(value).unwrap())
            },
        )
        .unwrap();
        let handle = registry
            .register(
                Box::new(adapter),
                BTreeSet::from([CONVERT_CAPABILITY.into()]),
            )
            .unwrap();
        let receipt = registry
            .dispatch(&handle, CONVERT_CAPABILITY, payload())
            .unwrap();
        let PluginDispatch::Immediate(Ok(reply)) = receipt.outcome else {
            panic!("Expected canonical conversion result");
        };
        let result: convert::ConversionResult = reply.decode(&pins().reply_schema).unwrap();
        assert_eq!(result.output_path, "new.vtt");
        assert_eq!(result.cue_count, 1);
        assert!(!result.overwrite);
        assert!(storage.0.borrow().as_ref().unwrap().contains("안녕하세요"));
        registry.unregister(&handle).unwrap();
    }
}
