//! Original metadata remains in its admission until one retained issue attempt.
use super::*;

pub(super) fn check_preparation(releasing: bool, deadline: Instant) -> Result<(), ApiError> {
    if releasing {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Original replay owner is releasing",
        ));
    }
    if Instant::now() >= deadline {
        return Err(ApiError::new(
            ErrorCode::Timeout,
            "Original replay preparation deadline expired",
        ));
    }
    Ok(())
}

pub(super) enum Grant {
    Ready(OwnedMediaReplay),
    Pending(Option<(OwnedMediaStream, Arc<dyn MediaReplayPolicy>)>),
}

impl Grant {
    pub fn ready(&self) -> Result<&OwnedMediaReplay, ApiError> {
        match self {
            Self::Ready(grant) => Ok(grant),
            Self::Pending(_) => Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Original replay grant is not admitted",
            )),
        }
    }
    pub fn prepare(
        &mut self,
        provider: &Arc<dyn BrowserProviderPort>,
        owner: &impl ReplayOriginOwner,
    ) -> Result<(), ApiError> {
        let Self::Pending(pending) = self else {
            return Ok(());
        };
        // Spend before any provider/policy callback, including error or unwind.
        let (stream, policy) = pending.take().ok_or_else(|| {
            ApiError::new(
                ErrorCode::SourceExpired,
                "Original replay issue was already attempted",
            )
        })?;
        owner.check_stream(provider, &stream)?;
        provider.check_media_stream(&stream)?;
        let grant = provider.issue_media_replay(stream, policy.as_ref())?;
        owner.check_origin(provider, &grant)?;
        *self = Self::Ready(grant);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_expired_deadline_is_timeout_without_renewal() {
        let original = Instant::now();
        assert_eq!(
            check_preparation(false, original).unwrap_err().code,
            ErrorCode::Timeout
        );
        assert_eq!(
            check_preparation(false, original).unwrap_err().code,
            ErrorCode::Timeout
        );
    }
    #[test]
    fn releasing_origin_cannot_prepare_with_a_later_deadline() {
        let later = Instant::now() + std::time::Duration::from_secs(5);
        assert!(check_preparation(false, later).is_ok());
        assert_eq!(
            check_preparation(true, later).unwrap_err().code,
            ErrorCode::SourceExpired
        );
    }
}
