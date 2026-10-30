//! Shared fixed first-stream CPU capture command. No tool discovery or execution.
use crate::application::api::{ApiError, ErrorCode};
use std::{ffi::OsString, path::Path};
#[derive(Clone, Copy)]
pub(crate) enum Selection {
    FirstDecoded,
    AtOrAfter(u128),
}
pub(crate) fn arguments(
    input: &Path,
    demuxer: &str,
    selection: Selection,
    seek: Option<&str>,
    threads: u8,
) -> Result<Vec<OsString>, ApiError> {
    if !["mov", "matroska"].contains(&demuxer) || !(1..=2).contains(&threads) {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Unsupported capture demuxer or thread budget",
        ));
    }
    let filter = match selection {
        Selection::FirstDecoded => "select=eq(n\\,0),showinfo".to_owned(),
        Selection::AtOrAfter(threshold) if threshold <= 1u128 << 53 => {
            format!("select=gte(pts\\,{threshold}),showinfo")
        }
        Selection::AtOrAfter(_) => {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Capture threshold exceeds exact FFmpeg expression integer range",
            ));
        }
    };
    let count = threads.to_string();
    let mut arguments: Vec<OsString> = [
        "-hide_banner",
        "-nostdin",
        "-loglevel",
        "info",
        "-filter_threads",
        "1",
        "-max_alloc",
        "268435456",
        "-threads",
        &count,
        "-copyts",
        "-noautorotate",
        "-protocol_whitelist",
        "file",
        "-f",
        demuxer,
    ]
    .into_iter()
    .map(OsString::from)
    .collect();
    if demuxer == "mov" {
        arguments.extend(["-enable_drefs", "0"].into_iter().map(OsString::from));
    }
    if let Some(timestamp) = seek {
        arguments.extend(
            ["-seek_timestamp", "1", "-noaccurate_seek", "-ss", timestamp]
                .into_iter()
                .map(OsString::from),
        );
    }
    arguments.push("-i".into());
    arguments.push(input.as_os_str().to_owned());
    arguments.extend(
        [
            "-map",
            "0:v:0",
            "-an",
            "-sn",
            "-dn",
            "-vf",
            &filter,
            "-frames:v",
            "1",
            "-fps_mode",
            "passthrough",
            "-c:v",
            "png",
            "-pix_fmt",
            "rgb24",
            "-threads",
            &count,
            "-f",
            "image2pipe",
            "pipe:1",
        ]
        .into_iter()
        .map(OsString::from),
    );
    Ok(arguments)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_frame_and_exact_integer_threshold_share_fixed_source_output_contract() {
        let source = Path::new("C:/private/source.mp4");
        let origin = arguments(source, "mov", Selection::FirstDecoded, None, 1).unwrap();
        assert!(origin.contains(&OsString::from("select=eq(n\\,0),showinfo")));
        assert!(!origin.contains(&OsString::from("-ss")));
        let endpoint = arguments(
            source,
            "mov",
            Selection::AtOrAfter(208896),
            Some("15.000000"),
            2,
        )
        .unwrap();
        assert!(endpoint.contains(&OsString::from("select=gte(pts\\,208896),showinfo")));
        assert_eq!(endpoint.last().unwrap(), "pipe:1");
        assert!(
            arguments(
                source,
                "mov",
                Selection::AtOrAfter((1u128 << 53) + 1),
                None,
                1
            )
            .is_err()
        );
        assert!(arguments(source, "concat", Selection::FirstDecoded, None, 1).is_err());
        assert!(arguments(source, "mov", Selection::FirstDecoded, None, 0).is_err());
    }
}
