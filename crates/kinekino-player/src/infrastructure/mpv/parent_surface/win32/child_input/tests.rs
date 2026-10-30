use super::InputReadback;

#[test]
fn late_destroy_ack_requests_explicit_class_finalization_without_os_reissue() {
    use super::{ProbeError, ProbeRelease, release_observation};
    assert_eq!(
        release_observation(false, false, 7, false).unwrap(),
        ProbeRelease::Unobserved
    );
    assert!(matches!(
        release_observation(true, false, 7, false),
        Err(ProbeError::ClassFinalizationRequired)
    ));
    assert_eq!(
        release_observation(false, true, 7, false).unwrap(),
        ProbeRelease::Complete { generation: 7 }
    );
}

#[test]
fn every_readback_requirement_is_fail_closed() {
    let good = InputReadback {
        installed: true,
        destroying: false,
        callback_fault: false,
        same_process: true,
        same_thread: true,
        enabled: false,
        visible: false,
    };
    assert!(good.accepts());
    for bad in [
        InputReadback {
            installed: false,
            ..good
        },
        InputReadback {
            destroying: true,
            ..good
        },
        InputReadback {
            callback_fault: true,
            ..good
        },
        InputReadback {
            same_process: false,
            ..good
        },
        InputReadback {
            same_thread: false,
            ..good
        },
        InputReadback {
            enabled: true,
            ..good
        },
        InputReadback {
            visible: true,
            ..good
        },
    ] {
        assert!(!bad.accepts());
    }
}
