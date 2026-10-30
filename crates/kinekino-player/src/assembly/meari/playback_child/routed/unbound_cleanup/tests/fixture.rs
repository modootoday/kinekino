//! Public disabled NoMedia engine constructor; does not synthesize a window proof or Ready.
use crate::infrastructure::{
    catalog::CatalogFiles,
    mpv::{NativeEpisodeEngine, TrustedMpvExecutable},
};
use std::{
    fs,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

pub(super) fn engine() -> Arc<NativeEpisodeEngine> {
    use sha2::{Digest, Sha256};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "unbound-task-ordering-{}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    eprintln!("unbound_task_ordering_fixture={}", root.display());
    fs::create_dir(root.join("catalog")).unwrap();
    let bytes = b"inert unbound task ordering artifact; never executed";
    let executable = root.join("mpv.exe");
    fs::write(&executable, bytes).unwrap();
    let trusted =
        TrustedMpvExecutable::open(&executable, &format!("{:x}", Sha256::digest(bytes))).unwrap();
    Arc::new(
        NativeEpisodeEngine::new(
            trusted,
            Some(1),
            Arc::new(CatalogFiles::open(&root.join("catalog")).unwrap()),
        )
        .unwrap(),
    )
}
