//! Events an engine reports back to the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserEvent {
    NavigationStarted { url: String },
    Loaded { url: String },
    RequestBlocked { url: String },
    NewWindowRequested { url: String },
}
