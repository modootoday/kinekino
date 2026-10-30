use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub enum Value {
    Text(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
    Literal(String),
}
impl Value {
    pub fn array(&self) -> Result<&[Value], String> {
        if let Self::Array(v) = self {
            Ok(v)
        } else {
            Err("array required".into())
        }
    }
    pub fn field(&self, key: &str) -> Result<&Value, String> {
        if let Self::Object(v) = self {
            v.get(key).ok_or_else(|| format!("missing {key}"))
        } else {
            Err("object required".into())
        }
    }
    pub fn text(&self) -> Result<&str, String> {
        if let Self::Text(v) = self {
            Ok(v)
        } else {
            Err("text required".into())
        }
    }
}
pub fn parse(bytes: &[u8]) -> Result<Value, String> {
    parse_mode(bytes, false)
}
pub fn parse_observation(bytes: &[u8]) -> Result<Value, String> {
    parse_mode(bytes, true)
}
fn parse_mode(bytes: &[u8], signed_observation: bool) -> Result<Value, String> {
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("JSON too large".into());
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|e| e.to_string())?
        .trim_start_matches('\u{feff}');
    let mut reader = Reader {
        bytes: text.as_bytes(),
        at: 0,
        signed_observation,
    };
    let value = reader.value(0)?;
    reader.space();
    if reader.at != reader.bytes.len() {
        return Err("trailing JSON".into());
    }
    Ok(value)
}
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
    signed_observation: bool,
}
impl Reader<'_> {
    fn space(&mut self) {
        while self
            .bytes
            .get(self.at)
            .is_some_and(|b| b" \r\n\t".contains(b))
        {
            self.at += 1;
        }
    }
    fn take(&mut self, byte: u8) -> Result<(), String> {
        self.space();
        if self.bytes.get(self.at) != Some(&byte) {
            return Err(format!("expected byte {byte} at {}", self.at));
        }
        self.at += 1;
        Ok(())
    }
    fn hex4(&mut self) -> Result<u16, String> {
        let raw = self
            .bytes
            .get(self.at..self.at + 4)
            .ok_or("short Unicode escape")?;
        let text = std::str::from_utf8(raw).map_err(|e| e.to_string())?;
        let value = u16::from_str_radix(text, 16).map_err(|e| e.to_string())?;
        self.at += 4;
        Ok(value)
    }
    fn string(&mut self) -> Result<String, String> {
        self.take(b'"')?;
        let mut output = Vec::new();
        loop {
            let byte = *self.bytes.get(self.at).ok_or("unclosed string")?;
            self.at += 1;
            match byte {
                b'"' => return String::from_utf8(output).map_err(|e| e.to_string()),
                0..=31 => return Err("unescaped control".into()),
                b'\\' => {
                    let escaped = *self.bytes.get(self.at).ok_or("short escape")?;
                    self.at += 1;
                    match escaped {
                        b'"' | b'\\' | b'/' => output.push(escaped),
                        b'b' => output.push(8),
                        b'f' => output.push(12),
                        b'n' => output.push(10),
                        b'r' => output.push(13),
                        b't' => output.push(9),
                        b'u' => {
                            let high = self.hex4()?;
                            let scalar = if (0xd800..=0xdbff).contains(&high) {
                                if self.bytes.get(self.at..self.at + 2) != Some(b"\\u") {
                                    return Err("unpaired surrogate".into());
                                }
                                self.at += 2;
                                let low = self.hex4()?;
                                if !(0xdc00..=0xdfff).contains(&low) {
                                    return Err("invalid low surrogate".into());
                                }
                                0x10000 + ((u32::from(high) - 0xd800) << 10) + u32::from(low)
                                    - 0xdc00
                            } else {
                                u32::from(high)
                            };
                            let c = char::from_u32(scalar).ok_or("invalid Unicode scalar")?;
                            let mut buffer = [0; 4];
                            output.extend_from_slice(c.encode_utf8(&mut buffer).as_bytes());
                        }
                        _ => return Err("invalid escape".into()),
                    }
                }
                _ => output.push(byte),
            }
        }
    }
    fn value(&mut self, depth: usize) -> Result<Value, String> {
        if depth > 32 {
            return Err("JSON depth".into());
        }
        self.space();
        match self.bytes.get(self.at).copied().ok_or("missing value")? {
            b'"' => Ok(Value::Text(self.string()?)),
            b'[' => {
                self.at += 1;
                let mut rows = Vec::new();
                self.space();
                if self.bytes.get(self.at) == Some(&b']') {
                    self.at += 1;
                    return Ok(Value::Array(rows));
                }
                loop {
                    rows.push(self.value(depth + 1)?);
                    self.space();
                    if self.bytes.get(self.at) == Some(&b']') {
                        self.at += 1;
                        break;
                    }
                    self.take(b',')?;
                }
                Ok(Value::Array(rows))
            }
            b'{' => {
                self.at += 1;
                let mut rows = BTreeMap::new();
                self.space();
                if self.bytes.get(self.at) == Some(&b'}') {
                    self.at += 1;
                    return Ok(Value::Object(rows));
                }
                loop {
                    let key = self.string()?;
                    self.take(b':')?;
                    let value = self.value(depth + 1)?;
                    if rows.insert(key, value).is_some() {
                        return Err("duplicate key".into());
                    }
                    self.space();
                    if self.bytes.get(self.at) == Some(&b'}') {
                        self.at += 1;
                        break;
                    }
                    self.take(b',')?;
                }
                Ok(Value::Object(rows))
            }
            _ => {
                let start = self.at;
                while self
                    .bytes
                    .get(self.at)
                    .is_some_and(|b| !b" \r\n\t,]}".contains(b))
                {
                    self.at += 1;
                }
                let raw =
                    std::str::from_utf8(&self.bytes[start..self.at]).map_err(|e| e.to_string())?;
                // Frozen manifests remain nonnegative. Signed integers are opt-in for observations.
                let digits = if self.signed_observation {
                    raw.strip_prefix('-').unwrap_or(raw)
                } else {
                    raw
                };
                if !matches!(raw, "true" | "false" | "null")
                    && (digits.is_empty()
                        || !digits.bytes().all(|b| b.is_ascii_digit())
                        || digits.len() > 1 && digits.starts_with('0'))
                {
                    return Err("unsupported or invalid literal".into());
                }
                Ok(Value::Literal(raw.into()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signed_observation_does_not_relax_frozen_manifest_literals() {
        assert!(parse(b"-1").is_err());
        assert_eq!(
            parse_observation(b"-1").unwrap(),
            Value::Literal("-1".into())
        );
        for invalid in ["-01", "--1", "+1", "1.2", "-", "NaN"] {
            assert!(parse_observation(invalid.as_bytes()).is_err());
        }
    }
    #[test]
    fn frozen_shapes_unicode_and_escapes() {
        let value = parse(
            br#"[{"source":"D:\\x","ignored":false,"features":["window"],"text":"\uD83D\uDE00"}]"#,
        )
        .unwrap();
        assert_eq!(
            value.array().unwrap()[0]
                .field("text")
                .unwrap()
                .text()
                .unwrap(),
            "😀"
        );
    }
    #[test]
    fn malformed_and_duplicate_rejected() {
        for input in [
            "{\"a\":1,\"a\":2}",
            "[1,]",
            "{\"a\":\"\\uD800\"}",
            "true false",
            "01",
            "NaN",
            "\"bad\ntext\"",
        ] {
            assert!(parse(input.as_bytes()).is_err(), "{input}");
        }
    }
}
