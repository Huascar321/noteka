//! Proveedores locales/remotos opcionales: OAuth, credenciales y APIs.
//! No lo usa el núcleo offline; sólo la CLI de autenticación y el harness cuando
//! el usuario configura un proveedor explícitamente.
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

pub mod api;
pub mod google;
pub mod openai;
pub mod proxy;

#[derive(Debug)]
pub struct ProviderError {
    pub code: &'static str,
    pub message: String,
}
impl ProviderError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new("validation_error", message)
    }
}
pub type Result<T> = std::result::Result<T, ProviderError>;

pub fn config_dir() -> Result<PathBuf> {
    let dir = dirs::data_local_dir()
        .ok_or_else(|| ProviderError::new("storage_error", "Sin directorio de datos local"))?
        .join("Noteka")
        .join("providers");
    fs::create_dir_all(&dir).map_err(|e| {
        ProviderError::new("storage_error", format!("No se pudo crear {dir:?}: {e}"))
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
    }
    Ok(dir)
}

/// Identificador estable de este host para el registro dinámico de ChatGPT.
pub fn host_id() -> Result<String> {
    let path = config_dir()?.join("host-id");
    if let Ok(value) = fs::read_to_string(&path) {
        let value = value.trim().to_owned();
        if let Some(uuid) = value.strip_prefix("urn:uuid:")
            && Uuid::parse_str(uuid).is_ok()
        {
            return Ok(value);
        }
    }
    // El registro dinámico exige un identificador de host estable con forma URN UUID.
    let value = format!("urn:uuid:{}", Uuid::new_v4());
    write_private(&path, value.as_bytes())?;
    Ok(value)
}
fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    fs::write(path, bytes).map_err(|e| {
        ProviderError::new(
            "storage_error",
            format!("No se pudo escribir configuración: {e}"),
        )
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

pub trait CredentialStore: Send + Sync {
    fn get(&self, user: &str) -> Result<Option<String>>;
    fn set(&self, user: &str, value: &str) -> Result<()>;
    fn delete(&self, user: &str) -> Result<()>;
}
pub struct KeyringStore;
impl CredentialStore for KeyringStore {
    fn get(&self, user: &str) -> Result<Option<String>> {
        let entry = keyring::Entry::new("noteka", user)
            .map_err(|e| ProviderError::new("storage_error", format!("Keychain: {e}")))?;
        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(ProviderError::new(
                "storage_error",
                format!("Keychain: {e}"),
            )),
        }
    }
    fn set(&self, user: &str, value: &str) -> Result<()> {
        keyring::Entry::new("noteka", user)
            .map_err(|e| ProviderError::new("storage_error", format!("Keychain: {e}")))?
            .set_password(value)
            .map_err(|e| ProviderError::new("storage_error", format!("Keychain: {e}")))
    }
    fn delete(&self, user: &str) -> Result<()> {
        let entry = keyring::Entry::new("noteka", user)
            .map_err(|e| ProviderError::new("storage_error", format!("Keychain: {e}")))?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(ProviderError::new(
                "storage_error",
                format!("Keychain: {e}"),
            )),
        }
    }
}
#[derive(Default)]
pub struct MemoryStore(Mutex<HashMap<String, String>>);
impl CredentialStore for MemoryStore {
    fn get(&self, user: &str) -> Result<Option<String>> {
        Ok(self.0.lock().unwrap().get(user).cloned())
    }
    fn set(&self, user: &str, value: &str) -> Result<()> {
        self.0.lock().unwrap().insert(user.into(), value.into());
        Ok(())
    }
    fn delete(&self, user: &str) -> Result<()> {
        self.0.lock().unwrap().remove(user);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthSession {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub scopes: Vec<String>,
    pub expires_at: u64,
    pub client_id: String,
    pub subject: String,
    pub email: Option<String>,
}
impl OAuthSession {
    pub fn expired(&self) -> bool {
        now() + 60 >= self.expires_at
    }
}
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn b64url(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}
pub fn pkce_challenge(verifier: &str) -> String {
    b64url(&Sha256::digest(verifier.as_bytes()))
}
pub fn pkce_verifier() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}
pub fn random_state() -> String {
    Uuid::new_v4().simple().to_string()
}

#[derive(Debug, Clone)]
pub struct Callback {
    pub code: String,
    pub state: String,
    pub issued_client_id: Option<String>,
    pub picked_file_ids: Vec<String>,
}
/// Escucha el callback loopback con deadline. Sólo acepta 127.0.0.1 y la ruta exacta.
pub fn wait_callback(
    port: u16,
    path: &str,
    expected_state: &str,
    timeout: Duration,
) -> Result<Callback> {
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| {
        ProviderError::new(
            "callback_error",
            format!("No se pudo escuchar el callback: {e}"),
        )
    })?;
    listener
        .set_nonblocking(true)
        .map_err(|e| ProviderError::new("callback_error", format!("{e}")))?;
    let deadline = Instant::now() + timeout;
    loop {
        if Instant::now() >= deadline {
            return Err(ProviderError::new(
                "auth_timeout",
                "No se recibió el callback de autorización",
            ));
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                let request = read_request(&mut stream);
                let first = request.lines().next().unwrap_or("");
                let target = first.split_whitespace().nth(1).unwrap_or("");
                let query = target.split_once('?').map(|(_, q)| q).unwrap_or("");
                if !target.starts_with(path) {
                    respond(&mut stream, 404, "Not found");
                    continue;
                }
                let params = parse_query(query);
                if params.contains_key("error") {
                    respond(&mut stream, 400, "Autorización rechazada");
                    return Err(ProviderError::new(
                        "access_denied",
                        "El usuario rechazó la autorización",
                    ));
                }
                let state = params.get("state").cloned().unwrap_or_default();
                if state != expected_state {
                    respond(&mut stream, 400, "Estado inválido");
                    return Err(ProviderError::new(
                        "state_mismatch",
                        "El callback no coincide con el intento",
                    ));
                }
                let code = params
                    .get("code")
                    .cloned()
                    .ok_or_else(|| ProviderError::new("callback_error", "Falta el código"))?;
                respond(
                    &mut stream,
                    200,
                    "Noteka autorizado. Podés cerrar esta pestaña.",
                );
                return Ok(Callback {
                    code,
                    state,
                    issued_client_id: params.get("client_id").cloned(),
                    picked_file_ids: parse_picked_ids(
                        params.get("picked_file_ids").map(String::as_str),
                    ),
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(e) => return Err(ProviderError::new("callback_error", format!("{e}"))),
        }
    }
}
fn read_request(stream: &mut TcpStream) -> String {
    let mut data = Vec::new();
    let mut buffer = [0u8; 4096];
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                data.extend_from_slice(&buffer[..n]);
                if data.windows(4).any(|w| w == b"\r\n\r\n") || data.len() > 64 * 1024 {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    String::from_utf8_lossy(&data).into_owned()
}
fn respond(stream: &mut TcpStream, status: u16, message: &str) {
    let body = format!("<html><body><p>{message}</p></body></html>");
    let _ = write!(
        stream,
        "HTTP/1.1 {status} OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
}
pub fn parse_query(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .map(|(k, v)| (url_decode(k), url_decode(v)))
        .collect()
}
pub fn parse_picked_ids(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}
pub fn url_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                if let Ok(byte) = u8::from_str_radix(&value[i + 1..i + 3], 16) {
                    out.push(byte);
                    i += 3;
                } else {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            byte => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}
pub fn open_browser(url: &str) -> Result<()> {
    let (program, args): (&str, Vec<&str>) = if cfg!(target_os = "macos") {
        ("open", vec![url])
    } else if cfg!(target_os = "windows") {
        ("cmd", vec!["/C", "start", "", url])
    } else {
        ("xdg-open", vec![url])
    };
    std::process::Command::new(program)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| {
            ProviderError::new(
                "browser_error",
                format!("No se pudo abrir el navegador: {e}"),
            )
        })?;
    Ok(())
}

pub fn https_endpoint(override_value: Option<&str>, default: &str) -> Result<String> {
    let url = override_value.unwrap_or(default).trim_end_matches('/');
    if url.starts_with("https://") {
        return Ok(url.to_string());
    }
    if let Some(rest) = url.strip_prefix("http://127.0.0.1")
        && (rest.is_empty() || rest.starts_with(':'))
    {
        return Ok(url.to_string());
    }
    Err(ProviderError::validation(
        "El endpoint debe ser HTTPS (o loopback explícito para pruebas)",
    ))
}
pub fn post_form(url: &str, form: &[(&str, &str)], timeout: Duration) -> Result<serde_json::Value> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .http_status_as_error(false)
        .build()
        .new_agent();
    let mut response = agent
        .post(url)
        .send_form(form.iter().copied())
        .map_err(|e| ProviderError::new("auth_http", format!("Token endpoint: {e}")))?;
    if !response.status().is_success() {
        let body = response.body_mut().read_to_string().unwrap_or_default();
        return Err(ProviderError::new(
            "auth_http",
            format!("Token endpoint {}: {body}", response.status()),
        ));
    }
    response
        .body_mut()
        .read_json()
        .map_err(|e| ProviderError::new("auth_http", format!("Respuesta inválida: {e}")))
}
pub fn get_json(url: &str, bearer: Option<&str>, timeout: Duration) -> Result<serde_json::Value> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .http_status_as_error(false)
        .build()
        .new_agent();
    let mut request = agent.get(url);
    if let Some(token) = bearer {
        request = request.header("Authorization", &format!("Bearer {token}"));
    }
    let mut response = request
        .call()
        .map_err(|e| ProviderError::new("provider_http", format!("Proveedor: {e}")))?;
    if !response.status().is_success() {
        let body = response.body_mut().read_to_string().unwrap_or_default();
        return Err(http_status_error(response.status().as_u16(), &body));
    }
    response
        .body_mut()
        .read_json()
        .map_err(|e| ProviderError::new("provider_http", format!("Respuesta inválida: {e}")))
}
pub fn post_stream_text(
    url: &str,
    bearer: Option<&str>,
    body: &serde_json::Value,
    timeout: Duration,
) -> Result<String> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .http_status_as_error(false)
        .build()
        .new_agent();
    let mut request = agent.post(url);
    if let Some(token) = bearer {
        request = request.header("Authorization", &format!("Bearer {token}"));
    }
    let mut response = request
        .send_json(body)
        .map_err(|e| ProviderError::new("provider_http", format!("Proveedor: {e}")))?;
    if !response.status().is_success() {
        let body = response.body_mut().read_to_string().unwrap_or_default();
        return Err(ProviderError::new(
            "provider_http",
            format!("Proveedor {}: {}", response.status(), body),
        ));
    }
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned();
    let raw = response
        .body_mut()
        .read_to_string()
        .map_err(|e| ProviderError::new("provider_http", format!("Respuesta inválida: {e}")))?;
    let trimmed = raw.trim_start();
    if content_type.contains("text/event-stream")
        || trimmed.starts_with("data:")
        || trimmed.starts_with("event:")
        || raw.contains("\ndata:")
    {
        return parse_sse_text(&raw);
    }
    let value: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| ProviderError::new("provider_http", format!("Respuesta inválida: {e}")))?;
    crate::openai::response_text(&value)
        .ok_or_else(|| ProviderError::new("provider_http", "Respuesta sin texto"))
}
fn parse_sse_text(raw: &str) -> Result<String> {
    let mut text = String::new();
    let mut completed = None;
    for line in raw.lines() {
        let Some(data) = line.strip_prefix("data:") else {
            continue;
        };
        let data = data.trim();
        if data.is_empty() || data == "[DONE]" {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(data)
            .map_err(|e| ProviderError::new("provider_http", format!("SSE inválido: {e}")))?;
        match value.get("type").and_then(|v| v.as_str()) {
            Some("response.output_text.delta") => {
                if let Some(delta) = value.get("delta").and_then(|v| v.as_str()) {
                    text.push_str(delta);
                }
            }
            Some("response.completed") => {
                completed = value.get("response").cloned();
            }
            Some("error") => {
                let message = value
                    .get("error")
                    .and_then(|v| v.get("message"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("error del proveedor");
                return Err(ProviderError::new("provider_http", message.to_owned()));
            }
            _ => {}
        }
    }
    if text.is_empty()
        && let Some(response) = completed
        && let Some(done) = crate::openai::response_text(&response)
    {
        text = done;
    }
    if text.is_empty() {
        return Err(ProviderError::new("provider_http", "Respuesta sin texto"));
    }
    Ok(text)
}
fn http_status_error(status: u16, body: &str) -> ProviderError {
    let message = format!("Proveedor {status}: {body}");
    let code = match status {
        401 => "auth_required",
        403 => "permission_denied",
        404 => "not_found",
        _ => "provider_http",
    };
    ProviderError::new(code, message)
}
pub fn get_text(
    url: &str,
    bearer: Option<&str>,
    timeout: Duration,
    max_bytes: usize,
) -> Result<String> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .http_status_as_error(false)
        .build()
        .new_agent();
    let mut request = agent.get(url);
    if let Some(token) = bearer {
        request = request.header("Authorization", &format!("Bearer {token}"));
    }
    let mut response = request
        .call()
        .map_err(|e| ProviderError::new("provider_http", format!("Proveedor: {e}")))?;
    if !response.status().is_success() {
        let body = response.body_mut().read_to_string().unwrap_or_default();
        return Err(http_status_error(response.status().as_u16(), &body));
    }
    let text = response
        .body_mut()
        .read_to_string()
        .map_err(|e| ProviderError::new("provider_http", format!("Respuesta inválida: {e}")))?;
    if text.len() > max_bytes {
        return Err(ProviderError::new(
            "preview_too_large",
            format!("La previsualización supera {max_bytes} bytes."),
        ));
    }
    Ok(text)
}
pub fn post_json(
    url: &str,
    bearer: Option<&str>,
    body: &serde_json::Value,
    timeout: Duration,
) -> Result<serde_json::Value> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .http_status_as_error(false)
        .build()
        .new_agent();
    let mut request = agent.post(url);
    if let Some(token) = bearer {
        request = request.header("Authorization", &format!("Bearer {token}"));
    }
    let mut response = request
        .send_json(body)
        .map_err(|e| ProviderError::new("provider_http", format!("Proveedor: {e}")))?;
    if !response.status().is_success() {
        let body = response.body_mut().read_to_string().unwrap_or_default();
        return Err(ProviderError::new(
            "provider_http",
            format!("Proveedor {}: {}", response.status(), body),
        ));
    }
    response
        .body_mut()
        .read_json()
        .map_err(|e| ProviderError::new("provider_http", format!("Respuesta inválida: {e}")))
}
pub fn parse_scopes(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or_default()
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}
pub fn store_json<T: Serialize>(store: &dyn CredentialStore, user: &str, value: &T) -> Result<()> {
    let raw = serde_json::to_string(value)
        .map_err(|e| ProviderError::new("storage_error", format!("Serialización: {e}")))?;
    store.set(user, &raw)
}
pub fn load_json<T: for<'de> Deserialize<'de>>(
    store: &dyn CredentialStore,
    user: &str,
) -> Result<Option<T>> {
    match store.get(user)? {
        None => Ok(None),
        Some(raw) => serde_json::from_str(&raw)
            .map(Some)
            .map_err(|e| ProviderError::new("storage_error", format!("Credencial inválida: {e}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pkce_rfc_vector_and_query_parser() {
        assert_eq!(
            pkce_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
        let q = parse_query("code=a%2Bb&state=x+y&client_id=oaiapp_1");
        assert_eq!(q["code"], "a+b");
        assert_eq!(q["state"], "x y");
        assert_eq!(q["client_id"], "oaiapp_1");
        assert_eq!(
            parse_picked_ids(Some("id-1, id-2,,id-3")),
            vec!["id-1", "id-2", "id-3"]
        );
        assert!(parse_picked_ids(None).is_empty());
    }
    #[test]
    fn endpoints_and_provider_parsers() {
        assert!(https_endpoint(Some("http://127.0.0.1:9/v1"), "").is_ok());
        assert!(https_endpoint(Some("http://example.com"), "").is_err());
        assert!(https_endpoint(Some("https://api.openai.com"), "").is_ok());
        let value =
            serde_json::json!({"output":[{"content":[{"type":"output_text","text":"hola"}]}]});
        assert_eq!(
            crate::openai::response_text(&value).as_deref(),
            Some("hola")
        );
        let gemini = serde_json::json!({"candidates":[{"content":{"parts":[{"text":"mundo"}]}}]});
        assert_eq!(crate::api::gemini_text(&gemini).as_deref(), Some("mundo"));
    }
    #[test]
    fn memory_store_roundtrip() {
        let s = MemoryStore::default();
        store_json(&s, "u", &serde_json::json!({"a":1})).unwrap();
        let v: Option<serde_json::Value> = load_json(&s, "u").unwrap();
        assert_eq!(v.unwrap()["a"], 1);
        s.delete("u").unwrap();
        assert!(s.get("u").unwrap().is_none());
    }
    #[test]
    fn http_status_maps_permission_and_auth_errors() {
        assert_eq!(http_status_error(401, "").code, "auth_required");
        assert_eq!(http_status_error(403, "").code, "permission_denied");
        assert_eq!(http_status_error(404, "").code, "not_found");
        assert_eq!(http_status_error(500, "").code, "provider_http");
    }

    #[test]
    fn provider_request_timeout_is_visible() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (_stream, _) = listener.accept().unwrap();
            std::thread::sleep(Duration::from_millis(200));
        });
        let error = post_stream_text(
            &format!("http://{address}/responses"),
            None,
            &serde_json::json!({}),
            Duration::from_millis(30),
        )
        .unwrap_err();
        assert_eq!(error.code, "provider_http");
        server.join().unwrap();
    }

    #[test]
    fn sse_accumulates_deltas_and_ignores_events() {
        let raw = concat!(
            "event: response.output_text.delta\n",
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"conec\"}\n\n",
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"tado\"}\n\n",
            "data: [DONE]\n\n"
        );
        assert_eq!(parse_sse_text(raw).unwrap(), "conectado");
    }
    #[test]
    fn sse_falls_back_to_completed_response() {
        let raw = concat!(
            "data: {\"type\":\"response.completed\",\"response\":{\"output\":",
            "[{\"content\":[{\"type\":\"output_text\",\"text\":\"final\"}]}]}}\n\n"
        );
        assert_eq!(parse_sse_text(raw).unwrap(), "final");
    }
    #[test]
    fn sse_reports_provider_error() {
        let raw = "data: {\"type\":\"error\",\"error\":{\"message\":\"cuota agotada\"}}\n\n";
        let error = parse_sse_text(raw).unwrap_err();
        assert_eq!(error.code, "provider_http");
        assert_eq!(error.message, "cuota agotada");
    }
}
