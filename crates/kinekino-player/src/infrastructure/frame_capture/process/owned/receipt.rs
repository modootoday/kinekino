use kinekino_process::{
    application::supervisor::ReceiptSink,
    domain::observation::{FamilyObservation, ProcessReceipt},
};
use std::{
    fs::OpenOptions,
    io::{self, Write},
    path::PathBuf,
};

pub(super) struct FileSink {
    pub root: PathBuf,
    next: u64,
}
impl FileSink {
    pub fn new(root: PathBuf) -> Self {
        Self { root, next: 0 }
    }
}
impl ReceiptSink for FileSink {
    fn record(&mut self, receipt: &ProcessReceipt) -> io::Result<()> {
        let index = self.next;
        self.next = self
            .next
            .checked_add(1)
            .ok_or_else(|| io::Error::other("Receipt sequence exhausted"))?;
        let family = match receipt.family {
            FamilyObservation::Empty => serde_json::json!("Empty"),
            FamilyObservation::Active(count) => serde_json::json!({"Active":count}),
            FamilyObservation::Unknown => serde_json::Value::Null,
        };
        let record = serde_json::json!({"schema":"frame-owned-process/1","identity":{"pid":receipt.identity.pid,"creation_filetime":receipt.identity.creation_filetime,"executable":receipt.identity.executable},"child_exit":receipt.child_exit,"family":family,"forced":receipt.forced,"stdin_eof_requested":receipt.stdin_eof_requested,"output_transport":"bounded_regular_files","pipe_eof_observed":false});
        let bytes = serde_json::to_vec(&record).map_err(io::Error::other)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.root.join(format!("receipt-{index:04}.json")))?;
        file.write_all(&bytes)?;
        file.sync_all()
    }
}
