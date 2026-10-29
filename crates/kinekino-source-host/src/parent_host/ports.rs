//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait ParentReceiptSink: ReceiptSink {
    fn record_unassigned(&mut self, receipt: &UnassignedCloseReceipt) -> io::Result<()>;
}
