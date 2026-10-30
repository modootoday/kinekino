use crate::application::api::{ApiError, ErrorCode};
use std::collections::BTreeSet;
#[derive(Clone, Copy)]
pub(super) enum Table {
    Encoders,
    Filters,
    Muxers,
}
pub(super) fn names(bytes: &[u8], table: Table) -> Result<BTreeSet<String>, ApiError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| ApiError::new(ErrorCode::NeedsInput, "Encoder inventory is not UTF-8"))?;
    let mut names = BTreeSet::new();
    for line in text.lines() {
        let mut fields = line.split_whitespace();
        let Some(flags) = fields.next() else { continue };
        let Some(name) = fields.next() else { continue };
        let row = match table {
            Table::Encoders => {
                flags.len() == 6
                    && flags
                        .bytes()
                        .next()
                        .is_some_and(|flag| matches!(flag, b'V' | b'A' | b'S'))
                    && flags
                        .bytes()
                        .all(|flag| flag.is_ascii_uppercase() || flag == b'.')
            }
            Table::Filters => {
                matches!(flags.len(), 2 | 3)
                    && flags
                        .bytes()
                        .all(|flag| matches!(flag, b'T' | b'S' | b'C' | b'.'))
            }
            Table::Muxers => {
                flags.contains('E')
                    && flags.len() <= 2
                    && flags.bytes().all(|flag| matches!(flag, b'D' | b'E'))
            }
        };
        if !row || name == "=" {
            continue;
        }
        for name in name.split(',') {
            if name.is_empty()
                || name.len() > 128
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
            {
                return Err(ApiError::new(
                    ErrorCode::NeedsInput,
                    "Invalid encoder feature name",
                ));
            }
            names.insert(name.into());
        }
    }
    if names.is_empty() {
        return Err(ApiError::new(
            ErrorCode::NeedsInput,
            "Encoder feature table has no recognized entries",
        ));
    }
    Ok(names)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn table_flags_names_and_aliases_are_distinct_from_legends_and_descriptions() {
        let encoders = names(
            b"Encoders:\n V..... = Video\n V....D libx264 H264\n A....D aac AAC\n",
            Table::Encoders,
        )
        .unwrap();
        assert_eq!(encoders, BTreeSet::from(["aac".into(), "libx264".into()]));
        assert_eq!(
            names(b" TSC crop V->V\n ... subtitles V->V\n", Table::Filters)
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            names(
                b" T.. = Timeline support\n TS crop V->V\n .. subtitles V->V\n",
                Table::Filters
            )
            .unwrap(),
            BTreeSet::from(["crop".into(), "subtitles".into()])
        );
        assert_eq!(
            names(
                b" E mp4,mov MP4\n D input Demuxer\n E = Muxing\n",
                Table::Muxers
            )
            .unwrap(),
            BTreeSet::from(["mov".into(), "mp4".into()])
        );
        assert!(names(b" E bad/name description\n", Table::Muxers).is_err());
        assert!(names(b"Header only", Table::Encoders).is_err());
    }
}
