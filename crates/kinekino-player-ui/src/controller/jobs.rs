//! Plugin calls off the UI thread. Each call runs on its own short-lived thread and
//! answers through a channel; the controller polls it from the window's tick. A newer
//! request replaces the pending one, so a late answer to an abandoned request is dropped.
use crate::catalog::Catalog;
use kinekino_core::source::{SourceError, SourceItem, SourcePage, StreamCandidate};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::Duration;

/// What to do with the trail once a listing arrives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Then {
    /// Opened from the shown listing: push it.
    Push,
    /// "More" on the top listing: append and remember the page.
    Page,
    /// Back to the parent: the trail is already popped.
    Stay,
}

/// One opened listing: the item it lists, the last page shown, and the search query
/// when it is a search result (then `item` is the source root it searched).
#[derive(Debug, Clone, PartialEq)]
pub struct Level {
    pub item: SourceItem,
    pub page: u32,
    pub query: Option<String>,
}

pub enum Job {
    List { level: Level, then: Then },
    Resolve { index: usize },
}

pub enum Reply {
    Page(Result<SourcePage, SourceError>),
    Streams(Result<Vec<StreamCandidate>, SourceError>),
}

pub struct Pending {
    pub job: Job,
    rx: Receiver<Reply>,
}

impl Pending {
    /// Starts `job` on a worker thread; `call` is the plugin call it makes.
    pub fn start(
        catalog: &Arc<dyn Catalog>,
        job: Job,
        call: impl FnOnce(&dyn Catalog) -> Reply + Send + 'static,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        let catalog = Arc::clone(catalog);
        std::thread::spawn(move || {
            // The receiver is gone when the request was replaced; nothing to do then.
            let _ = tx.send(call(catalog.as_ref()));
        });
        Self { job, rx }
    }

    /// The answer, if it has come (`Some(Err)` when the worker died without one).
    pub fn take(&self, wait: Option<Duration>) -> Option<Result<Reply, ()>> {
        let got = match wait {
            Some(d) => self.rx.recv_timeout(d).map_err(|e| match e {
                mpsc::RecvTimeoutError::Timeout => TryRecvError::Empty,
                mpsc::RecvTimeoutError::Disconnected => TryRecvError::Disconnected,
            }),
            None => self.rx.try_recv(),
        };
        match got {
            Ok(r) => Some(Ok(r)),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Err(())),
        }
    }
}
