//! Inference registry and the `settings.inference.mode` → registry policy mapping.
//! Adapter crates register their factories; the composition root only consumes this.
use crate::mock::MockBackend;
use crate::port::{InferenceBackend, InferenceCaps, InferenceKind};
use crate::settings::{InferenceMode, InferenceSettings};
use kinekino_core::port::OsId;
use kinekino_core::port::registry::{
    Availability, PlatformRule, Policy, Registration, Registry, Role,
};

pub type DynInference = dyn InferenceBackend;
pub type InferenceRegistry =
    Registry<DynInference, InferenceKind, InferenceCaps, InferenceSettings>;

pub fn inference_registry() -> InferenceRegistry {
    Registry::new("inference")
}

/// Rules for an OS-independent adapter on every supported OS.
pub fn everywhere(role: Role) -> Vec<PlatformRule> {
    [OsId::Linux, OsId::Windows, OsId::MacOs]
        .into_iter()
        .map(|os| PlatformRule::new(os, None, role))
        .collect()
}

/// `MockBackend` registration (explicit only: never chosen by platform order).
pub fn register_mock(reg: &mut InferenceRegistry) {
    reg.register(Registration::new(
        InferenceKind::Mock,
        InferenceCaps::all(),
        everywhere(Role::Explicit),
        || Availability::Available,
        |_cfg: &InferenceSettings| Ok(Box::new(MockBackend::default()) as Box<DynInference>),
    ));
}

/// Mode → policy: `cloud` prefers HF then falls back to local packs, `local_first` prefers
/// local packs, `local_only` allows only local packs.
pub fn policy_for(mode: InferenceMode) -> Policy<InferenceKind> {
    match mode {
        InferenceMode::Cloud => Policy::Prefer(InferenceKind::HfCloud),
        InferenceMode::LocalFirst => Policy::Prefer(InferenceKind::LocalPack),
        InferenceMode::LocalOnly => Policy::Only(InferenceKind::LocalPack),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinekino_core::port::{CapabilityTable, TargetPlatform};

    #[test]
    fn mock_is_explicit_and_policy_maps() {
        let mut reg = inference_registry();
        register_mock(&mut reg);
        let p = TargetPlatform {
            os: OsId::Linux,
            display: None,
        };
        assert!(
            reg.explain(p, InferenceCaps::empty(), Policy::PlatformOrder)
                .selected
                .is_none()
        );
        let d = reg.explain(p, InferenceCaps::all(), Policy::Only(InferenceKind::Mock));
        assert_eq!(d.selected.map(|s| s.kind), Some(InferenceKind::Mock));
        assert_eq!(
            policy_for(InferenceMode::LocalOnly),
            Policy::Only(InferenceKind::LocalPack)
        );
    }
}
