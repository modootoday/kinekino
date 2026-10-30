use super::{RequestIds, results::Identity};
use crate::application::api::{ApiError, ErrorCode};
use serde_json::{Value, json};
use std::time::Instant;
const PROPERTIES: [&str; 6] = [
    "time-pos",
    "duration",
    "pause",
    "idle-active",
    "seekable",
    "partially-seekable",
];
#[derive(Clone, Copy, Debug)]
pub(in crate::infrastructure::mpv) struct Sample {
    pub entry: i64,
    pub position: f64,
    pub duration: f64,
    pub paused: bool,
    pub idle: bool,
}
#[derive(Clone, Copy)]
enum Phase {
    Before(i64),
    Properties([i64; 6]),
    After(i64),
    Closed,
}
/// Scalar-only retained state; playlist JSON is inspected by reference and never cloned.
pub(in crate::infrastructure::mpv) struct Barrier {
    identity: Identity,
    deadline: Instant,
    phase: Phase,
    entry: Option<i64>,
    seconds: [Option<f64>; 2],
    flags: [Option<bool>; 4],
    accepted: [bool; 6],
}
pub(in crate::infrastructure::mpv) enum BarrierUpdate {
    Ignored,
    Send(Vec<Value>),
    Complete(Sample),
    Rejected,
}
impl Barrier {
    pub fn start(
        identity: Identity,
        deadline: Instant,
        ids: &mut RequestIds,
    ) -> Result<(Self, Value), ApiError> {
        let id = ids.next()?;
        Ok((
            Self {
                identity,
                deadline,
                phase: Phase::Before(id),
                entry: None,
                seconds: [None; 2],
                flags: [None; 4],
                accepted: [false; 6],
            },
            request(id, "playlist"),
        ))
    }
    pub fn cancel(&mut self) {
        self.phase = Phase::Closed;
    }
    pub fn expired(&self, now: Instant) -> bool {
        now >= self.deadline
    }
    pub fn receive(
        &mut self,
        message: &Value,
        identity: &Identity,
        now: Instant,
        ids: &mut RequestIds,
    ) -> Result<BarrierUpdate, ApiError> {
        if matches!(self.phase, Phase::Closed) {
            return Ok(BarrierUpdate::Ignored);
        }
        if identity != &self.identity
            || self.expired(now)
            || matches!(
                message["event"].as_str(),
                Some("start-file" | "end-file" | "shutdown")
            )
        {
            self.cancel();
            return Ok(BarrierUpdate::Rejected);
        }
        let Some(id) = message["request_id"].as_i64() else {
            return Ok(BarrierUpdate::Ignored);
        };
        let slot = match self.phase {
            Phase::Before(expected) if expected == id => None,
            Phase::After(expected) if expected == id => None,
            Phase::Properties(expected) => {
                match expected.iter().position(|expected| *expected == id) {
                    Some(index) if !self.accepted[index] => Some(index),
                    _ => return Ok(BarrierUpdate::Ignored),
                }
            }
            _ => return Ok(BarrierUpdate::Ignored),
        };
        if message["error"].as_str() != Some("success") {
            self.cancel();
            return Ok(BarrierUpdate::Rejected);
        }
        match self.phase {
            Phase::Before(_) => {
                let Some(entry) = playing_entry(&message["data"]) else {
                    self.cancel();
                    return Ok(BarrierUpdate::Rejected);
                };
                if self
                    .identity
                    .entry
                    .is_some_and(|expected| entry != expected)
                {
                    self.cancel();
                    return Ok(BarrierUpdate::Rejected);
                }
                self.entry = Some(entry);
                let reserved = match ids.reserve(6) {
                    Ok(ids) => ids,
                    Err(error) => {
                        self.cancel();
                        return Err(error);
                    }
                };
                let expected: [i64; 6] = reserved.try_into().map_err(|_| {
                    ApiError::new(ErrorCode::Internal, "Native barrier request count changed")
                })?;
                self.phase = Phase::Properties(expected);
                Ok(BarrierUpdate::Send(
                    expected
                        .iter()
                        .zip(PROPERTIES)
                        .map(|(id, property)| request(*id, property))
                        .collect(),
                ))
            }
            Phase::Properties(_) => {
                let index = slot.expect("property phase has a correlated slot");
                if index < 2 {
                    self.seconds[index] = message["data"]
                        .as_f64()
                        .filter(|value| value.is_finite() && *value >= 0.0);
                } else {
                    self.flags[index - 2] = message["data"].as_bool();
                }
                self.accepted[index] = true;
                if (index < 2 && self.seconds[index].is_none())
                    || (index >= 2 && self.flags[index - 2].is_none())
                {
                    self.cancel();
                    return Ok(BarrierUpdate::Rejected);
                }
                if self.accepted.iter().all(|accepted| *accepted) {
                    let id = match ids.next() {
                        Ok(id) => id,
                        Err(error) => {
                            self.cancel();
                            return Err(error);
                        }
                    };
                    self.phase = Phase::After(id);
                    Ok(BarrierUpdate::Send(vec![request(id, "playlist")]))
                } else {
                    Ok(BarrierUpdate::Ignored)
                }
            }
            Phase::After(_) => {
                let same =
                    playing_entry(&message["data"]).is_some_and(|entry| Some(entry) == self.entry);
                self.cancel();
                if !same
                    || self.flags[1] != Some(false)
                    || self.flags[2] != Some(true)
                    || self.flags[3] != Some(false)
                {
                    return Ok(BarrierUpdate::Rejected);
                }
                match (
                    self.seconds[0],
                    self.seconds[1],
                    self.flags[0],
                    self.flags[1],
                ) {
                    (Some(position), Some(duration), Some(paused), Some(idle)) => {
                        Ok(BarrierUpdate::Complete(Sample {
                            entry: self.entry.expect("Accepted before/after entry"),
                            position,
                            duration,
                            paused,
                            idle,
                        }))
                    }
                    _ => Ok(BarrierUpdate::Rejected),
                }
            }
            Phase::Closed => Ok(BarrierUpdate::Ignored),
        }
    }
}
fn request(id: i64, property: &str) -> Value {
    json!({"command":["get_property",property],"request_id":id})
}
fn playing_entry(data: &Value) -> Option<i64> {
    let entries = data.as_array()?;
    if entries.is_empty() || entries.len() > 64 {
        return None;
    }
    let mut playing = None;
    for entry in entries {
        // Missing playing is mpv's normal representation for non-current entries.
        match entry.get("playing") {
            Some(Value::Bool(true)) => {
                if playing.is_some() {
                    return None;
                }
                playing = Some(entry["id"].as_i64().filter(|id| *id >= 0)?);
            }
            None | Some(Value::Bool(false)) => (),
            _ => return None,
        }
    }
    playing
}
