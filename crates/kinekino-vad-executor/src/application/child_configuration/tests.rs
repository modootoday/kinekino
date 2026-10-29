use super::*;
struct Fixed(VadProfile);
impl ProfileAdmission for Fixed {
    fn authorize(&self, p: &VadProfile) -> Result<(), ApiError> {
        if p == &self.0 {
            Ok(())
        } else {
            Err(ApiError::new(
                ErrorCode::PermissionDenied,
                "test-only fixed release",
            ))
        }
    }
}
fn pin(id: &str, digest: char) -> ArtifactPin {
    ArtifactPin::new(id.into(), digest.to_string().repeat(64), DecimalU64(16)).unwrap()
}
fn selected() -> ParentSelectedRelease {
    let p = VadProfile::cached_silero(pin("license", 'a'), pin("worker", 'b')).unwrap();
    ParentSelectedRelease::authorize(p.clone(), pin("manifest", 'c'), &Fixed(p)).unwrap()
}
fn deployment_with_notices(sizes: &[u64]) -> ChildNativeDeployment {
    ChildNativeDeployment {
        worker_path: "C:/technical-only/worker.exe".into(),
        modules: vec![ChildNativeModule {
            file: ChildPinnedFile {
                path: "C:/technical-only/DirectML.dll".into(),
                pin: pin("directml", 'e'),
            },
            required_at_startup: true,
        }],
        notices: sizes
            .iter()
            .enumerate()
            .map(|(i, size)| ChildPinnedFile {
                path: format!("C:/technical-only/notice-{i}.txt"),
                pin: ArtifactPin::new(format!("notice-{i}"), "f".repeat(64), DecimalU64(*size))
                    .unwrap(),
            })
            .collect(),
        system_module_names: vec!["ntdll.dll".into()],
        minimum_windows_build: 1,
    }
}
#[test]
fn native_notice_sizes_survive_existing_bounded_parent_wire() {
    let selected = selected()
        .with_native_deployment(deployment_with_notices(&[338_088, 1073]))
        .unwrap();
    let request = pin("request", 'd');
    let (bytes, config) = selected.encode_for_request(&request).unwrap();
    assert!(bytes.len() as u64 <= MAX_CHILD_CONFIGURATION_BYTES);
    let delegated = decode_parent_configuration(&bytes, &config, &request).unwrap();
    let notices = &delegated.native_deployment().unwrap().notices;
    assert_eq!(
        notices.iter().map(|n| n.pin.bytes()).collect::<Vec<_>>(),
        [338_088, 1073]
    );
}
#[test]
fn optional_deployment_stays_absent_and_native_notice_presence_stays_required() {
    let selected = selected();
    let request = pin("request", 'd');
    let (bytes, config) = selected.encode_for_request(&request).unwrap();
    assert!(
        decode_parent_configuration(&bytes, &config, &request)
            .unwrap()
            .native_deployment()
            .is_none()
    );
    assert_eq!(
        deployment_with_notices(&[]).validate().err().unwrap().code,
        ErrorCode::SourceExpired
    );
}
#[test]
fn delegated_native_notice_limits_reject_before_worker_io() {
    use kinekino_vad::domain::{MAX_RUNTIME_NOTICE_BYTES, MAX_RUNTIME_NOTICES_TOTAL_BYTES};
    let half = MAX_RUNTIME_NOTICES_TOTAL_BYTES / 2;
    for sizes in [
        vec![MAX_RUNTIME_NOTICE_BYTES],
        vec![half, half],
        vec![65_536; 16],
    ] {
        deployment_with_notices(&sizes).validate().unwrap();
    }
    for sizes in [
        vec![MAX_RUNTIME_NOTICE_BYTES + 1],
        vec![half, half + 1],
        vec![1; 17],
    ] {
        assert_eq!(
            deployment_with_notices(&sizes)
                .validate()
                .err()
                .unwrap()
                .code,
            ErrorCode::SourceExpired
        );
    }
}
#[test]
fn matching_host_selection_supplies_request_bound_child_configuration() {
    let s = selected();
    let request = pin("request", 'd');
    let (bytes, config) = s
        .encode_for_attempt(&request, s.profile(), s.manifest(), s.profile().runtime())
        .unwrap();
    let delegated = decode_parent_configuration(&bytes, &config, &request).unwrap();
    delegated.check_request(s.profile(), s.manifest()).unwrap();
    assert_eq!(delegated.profile(), s.profile());
}
#[test]
fn same_size_other_worker_digest_cannot_produce_child_configuration() {
    let s = selected();
    assert_eq!(
        s.encode_for_attempt(
            &pin("request", 'd'),
            s.profile(),
            s.manifest(),
            &pin("worker", 'e')
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
}
#[test]
fn cross_release_cannot_produce_child_configuration_even_with_same_worker() {
    let s = selected();
    let p = s.profile();
    let other = VadProfile::new(
        p.id().into(),
        p.repo().into(),
        "e".repeat(40),
        p.weights().clone(),
        p.license_document().clone(),
        p.runtime().clone(),
    )
    .unwrap();
    assert_eq!(
        s.encode_for_attempt(&pin("request", 'd'), &other, s.manifest(), p.runtime())
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        s.encode_for_attempt(&pin("request", 'd'), p, &pin("manifest", 'e'), p.runtime())
            .unwrap_err()
            .code,
        ErrorCode::SourceExpired
    );
}
#[test]
fn same_bytes_other_worker_identity_cannot_produce_child_configuration() {
    let s = selected();
    assert_eq!(
        s.encode_for_attempt(
            &pin("request", 'd'),
            s.profile(),
            s.manifest(),
            &pin("alias", 'b')
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
}
#[test]
fn configuration_replay_for_another_request_is_rejected() {
    let s = selected();
    let (bytes, config) = s.encode_for_request(&pin("request", 'd')).unwrap();
    assert!(decode_parent_configuration(&bytes, &config, &pin("request", 'e')).is_err());
}
#[test]
fn edited_configuration_cannot_reuse_original_pin() {
    let s = selected();
    let request = pin("request", 'd');
    let (mut bytes, config) = s.encode_for_request(&request).unwrap();
    bytes[0] = b' ';
    assert!(decode_parent_configuration(&bytes, &config, &request).is_err());
}
#[test]
fn fixed_native_expectations_survive_delegation_without_becoming_a_grant() {
    let s = selected()
        .with_native_deployment(ChildNativeDeployment {
            worker_path: "C:/technical-only/worker.exe".into(),
            modules: vec![ChildNativeModule {
                file: ChildPinnedFile {
                    path: "C:/technical-only/DirectML.dll".into(),
                    pin: pin("directml", 'e'),
                },
                required_at_startup: true,
            }],
            notices: vec![ChildPinnedFile {
                path: "C:/technical-only/notice.txt".into(),
                pin: pin("notice", 'f'),
            }],
            system_module_names: vec!["ntdll.dll".into()],
            minimum_windows_build: 1,
        })
        .unwrap();
    let request = pin("request", 'd');
    let (bytes, pin) = s
        .encode_for_attempt(&request, s.profile(), s.manifest(), s.profile().runtime())
        .unwrap();
    let child = decode_parent_configuration(&bytes, &pin, &request).unwrap();
    let d = child.native_deployment().unwrap();
    assert_eq!(d.worker_path, "C:/technical-only/worker.exe");
    assert_eq!(d.modules[0].file.pin.id(), "directml");
    assert!(d.modules[0].required_at_startup);
    assert_eq!(d.notices[0].pin.id(), "notice");
    assert_eq!(d.system_module_names, ["ntdll.dll"]);
    assert_eq!(d.minimum_windows_build, 1);
}
