use super::{HlsError, HlsPlaylistKind, HlsSummary, VerifiedHlsManifest, attributes};
use crate::domain::navigation::media_transport_allowed;
use url::Url;

enum Pending {
    Variant,
    Segment,
}

pub fn verify(base_url: &str, bytes: &[u8]) -> Result<VerifiedHlsManifest, HlsError> {
    if bytes.len() > 2 * 1024 * 1024 {
        return Err(HlsError::Limit);
    }
    if !media_transport_allowed(base_url) {
        return Err(HlsError::UnsafeTransport);
    }
    let base = Url::parse(base_url).map_err(|_| HlsError::Invalid)?;
    let text = std::str::from_utf8(bytes).map_err(|_| HlsError::Invalid)?;
    if text.starts_with('\u{feff}') {
        return Err(HlsError::Invalid);
    }
    let mut lines = text.lines();
    if lines.next() != Some("#EXTM3U") {
        return Err(HlsError::Invalid);
    }
    let mut summary = HlsSummary {
        kind: HlsPlaylistKind::Media,
        variants: 0,
        segments: 0,
        resource_references: 0,
        encrypted: false,
        end_list: false,
    };
    let mut resources = Vec::new();
    let mut pending = None;
    let mut target_duration = None;
    let mut max_duration = 0.0f64;
    let mut version_seen = false;
    let mut master_only = false;
    let mut media_only = false;
    for (index, raw) in lines.enumerate() {
        if index >= 16384 || raw.len() > 8192 {
            return Err(HlsError::Limit);
        }
        let line = raw.trim_end_matches('\r');
        if line.chars().any(char::is_control) {
            return Err(HlsError::Invalid);
        }
        if line.is_empty() {
            continue;
        }
        if !line.starts_with('#') {
            add_resource(&base, line, &mut resources)?;
            match pending.take().ok_or(HlsError::Invalid)? {
                Pending::Variant => summary.variants += 1,
                Pending::Segment => summary.segments += 1,
            }
            continue;
        }
        let (tag, value) = line.split_once(':').unwrap_or((line, ""));
        master_only |= matches!(
            tag,
            "#EXT-X-STREAM-INF"
                | "#EXT-X-I-FRAME-STREAM-INF"
                | "#EXT-X-MEDIA"
                | "#EXT-X-SESSION-DATA"
                | "#EXT-X-SESSION-KEY"
        );
        media_only |= matches!(
            tag,
            "#EXTINF"
                | "#EXT-X-TARGETDURATION"
                | "#EXT-X-MEDIA-SEQUENCE"
                | "#EXT-X-DISCONTINUITY-SEQUENCE"
                | "#EXT-X-ENDLIST"
                | "#EXT-X-KEY"
                | "#EXT-X-MAP"
                | "#EXT-X-DISCONTINUITY"
                | "#EXT-X-BYTERANGE"
                | "#EXT-X-PROGRAM-DATE-TIME"
                | "#EXT-X-PLAYLIST-TYPE"
                | "#EXT-X-I-FRAMES-ONLY"
        );
        if master_only && media_only {
            return Err(HlsError::Invalid);
        }
        if matches!(
            tag,
            "#EXT-X-ENDLIST"
                | "#EXT-X-DISCONTINUITY"
                | "#EXT-X-INDEPENDENT-SEGMENTS"
                | "#EXT-X-I-FRAMES-ONLY"
        ) && line != tag
        {
            return Err(HlsError::Invalid);
        }
        match tag {
            "#EXTINF" => {
                let number = value.split_once(',').ok_or(HlsError::Invalid)?.0;
                if number
                    .bytes()
                    .any(|byte| !byte.is_ascii_digit() && byte != b'.')
                {
                    return Err(HlsError::Invalid);
                }
                let duration: f64 = number.parse().map_err(|_| HlsError::Invalid)?;
                if pending.is_some() || !duration.is_finite() || duration <= 0.0 {
                    return Err(HlsError::Invalid);
                }
                max_duration = max_duration.max(duration);
                pending = Some(Pending::Segment);
            }
            "#EXT-X-STREAM-INF" => {
                if pending.is_some() {
                    return Err(HlsError::Invalid);
                }
                let attrs = attributes::parse(value)?;
                bandwidth(&attrs)?;
                for (_, uri) in attrs.iter().filter(|(name, _)| *name == "URI") {
                    add_resource(&base, uri, &mut resources)?;
                }
                pending = Some(Pending::Variant);
            }
            "#EXT-X-I-FRAME-STREAM-INF"
            | "#EXT-X-MEDIA"
            | "#EXT-X-MAP"
            | "#EXT-X-KEY"
            | "#EXT-X-SESSION-KEY"
            | "#EXT-X-SESSION-DATA"
            | "#EXT-X-START" => {
                let attrs = attributes::parse(value)?;
                let uri = attrs
                    .iter()
                    .find(|(name, _)| *name == "URI")
                    .map(|(_, value)| *value);
                if let Some((_, range)) = attrs.iter().find(|(name, _)| *name == "BYTERANGE") {
                    byte_range(range)?;
                }
                if let Some((_, iv)) = attrs.iter().find(|(name, _)| *name == "IV") {
                    let digits = iv.strip_prefix("0x").ok_or(HlsError::Invalid)?;
                    if digits.is_empty()
                        || digits.len() > 32
                        || !digits.bytes().all(|byte| byte.is_ascii_hexdigit())
                    {
                        return Err(HlsError::Invalid);
                    }
                }
                for (name, value) in &attrs {
                    if matches!(*name, "DEFAULT" | "AUTOSELECT" | "FORCED" | "PRECISE")
                        && !matches!(*value, "YES" | "NO")
                    {
                        return Err(HlsError::Invalid);
                    }
                }
                if tag == "#EXT-X-MEDIA" {
                    let get = |key| {
                        attrs
                            .iter()
                            .find(|(name, _)| *name == key)
                            .map(|(_, value)| *value)
                    };
                    if get("GROUP-ID").is_none_or(str::is_empty)
                        || get("NAME").is_none_or(str::is_empty)
                    {
                        return Err(HlsError::Invalid);
                    }
                    match get("TYPE") {
                        Some("AUDIO" | "VIDEO") => {}
                        Some("SUBTITLES") if uri.is_some() => {}
                        Some("CLOSED-CAPTIONS")
                            if uri.is_none() && get("INSTREAM-ID").is_some() => {}
                        _ => return Err(HlsError::Invalid),
                    }
                    if get("DEFAULT") == Some("YES") && get("AUTOSELECT") == Some("NO") {
                        return Err(HlsError::Invalid);
                    }
                }
                if tag == "#EXT-X-SESSION-DATA" {
                    let get = |key| {
                        attrs
                            .iter()
                            .find(|(name, _)| *name == key)
                            .map(|(_, value)| *value)
                    };
                    if get("DATA-ID").is_none_or(str::is_empty)
                        || uri.is_some() == get("VALUE").is_some()
                    {
                        return Err(HlsError::Invalid);
                    }
                }
                if tag == "#EXT-X-START" {
                    let number = attrs
                        .iter()
                        .find(|(name, _)| *name == "TIME-OFFSET")
                        .map(|(_, value)| *value)
                        .ok_or(HlsError::Invalid)?;
                    if number
                        .strip_prefix('-')
                        .unwrap_or(number)
                        .bytes()
                        .any(|byte| !byte.is_ascii_digit() && byte != b'.')
                        || number
                            .parse::<f64>()
                            .ok()
                            .is_none_or(|number| !number.is_finite())
                    {
                        return Err(HlsError::Invalid);
                    }
                }
                if tag == "#EXT-X-KEY" || tag == "#EXT-X-SESSION-KEY" {
                    match attrs
                        .iter()
                        .find(|(name, _)| *name == "METHOD")
                        .map(|(_, value)| *value)
                    {
                        Some("NONE") if uri.is_none() && tag != "#EXT-X-SESSION-KEY" => {}
                        Some("AES-128") if uri.is_some() => summary.encrypted = true,
                        Some("SAMPLE-AES") => return Err(HlsError::Unsupported),
                        _ => return Err(HlsError::Invalid),
                    }
                    if attrs
                        .iter()
                        .any(|(name, value)| *name == "KEYFORMAT" && *value != "identity")
                    {
                        return Err(HlsError::Unsupported);
                    }
                }
                if tag == "#EXT-X-I-FRAME-STREAM-INF" {
                    bandwidth(&attrs)?;
                    if uri.is_none() {
                        return Err(HlsError::Invalid);
                    }
                    summary.variants += 1;
                }
                if tag == "#EXT-X-MAP" && uri.is_none() {
                    return Err(HlsError::Invalid);
                }
                if let Some(uri) = uri {
                    add_resource(&base, uri, &mut resources)?;
                }
            }
            "#EXT-X-TARGETDURATION" => {
                let duration: u32 = integer(value)?.try_into().map_err(|_| HlsError::Invalid)?;
                if duration == 0 || target_duration.replace(duration).is_some() {
                    return Err(HlsError::Invalid);
                }
            }
            "#EXT-X-ENDLIST" => summary.end_list = true,
            "#EXT-X-VERSION" => {
                let version = integer(value)?;
                if version == 0 || std::mem::replace(&mut version_seen, true) {
                    return Err(HlsError::Invalid);
                }
            }
            "#EXT-X-MEDIA-SEQUENCE" | "#EXT-X-DISCONTINUITY-SEQUENCE" => {
                integer(value)?;
            }
            "#EXT-X-PLAYLIST-TYPE" if !matches!(value, "VOD" | "EVENT") => {
                return Err(HlsError::Invalid);
            }
            "#EXT-X-BYTERANGE" => {
                byte_range(value)?;
            }
            "#EXT-X-PROGRAM-DATE-TIME" | "#EXT-X-DATERANGE" => return Err(HlsError::Unsupported),
            "#EXT-X-PLAYLIST-TYPE"
            | "#EXT-X-DISCONTINUITY"
            | "#EXT-X-INDEPENDENT-SEGMENTS"
            | "#EXT-X-I-FRAMES-ONLY" => {}
            "#EXTM3U" => return Err(HlsError::Invalid),
            _ if tag.starts_with("#EXT-X-") => return Err(HlsError::Unsupported),
            _ => {}
        }
    }
    if pending.is_some() || summary.variants > 0 && summary.segments > 0 {
        return Err(HlsError::Invalid);
    }
    if summary.variants > 0 {
        summary.kind = HlsPlaylistKind::Master;
    } else if summary.segments == 0
        || target_duration.is_none_or(|target| max_duration.round() > f64::from(target))
    {
        return Err(HlsError::Invalid);
    }
    summary.resource_references = resources.len() as u32;
    Ok(VerifiedHlsManifest {
        source_url: base_url.into(),
        summary,
        resources,
    })
}
fn bandwidth(attrs: &[(&str, &str)]) -> Result<(), HlsError> {
    if let Some((_, average)) = attrs.iter().find(|(name, _)| *name == "AVERAGE-BANDWIDTH")
        && integer(average)? == 0
    {
        return Err(HlsError::Invalid);
    }
    if attrs
        .iter()
        .find(|(name, _)| *name == "BANDWIDTH")
        .and_then(|(_, value)| integer(value).ok())
        .is_none_or(|value| value == 0)
    {
        return Err(HlsError::Invalid);
    }
    Ok(())
}
fn byte_range(value: &str) -> Result<(), HlsError> {
    let (length, offset) = value
        .split_once('@')
        .map_or((value, None), |(length, offset)| (length, Some(offset)));
    if integer(length)? == 0 {
        return Err(HlsError::Invalid);
    }
    if let Some(offset) = offset {
        integer(offset)?;
    }
    Ok(())
}
fn integer(value: &str) -> Result<u64, HlsError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(HlsError::Invalid);
    }
    value.parse().map_err(|_| HlsError::Invalid)
}
fn add_resource(base: &Url, value: &str, output: &mut Vec<String>) -> Result<(), HlsError> {
    if output.len() >= 4096 {
        return Err(HlsError::Limit);
    }
    if value.is_empty() || value.len() > 8192 || value.chars().any(char::is_control) {
        return Err(HlsError::Invalid);
    }
    let url = base.join(value).map_err(|_| HlsError::Invalid)?;
    if url.as_str().len() > 8192 {
        return Err(HlsError::Limit);
    }
    if !media_transport_allowed(url.as_str()) {
        return Err(HlsError::UnsafeTransport);
    }
    output.push(url.into());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bom_duplicate_version_mixed_tags_and_wrong_attribute_types_are_rejected() {
        for input in [
            "\u{feff}#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1\na.txt",
            "#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-VERSION:3\n#EXT-X-STREAM-INF:BANDWIDTH=1\na.txt",
            "#EXTM3U\n#EXT-X-TARGETDURATION:5\n#EXT-X-STREAM-INF:BANDWIDTH=1\na.txt",
            "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=\"1\"\na.txt",
            "#EXTM3U\n#EXT-X-I-FRAME-STREAM-INF:BANDWIDTH=1,URI=a.txt",
            "#EXTM3U\n#EXT-X-TARGETDURATION:5\n#EXTINF:1e0,\na.ts",
            "#EXTM3U\n#EXT-X-TARGETDURATION:5\n#EXTINF:1,\na.ts\n#EXT-X-ENDLIST:unexpected",
        ] {
            assert!(verify("https://cdn.example/master.txt", input.as_bytes()).is_err());
        }
    }
    #[test]
    fn misleading_extension_is_validated_from_complete_structure_and_secure_references() {
        let media = verify(
            "https://cdn.example/master.txt",
            b"#EXTM3U\n#EXT-X-TARGETDURATION:5\n#EXTINF:4.2,\nsegment.ts\n#EXT-X-ENDLIST\n",
        )
        .unwrap();
        assert_eq!(media.summary().kind, HlsPlaylistKind::Media);
        assert_eq!(media.summary().segments, 1);
        assert_eq!(media.resources(), &["https://cdn.example/segment.ts"]);
        let master = verify(
            "https://cdn.example/master.txt",
            b"#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1000,CODECS=\"a,b\"\nvariant.txt\n",
        )
        .unwrap();
        assert_eq!(master.summary().kind, HlsPlaylistKind::Master);
        assert_eq!(master.summary().variants, 1);
    }
    #[test]
    fn prefix_only_html_missing_uri_and_privileged_references_are_rejected() {
        for bytes in [
            b"#EXTM3U\n#EXT-X-VERSION:3\n".as_slice(),
            b"<html>#EXTM3U</html>",
            b"#EXTM3Uevil\n#EXT-X-VERSION:3",
            b"#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1\n",
            b"#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1\nfile:///private",
        ] {
            assert!(verify("https://cdn.example/master.txt", bytes).is_err());
        }
        assert_eq!(verify("https://cdn.example/master.txt", b"#EXTM3U\n#EXT-X-TARGETDURATION:5\n#EXT-X-KEY:METHOD=AES-128,URI=\"file:///private\"\n#EXTINF:4,\nsegment.ts").unwrap_err(), HlsError::UnsafeTransport);
    }
}
