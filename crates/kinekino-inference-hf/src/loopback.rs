//! Minimal blocking HTTP/1.1 client restricted to `127.0.0.1`/`localhost` (mock servers).
use crate::port::{HttpRequest, HttpResponse, HttpTransport, TransportError};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

#[derive(Debug, Clone, Copy, Default)]
pub struct LoopbackHttp;

fn split_url(url: &str) -> Result<(String, String), TransportError> {
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| TransportError::Refused(format!("only http:// loopback: {url}")))?;
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let host = authority.rsplit_once(':').map_or(authority, |(h, _)| h);
    if !matches!(host, "127.0.0.1" | "localhost") {
        return Err(TransportError::Refused(format!("non-loopback host {host}")));
    }
    Ok((authority.to_string(), path.to_string()))
}

impl HttpTransport for LoopbackHttp {
    fn post_json(
        &self,
        req: &HttpRequest,
        connect_timeout: Duration,
        total_timeout: Duration,
    ) -> Result<HttpResponse, TransportError> {
        let (authority, path) = split_url(&req.url)?;
        let addr: SocketAddr = authority
            .to_socket_addrs()
            .map_err(|e| TransportError::Connect(e.to_string()))?
            .find(|a| a.ip().is_loopback())
            .ok_or_else(|| TransportError::Refused("no loopback address".into()))?;
        let mut s = TcpStream::connect_timeout(&addr, connect_timeout)
            .map_err(|e| TransportError::Connect(e.to_string()))?;
        let io = |e: std::io::Error| match e.kind() {
            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut => {
                TransportError::Timeout(e.to_string())
            }
            _ => TransportError::Connect(e.to_string()),
        };
        s.set_read_timeout(Some(total_timeout)).map_err(io)?;
        s.set_write_timeout(Some(total_timeout)).map_err(io)?;
        let mut head = format!(
            "POST {path} HTTP/1.1\r\nHost: {authority}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
            req.body.len()
        );
        if let Some(t) = &req.bearer {
            head.push_str(&format!("Authorization: Bearer {t}\r\n"));
        }
        head.push_str("\r\n");
        s.write_all(head.as_bytes()).map_err(io)?;
        s.write_all(&req.body).map_err(io)?;
        let mut raw = Vec::new();
        s.read_to_end(&mut raw).map_err(io)?;
        parse_response(&raw)
    }
}

fn parse_response(raw: &[u8]) -> Result<HttpResponse, TransportError> {
    let bad = || TransportError::Connect("malformed HTTP response".into());
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(bad)?;
    let head = std::str::from_utf8(&raw[..split]).map_err(|_| bad())?;
    let status = head
        .split_whitespace()
        .nth(1)
        .and_then(|c| c.parse().ok())
        .ok_or_else(bad)?;
    Ok(HttpResponse {
        status,
        body: raw[split + 4..].to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_non_loopback() {
        for url in [
            "https://router.huggingface.co/v1",
            "http://example.com/x",
            "http://10.0.0.1:80/",
        ] {
            let r = LoopbackHttp.post_json(
                &HttpRequest {
                    url: url.into(),
                    bearer: None,
                    body: vec![],
                },
                Duration::from_millis(10),
                Duration::from_millis(10),
            );
            assert!(matches!(r, Err(TransportError::Refused(_))), "{url}");
        }
    }
}
