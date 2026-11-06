//! OS-specific stdio helpers for plugin worker spawn (pipes + null sink).
//! Platform `cfg` lives here only; domain worker logic stays cfg-free (structure r-cfg).
use std::fs::{File, OpenOptions};

pub(super) fn null_sink() -> std::io::Result<File> {
    #[cfg(windows)]
    {
        OpenOptions::new().write(true).open("NUL")
    }
    #[cfg(not(windows))]
    {
        OpenOptions::new().write(true).open("/dev/null")
    }
}

pub(super) fn file_from_pipe_reader(reader: std::io::PipeReader) -> File {
    #[cfg(unix)]
    {
        File::from(std::os::fd::OwnedFd::from(reader))
    }
    #[cfg(windows)]
    {
        File::from(std::os::windows::io::OwnedHandle::from(reader))
    }
}

pub(super) fn file_from_pipe_writer(writer: std::io::PipeWriter) -> File {
    #[cfg(unix)]
    {
        File::from(std::os::fd::OwnedFd::from(writer))
    }
    #[cfg(windows)]
    {
        File::from(std::os::windows::io::OwnedHandle::from(writer))
    }
}
