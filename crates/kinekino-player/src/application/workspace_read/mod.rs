#[path = "ports.rs"]
mod ports;
pub use ports::WorkspaceReadPort;
pub mod canvas;
pub mod mask;
mod preview_mask;
pub use mask::*;
pub mod inspector;
pub mod preview;
mod preview_canvas;
mod preview_inspector;
mod preview_tables;
pub mod queue;
pub mod source;
pub mod table;
pub mod timeline;
pub use canvas::*;
pub use inspector::*;
pub use queue::*;
use serde::Serialize;
pub use source::*;
pub use table::*;
pub use timeline::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceId {
    Seg,
    Sub,
    Obj,
    Clip,
    Frame,
    Card,
    A2d,
    Proj,
    Download,
    Model,
    Job,
    Out,
}
impl WorkspaceId {
    pub const ALL: [Self; 12] = [
        Self::Seg,
        Self::Sub,
        Self::Obj,
        Self::Clip,
        Self::Frame,
        Self::Card,
        Self::A2d,
        Self::Proj,
        Self::Download,
        Self::Model,
        Self::Job,
        Self::Out,
    ];
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ReadOrigin {
    Live,
    DesignSample,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ReadState {
    Unsupported,
    Unknown,
    NeedsInput,
    Ready,
}
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceSnapshot {
    pub workspace_id: WorkspaceId,
    pub origin: ReadOrigin,
    pub state: ReadState,
    pub source: Option<SourceSnapshot>,
    pub preview: PreviewSnapshot,
    pub canvas: Option<CanvasSnapshot>,
    pub mask: Option<MaskSnapshot>,
    pub timeline: Vec<TimelineTrack>,
    pub table: TableSnapshot,
    pub inspector: Vec<InspectorField>,
    pub queue: Vec<QueueItem>,
    pub notice: Option<String>,
}
#[derive(Default)]
pub struct LiveWorkspaceProvider;
impl WorkspaceReadPort for LiveWorkspaceProvider {
    fn snapshot(&self, workspace: WorkspaceId) -> WorkspaceSnapshot {
        WorkspaceSnapshot {
            workspace_id: workspace,
            origin: ReadOrigin::Live,
            state: ReadState::Unsupported,
            source: None,
            preview: PreviewSnapshot::default(),
            canvas: None,
            mask: None,
            timeline: Vec::new(),
            table: TableSnapshot::default(),
            inspector: Vec::new(),
            queue: Vec::new(),
            notice: Some("Workspace live read adapter is not implemented".into()),
        }
    }
}
