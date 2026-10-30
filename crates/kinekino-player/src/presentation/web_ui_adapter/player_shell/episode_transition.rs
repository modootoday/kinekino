//! Presentation values only; SDK owns adjacency, replacement, cancellation and release.
use meari::domain::Value;
use std::collections::BTreeMap;
#[derive(Default)]
pub struct TransitionControlsView {
    pub prepare: bool,
    pub observe: bool,
    pub commit: bool,
    pub cancel: bool,
    pub read_job: bool,
    pub release: bool,
    pub status: String,
    pub error: String,
}
impl TransitionControlsView {
    pub fn props(self) -> BTreeMap<String, Value> {
        let mut props = BTreeMap::from([
            ("transitionStatus".into(), Value::String(self.status)),
            ("transitionError".into(), Value::String(self.error)),
        ]);
        for (name, enabled) in [
            ("transitionPrepareDisabled", self.prepare),
            ("transitionObserveDisabled", self.observe),
            ("transitionCommitDisabled", self.commit),
            ("transitionCancelDisabled", self.cancel),
            ("transitionReadJobDisabled", self.read_job),
            ("transitionReleaseDisabled", self.release),
        ] {
            props.insert(name.into(), Value::Bool(!enabled));
        }
        props
    }
}
