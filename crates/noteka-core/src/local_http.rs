//! Cliente HTTP/1.1 mínimo para runtimes locales. Sólo loopback, sin TLS,
//! sin proxies, sin redirecciones y con topes explícitos.
use crate::{error::*, semantic_contracts::MAX_ENDPOINT_BYTES};
use serde_json::Value;
use std::{
    io::{Read, Write},
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, TcpStream},
    time::{Duration, Instant},
};

const MAX_HEADER_BYTES: usize = 64 * 1024;
const MAX_REQUEST_BYTES: usize = 8 * 1024 * 1024;

fn fail(reason: &str) -> CoreError {
    let mut e = CoreError::new(
        ErrorCode::StorageError,
        "El runtime local no respondió con un contrato HTTP válido.",
    );
    e.details = serde_json::json!({"reason":reason});
    e
}

pub(crate) fn loopback_target(url: &str) -> Result<(SocketAddr, String, String)> {
    if url.len() > MAX_ENDPOINT_BYTES {
        return Err(fail("endpoint_size_limit"));
    }
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| fail("endpoint_requires_http_loopback"))?;
    if rest.contains('@') || rest.contains('#') || rest.contains('?') {
        return Err(fail("endpoint_url_not_allowed"));
    }
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    if path.is_empty() || !path.starts_with('/') || path.len() > 1024 {
        return Err(fail("endpoint_path_invalid"));
    }
    let (v4, v6, port) = if let Some(after) = authority.strip_prefix('[') {
        let end = after
            .find(']')
            .ok_or_else(|| fail("endpoint_ipv6_invalid"))?;
        let host = &after[..end];
        if host != "::1" {
            return Err(fail("endpoint_must_be_loopback"));
        }
        let tail = &after[end + 1..];
        let port = if tail.is_empty() {
            80
        } else {
            tail.strip_prefix(':')
                .ok_or_else(|| fail("endpoint_port_invalid"))?
                .parse::<u16>()
                .map_err(|_| fail("endpoint_port_invalid"))?
        };
        (None, Some(Ipv6Addr::LOCALHOST), port)
    } else {
        let (host, port) = match authority.rsplit_once(':') {
            Some((h, p)) if !h.contains(':') => (
                h,
                p.parse::<u16>()
                    .map_err(|_| fail("endpoint_port_invalid"))?,
            ),
            Some(_) => return Err(fail("endpoint_ipv6_requires_brackets")),
            None => (authority, 80),
        };
        if host != "127.0.0.1" {
            return Err(fail("endpoint_must_be_loopback"));
        }
        (Some(Ipv4Addr::LOCALHOST), None, port)
    };
    if port == 0 {
        return Err(fail("endpoint_port_invalid"));
    }
    let (addr, host_header): (SocketAddr, String) = match (v4, v6) {
        (Some(v4), _) => (
            SocketAddr::V4(SocketAddrV4::new(v4, port)),
            format!("127.0.0.1:{port}"),
        ),
        (_, Some(v6)) => (
            SocketAddr::V6(SocketAddrV6::new(v6, port, 0, 0)),
            format!("[::1]:{port}"),
        ),
        _ => return Err(fail("endpoint_must_be_loopback")),
    };
    Ok((addr, host_header, path.to_string()))
}

pub(crate) fn post_json(
    url: &str,
    body: &Value,
    max_response: usize,
    timeout: Duration,
) -> Result<Value> {
    let (addr, host, path) = loopback_target(url)?;
    let payload = serde_json::to_vec(body)?;
    if payload.len() > MAX_REQUEST_BYTES {
        return Err(fail("provider_request_size_limit"));
    }
    let mut stream =
        TcpStream::connect_timeout(&addr, timeout).map_err(|_| fail("provider_connect_failed"))?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|_| fail("provider_timeout"))?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|_| fail("provider_timeout"))?;
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nAccept: application/json\r\nConnection: close\r\nContent-Length: {}\r\n\r\n",
        payload.len()
    );
    let started = Instant::now();
    stream
        .write_all(request.as_bytes())
        .and_then(|_| stream.write_all(&payload))
        .and_then(|_| stream.flush())
        .map_err(|_| fail("provider_write_failed"))?;
    let mut raw = Vec::new();
    let mut buf = [0u8; 16 * 1024];
    loop {
        if started.elapsed() > timeout {
            return Err(fail("provider_timeout"));
        }
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                raw.extend_from_slice(&buf[..n]);
                if raw.len() > max_response + MAX_HEADER_BYTES {
                    return Err(fail("provider_response_size_limit"));
                }
                if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&raw[..pos]);
                    let length = head.lines().find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().ok())
                            .flatten()
                    });
                    if length.is_some_and(|length| raw.len() >= pos + 4 + length) {
                        break;
                    }
                }
            }
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                return Err(fail("provider_timeout"));
            }
            Err(e) if e.kind() == std::io::ErrorKind::ConnectionReset && !raw.is_empty() => {
                break;
            }
            Err(_) => return Err(fail("provider_read_failed")),
        }
    }
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| fail("provider_http_invalid"))?;
    let head = std::str::from_utf8(&raw[..split]).map_err(|_| fail("provider_http_invalid"))?;
    let mut lines = head.split("\r\n");
    let status = lines
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .ok_or_else(|| fail("provider_http_invalid"))?;
    let mut chunked = false;
    let mut length = None;
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            let name = name.trim().to_ascii_lowercase();
            let value = value.trim();
            if name == "transfer-encoding" && value.to_ascii_lowercase().contains("chunked") {
                chunked = true;
            } else if name == "content-length" {
                length = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| fail("provider_http_invalid"))?,
                );
            }
        }
    }
    if !(200..300).contains(&status) {
        let mut e = fail("provider_http_status");
        e.details["status"] = serde_json::json!(status);
        return Err(e);
    }
    let body = &raw[split + 4..];
    let body = if chunked {
        decode_chunked(body, max_response)?
    } else {
        if let Some(n) = length {
            if n > max_response || body.len() < n {
                return Err(fail("provider_response_size_limit"));
            }
            body[..n].to_vec()
        } else {
            body.to_vec()
        }
    };
    if body.len() > max_response {
        return Err(fail("provider_response_size_limit"));
    }
    serde_json::from_slice(&body).map_err(|_| fail("provider_json_invalid"))
}
fn decode_chunked(mut body: &[u8], max: usize) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    loop {
        let line_end = body
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or_else(|| fail("provider_chunk_invalid"))?;
        let line =
            std::str::from_utf8(&body[..line_end]).map_err(|_| fail("provider_chunk_invalid"))?;
        let size_text = line.split(';').next().unwrap_or("").trim();
        let size =
            usize::from_str_radix(size_text, 16).map_err(|_| fail("provider_chunk_invalid"))?;
        body = &body[line_end + 2..];
        if size == 0 {
            return Ok(out);
        }
        if body.len() < size + 2 || out.len().saturating_add(size) > max {
            return Err(fail("provider_response_size_limit"));
        }
        out.extend_from_slice(&body[..size]);
        if &body[size..size + 2] != b"\r\n" {
            return Err(fail("provider_chunk_invalid"));
        }
        body = &body[size + 2..];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_loopback_urls_are_accepted() {
        assert!(loopback_target("http://127.0.0.1:1234/v1/embeddings").is_ok());
        assert!(loopback_target("http://[::1]:1234/v1/embeddings").is_ok());
        for url in [
            "https://127.0.0.1:1234/v1/embeddings",
            "http://localhost:1234/v1/embeddings",
            "http://192.168.1.2:1234/v1/embeddings",
            "http://user@127.0.0.1:1234/v1/embeddings",
            "http://127.0.0.1:1234/v1/embeddings?x=1",
            "http://127.0.0.1:0/v1/embeddings",
        ] {
            assert!(loopback_target(url).is_err(), "{url}");
        }
    }
}
