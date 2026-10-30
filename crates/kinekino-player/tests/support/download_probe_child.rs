//! Standalone std-only synthetic child, compiled by explicitly allocated rustc.
//! Exercises original regular-file stdin/seek and ownership; never a real decoder.
use std::{
    ffi::c_void,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    os::windows::io::BorrowedHandle,
    time::Duration,
};

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetStdHandle(kind: u32) -> *mut c_void;
}
fn main() {
    if let Err(error) = run() {
        eprintln!("download_probe_fixture_error={error}");
        std::process::exit(64);
    }
}
fn run() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let capability = args == ["-v", "error", "-protocols"];
    let held = std::env::current_exe()?
        .file_stem()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.contains("held"));
    if capability {
        if held {
            let mut ready = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open("probe.ready")?;
            ready.write_all(b"ready\n")?;
            ready.sync_all()?;
            std::thread::sleep(Duration::from_secs(30));
        }
        println!("Supported protocols:\nInput:\n  fd\nOutput:\n  fd");
        return Ok(());
    }
    let expected = [
        "-v",
        "error",
        "-protocol_whitelist",
        "fd",
        "-f",
        "mov",
        "-enable_drefs",
        "0",
        "-fd",
        "0",
        "-blocksize",
        "65536",
        "-count_packets",
        "-show_entries",
        "stream=index,codec_type,codec_name,width,height,channels,sample_rate,nb_read_packets:format=format_name,duration",
        "-of",
        "json",
        "-i",
        "fd:",
    ];
    if args != expected {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "unexpected fixed probe arguments",
        ));
    }
    // Borrow original inherited stdin, duplicate privately, and exercise actual
    // regular-file seek. Never open a pathname to replace the supplied input.
    let raw = unsafe { GetStdHandle((-10_i32) as u32) };
    if raw.is_null() || raw as isize == -1 {
        return Err(std::io::Error::other("original stdin is unavailable"));
    }
    let input = unsafe { BorrowedHandle::borrow_raw(raw) }.try_clone_to_owned()?;
    let mut input = File::from(input);
    if !input.metadata()?.is_file() || input.seek(SeekFrom::End(0))? != 1024 {
        return Err(std::io::Error::other(
            "original stdin is not the expected seekable file",
        ));
    }
    input.seek(SeekFrom::Start(128))?;
    let mut sample = [0_u8; 1];
    input.read_exact(&mut sample)?;
    if sample != [7] {
        return Err(std::io::Error::other("original source byte differs"));
    }
    input.seek(SeekFrom::Start(0))?;
    let mut bytes = Vec::new();
    input.take(1025).read_to_end(&mut bytes)?;
    if bytes.len() != 1024 || bytes.iter().any(|byte| *byte != 7) {
        return Err(std::io::Error::other("original stdin EOF/content mismatch"));
    }
    println!(
        "{}",
        r#"{"format":{"format_name":"mov,mp4,m4a,3gp,3g2,mj2","duration":"1.000000"},"streams":[{"index":0,"codec_type":"video","codec_name":"h264","width":640,"height":360,"nb_read_packets":"25"},{"index":1,"codec_type":"audio","codec_name":"aac","channels":2,"sample_rate":"48000","nb_read_packets":"47"}],"fixture_original_stdin_seek":true,"fixture_original_stdin_eof":true}"#
    );
    Ok(())
}
