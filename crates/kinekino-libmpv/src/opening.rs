//! Step-wise open: `begin_open` queues the file and returns at once; `open_ready` drains
//! events without waiting, so a slow source never blocks the caller's thread. The engine
//! stays on one thread because the render context must not be used concurrently.
use super::{LibMpvSurface, ffi};
use kinekino_core::port::error::{ErrorCode, PortError, PortResult};
use std::ffi::{CString, c_void};
use std::time::{Duration, Instant};

/// Time allowed until the source starts playing, then until its first frame arrives.
const START_LIMIT: Duration = Duration::from_secs(10);
const FRAME_LIMIT: Duration = Duration::from_secs(5);

pub(crate) struct Opening {
    /// libmpv playlist entry of the file being opened; events of older files are ignored.
    entry: i64,
    /// The entry redirected (playlist, stream resolver); the next started file is ours.
    follow: bool,
    started: bool,
    restarted: Option<Instant>,
    since: Instant,
}

impl LibMpvSurface {
    /// Queue `source` paused at `start_secs`; an earlier unfinished open is abandoned.
    pub fn begin_open(&mut self, source: &str, start_secs: f64) -> PortResult<()> {
        self.opening = None;
        let opts = format!("start={start_secs},pause=yes");
        self.command(&["loadfile", source, "replace", "-1", &opts])?;
        // `replace` leaves the new file as the last playlist entry.
        let last = self.property_i64("playlist/count").unwrap_or(1) - 1;
        let entry = self
            .property_i64(&format!("playlist/{last}/id"))
            .ok_or_else(|| PortError::new(ErrorCode::Internal, "libmpv gave no playlist entry"))?;
        self.opening = Some(Opening {
            entry,
            follow: false,
            started: false,
            restarted: None,
            since: Instant::now(),
        });
        Ok(())
    }

    /// Ok(true) once the opened file has a decoded frame, Ok(false) while still opening.
    /// Errors end the open. With nothing being opened this is Ok(true).
    pub fn open_ready(&mut self) -> PortResult<bool> {
        let Some(mut opening) = self.opening.take() else {
            return Ok(true);
        };
        let state = self.advance(&mut opening);
        if matches!(state, Ok(false)) {
            self.opening = Some(opening);
        }
        state
    }

    fn advance(&mut self, o: &mut Opening) -> PortResult<bool> {
        loop {
            // SAFETY: returned event is owned by mpv and valid until the next wait call.
            let ev = unsafe { &*(self.api.wait_event)(self.handle, 0.0) };
            match ev.event_id {
                ffi::EVENT_NONE => break,
                ffi::EVENT_START_FILE if o.follow || entry_of_start(ev) == Some(o.entry) => {
                    if let Some(id) = entry_of_start(ev) {
                        o.entry = id;
                    }
                    (o.follow, o.started, self.fresh) = (false, true, false);
                }
                ffi::EVENT_END_FILE => {
                    if let Some(end) = end_file(ev).filter(|e| e.playlist_entry_id == o.entry) {
                        if end.reason != ffi::END_FILE_REASON_REDIRECT {
                            return Err(PortError::new(
                                ErrorCode::InvalidInput,
                                "libmpv could not load the source",
                            ));
                        }
                        o.follow = true;
                    }
                }
                ffi::EVENT_PLAYBACK_RESTART if o.started => {
                    o.restarted.get_or_insert_with(Instant::now);
                }
                _ => {}
            }
        }
        let Some(at) = o.restarted else {
            if o.since.elapsed() > START_LIMIT {
                return Err(PortError::new(ErrorCode::Timeout, "libmpv event timeout"));
            }
            return Ok(false);
        };
        self.poll_update();
        if self.fresh {
            return Ok(true);
        }
        if at.elapsed() > FRAME_LIMIT {
            return Err(PortError::new(
                ErrorCode::Timeout,
                "libmpv produced no frame",
            ));
        }
        Ok(false)
    }

    fn property_i64(&mut self, name: &str) -> Option<i64> {
        let n = CString::new(name).ok()?;
        let mut v = 0i64;
        // SAFETY: FORMAT_INT64 writes one i64 into `v`.
        let rc = unsafe {
            (self.api.get_property)(
                self.handle,
                n.as_ptr(),
                ffi::FORMAT_INT64,
                (&mut v as *mut i64).cast::<c_void>(),
            )
        };
        (rc >= 0).then_some(v)
    }
}

fn entry_of_start(ev: &ffi::Event) -> Option<i64> {
    // SAFETY: START_FILE events carry an mpv_event_start_file valid until the next wait.
    (!ev.data.is_null()).then(|| unsafe { (*ev.data.cast::<ffi::StartFile>()).playlist_entry_id })
}

fn end_file(ev: &ffi::Event) -> Option<&ffi::EndFile> {
    // SAFETY: END_FILE events carry an mpv_event_end_file valid until the next wait.
    (!ev.data.is_null()).then(|| unsafe { &*ev.data.cast::<ffi::EndFile>() })
}
