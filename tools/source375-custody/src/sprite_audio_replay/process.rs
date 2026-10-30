use crate::infrastructure as io;
use std::{
    fs::{self, File},
    os::windows::process::CommandExt,
    path::Path,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
struct Owned(Child);
impl Drop for Owned {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
pub(super) fn decode(
    tool: &Path,
    source: &Path,
    raw: &Path,
    output: &Path,
    trace: bool,
) -> Result<(), String> {
    let stderr = output.join("decode.stderr");
    let started = Instant::now();
    let mut command = Command::new(tool);
    command.args([
        "-hide_banner",
        "-nostdin",
        "-nostats",
        "-loglevel",
        "info",
        "-threads",
        "2",
        "-filter_threads",
        "1",
        "-protocol_whitelist",
        "file",
        "-f",
        "mov",
        "-enable_drefs",
        "0",
    ]);
    if trace {
        command.arg("-copyts");
    }
    command
        .arg("-i")
        .arg(source)
        .args(["-map", "0:a:0", "-vn", "-sn", "-dn"]);
    if trace {
        command.args([
            "-af",
            "aresample=16000:osf=s16:ochl=mono:async=0,asettb=expr=1/16000,ashowinfo",
        ]);
    } else {
        command.args(["-ac", "1", "-ar", "16000"]);
    }
    command
        .args(["-c:a", "pcm_s16le", "-f", "s16le", "pipe:1"])
        .creation_flags(0x08000000)
        .stdin(Stdio::null())
        .stdout(File::create(raw).map_err(|e| e.to_string())?)
        .stderr(File::create(&stderr).map_err(|e| e.to_string())?);
    io::write_new(&output.join("command.txt"),format!("{command:?}\naccepted_pcm_bytes_max=25165824\naccepted_stderr_bytes_max=12582912\ndeadline_ms=30000\ntrace={trace}\n").as_bytes())?;
    let mut child = Owned(command.spawn().map_err(|e| e.to_string())?);
    let result = loop {
        let large = fs::metadata(raw).map_err(|e| e.to_string())?.len() > 24 * 1024 * 1024
            || fs::metadata(&stderr).map_err(|e| e.to_string())?.len() > 12 * 1024 * 1024;
        if large || started.elapsed() > Duration::from_secs(30) {
            break Err(if large {
                "decode response exceeded bound"
            } else {
                "decode deadline elapsed"
            }
            .to_string());
        }
        if let Some(status) = child.0.try_wait().map_err(|e| e.to_string())? {
            break if status.success() {
                Ok(())
            } else {
                Err(format!("decode exit={:?}", status.code()))
            };
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    if result.is_err() {
        let _ = child.0.kill();
        let _ = child.0.wait();
    }
    io::write_new(
        &output.join("process.txt"),
        format!(
            "pid={}\nresult={result:?}\nelapsed_ms={}\nterminal=true\n",
            child.0.id(),
            started.elapsed().as_millis()
        )
        .as_bytes(),
    )?;
    result?;
    if fs::metadata(raw).map_err(|e| e.to_string())?.len() > 24 * 1024 * 1024
        || fs::metadata(stderr).map_err(|e| e.to_string())?.len() > 12 * 1024 * 1024
    {
        return Err("final decode response exceeded bound".into());
    }
    Ok(())
}
