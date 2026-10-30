//! CLI 전송 기한. SDK 작업 timeout 및 자동 재시도와 분리한다.
use super::*;
pub(super) fn from_args(args: &[String]) -> Result<std::time::Duration, ApiError> {
    let value = args
        .windows(2)
        .find(|pair| pair[0] == "--transport-timeout-ms")
        .map(|pair| pair[1].as_str());
    if value.is_some() && !args.iter().any(|arg| arg == "--endpoint") {
        return Err(invalid("Transport timeout requires --endpoint"));
    }
    let millis = match value {
        None => kinekino_player::application::local_host::DEFAULT_TIMEOUT_MS,
        Some(value) => {
            if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(invalid("Invalid transport timeout"));
            }
            let millis: u64 = value
                .parse()
                .map_err(|_| invalid("Invalid transport timeout"))?;
            if !(10..=30_000).contains(&millis) {
                return Err(invalid("Transport timeout must be 10..30000 milliseconds"));
            }
            millis
        }
    };
    Ok(std::time::Duration::from_millis(millis))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn args(value: &str) -> Vec<String> {
        [
            "version",
            "--endpoint",
            "explicit",
            "--transport-timeout-ms",
            value,
        ]
        .map(str::to_owned)
        .to_vec()
    }
    #[test]
    fn explicit_transport_budget_is_bounded_and_does_not_change_request_timeout() {
        for millis in ["10", "30000"] {
            let args = args(millis);
            assert_eq!(from_args(&args).unwrap().as_millis().to_string(), millis);
            assert_eq!(parse(&args).unwrap().context.timeout_ms, None);
        }
        assert_eq!(from_args(&[]).unwrap().as_millis(), 5000);
        for value in [
            "0",
            "9",
            "30001",
            "-1",
            "+10",
            "1e3",
            "18446744073709551616",
        ] {
            assert!(parse(&args(value)).is_err(), "{value}");
        }
        assert!(parse(&["version", "--transport-timeout-ms", "10000"].map(str::to_owned)).is_err());
        let mut duplicate = args("10000");
        duplicate.extend(["--transport-timeout-ms", "20000"].map(str::to_owned));
        assert!(parse(&duplicate).is_err());
    }
}
