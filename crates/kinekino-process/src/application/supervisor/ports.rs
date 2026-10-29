//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait ReceiptSink {
    fn record(&mut self, receipt: &ProcessReceipt) -> io::Result<()>;
}
