//! Proxy loopback OpenAI chat-completions sobre una sesión ChatGPT.
//! El núcleo/harness sólo ve un endpoint local; el token queda en Keychain.
use crate::{CredentialStore, ProviderError, openai};
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    time::Duration,
};

pub fn serve(store: &dyn CredentialStore, port: u16) -> Result<(), ProviderError> {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|e| ProviderError::new("proxy_error", format!("No se pudo escuchar: {e}")))?;
    eprintln!("Proxy ChatGPT en http://127.0.0.1:{port}/v1/chat/completions");
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(error) = handle(store, &mut stream) {
                    let _ = respond(
                        &mut stream,
                        500,
                        &json!({"error":{"message":error.message}}),
                    );
                }
            }
            Err(_) => continue,
        }
    }
    Ok(())
}
fn handle(store: &dyn CredentialStore, stream: &mut TcpStream) -> Result<(), ProviderError> {
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
    let mut data = Vec::new();
    let mut buffer = [0u8; 8192];
    let mut length = None;
    loop {
        let n = stream
            .read(&mut buffer)
            .map_err(|e| ProviderError::new("proxy_error", format!("{e}")))?;
        if n == 0 {
            break;
        }
        data.extend_from_slice(&buffer[..n]);
        if length.is_none()
            && let Some(pos) = data.windows(4).position(|w| w == b"\r\n\r\n")
        {
            let head = String::from_utf8_lossy(&data[..pos]);
            length = head.lines().find_map(|line| {
                let (k, v) = line.split_once(':')?;
                k.eq_ignore_ascii_case("content-length")
                    .then(|| v.trim().parse::<usize>().ok())
                    .flatten()
            });
            if let Some(n) = length
                && data.len() >= pos + 4 + n
            {
                break;
            }
        }
        if data.len() > 2 * 1024 * 1024 {
            return Err(ProviderError::validation("request demasiado grande"));
        }
    }
    let split = data
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| ProviderError::validation("HTTP inválido"))?;
    let head = String::from_utf8_lossy(&data[..split]);
    let mut lines = head.lines();
    let request = lines.next().unwrap_or("");
    let path = request.split_whitespace().nth(1).unwrap_or("");
    if path != "/v1/chat/completions" {
        return respond(
            stream,
            404,
            &json!({"error":{"message":"ruta no soportada"}}),
        );
    }
    let body = &data[split + 4..];
    let body: Value =
        serde_json::from_slice(body).map_err(|_| ProviderError::validation("JSON inválido"))?;
    let model = body["model"].as_str().unwrap_or_default();
    if model.is_empty() || model.len() > 160 {
        return respond(stream, 400, &json!({"error":{"message":"model requerido"}}));
    }
    let prompt = body["messages"]
        .as_array()
        .map(|messages| {
            messages
                .iter()
                .filter_map(|m| {
                    let role = m["role"].as_str().unwrap_or("user");
                    let content = m["content"].as_str().unwrap_or("");
                    (!content.is_empty()).then(|| format!("{role}: {content}"))
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    if prompt.is_empty() {
        return respond(
            stream,
            400,
            &json!({"error":{"message":"messages requerido"}}),
        );
    }
    let max = body["max_tokens"].as_u64().unwrap_or(512).clamp(1, 4096) as u32;
    let text = openai::chat(store, model, &prompt, max)?;
    respond(
        stream,
        200,
        &json!({"choices":[{"message":{"role":"assistant","content":text}}]}),
    )
}
fn respond(stream: &mut TcpStream, status: u16, value: &Value) -> Result<(), ProviderError> {
    let body = value.to_string();
    write!(
        stream,
        "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    ).map_err(|e| ProviderError::new("proxy_error", format!("{e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MemoryStore;
    use std::net::TcpListener;
    use std::thread;

    fn request(raw: &str) -> String {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            handle(&MemoryStore::default(), &mut stream).unwrap();
        });
        let mut client = TcpStream::connect(address).unwrap();
        client.write_all(raw.as_bytes()).unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        server.join().unwrap();
        response
    }

    #[test]
    fn rejects_unknown_paths_and_missing_models_without_credentials() {
        let path = request("POST /otra HTTP/1.1\r\nContent-Length: 2\r\n\r\n{}");
        assert!(path.starts_with("HTTP/1.1 404"));
        let model = request(
            "POST /v1/chat/completions HTTP/1.1\r\nContent-Length: 14\r\n\r\n{\"messages\":[]}",
        );
        assert!(model.starts_with("HTTP/1.1 400"));
        assert!(!path.contains("secret"));
        assert!(!model.contains("secret"));
    }
}
