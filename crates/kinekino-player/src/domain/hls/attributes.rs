use super::HlsError;

pub(super) fn parse(value: &str) -> Result<Vec<(&str, &str)>, HlsError> {
    let mut output = Vec::new();
    let mut quoted = false;
    let mut start = 0;
    for (offset, byte) in value.bytes().chain(std::iter::once(b',')).enumerate() {
        if byte == b'"' {
            quoted = !quoted;
        }
        if byte != b',' || quoted {
            continue;
        }
        let (name, value) = value[start..offset]
            .split_once('=')
            .ok_or(HlsError::Invalid)?;
        if name.is_empty()
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-')
            || value.is_empty()
            || output.iter().any(|(known, _)| *known == name)
        {
            return Err(HlsError::Invalid);
        }
        let is_quoted = value.starts_with('"');
        if matches!(
            name,
            "URI"
                | "CODECS"
                | "NAME"
                | "GROUP-ID"
                | "LANGUAGE"
                | "ASSOC-LANGUAGE"
                | "INSTREAM-ID"
                | "CHARACTERISTICS"
                | "CHANNELS"
                | "KEYFORMAT"
                | "KEYFORMATVERSIONS"
                | "DATA-ID"
                | "VALUE"
                | "BYTERANGE"
        ) && !is_quoted
        {
            return Err(HlsError::Invalid);
        }
        if matches!(
            name,
            "BANDWIDTH"
                | "AVERAGE-BANDWIDTH"
                | "METHOD"
                | "TYPE"
                | "DEFAULT"
                | "AUTOSELECT"
                | "FORCED"
                | "PRECISE"
                | "IV"
                | "RESOLUTION"
                | "FRAME-RATE"
                | "TIME-OFFSET"
        ) && is_quoted
        {
            return Err(HlsError::Invalid);
        }
        if !is_quoted
            && value
                .bytes()
                .any(|byte| !byte.is_ascii_alphanumeric() && !matches!(byte, b'-' | b'_' | b'.'))
        {
            return Err(HlsError::Invalid);
        }
        let value = if value.starts_with('"') {
            value
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .ok_or(HlsError::Invalid)?
        } else {
            value
        };
        if value.contains('"') {
            return Err(HlsError::Invalid);
        }
        output.push((name, value));
        start = offset + 1;
    }
    if quoted || start != value.len() + 1 {
        return Err(HlsError::Invalid);
    }
    Ok(output)
}
