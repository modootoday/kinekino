//! CLOCK_MONOTONIC domain anchored on a live original parent.
//!
//! CLOCK_MONOTONIC is host-wide, so parent and worker observe one domain. The
//! creator stays the same live process exactly while `getppid()` still
//! returns its pid (an orphan is re-parented) and its start stamp is unchanged.
use kinekino_core::platform::clock::{CounterIdentity, CounterSample, LiveCreator, ParentCounter};
use std::io;

const KIND: &str = "posix-monotonic/1";
const NANOS: u64 = 1_000_000_000;

/// POSIX monotonic counter adapter.
pub struct UnixCounter;
/// Process-wide adapter instance.
pub static COUNTER: UnixCounter = UnixCounter;

fn ticks() -> io::Result<u64> {
    let mut now = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: `now` is a valid out-pointer for clock_gettime.
    if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut now) } != 0 {
        return Err(io::Error::last_os_error());
    }
    let seconds = u64::try_from(now.tv_sec).map_err(|_| unsupported())?;
    let nanos = u64::try_from(now.tv_nsec).map_err(|_| unsupported())?;
    seconds
        .checked_mul(NANOS)
        .and_then(|value| value.checked_add(nanos))
        .ok_or_else(unsupported)
}

fn unsupported() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "Monotonic counter range is unsupported",
    )
}

/// Start stamp of `pid` (boot-relative clock ticks); never zero.
fn creation(pid: u32) -> io::Result<u64> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat"))?;
    super::ownership::stat_field(&stat, 22)
        .filter(|value| *value != 0)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Process start stamp unreadable"))
}

fn sample_for(pid: u32) -> io::Result<CounterSample> {
    let parent_creation = creation(pid)?;
    let ticks = ticks()?;
    Ok(CounterSample {
        identity: CounterIdentity {
            kind: KIND,
            frequency_hz: NANOS,
            parent_pid: pid,
            parent_creation,
        },
        ticks,
    })
}

fn current_creator() -> u32 {
    // SAFETY: getppid has no preconditions and cannot fail.
    let pid = unsafe { libc::getppid() };
    u32::try_from(pid).unwrap_or(0)
}

struct UnixCreator {
    pid: u32,
    creation: u64,
}
impl UnixCreator {
    fn live(&self) -> io::Result<bool> {
        if current_creator() != self.pid {
            return Ok(false);
        }
        match creation(self.pid) {
            Ok(value) => Ok(value == self.creation),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }
}
impl LiveCreator for UnixCreator {
    fn pid(&self) -> u32 {
        self.pid
    }
    fn sample(&self) -> io::Result<Option<CounterSample>> {
        if !self.live()? {
            return Ok(None);
        }
        let sample = sample_for(self.pid)?;
        Ok(self.live()?.then_some(sample))
    }
}

impl ParentCounter for UnixCounter {
    fn kinds(&self) -> &'static [&'static str] {
        &[KIND]
    }
    fn sample_current(&self) -> io::Result<CounterSample> {
        sample_for(std::process::id())
    }
    fn open_creator(&self) -> io::Result<Box<dyn LiveCreator>> {
        let pid = current_creator();
        if pid <= 1 {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No live creator"));
        }
        let creation = creation(pid)?;
        Ok(Box::new(UnixCreator { pid, creation }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_sample_is_anchored_and_monotonic() {
        let first = COUNTER.sample_current().unwrap();
        let second = COUNTER.sample_current().unwrap();
        assert_eq!(first.identity, second.identity);
        assert_eq!(first.identity.parent_pid, std::process::id());
        assert!(second.ticks >= first.ticks);
    }

    #[test]
    fn creator_shares_the_host_domain() {
        let creator = COUNTER.open_creator().unwrap();
        let sample = creator
            .sample()
            .unwrap()
            .expect("test harness parent is live");
        assert_eq!(sample.identity.parent_pid, creator.pid());
        assert!(
            sample.ticks
                >= COUNTER
                    .sample_current()
                    .unwrap()
                    .ticks
                    .saturating_sub(NANOS)
        );
    }
}
