use crate::application::review_playback::ReviewContentClass;
use std::path::Path;

pub(super) fn arguments(
    source: &Path,
    pipe: &str,
    threads: u32,
    classification: &ReviewContentClass,
) -> Vec<std::ffi::OsString> {
    let mut args = [
        "--no-config",
        "--load-scripts=no",
        "--ytdl=no",
        "--autoload-files=no",
        "--load-unsafe-playlists=no",
        "--sub-auto=no",
        "--audio-file-auto=no",
        "--resume-playback=no",
        "--save-position-on-quit=no",
        "--input-terminal=no",
        "--terminal=no",
        "--osc=no",
        "--input-default-bindings=no",
        "--input-vo-keyboard=no",
        "--vo=null",
        "--ao=null",
        "--hwdec=no",
        "--pause=yes",
        "--keep-open=yes",
        "--demuxer=lavf",
        "--demuxer-lavf-o=protocol_whitelist=file",
    ]
    .map(std::ffi::OsString::from)
    .to_vec();
    match classification {
        ReviewContentClass::IsoBmffHeader => args.extend(
            [
                "--demuxer-lavf-format=mov",
                "--demuxer-lavf-o-append=enable_drefs=0",
            ]
            .map(std::ffi::OsString::from),
        ),
        ReviewContentClass::MatroskaHeader => args.push("--demuxer-lavf-format=matroska".into()),
    }
    args.extend([
        format!("--vd-lavc-threads={threads}").into(),
        format!("--ad-lavc-threads={threads}").into(),
        format!("--input-ipc-server={pipe}").into(),
        "--".into(),
        source.as_os_str().to_owned(),
    ]);
    args
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn admitted_container_selects_demuxer_without_external_loading() {
        for class in [
            ReviewContentClass::IsoBmffHeader,
            ReviewContentClass::MatroskaHeader,
        ] {
            let args = arguments(Path::new("C:/opaque-object"), "owned-pipe", 1, &class);
            let strings: Vec<_> = args.iter().map(|arg| arg.to_str().unwrap()).collect();
            let mov = class == ReviewContentClass::IsoBmffHeader;
            assert_eq!(strings.contains(&"--demuxer-lavf-format=mov"), mov);
            assert_eq!(
                strings.contains(&"--demuxer-lavf-o-append=enable_drefs=0"),
                mov
            );
            assert_eq!(strings.contains(&"--demuxer-lavf-format=matroska"), !mov);
            for required in [
                "--no-config",
                "--load-scripts=no",
                "--ytdl=no",
                "--autoload-files=no",
                "--load-unsafe-playlists=no",
                "--sub-auto=no",
                "--audio-file-auto=no",
                "--demuxer-lavf-o=protocol_whitelist=file",
                "--vo=null",
                "--ao=null",
            ] {
                assert!(strings.contains(&required));
            }
            assert_eq!(&strings[strings.len() - 2..], &["--", "C:/opaque-object"]);
        }
    }
}
