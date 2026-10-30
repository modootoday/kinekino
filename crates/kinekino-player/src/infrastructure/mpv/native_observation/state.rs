use crate::application::playback_session::{
    FileAttribution, NativePropertySample, NativeReceiveObservation,
};
use serde_json::Value;
use std::time::Instant;

struct Property<T> {
    value: Option<T>,
    sequence: Option<u64>,
    received: Option<Instant>,
    lifecycle: Option<u64>,
}
impl<T> Default for Property<T> {
    fn default() -> Self {
        Self {
            value: None,
            sequence: None,
            received: None,
            lifecycle: None,
        }
    }
}
impl<T: Copy> Property<T> {
    fn receive(&mut self, value: Option<T>, sequence: u64, now: Instant, lifecycle: u64) {
        self.value = value;
        self.sequence = Some(sequence);
        self.received = Some(now);
        self.lifecycle = Some(lifecycle);
    }
    fn report(&self, now: Instant) -> NativePropertySample<T> {
        NativePropertySample {
            value: self.value,
            receive_sequence: self.sequence,
            received_age: self
                .received
                .map(|received| now.saturating_duration_since(received)),
            lifecycle_generation_at_receive: self.lifecycle,
            file_attribution: FileAttribution::Unverified,
        }
    }
}
#[derive(Default)]
pub(crate) struct ReceiveState {
    connected: bool,
    exhausted: bool,
    connection: u64,
    lifecycle: u64,
    sequence: u64,
    position: Property<f64>,
    duration: Property<f64>,
    paused: Property<bool>,
    idle: Property<bool>,
    seekable: Property<bool>,
    partial: Property<bool>,
}
impl ReceiveState {
    pub(crate) fn generations(&self) -> Option<(u64, u64)> {
        (self.connected && !self.exhausted).then_some((self.connection, self.lifecycle))
    }
    fn invalidate(&mut self) {
        self.position.value = None;
        self.duration.value = None;
        self.paused.value = None;
        self.idle.value = None;
        self.seekable.value = None;
        self.partial.value = None;
    }
    fn fail(&mut self) {
        self.exhausted = true;
        self.connected = false;
        self.invalidate();
    }
    pub fn connect(&mut self) -> Result<(), &'static str> {
        let Some(connection) = self.connection.checked_add(1) else {
            self.fail();
            return Err("Native connection generation exhausted");
        };
        let Some(lifecycle) = self.lifecycle.checked_add(1) else {
            self.fail();
            return Err("Native lifecycle generation exhausted");
        };
        if self.exhausted {
            return Err("Native receive counters exhausted");
        }
        self.connection = connection;
        self.lifecycle = lifecycle;
        self.connected = true;
        self.invalidate();
        Ok(())
    }
    pub fn disconnect(&mut self) {
        self.connected = false;
        self.invalidate();
    }
    pub fn receive(&mut self, message: &Value, now: Instant) -> Result<(), &'static str> {
        if !self.connected || self.exhausted {
            return Ok(());
        }
        let Some(sequence) = self.sequence.checked_add(1) else {
            self.fail();
            return Err("Native receive sequence exhausted");
        };
        self.sequence = sequence;
        match message["event"].as_str() {
            Some("start-file" | "end-file") => {
                let Some(lifecycle) = self.lifecycle.checked_add(1) else {
                    self.fail();
                    return Err("Native lifecycle generation exhausted");
                };
                self.lifecycle = lifecycle;
                self.invalidate();
            }
            Some("property-change") => {
                let data = &message["data"];
                let seconds = || {
                    data.as_f64()
                        .filter(|value| value.is_finite() && *value >= 0.0)
                };
                match message["name"].as_str() {
                    Some("time-pos") => {
                        self.position
                            .receive(seconds(), sequence, now, self.lifecycle)
                    }
                    Some("duration") => {
                        self.duration
                            .receive(seconds(), sequence, now, self.lifecycle)
                    }
                    Some("pause") => {
                        self.paused
                            .receive(data.as_bool(), sequence, now, self.lifecycle)
                    }
                    Some("idle-active") => {
                        self.idle
                            .receive(data.as_bool(), sequence, now, self.lifecycle)
                    }
                    Some("seekable") => {
                        self.seekable
                            .receive(data.as_bool(), sequence, now, self.lifecycle)
                    }
                    Some("partially-seekable") => {
                        self.partial
                            .receive(data.as_bool(), sequence, now, self.lifecycle)
                    }
                    _ => (),
                }
            }
            _ => (),
        }
        Ok(())
    }
    pub fn report(&self, now: Instant) -> NativeReceiveObservation {
        NativeReceiveObservation {
            connected: self.connected,
            counter_exhausted: self.exhausted,
            connection_generation: self.connection,
            lifecycle_generation: self.lifecycle,
            connection_receive_sequence: self.sequence,
            position_seconds: self.position.report(now),
            duration_seconds: self.duration.report(now),
            paused: self.paused.report(now),
            idle: self.idle.report(now),
            seekable: self.seekable.report(now),
            partially_seekable: self.partial.report(now),
        }
    }
    #[cfg(test)]
    pub(super) fn exhaust_sequence(&mut self) {
        self.sequence = u64::MAX;
    }
}
