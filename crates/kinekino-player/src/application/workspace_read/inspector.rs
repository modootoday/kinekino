use super::{CellValue, ReadState};
use serde::Serialize;
#[derive(Debug, Clone, Serialize)]
pub struct InspectorField {
    pub field_id: String,
    pub label: String,
    pub value: CellValue,
    pub state: ReadState,
}
