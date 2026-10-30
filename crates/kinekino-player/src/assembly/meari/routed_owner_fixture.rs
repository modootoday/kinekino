//! cfg(test) original-window close bridge; a marker requests intent, never attests closure.
use meari_native::infrastructure::window::NativeWindowWake;
use std::{
    cell::RefCell,
    fs::File,
    io::{self, Read},
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

type Close = Box<dyn Fn() -> Result<bool, &'static str>>;
#[cfg(test)]
mod child;
#[cfg(test)]
mod tests;
struct State {
    path: PathBuf,
    expected: Vec<u8>,
    deadline: Instant,
    requested: Arc<AtomicBool>,
    close: Option<Close>,
    cancel: Option<mpsc::Sender<()>>,
    watcher: Option<JoinHandle<io::Result<()>>>,
    close_result: Option<Result<bool, &'static str>>,
}
thread_local! {
    static ACTIVE: RefCell<Option<Rc<RefCell<State>>>> = const { RefCell::new(None) };
}
pub(crate) struct Scope {
    state: Rc<RefCell<State>>,
    previous: Option<Rc<RefCell<State>>>,
}
pub(crate) struct CloseEvidence {
    pub requested: bool,
    pub delivered: Option<Result<bool, &'static str>>,
    pub watcher: io::Result<()>,
}
impl Scope {
    pub(crate) fn enter(path: PathBuf, nonce: &str, deadline: Instant) -> io::Result<Self> {
        if !path.is_absolute()
            || path.exists()
            || nonce.is_empty()
            || nonce.len() > 128
            || !nonce
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return Err(io::Error::other("Invalid fresh fixture close marker"));
        }
        let state = Rc::new(RefCell::new(State {
            path,
            expected: format!("{nonce}\n").into_bytes(),
            deadline,
            requested: Arc::new(AtomicBool::new(false)),
            close: None,
            cancel: None,
            watcher: None,
            close_result: None,
        }));
        let previous = ACTIVE.with(|active| active.replace(Some(state.clone())));
        Ok(Self { state, previous })
    }
    pub(crate) fn finish(&mut self) -> CloseEvidence {
        let (cancel, watcher, requested, delivered) = {
            let mut state = self.state.borrow_mut();
            (
                state.cancel.take(),
                state.watcher.take(),
                state.requested.load(Ordering::Acquire),
                state.close_result,
            )
        };
        if let Some(cancel) = cancel {
            let _ = cancel.send(());
        }
        let watcher = watcher.map_or_else(
            || Err(io::Error::other("Original window never attached")),
            |watcher| {
                watcher
                    .join()
                    .unwrap_or_else(|_| Err(io::Error::other("Close watcher panicked")))
            },
        );
        CloseEvidence {
            requested,
            delivered,
            watcher,
        }
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        // Cancel the private observer only. Drop never grants window-close permission.
        let (cancel, watcher) = {
            let mut state = self.state.borrow_mut();
            (state.cancel.take(), state.watcher.take())
        };
        if let Some(cancel) = cancel {
            let _ = cancel.send(());
        }
        if let Some(watcher) = watcher {
            let _ = watcher.join();
        }
        ACTIVE.with(|active| *active.borrow_mut() = self.previous.take());
    }
}
pub(crate) fn attached(
    wake: NativeWindowWake,
    close: impl Fn() -> Result<bool, &'static str> + 'static,
) {
    let Some(state) = ACTIVE.with(|active| active.borrow().clone()) else {
        return;
    };
    let mut state = state.borrow_mut();
    if state.close.is_some() {
        state.close_result = Some(Err("Duplicate original attachment"));
        return;
    }
    state.close = Some(Box::new(close));
    let path = state.path.clone();
    let expected = state.expected.clone();
    let deadline = state.deadline;
    let requested = state.requested.clone();
    let (cancel, receiver) = mpsc::channel();
    state.cancel = Some(cancel);
    state.watcher = Some(thread::spawn(move || {
        loop {
            if Instant::now() >= deadline {
                // Deadline requests original cleanup, but the watcher error prevents acceptance.
                requested.store(true, Ordering::Release);
                let _ = wake.wake();
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Original fixture close deadline",
                ));
            }
            match File::open(&path) {
                Ok(file) => {
                    let mut bytes = Vec::new();
                    file.take(1025).read_to_end(&mut bytes)?;
                    if bytes != expected {
                        return Err(io::Error::other("Foreign or partial close marker"));
                    }
                    requested.store(true, Ordering::Release);
                    wake.wake()
                        .map_err(|error| io::Error::other(format!("{error:?}")))?;
                    return Ok(());
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => (),
                Err(error) => return Err(error),
            }
            match receiver.recv_timeout(Duration::from_millis(10)) {
                Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
                Err(mpsc::RecvTimeoutError::Timeout) => (),
            }
        }
    }));
}
pub(crate) fn after_dispatch() {
    let Some(state) = ACTIVE.with(|active| active.borrow().clone()) else {
        return;
    };
    let mut state = state.borrow_mut();
    if state.close_result.is_none() && state.requested.load(Ordering::Acquire) {
        state.close_result = Some(
            state
                .close
                .as_ref()
                .map_or(Err("Original close unavailable"), |close| close()),
        );
    }
}
