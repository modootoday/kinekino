//! Private trusted child entry; parent source factory remains unregistered.
#[cfg(windows)]
pub fn main() {
    use kinekino_process::{domain::source_clock::SourceCutoffs, infrastructure::ipc::*};
    use kinekino_source_host::{
        worker_ipc::WorkerIpc,
        worker_loop::{LoopStatus, WorkerLoop},
    };
    use std::io::Write;
    // Only inherited handle numbers and absolute clock cutoffs are argv data.
    // URL/header/request body arrive over the original private control endpoint.
    let args: Vec<_> = std::env::args().skip(1).collect();
    let read = |prefix: &str| -> Option<u64> {
        let mut values = args.iter().filter_map(|arg| arg.strip_prefix(prefix));
        let value = values.next()?.parse().ok()?;
        if values.next().is_some() {
            return None;
        }
        Some(value)
    };
    let parsed = (|| {
        if args.len() != 4 {
            return None;
        }
        let control = usize::try_from(read(CONTROL_ARGUMENT)?).ok()?;
        let data = usize::try_from(read(DATA_ARGUMENT)?).ok()?;
        let cutoffs = SourceCutoffs::new(read(OPERATION_ARGUMENT)?, read(END_ARGUMENT)?).ok()?;
        Some((control, data, cutoffs))
    })();
    let Some((control, data, cutoffs)) = parsed else {
        std::process::exit(20);
    };
    // This executable is only for spawn_source's trusted inherited-handle path;
    // it grants no original browser/provider replay admission to any caller.
    let Ok(ipc) = (unsafe { WorkerIpc::from_inherited(control, data, cutoffs) }) else {
        std::process::exit(21);
    };
    let mut owner = WorkerLoop::new(ipc, cutoffs);
    let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    else {
        std::process::exit(22);
    };
    std::panic::set_hook(Box::new(|_| eprintln!("source worker panic")));
    let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // Fixed diagnostic only; not request admission or physical-close ACK.
        std::io::stdout()
            .write_all(b"source-worker-ipc-ready\n")
            .expect("worker readiness IO");
        std::io::stdout().flush().expect("worker readiness flush");
        runtime.block_on(owner.run())
    }));
    let status = match run {
        Ok(status) => status,
        Err(panic) => {
            owner.begin_unwind_close(panic);
            runtime.block_on(owner.close())
        }
    };
    if status == LoopStatus::Incomplete {
        // Process exit, not a fabricated local-IO receipt. Parent must observe
        // this failure and original Job/family close using its retained handles.
        std::process::exit(124);
    }
    let failed = owner.first_error().is_some();
    // All variants contain only finite enums; never format raw reqwest errors,
    // URLs, headers, panic payloads or media bytes. This diagnostic is not an ACK.
    if writeln!(
        std::io::stdout(),
        "source-worker-loop:first={:?};secondary={};overflow={}",
        owner.first_error(),
        owner.secondary_errors().len(),
        owner.secondary_overflow()
    )
    .and_then(|()| std::io::stdout().flush())
    .is_err()
    {
        std::process::exit(26);
    }
    drop(owner);
    // Runtime Drop may await resolver threads. Parent's original Job deadline
    // bounds this child; no shutdown_timeout result is presented as physical ACK.
    drop(runtime);
    if failed {
        std::process::exit(23);
    }
}
#[cfg(not(windows))]
pub fn main() {
    std::process::exit(24);
}
