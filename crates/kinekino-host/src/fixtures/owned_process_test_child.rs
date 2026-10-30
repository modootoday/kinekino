//! Explicit test-support executable; no shell, network, browser or media dependencies.
use std::{
    env,
    io::{self, Read, Write},
    process::{Command, Stdio},
    thread,
    time::Duration,
};

pub fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let mode = args.first().map(String::as_str).unwrap_or("eof");
    match mode {
        #[cfg(windows)]
        "source-ipc-echo" | "source-ipc-hold" | "source-ipc-exit" => {
            source_ipc_stimulus(mode, &args)
        }
        "args" => {
            for value in args.iter().skip(1) {
                let hex = value
                    .as_bytes()
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>();
                println!("arg:{}:{hex}", value.len());
            }
        }
        "eof" => {
            println!("ready");
            let _ = io::stdout().flush();
            let mut byte = [0u8; 1];
            loop {
                match io::stdin().read(&mut byte) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(_) => std::process::exit(3),
                }
            }
            println!("terminal");
        }
        "hang" | "sentinel" => loop {
            thread::sleep(Duration::from_millis(50));
        },
        "silent" => {
            let _ = io::stdin().read(&mut [0u8; 1]);
        }
        "partial" => {
            print!("ready");
            let _ = io::stdout().flush();
            loop {
                thread::sleep(Duration::from_millis(50));
            }
        }
        "flood" => {
            let bytes = [b'x'; 4096];
            for _ in 0..16 {
                io::stdout().write_all(&bytes).unwrap();
            }
            loop {
                thread::sleep(Duration::from_millis(50));
            }
        }
        "malformed" => {
            println!("{{not-json}}");
        }
        "descendant" | "parent-exit" => {
            descendant_stimulus(mode == "parent-exit");
        }
        "exit" => std::process::exit(7),
        _ => std::process::exit(2),
    }
}

#[cfg(windows)]
fn source_ipc_stimulus(mode: &str, args: &[String]) {
    use kinekino_process::{domain::source_clock::SourceCutoffs, infrastructure::ipc::*};
    let value = |prefix: &str| -> u64 {
        let values: Vec<_> = args
            .iter()
            .filter_map(|arg| arg.strip_prefix(prefix))
            .collect();
        if values.len() != 1 {
            std::process::exit(20);
        }
        values[0].parse().unwrap_or_else(|_| std::process::exit(21))
    };
    let cutoffs = SourceCutoffs::new(value(OPERATION_ARGUMENT), value(END_ARGUMENT))
        .unwrap_or_else(|_| std::process::exit(22));
    let control =
        usize::try_from(value(CONTROL_ARGUMENT)).unwrap_or_else(|_| std::process::exit(23));
    let data = usize::try_from(value(DATA_ARGUMENT)).unwrap_or_else(|_| std::process::exit(23));
    // Only the explicit test-support child under spawn_source consumes these
    // original inherited handles. These arguments issue no application authority.
    let mut ipc = unsafe { SourceIpc::from_inherited(control, data, cutoffs) }
        .unwrap_or_else(|_| std::process::exit(24));
    if mode == "source-ipc-exit" {
        return;
    }
    ipc.control().start_read(MAX_CONTROL_SEGMENT).unwrap();
    let echo = mode == "source-ipc-echo";
    if echo {
        ipc.data().start_read(MAX_DATA_SEGMENT).unwrap();
    }
    let mut writing = false;
    let mut pending = Vec::new();
    let mut offset = 0;
    while let Ok(IoPoll::Pending) = ipc.control().poll_read() {
        if echo {
            let result = if writing {
                ipc.data().poll_write()
            } else {
                ipc.data().poll_read()
            };
            match result {
                Ok(IoPoll::Pending) => {}
                Ok(IoPoll::Read(bytes)) => {
                    pending = bytes;
                    offset = 0;
                    ipc.data().start_write(&pending).unwrap();
                    writing = true;
                }
                Ok(IoPoll::Written(count)) if count > 0 => {
                    offset += count;
                    if offset < pending.len() {
                        ipc.data().start_write(&pending[offset..]).unwrap();
                    } else {
                        pending.clear();
                        ipc.data().start_read(MAX_DATA_SEGMENT).unwrap();
                        writing = false;
                    }
                }
                Ok(_) | Err(_) => break,
            }
        }
        if uptime_millis() >= cutoffs.operation() {
            break;
        }
        thread::sleep(Duration::from_millis(1));
    }
    loop {
        if ipc.poll_close().io_reaped {
            break;
        }
        if uptime_millis() >= cutoffs.end() {
            std::process::exit(25);
        }
        thread::sleep(Duration::from_millis(1));
    }
}

/// Used by the parent-exit/outside-sentinel regression; a live descendant is the required input.
#[expect(
    clippy::zombie_processes,
    reason = "The separate test supervisor owns this dedicated family Job; waiting before parent exit would erase the regression"
)]
fn descendant_stimulus(exit_parent: bool) {
    let mut command = Command::new(env::current_exe().unwrap());
    command
        .arg("hang")
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    kinekino_platform::process::hide_console_window(&mut command);
    let mut child = command.spawn().unwrap();
    println!("descendant:{}", child.id());
    let _ = io::stdout().flush();
    if exit_parent {
        return;
    }
    let _ = child.wait(); // Deliberate stimulus; never supervisor/Drop behavior.
}
