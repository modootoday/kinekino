//! Explicit reusable inspector mount. No OS/window, new worker, route selection or implicit read.
use super::*;
use crate::assembly::meari::player_shell_document::DocumentOptions;
use meari::{
    application::source_modules::ModuleProgram,
    domain::{build::BuildPath, source_modules::SourceModuleGraph},
};
use meari_native::infrastructure::{
    document::InteractiveDocument,
    native_host::{HostLifecycle, HostStamp, NativeDocumentHandle},
};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
type OwnerSlot = Rc<RefCell<Option<JobDetailsOwner>>>;

pub fn sources() -> (BuildPath, BTreeMap<BuildPath, String>) {
    let path = |value: &str| BuildPath::try_from(value.to_owned()).expect("fixed JOB module path");
    let entry = path("studio/jobs/detail-panel.tsx");
    (
        entry.clone(),
        BTreeMap::from([
            (
                entry,
                include_str!("../../../presentation/web_ui/studio/jobs/detail-panel.tsx").into(),
            ),
            (
                path("components/notice.tsx"),
                include_str!("../../../presentation/web_ui/components/notice.tsx").into(),
            ),
        ]),
    )
}
pub fn prepare() -> Result<SourceModuleGraph, String> {
    let (entry, sources) = sources();
    meari::parse_module_graph(&entry, &sources).map_err(|error| format!("{error:?}"))
}

#[must_use = "Retain the original mount failure and query owner through cleanup"]
pub struct JobDetailsMountFailure {
    pub diagnostic: String,
    pub owner: OwnerSlot,
    pub attachment: Arc<ExecutionAttachment>,
}
#[must_use = "Parent must retain this inspector until close_query_owner returns true"]
pub struct MountedJobDetails {
    pub document: NativeDocumentHandle,
    pub owner: OwnerSlot,
}
impl MountedJobDetails {
    /// This mounts one selected inspector only; the caller owns StudioFrame/route/SDK policy.
    pub fn mount(
        attachment: ExecutionAttachment,
        target: GetJob,
        context: RequestContext,
        options: DocumentOptions,
        css: &str,
        assets: Option<meari::application::assets::VerifiedAssets>,
    ) -> Result<Self, JobDetailsMountFailure> {
        let attachment = Arc::new(attachment);
        let owner = Rc::new(RefCell::new(None));
        let result = (|| {
            let program = ModuleProgram::mount(
                prepare()?,
                JobDetailsProjection::new(target.job_id.clone()).props(),
                options.namespace,
                options.epoch,
            )
            .map_err(|error| format!("{error:?}"))?;
            let exporting = owner.clone();
            let attaching = attachment.clone();
            let document = InteractiveDocument::from_module_program_with_callbacks(
                program,
                css,
                options.viewport,
                assets,
                options.source_generation,
                move |scope| {
                    let state = JobDetailsOwner::new(attaching, scope, target, context)
                        .map_err(|error| error.message)?;
                    // Retain before binding export can fail.
                    *exporting.borrow_mut() = Some(state);
                    exporting
                        .borrow()
                        .as_ref()
                        .expect("stored owner")
                        .bindings()
                        .map_err(|error| error.message)
                },
            )?;
            let document = NativeDocumentHandle::new(document, options.source_generation)
                .map_err(|error| format!("{error:?}"))?;
            Ok::<_, String>(document)
        })();
        match result {
            Ok(document) => Ok(Self { document, owner }),
            Err(diagnostic) => Err(JobDetailsMountFailure {
                diagnostic,
                owner,
                attachment,
            }),
        }
    }
    pub fn admit(&self, expected: &HostStamp, callback: &PendingCallback) -> Result<(), String> {
        if &self
            .document
            .stamp()
            .map_err(|error| format!("{error:?}"))?
            != expected
        {
            return Err("Stale JOB document stamp".into());
        }
        self.owner
            .try_borrow_mut()
            .map_err(|_| "JOB owner is borrowed")?
            .as_mut()
            .ok_or("JOB owner is missing")?
            .dispatch(callback, &expected.owner)
            .map_err(|error| error.message)
    }
    /// Respect the parent's drained callback batch barrier before replacing exports.
    pub fn after_dispatch(&self, remaining_callbacks: usize) -> Result<bool, String> {
        if remaining_callbacks != 0 {
            return Ok(false);
        }
        let mut slot = self
            .owner
            .try_borrow_mut()
            .map_err(|_| "JOB owner is borrowed")?;
        let owner = slot.as_mut().ok_or("JOB owner is missing")?;
        let polled = owner.poll();
        let changed = owner.dirty;
        if changed {
            let props = owner.view.props();
            let bindings = owner.bindings().map_err(|error| error.message)?;
            self.document
                .try_reproject(
                    &self
                        .document
                        .stamp()
                        .map_err(|error| format!("{error:?}"))?,
                    |document| document.update_module_props_with_callbacks(props, bindings),
                )
                .map_err(|error| format!("{error:?}"))?;
            owner.dirty = false;
        }
        polled.map_err(|error| error.message)?;
        Ok(changed)
    }
    /// Completion means query attachment detached, not parent worker/window shutdown.
    pub fn close_query_owner(&self) -> Result<bool, PluginError> {
        let mut slot = self
            .owner
            .try_borrow_mut()
            .map_err(|_| error(PluginErrorCode::Busy, "JOB owner is borrowed"))?;
        let owner = slot
            .as_mut()
            .ok_or_else(|| error(PluginErrorCode::Stale, "JOB owner is missing"))?;
        // The parent may already have revoked the document before host cleanup.
        // Paint disabled controls only while Open; never block SDK drain on a closed DOM.
        if !owner.invalidated && self.document.lifecycle() == HostLifecycle::Open {
            owner.view.closing = true;
            owner.view.status = "작업 정보 조회를 정리하고 있습니다.".into();
            owner.dirty = true;
            let props = owner.view.props();
            let bindings = owner.bindings()?;
            self.document
                .try_reproject(
                    &self.document.stamp().map_err(|_| {
                        error(PluginErrorCode::Stale, "JOB document is unavailable")
                    })?,
                    |document| document.update_module_props_with_callbacks(props, bindings),
                )
                .map_err(|failure| error(PluginErrorCode::Adapter, &format!("{failure:?}")))?;
            owner.dirty = false;
        }
        owner.close()
    }
}
