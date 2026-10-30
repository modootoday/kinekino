use crate::cpu_selection::{first_n, first_two};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    Conservative2,
    Build4,
}
impl Profile {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "conservative2" => Some(Self::Conservative2),
            "build4" => Some(Self::Build4),
            _ => None,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Conservative2 => "conservative2",
            Self::Build4 => "build4",
        }
    }
    pub fn requested_cpus(self) -> u32 {
        match self {
            Self::Conservative2 => 2,
            Self::Build4 => 4,
        }
    }
    pub fn select(self, permitted: usize) -> Option<usize> {
        match self {
            Self::Conservative2 => (permitted != 0).then(|| first_two(permitted)),
            Self::Build4 => (permitted.count_ones() >= 4).then(|| first_n(permitted, 4)),
        }
    }
}

pub const MIN_FREE_BYTES: u64 = 16 * 1024 * 1024 * 1024;

#[derive(Clone, Copy, Debug)]
pub struct Snapshot {
    pub idle: u64,
    pub kernel: u64,
    pub user: u64,
    pub free_bytes: u64,
}
#[derive(Clone, Copy, Debug)]
pub struct Interval {
    pub busy: u64,
    pub total: u64,
    pub minimum_free_bytes: u64,
    pub elapsed_ms: u128,
}
impl Interval {
    pub fn between(start: Snapshot, end: Snapshot, elapsed_ms: u128) -> Option<Self> {
        let total = end
            .kernel
            .checked_sub(start.kernel)?
            .checked_add(end.user.checked_sub(start.user)?)?;
        let busy = total.checked_sub(end.idle.checked_sub(start.idle)?)?;
        (total != 0).then_some(Self {
            busy,
            total,
            minimum_free_bytes: start.free_bytes.min(end.free_bytes),
            elapsed_ms,
        })
    }
    pub fn rejection(self) -> Option<&'static str> {
        if self.elapsed_ms < 1000 {
            Some("sample_interval_too_short")
        } else if u128::from(self.busy) * 100 >= u128::from(self.total) * 50 {
            Some("cpu_not_below_50_percent")
        } else if self.minimum_free_bytes < MIN_FREE_BYTES {
            Some("free_ram_below_16_gib")
        } else {
            None
        }
    }
}

#[cfg(test)]
#[path = "execution_profile/tests.rs"]
mod tests;
