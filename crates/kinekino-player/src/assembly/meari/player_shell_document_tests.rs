use super::*;
use crate::{
    application::api::{ApplicationClientHandle, PlayerSdk},
    assembly::meari::player_shell_startup::prepare_shell,
    presentation::web_ui_adapter::player_shell::{ShellPhase, document::StartLabels},
};
use std::{rc::Rc, sync::Arc};

use crate::presentation::web_ui_adapter::player_shell::document::video::ViewingVideoId;

fn set_fixture_source(prepared: &mut PreparedShell<Rc<()>>, source: String) {
    use meari::domain::build::BuildPath;
    let path: BuildPath = "viewing/v1-fixture.tsx".to_owned().try_into().unwrap();
    let inventory = BTreeMap::from([(path.clone(), source)]);
    prepared.request.document.graph = meari::parse_module_graph(&path, &inventory).unwrap();
    prepared.request.document.inventory = inventory;
    prepared.request.document.props = BTreeMap::new();
}

fn fixture_source(markup: &str) -> String {
    format!("type Props={{}};export function App(props:Props){{return {markup};}}")
}

#[test]
fn actual_viewing_video_is_single_inactive_and_bound_to_the_original_full_stamp() {
    let (prepared, client, owner) = prepared();
    let mounted = mount_prepared_shell(prepared, options())
        .unwrap_or_else(|failure| panic!("{:?}: {}", failure.stage, failure.diagnostic));
    let before = mounted.document.stamp().unwrap();
    let video = mounted.inspect_viewing_video().unwrap();
    assert_eq!(video.id(), ViewingVideoId::Primary);
    assert_eq!(video.origin(), &before);
    assert_eq!(before.owner.source_generation, 1);
    assert_eq!(before.document_revision, 1);
    assert_eq!(before.turn_sequence, 0);
    mounted
        .document
        .try_with(|document| {
            let videos: Vec<_> = document
                .elements()
                .iter()
                .filter(|element| element.dom_tag == "video")
                .collect();
            assert_eq!(videos.len(), 1);
            assert_eq!(videos[0].id, ViewingVideoId::Primary.as_str());
            assert_eq!(
                videos[0].attributes.get("tabindex").map(String::as_str),
                Some("-1")
            );
            assert_eq!(
                videos[0].attributes.get("aria-hidden").map(String::as_str),
                Some("true")
            );
            assert!(
                document
                    .module_projection()
                    .unwrap()
                    .callback_bindings()
                    .is_empty()
            );
        })
        .unwrap();
    assert_eq!(mounted.document.stamp().unwrap(), before);
    assert!(Arc::ptr_eq(&client, &mounted.inputs.client));
    assert!(Rc::ptr_eq(&owner, &mounted.inputs.host_owner));
    assert!(mounted.state.observed_route().is_none());
}

#[test]
fn interactive_read_root_keeps_one_video_and_existing_local_callback_bindings() {
    use crate::presentation::web_ui_adapter::player_shell::document::{
        enable_read_panel, read_props,
    };
    use meari::{
        application::host_callbacks::HostCallbacks,
        domain::{Value, host_callbacks::CallbackSignature},
    };
    use std::cell::RefCell;
    let (mut prepared, _, _) = prepared();
    let props = read_props(&StartLabels {
        connection: "관측 없음".into(),
        time: "관측 없음".into(),
    });
    let references: Vec<_> = props
        .values()
        .filter_map(|value| match value {
            Value::CallbackReference(reference) => Some(reference.binding.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(references.len(), 8);
    assert_eq!(
        references
            .iter()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>(),
        std::collections::BTreeSet::from([
            "onOpenJobs",
            "onReadCapabilities",
            "onReadEngine",
            "onReadStatus",
            "onRetryCapabilitiesAck",
            "onRetryEngineAck",
            "onRetryStatusAck",
            "onRefreshReadProjection",
        ])
    );
    let expected_bindings = references
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    enable_read_panel(&mut prepared.request.document, props)
        .ok()
        .unwrap();
    let callback_owner = Rc::new(RefCell::new(None));
    let retained = callback_owner.clone();
    let mounted = mount_prepared_shell_with_callbacks(
        prepared,
        options(),
        Box::new(move |scope| {
            let mut callbacks = HostCallbacks::new(scope);
            for name in references {
                callbacks
                    .bind_local(name, CallbackSignature::Action)
                    .map_err(|error| error.to_string())?;
            }
            let bindings = callbacks
                .export_bindings()
                .map_err(|error| error.to_string())?;
            *retained.borrow_mut() = Some(callbacks);
            Ok(bindings)
        }),
    )
    .unwrap_or_else(|failure| panic!("{:?}: {}", failure.stage, failure.diagnostic));
    let metadata = mounted.inspect_viewing_video().unwrap();
    assert_eq!(metadata.origin(), &mounted.document.stamp().unwrap());
    mounted
        .document
        .try_with(|document| {
            assert_eq!(
                document
                    .module_projection()
                    .unwrap()
                    .callback_bindings()
                    .len(),
                8
            );
            assert_eq!(
                document
                    .module_projection()
                    .unwrap()
                    .callback_bindings()
                    .into_values()
                    .map(|callback| callback.reference.binding)
                    .collect::<std::collections::BTreeSet<_>>(),
                expected_bindings
            );
            assert_eq!(
                document
                    .elements()
                    .iter()
                    .filter(|element| element.dom_tag == "video")
                    .count(),
                1
            );
        })
        .unwrap();
    assert!(callback_owner.borrow().is_some());
}

#[test]
fn explicit_video_inspection_rejects_missing_wrong_tag_multiple_and_source_declarations() {
    use video::ViewingVideoError;
    let id = ViewingVideoId::Primary.as_str();
    for (markup, expected) in [
        ("<div/>".to_owned(), ViewingVideoError::Missing),
        (format!("<div id=\"{id}\"/>"), ViewingVideoError::WrongTag),
        (
            format!("<div><video id=\"{id}\"/><video/></div>"),
            ViewingVideoError::MultipleVideos,
        ),
        (
            format!("<video id=\"{id}\" src=\"\"/>"),
            ViewingVideoError::ActiveDeclaration,
        ),
    ] {
        let (mut prepared, _, _) = prepared();
        set_fixture_source(&mut prepared, fixture_source(&markup));
        let mounted = mount_prepared_shell(prepared, options())
            .unwrap_or_else(|failure| panic!("{}", failure.diagnostic));
        assert_eq!(mounted.inspect_viewing_video().unwrap_err(), expected);
    }
}

#[test]
fn historical_video_metadata_does_not_follow_another_document_or_reprojection() {
    let (first, _, _) = prepared();
    let first = mount_prepared_shell(first, options())
        .unwrap_or_else(|failure| panic!("{}", failure.diagnostic));
    let historical = first.inspect_viewing_video().unwrap();
    let (second, _, _) = prepared();
    let mut second_options = options();
    second_options.epoch = 2;
    second_options.source_generation = 2;
    let second = mount_prepared_shell(second, second_options)
        .unwrap_or_else(|failure| panic!("{}", failure.diagnostic));
    let other = second.inspect_viewing_video().unwrap();
    assert_eq!(historical.id(), other.id());
    assert_ne!(historical.origin().host_id, other.origin().host_id);
    assert_ne!(historical.origin().owner, other.origin().owner);
    assert_eq!(other.origin().owner.source_generation, 2);
    first
        .document
        .try_reproject(historical.origin(), |_| Ok::<_, std::io::Error>(()))
        .unwrap();
    let current = first.inspect_viewing_video().unwrap();
    assert_eq!(historical.id(), current.id());
    assert_eq!(historical.origin().host_id, current.origin().host_id);
    assert_ne!(
        historical.origin().document_revision,
        current.origin().document_revision
    );
    assert_eq!(current.origin(), &first.document.stamp().unwrap());
}

#[test]
fn optional_video_inspection_and_duplicate_mount_failure_retain_original_owners_and_ticket() {
    let (mut generic, client, owner) = prepared();
    let generation = generic.request.ticket.generation();
    set_fixture_source(&mut generic, fixture_source("<div/>"));
    let mounted = mount_prepared_shell(generic, options())
        .unwrap_or_else(|failure| panic!("{}", failure.diagnostic));
    assert_eq!(
        mounted.inspect_viewing_video().unwrap_err(),
        video::ViewingVideoError::Missing
    );
    assert!(Arc::ptr_eq(&client, &mounted.inputs.client));
    assert!(Rc::ptr_eq(&owner, &mounted.inputs.host_owner));
    assert_eq!(mounted.ticket.generation(), generation);
    assert!(mounted.state.observed_route().is_none());
    let (mut duplicate, client, owner) = prepared();
    let generation = duplicate.request.ticket.generation();
    let id = ViewingVideoId::Primary.as_str();
    set_fixture_source(
        &mut duplicate,
        fixture_source(&format!(
            "<div><video id=\"{id}\"/><video id=\"{id}\"/></div>"
        )),
    );
    let failure = match mount_prepared_shell(duplicate, options()) {
        Ok(_) => panic!("duplicate DOM identity must fail"),
        Err(failure) => failure,
    };
    assert_eq!(failure.stage, DocumentStage::InteractiveDocument);
    assert!(failure.diagnostic.contains("duplicate DOM id"));
    assert!(Arc::ptr_eq(&client, &failure.inputs.client));
    assert!(Rc::ptr_eq(&owner, &failure.inputs.host_owner));
    assert_eq!(failure.ticket.generation(), generation);
    assert!(failure.state.observed_route().is_none());
}

#[test]
fn video_callback_is_rejected_before_mount_without_consuming_original_preparation() {
    use meari::domain::build::BuildPath;
    let (prepared, client, owner) = prepared();
    let generation = prepared.request.ticket.generation();
    let inventory = prepared.request.document.inventory.clone();
    let props = prepared.request.document.props.clone();
    let id = ViewingVideoId::Primary.as_str();
    let path: BuildPath = "viewing/rejected-video.tsx".to_owned().try_into().unwrap();
    let rejected = BTreeMap::from([(
        path.clone(),
        format!(
            "type Props={{onVideoIntent:()=>void;}};export function App(props:Props){{return <video id=\"{id}\" onClick={{()=>props.onVideoIntent()}}/>;}}"
        ),
    )]);
    let diagnostic = match meari::parse_module_graph(&path, &rejected) {
        Ok(_) => panic!("video callback must be rejected during source preparation"),
        Err(diagnostic) => diagnostic,
    };
    assert_eq!(diagnostic.file, path);
    assert_eq!(diagnostic.diagnostic.code, "unsupported");
    assert!(
        diagnostic
            .diagnostic
            .message
            .contains("matching button/input handler")
    );
    assert_eq!(prepared.request.document.inventory, inventory);
    assert_eq!(prepared.request.document.props, props);
    assert_eq!(prepared.request.ticket.generation(), generation);
    assert!(Arc::ptr_eq(&client, &prepared.inputs.client));
    assert!(Rc::ptr_eq(&owner, &prepared.inputs.host_owner));
    assert!(prepared.state.observed_route().is_none());
    let mounted = mount_prepared_shell(prepared, options())
        .unwrap_or_else(|failure| panic!("{}", failure.diagnostic));
    assert_eq!(
        mounted.inspect_viewing_video().unwrap().origin(),
        &mounted.document.stamp().unwrap()
    );
    assert!(Arc::ptr_eq(&client, &mounted.inputs.client));
    assert!(Rc::ptr_eq(&owner, &mounted.inputs.host_owner));
}

fn prepared() -> (PreparedShell<Rc<()>>, ApplicationClientHandle, Rc<()>) {
    let client: ApplicationClientHandle = Arc::new(PlayerSdk::default());
    let owner = Rc::new(());
    let prepared = match prepare_shell(
        ShellInputs {
            client: client.clone(),
            host_owner: owner.clone(),
        },
        StartLabels {
            connection: "관측 없음".into(),
            time: "관측 없음".into(),
        },
    ) {
        Ok(prepared) => prepared,
        Err(_) => panic!("viewing graph should prepare"),
    };
    (prepared, client, owner)
}
fn options() -> DocumentOptions {
    DocumentOptions {
        namespace: "view-start-test".into(),
        epoch: 1,
        source_generation: 1,
        viewport: Viewport {
            width: 1280,
            height: 720,
            dark: true,
        },
    }
}

#[test]
fn actual_document_and_handle_preserve_external_owners_without_route_commit() {
    let (prepared, client, owner) = prepared();
    let mounted = match mount_prepared_shell(prepared, options()) {
        Ok(mounted) => mounted,
        Err(failure) => panic!("{:?}: {}", failure.stage, failure.diagnostic),
    };
    assert!(Arc::ptr_eq(&client, &mounted.inputs.client));
    assert!(Rc::ptr_eq(&owner, &mounted.inputs.host_owner));
    assert!(mounted.state.observed_route().is_none());
    assert_eq!(mounted.state.phase(), ShellPhase::AwaitingHost);
    assert_eq!(mounted.style_hashes.len(), 3);
    assert!(
        mounted
            .style_hashes
            .contains_key("crates/kinekino-player/src/presentation/web_ui/styles/viewing.css")
    );
}

#[test]
fn invalid_configuration_returns_original_owner_and_pending_ticket() {
    let (prepared, client, owner) = prepared();
    let generation = prepared.request.ticket.generation();
    let mut invalid = options();
    invalid.source_generation = 0;
    let failure = match mount_prepared_shell(prepared, invalid) {
        Err(failure) => failure,
        Ok(_) => panic!("invalid generation must fail"),
    };
    assert_eq!(failure.stage, DocumentStage::Configuration);
    assert!(Arc::ptr_eq(&client, &failure.inputs.client));
    assert!(Rc::ptr_eq(&owner, &failure.inputs.host_owner));
    assert_eq!(failure.ticket.generation(), generation);
    assert!(failure.state.observed_route().is_none());
}

#[test]
fn viewing_assets_include_pinned_fonts_and_start_mark() {
    let styles = prepare_viewing_styles().unwrap();
    assert_eq!(styles.source_hashes.len(), 3);
    assert!(styles.css.contains("aw-viewing-start"));
    assert!(!styles.css.contains("../../../../assets"));
    let manifest = styles.assets.manifest();
    assert!(
        manifest
            .files
            .keys()
            .any(|path| path.as_str().ends_with("start-mark.svg"))
    );
    assert_eq!(
        manifest
            .files
            .values()
            .filter(|asset| asset.role == meari::domain::build::InputRole::Font)
            .count(),
        3
    );
    for reference in styles.references {
        assert!(styles.assets.resource(&reference.output).is_some());
    }
}
