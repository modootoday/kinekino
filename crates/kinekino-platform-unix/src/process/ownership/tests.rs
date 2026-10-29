use super::*;
use kinekino_core::platform::process::{
    Deadline, Enforcement, FamilyObservation, OwnedFamily, ProcessOwnership, SpawnRequest,
};
use std::{
    fs::File,
    path::Path,
    thread,
    time::{Duration, Instant},
};

fn request<'a>(script: &str, root: &Path) -> SpawnRequest<'a> {
    let deadline = Deadline::new(
        Instant::now(),
        Duration::from_secs(20),
        Duration::from_secs(5),
    )
    .unwrap();
    SpawnRequest {
        executable: "/bin/sh".into(),
        cwd: root.to_path_buf(),
        arguments: vec!["-c".into(), script.into()],
        environment: None,
        stdin: None,
        stdout: File::create(root.join("out")).unwrap(),
        stderr: File::create(root.join("err")).unwrap(),
        deadline,
        source: None,
        fault: None,
    }
}
fn root(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("unix-owner-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir(&root).unwrap();
    root
}
fn wait_empty(family: &mut dyn OwnedFamily) -> FamilyObservation {
    let start = Instant::now();
    loop {
        let observed = family.poll_family().unwrap();
        if observed == FamilyObservation::Empty || start.elapsed() > Duration::from_secs(10) {
            return observed;
        }
        thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn capability_table_is_honest_for_this_os() {
    let caps = UnixOwnership.capabilities();
    assert_eq!(caps.family_termination, Enforcement::Advisory);
    assert!(!caps.exact_family_count);
    let expected = if cfg!(target_os = "linux") {
        Enforcement::Advisory
    } else {
        Enforcement::Unsupported
    };
    assert_eq!(caps.cleanup_on_owner_death, expected);
}

#[test]
fn terminating_the_owner_ends_descendants_in_the_group() {
    let root = root("group");
    let mut family = UnixOwnership
        .spawn(request("sleep 30 & sleep 30; wait", &root), None)
        .map_err(|failure| failure.error)
        .unwrap();
    thread::sleep(Duration::from_millis(100));
    assert!(matches!(
        family.poll_family().unwrap(),
        FamilyObservation::Active(_)
    ));
    family.terminate_owned_family().unwrap();
    assert_eq!(wait_empty(family.as_mut()), FamilyObservation::Empty);
    assert_eq!(
        family.poll_child().unwrap(),
        Some(128 + libc::SIGTERM as u32)
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn exit_code_and_probe_agree() {
    let root = root("exit");
    let mut family = UnixOwnership
        .spawn(request("exit 7", &root), None)
        .map_err(|failure| failure.error)
        .unwrap();
    let probe = family.exit_probe().unwrap();
    assert_eq!(wait_empty(family.as_mut()), FamilyObservation::Empty);
    assert_eq!(probe.poll().unwrap(), Some(7));
    assert!(family.memory_peaks().unwrap().job_peak_memory_bytes > 0);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn injected_post_create_fault_returns_the_owner_terminated() {
    let root = root("fault");
    let mut req = request("sleep 30", &root);
    req.fault = Some(kinekino_core::platform::process::FaultPoint::AfterCreate);
    let failure = UnixOwnership.spawn(req, None).err().unwrap();
    assert!(failure.termination_error.is_none());
    let mut owner = failure.owner.unwrap();
    assert_eq!(wait_empty(owner.as_mut()), FamilyObservation::Empty);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn budget_is_validated_and_read_back() {
    let budget = kinekino_core::platform::process::FamilyBudget {
        max_committed_bytes: 1 << 30,
        affinity_mask: 1,
    };
    let prepared = UnixOwnership.reserve_family(budget).unwrap();
    assert_eq!(prepared.budget(), budget);
    let zero = kinekino_core::platform::process::FamilyBudget {
        max_committed_bytes: 0,
        ..budget
    };
    assert!(UnixOwnership.reserve_family(zero).is_err());
}

#[test]
fn source_socket_pair_round_trips_and_reports_eof() {
    let made = UnixOwnership.source_pipe_pair(64).unwrap();
    assert!(!made.needs_connect);
    made.parent.verify_peer().unwrap();
    assert!(!made.parent.is_inheritable().unwrap());
    let child = made.child;
    let child_fd = {
        use std::os::fd::AsRawFd;
        child
            .as_any()
            .downcast_ref::<pipe::ChildSocket>()
            .unwrap()
            .0
            .as_raw_fd()
    };
    let mut write = made.parent.clone().write(b"ping").unwrap();
    let written = loop {
        if let Some(result) = write.poll() {
            break result.unwrap();
        }
    };
    assert_eq!(written.count, 4);
    let mut buffer = [0u8; 4];
    let count = unsafe { libc::read(child_fd, buffer.as_mut_ptr().cast(), 4) };
    assert_eq!((count, &buffer), (4, b"ping"));
    drop(child);
    let mut read = made.parent.clone().read(64).unwrap();
    let done = loop {
        if let Some(result) = read.poll() {
            break result.unwrap();
        }
        thread::sleep(Duration::from_millis(5));
    };
    assert!(done.eof);
}
