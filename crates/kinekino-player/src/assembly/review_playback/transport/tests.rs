use super::*;
use crate::{
    application::{
        review_playback::sessions::{Selection, SessionLauncher, SessionOwner},
        review_source::CaseLease,
    },
    domain::review_source::ReviewCase,
};
use std::{thread, time::Duration};

struct EmptyLauncher;
impl SessionLauncher for EmptyLauncher {
    fn retain_case(&self, _: &ReviewCase) -> Result<Box<dyn CaseLease + '_>, ApiError> {
        Err(ApiError::new(
            ErrorCode::NotFound,
            "No review case configured",
        ))
    }
    fn start(&self, _: &Selection) -> Result<Box<dyn SessionOwner>, ApiError> {
        panic!("empty host must not start media")
    }
}
pub(in crate::assembly::review_playback) fn sessions() -> Arc<ReviewPlaybackSessions> {
    Arc::new(ReviewPlaybackSessions::new("empty-host".into(), 4, EmptyLauncher).unwrap())
}

#[test]
fn invalid_transport_options_return_the_original_service_for_explicit_cleanup() {
    let sessions = sessions();
    let failure = match start_transport(
        PlayerSdk::default(),
        sessions.clone(),
        TransportOptions {
            timeout: Duration::ZERO,
        },
    ) {
        Ok(_) => panic!("invalid transport started"),
        Err(failure) => failure,
    };
    assert!(Arc::ptr_eq(&sessions, &failure.sessions));
    assert_eq!(failure.error.code, ErrorCode::InvalidArgument);
    assert!(failure.sessions.observe_shutdown().is_err());
    failure.sessions.begin_close().unwrap();
    assert!(failure.sessions.observe_shutdown().unwrap().complete);
}

#[test]
fn expired_observation_deadline_never_turns_into_success_after_real_worker_joins() {
    let sessions = sessions();
    let host = start_transport(
        PlayerSdk::default(),
        sessions.clone(),
        TransportOptions::default(),
    )
    .unwrap();
    let mut pending = host.begin_shutdown(Instant::now() - Duration::from_millis(1));
    assert!(Arc::ptr_eq(&sessions, &pending.sessions));
    let cutoff = Instant::now() + Duration::from_secs(5);
    loop {
        let observation = pending.poll();
        assert_eq!(observation.first_error.unwrap().code, ErrorCode::Timeout);
        assert!(!observation.succeeded());
        if observation.complete {
            assert!(observation.transport.is_quiescent());
            assert_eq!(observation.sessions.unwrap().owners, 0);
            break;
        }
        assert!(
            Instant::now() < cutoff,
            "original empty host workers did not join"
        );
        thread::sleep(Duration::from_millis(10));
    }
    assert!(!pending.poll().succeeded());
}
