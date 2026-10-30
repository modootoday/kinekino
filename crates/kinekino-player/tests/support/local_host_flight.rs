use super::cli_fixture::CliRunner;
use kinekino_player::{
    application::api::{ApplicationClient, PlayerSdk, Request, Response},
    infrastructure::local_host::{LocalHost, TransportDiagnostics, TransportOptions},
};
use std::{
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
enum Event {
    Started,
    Finished,
    Closed,
}
struct Sdk {
    gate: Arc<(Mutex<bool>, Condvar)>,
    events: mpsc::Sender<Event>,
    calls: Arc<AtomicUsize>,
}
impl ApplicationClient for Sdk {
    fn execute(&self, request: Request) -> Response {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let _ = self.events.send(Event::Started);
        let (lock, changed) = &*self.gate;
        let released = lock.lock().unwrap_or_else(|e| e.into_inner());
        drop(
            changed
                .wait_while(released, |released| !*released)
                .unwrap_or_else(|e| e.into_inner()),
        );
        let response = PlayerSdk::default().execute(request);
        let _ = self.events.send(Event::Finished);
        response
    }
}
impl Drop for Sdk {
    fn drop(&mut self) {
        let _ = self.events.send(Event::Closed);
    }
}
pub struct Flight {
    pub host: Option<LocalHost>,
    pub runner: Option<CliRunner>,
    pub calls: Arc<AtomicUsize>,
    pub trace: TransportDiagnostics,
    gate: Arc<(Mutex<bool>, Condvar)>,
    events: mpsc::Receiver<Event>,
    finished: bool,
    closed: bool,
}
impl Flight {
    pub fn start() -> Self {
        let mut fixture = Self::idle(TransportOptions::default());
        fixture.runner = Some(CliRunner::start(
            fixture.host.as_ref().unwrap().endpoint(),
            &Request::new("version"),
        ));
        fixture
    }
    pub fn idle(options: TransportOptions) -> Self {
        let gate = Arc::new((Mutex::new(false), Condvar::new()));
        let (events, receiver) = mpsc::channel();
        let calls = Arc::new(AtomicUsize::new(0));
        let trace = TransportDiagnostics::default();
        let host = LocalHost::start_observed(
            Arc::new(Sdk {
                gate: gate.clone(),
                events,
                calls: calls.clone(),
            }),
            options,
            trace.clone(),
        )
        .unwrap();
        Self {
            host: Some(host),
            runner: None,
            calls,
            trace,
            gate,
            events: receiver,
            finished: false,
            closed: false,
        }
    }
    pub fn await_started(&mut self) {
        self.await_started_for(Duration::from_secs(2));
    }
    pub fn await_started_for(&mut self, timeout: Duration) {
        let start = Instant::now();
        loop {
            if let Ok(Event::Started) = self.events.try_recv() {
                return;
            }
            if let Some(receipt) = self.runner.as_mut().and_then(CliRunner::poll) {
                panic!(
                    "CLI exited before SDK start: {receipt:?}; trace={:?}",
                    self.trace.snapshot()
                );
            }
            assert!(
                start.elapsed() < timeout,
                "SDK-start fixture watchdog (not transport timeout); calls={}; CLI={:?}; trace={:?}",
                self.calls.load(Ordering::SeqCst),
                self.runner.as_ref().map(CliRunner::progress),
                self.trace.snapshot()
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    pub fn release_and_finish(&mut self) {
        let (lock, changed) = &*self.gate;
        *lock.lock().unwrap_or_else(|e| e.into_inner()) = true;
        changed.notify_all();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !self.closed {
            match self
                .events
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            {
                Ok(Event::Finished) => self.finished = true,
                Ok(Event::Closed) => self.closed = true,
                Ok(Event::Started) => {}
                Err(_) => break,
            }
        }
        assert!(
            self.finished && self.closed,
            "SDK completion/owner-close barrier missing; calls={}; trace={:?}",
            self.calls.load(Ordering::SeqCst),
            self.trace.snapshot()
        );
    }
}
impl Drop for Flight {
    fn drop(&mut self) {
        drop(self.host.take());
        drop(self.runner.take());
        let (lock, changed) = &*self.gate;
        *lock.lock().unwrap_or_else(|e| e.into_inner()) = true;
        changed.notify_all();
        if !self.closed {
            while let Ok(event) = self.events.recv_timeout(Duration::from_secs(5)) {
                if matches!(event, Event::Closed) {
                    self.closed = true;
                    break;
                }
            }
            if !self.closed {
                eprintln!(
                    "fixture cleanup did not observe SDK owner-close; trace={:?}",
                    self.trace.snapshot()
                );
            }
        }
    }
}
