use super::ReadState;
use crate::domain::api_values::DecimalU64;
use serde::Serialize;
#[derive(Debug, Clone, Serialize, Default)]
pub struct TableSnapshot {
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRow>,
}
#[derive(Debug, Clone, Copy, Serialize)]
pub enum CellKind {
    Text,
    Time,
    Integer,
    Percent,
    Boolean,
}
#[derive(Debug, Clone, Serialize)]
pub enum CellValue {
    Text(String),
    TimeUs(DecimalU64),
    Integer(String),
    Percent(u8),
    Boolean(bool),
    Missing,
}
#[derive(Debug, Clone, Serialize)]
pub struct TableColumn {
    pub column_id: String,
    pub label: String,
    pub kind: CellKind,
}
#[derive(Debug, Clone, Serialize)]
pub struct TableCell {
    pub column_id: String,
    pub value: CellValue,
}
#[derive(Debug, Clone, Serialize)]
pub struct TableRow {
    pub row_id: String,
    pub cells: Vec<TableCell>,
    pub state: ReadState,
}
