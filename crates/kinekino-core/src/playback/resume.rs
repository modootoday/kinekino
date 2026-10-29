//! Resume policy (#9): when to save a position and where to restart.

/// Saved position for one media key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResumePoint {
    pub position_us: u64,
    pub duration_us: u64,
}

/// Thresholds; defaults skip the first 10 s and treat the last 5% (or 60 s) as finished.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResumePolicy {
    pub min_position_us: u64,
    pub finished_tail_us: u64,
    pub finished_fraction: f64,
    /// Restart this far before the saved point so context is not lost.
    pub rewind_us: u64,
}

impl Default for ResumePolicy {
    fn default() -> Self {
        Self {
            min_position_us: 10_000_000,
            finished_tail_us: 60_000_000,
            finished_fraction: 0.95,
            rewind_us: 3_000_000,
        }
    }
}

impl ResumePolicy {
    pub fn finished(&self, position_us: u64, duration_us: u64) -> bool {
        duration_us > 0
            && (position_us.saturating_add(self.finished_tail_us) >= duration_us
                || position_us as f64 >= duration_us as f64 * self.finished_fraction)
    }

    /// What to store on stop: `None` means clear (too early, finished or unknown length).
    pub fn record(&self, position_us: u64, duration_us: u64) -> Option<ResumePoint> {
        (duration_us > 0
            && position_us >= self.min_position_us
            && !self.finished(position_us, duration_us))
        .then_some(ResumePoint {
            position_us,
            duration_us,
        })
    }

    /// Start offset for a saved point; 0 when absent or no longer valid for `duration_us`.
    pub fn start(&self, saved: Option<ResumePoint>, duration_us: Option<u64>) -> u64 {
        let Some(p) = saved else { return 0 };
        let duration = duration_us.unwrap_or(p.duration_us);
        if self.record(p.position_us, duration).is_none() {
            return 0;
        }
        p.position_us.saturating_sub(self.rewind_us)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_middle_positions_only() {
        let p = ResumePolicy::default();
        let d = 1_200_000_000;
        assert_eq!(p.record(5_000_000, d), None);
        assert_eq!(
            p.record(600_000_000, d),
            Some(ResumePoint {
                position_us: 600_000_000,
                duration_us: d
            })
        );
        assert_eq!(p.record(1_150_000_000, d), None);
        assert_eq!(p.record(600_000_000, 0), None);
    }

    #[test]
    fn start_rewinds_and_revalidates_against_new_duration() {
        let p = ResumePolicy::default();
        let saved = Some(ResumePoint {
            position_us: 600_000_000,
            duration_us: 1_200_000_000,
        });
        assert_eq!(p.start(saved, None), 597_000_000);
        assert_eq!(p.start(saved, Some(620_000_000)), 0);
        assert_eq!(p.start(None, Some(1)), 0);
        let short = ResumePolicy {
            rewind_us: 30_000_000,
            ..p
        };
        assert_eq!(
            short.start(
                Some(ResumePoint {
                    position_us: 20_000_000,
                    duration_us: 1_200_000_000
                }),
                None
            ),
            0
        );
    }
}
