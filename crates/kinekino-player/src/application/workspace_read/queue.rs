use serde::Serialize;
#[derive(Debug, Clone, Copy, Serialize)]
pub enum QueueState {
    Unknown,
    NeedsInput,
    Queued,
    Running,
    Completed,
    Failed,
}
#[derive(Debug, Clone, Serialize)]
pub struct QueueItem {
    pub item_id: String,
    pub title: String,
    pub state: QueueState,
    pub progress_percent: Option<u8>,
}
