//! A pending automatic pause is bounded and reset when the observed condition clears.
use super::*;
#[derive(Default)]
pub(super) struct BoundaryPause {
    pending: Option<Instant>,
}
impl BoundaryPause {
    pub(super) fn should_publish(&mut self, needed: bool, now: Instant) -> Result<bool, ApiError> {
        if !needed {
            self.pending = None;
            return Ok(false);
        }
        match self.pending {
            Some(expires) if now >= expires => Err(ApiError::new(
                ErrorCode::Timeout,
                "Review boundary pause was not observed",
            )),
            Some(_) => Ok(false),
            None => Ok(true),
        }
    }
    pub(super) fn published(&mut self, now: Instant) {
        self.pending = Some(now.checked_add(Duration::from_millis(500)).unwrap_or(now));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_pause_is_published_until_observed_and_missing_completion_is_bounded() {
        let now = Instant::now();
        let mut pause = BoundaryPause::default();
        assert!(pause.should_publish(true, now).unwrap());
        pause.published(now);
        assert!(
            !pause
                .should_publish(true, now + Duration::from_millis(20))
                .unwrap()
        );
        assert_eq!(
            pause
                .should_publish(true, now + Duration::from_millis(500))
                .unwrap_err()
                .code,
            ErrorCode::Timeout
        );
    }
    #[test]
    fn seek_or_pause_that_clears_the_condition_cancels_the_old_completion_deadline() {
        let now = Instant::now();
        let mut pause = BoundaryPause::default();
        pause.published(now);
        assert!(
            !pause
                .should_publish(false, now + Duration::from_secs(1))
                .unwrap()
        );
        assert!(
            pause
                .should_publish(true, now + Duration::from_secs(1))
                .unwrap()
        );
    }
}
