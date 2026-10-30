//! Fixed argv for an explicit native attempt; no shell or externally supplied options.
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        playback_session::EpisodePlaybackSource,
    },
    domain::request_referer::RequestReferer,
};
use std::ffi::OsString;

pub(super) fn arguments(
    source: &EpisodePlaybackSource,
    paused: bool,
    volume_percent: f64,
) -> Result<Vec<OsString>, ApiError> {
    if !volume_percent.is_finite() || !(0.0..=100.0).contains(&volume_percent) {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Invalid native volume",
        ));
    }
    let mut args: Vec<OsString> = [
        "--no-config",
        "--load-scripts=no",
        "--ytdl=no",
        "--autoload-files=no",
        "--load-unsafe-playlists=no",
        "--resume-playback=no",
        "--save-position-on-quit=no",
        "--screenshot-png-compression=1",
        "--screenshot-png-filter=0",
        "--idle=yes",
        "--force-window=yes",
        "--hwdec=auto-safe",
        "--osc=no",
        "--input-default-bindings=no",
        "--input-vo-keyboard=no",
        "--tls-verify=yes",
        "--network-timeout=15",
        "--demuxer=lavf",
        "--demuxer-lavf-format=hls",
        "--demuxer-lavf-o=protocol_whitelist=%20%https,tls,tcp,crypto",
        "--demuxer-lavf-o-append=allowed_extensions=ALL",
        "--demuxer-lavf-o-append=extension_picky=0",
    ]
    .into_iter()
    .map(OsString::from)
    .collect();
    args.extend(super::pointer_policy::ARGUMENTS.map(OsString::from));
    referer_argument(&mut args, source.referer())?;
    args.push(format!("--title={}", crate::domain::product::PRODUCT.display_name).into());
    args.push(format!("--pause={}", if paused { "yes" } else { "no" }).into());
    args.push(format!("--volume={volume_percent}").into());
    Ok(args)
}

fn referer_argument(args: &mut Vec<OsString>, referer: &RequestReferer) -> Result<(), ApiError> {
    match referer {
        RequestReferer::Absent => Ok(()),
        RequestReferer::Empty => Err(ApiError::new(
            ErrorCode::Unsupported,
            "Explicit empty wire Referer transfer is not verified for this native adapter",
        )),
        RequestReferer::Url(value) if referer.allowed() => {
            args.push(format!("--referrer={value}").into());
            Ok(())
        }
        RequestReferer::Url(_) => Err(ApiError::new(
            ErrorCode::PermissionDenied,
            "Native source Referer is outside policy",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_launch_preserves_pointer_events_and_product_owned_visibility() {
        let source = EpisodePlaybackSource::synthetic_replacement_source(RequestReferer::Absent);
        let args = arguments(&source, false, 0.).unwrap();
        assert!(args.contains(&OsString::from("--input-cursor=yes")));
        assert!(args.contains(&OsString::from("--cursor-autohide=no")));
        assert!(!args.contains(&OsString::from("--input-cursor=no")));
        assert!(!args.contains(&OsString::from("--input-default-bindings=yes")));
    }
    #[test]
    fn absence_is_not_a_local_source_and_empty_is_not_silently_omitted() {
        let mut args = Vec::new();
        referer_argument(&mut args, &RequestReferer::Absent).unwrap();
        assert!(args.is_empty());
        assert_eq!(
            referer_argument(&mut args, &RequestReferer::Empty)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
        assert!(args.is_empty());
        referer_argument(
            &mut args,
            &RequestReferer::Url("https://media.example.test/".into()),
        )
        .unwrap();
        assert_eq!(
            args,
            vec![OsString::from("--referrer=https://media.example.test/")]
        );
        assert!(
            referer_argument(&mut args, &RequestReferer::Url("file:///private".into())).is_err()
        );
    }
}
