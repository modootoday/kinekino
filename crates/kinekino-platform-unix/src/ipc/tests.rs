use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn private_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "kk{tag}{:x}",
        u64::from_le_bytes(crate::id::fresh_guid().unwrap()[..8].try_into().unwrap())
    ));
    std::fs::DirBuilder::new().mode(0o700).create(&dir).unwrap();
    dir
}

#[test]
fn rejects_other_uid_and_root_mix() {
    let me = identity_of(1000);
    assert!(permits(&me, &identity_of(1000)));
    assert!(!permits(&me, &identity_of(1001)));
    assert!(!permits(&me, &identity_of(0)));
    assert!(!permits(&identity_of(0), &me));
}

#[test]
fn parses_proc_start_with_hostile_comm() {
    let fields: Vec<String> = (3..=22).map(|n| n.to_string()).collect();
    let stat = format!("7 (a) b) c) {}", fields.join(" "));
    assert_eq!(parse_start(&stat), Some(22));
    assert!(process_start(std::process::id()).unwrap() > 0);
}

#[test]
fn refuses_shared_or_foreign_runtime_dirs() {
    let dir = private_dir("mode");
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
    let ipc = UnixIpc {
        runtime_dir: Some(dir.clone()),
    };
    let id = ipc.fresh_endpoint("kinekino").unwrap();
    assert_eq!(
        ipc.address(&id).unwrap_err().kind(),
        io::ErrorKind::PermissionDenied
    );
    assert!(check_private_dir(&dir, current_uid().wrapping_add(1)).is_err());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn rejects_bad_product_names() {
    let dir = private_dir("name");
    let ipc = UnixIpc {
        runtime_dir: Some(dir.clone()),
    };
    let mut id = ipc.fresh_endpoint("kinekino").unwrap();
    for bad in ["../escape", "..", ".hidden", "a/b", ""] {
        id.product = bad.into();
        assert!(ipc.address(&id).is_err(), "{bad}");
    }
    id.product = "com.kinekino.player".into();
    assert!(ipc.address(&id).is_ok());
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn round_trip_reports_kernel_peer_and_cleans_up() {
    let dir = private_dir("rt");
    let ipc = UnixIpc {
        runtime_dir: Some(dir.clone()),
    };
    let id = ipc.fresh_endpoint("kinekino").unwrap();
    let path = PathBuf::from(ipc.address(&id).unwrap());
    let mut listener = ipc.listen(&id).unwrap();
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert!(ipc.listen(&id).is_err(), "second bind must fail");
    let mut client = ipc.connect(&id).unwrap();
    let mut server = listener.accept().await.unwrap();
    let peer = server.peer().unwrap();
    assert_eq!(peer.process_id, std::process::id());
    assert_eq!(peer.process_start, id.process_start);
    assert!(ipc.permits(&ipc.local_peer().unwrap(), &peer.identity));
    client.write_all(b"ping").await.unwrap();
    let mut buf = [0; 4];
    server.read_exact(&mut buf).await.unwrap();
    assert_eq!(&buf, b"ping");
    assert_eq!(client.peer().unwrap().process_id, std::process::id());
    listener.cancel();
    assert!(listener.accept().await.is_err());
    drop(listener);
    assert!(!path.exists());
    std::fs::remove_dir_all(dir).unwrap();
}
