//! Google OAuth desktop + Google Picker + Drive metadata (sólo lectura).
use crate::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

pub const USER: &str = "google-drive";
pub const PICKER_USER: &str = "google-drive-picker";
pub const SCOPES: &str = "openid email profile https://www.googleapis.com/auth/drive.file";
pub const PICKER_SCOPE: &str = "https://www.googleapis.com/auth/drive.file";
const MAX_PICKED: usize = 20;
pub const MAX_PREVIEW_BYTES: usize = 2_097_152;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    #[serde(flatten)]
    pub oauth: OAuthSession,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickerSession {
    #[serde(flatten)]
    pub oauth: OAuthSession,
    #[serde(default)]
    pub picked_file_ids: Vec<String>,
}
#[derive(Debug, Deserialize)]
struct TokenReply {
    access_token: String,
    refresh_token: Option<String>,
    token_type: String,
    expires_in: Option<u64>,
    scope: Option<String>,
}
fn client() -> Result<(String, String)> {
    let id = std::env::var("NOTEKA_GOOGLE_CLIENT_ID")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| {
            ProviderError::new(
                "google_client_missing",
                "Configurá NOTEKA_GOOGLE_CLIENT_ID con un cliente OAuth de escritorio",
            )
        })?;
    let secret = std::env::var("NOTEKA_GOOGLE_CLIENT_SECRET").unwrap_or_default();
    Ok((id, secret))
}
fn auth_base() -> Result<String> {
    https_endpoint(
        std::env::var("NOTEKA_GOOGLE_AUTH_BASE").ok().as_deref(),
        "https://accounts.google.com/o/oauth2/v2/auth",
    )
}
fn token_base() -> Result<String> {
    https_endpoint(
        std::env::var("NOTEKA_GOOGLE_TOKEN_BASE").ok().as_deref(),
        "https://oauth2.googleapis.com/token",
    )
}
fn api_base() -> Result<String> {
    https_endpoint(
        std::env::var("NOTEKA_GOOGLE_API_BASE").ok().as_deref(),
        "https://www.googleapis.com",
    )
}
fn callback_port() -> Result<u16> {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0))
        .map_err(|e| ProviderError::new("callback_error", format!("{e}")))?;
    Ok(listener.local_addr().unwrap().port())
}
fn oauth_url(
    base: &str,
    client_id: &str,
    redirect: &str,
    scope: &str,
    state: &str,
    challenge: &str,
    picker: bool,
) -> String {
    let mut fields = vec![
        ("client_id", client_id),
        ("redirect_uri", redirect),
        ("response_type", "code"),
        ("scope", scope),
        ("access_type", "offline"),
        ("prompt", "consent"),
        ("state", state),
        ("code_challenge_method", "S256"),
        ("code_challenge", challenge),
    ];
    if picker {
        fields.push(("trigger_onepick", "true"));
        fields.push(("allow_multiple", "true"));
    } else {
        fields.push(("include_granted_scopes", "true"));
    }
    format!(
        "{base}?{}",
        fields
            .iter()
            .map(|(key, value)| format!("{}={}", url_encode(key), url_encode(value)))
            .collect::<Vec<_>>()
            .join("&")
    )
}
pub fn login(store: &dyn CredentialStore) -> Result<Session> {
    let (client_id, client_secret) = client()?;
    let state = random_state();
    let verifier = pkce_verifier();
    let challenge = pkce_challenge(&verifier);
    let port = callback_port()?;
    let redirect = format!("http://127.0.0.1:{port}/auth/callback");
    let url = oauth_url(
        &auth_base()?,
        &client_id,
        &redirect,
        SCOPES,
        &state,
        &challenge,
        false,
    );
    eprintln!("Abrí esta URL para autorizar Google Drive:\n{url}");
    let _ = open_browser(&url);
    let callback = wait_callback(port, "/auth/callback", &state, Duration::from_secs(900))?;
    if callback.issued_client_id.is_some() {
        return Err(ProviderError::new(
            "callback_error",
            "Callback Google inesperado",
        ));
    }
    let token = exchange_code(
        &client_id,
        &client_secret,
        &redirect,
        &verifier,
        &callback.code,
    )?;
    let oauth = OAuthSession {
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        token_type: token.token_type,
        scopes: parse_scopes(token.scope.as_deref()),
        expires_at: now() + token.expires_in.unwrap_or(3600),
        client_id,
        subject: String::new(),
        email: None,
    };
    let session = enrich(oauth)?;
    store_json(store, USER, &session)?;
    Ok(session)
}
/// Abre Google Picker con `drive.file` y guarda la selección en una sesión
/// separada para no pisar la identidad obtenida por `login`.
pub fn pick(store: &dyn CredentialStore) -> Result<Value> {
    let identity: Session = load_json(store, USER)?
        .ok_or_else(|| ProviderError::new("auth_required", "Iniciá sesión con Google"))?;
    let (client_id, client_secret) = client()?;
    let state = random_state();
    let verifier = pkce_verifier();
    let challenge = pkce_challenge(&verifier);
    let port = callback_port()?;
    let redirect = format!("http://127.0.0.1:{port}/auth/callback");
    let url = oauth_url(
        &auth_base()?,
        &client_id,
        &redirect,
        PICKER_SCOPE,
        &state,
        &challenge,
        true,
    );
    eprintln!("Abrí Google Picker para elegir archivos:\n{url}");
    let _ = open_browser(&url);
    let callback = wait_callback(port, "/auth/callback", &state, Duration::from_secs(900))?;
    if callback.picked_file_ids.is_empty() {
        return Err(ProviderError::new(
            "picker_empty",
            "Google Picker no devolvió archivos seleccionados",
        ));
    }
    if callback.picked_file_ids.len() > MAX_PICKED {
        return Err(ProviderError::new(
            "picker_limit",
            format!("Seleccioná como máximo {MAX_PICKED} archivos"),
        ));
    }
    let token = exchange_code(
        &client_id,
        &client_secret,
        &redirect,
        &verifier,
        &callback.code,
    )?;
    let picker = PickerSession {
        oauth: OAuthSession {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            token_type: token.token_type,
            scopes: parse_scopes(token.scope.as_deref()),
            expires_at: now() + token.expires_in.unwrap_or(3600),
            client_id,
            subject: identity.oauth.subject,
            email: identity.oauth.email,
        },
        picked_file_ids: callback.picked_file_ids,
    };
    store_json(store, PICKER_USER, &picker)?;
    let account_id = picker.oauth.subject.clone();
    let files = fetch_selected(&picker.oauth.access_token, &picker.picked_file_ids)?;
    Ok(serde_json::json!({
        "account_id": account_id,
        "picked_file_ids": picker.picked_file_ids,
        "files": files,
    }))
}
pub fn selected(store: &dyn CredentialStore) -> Result<Value> {
    let mut picker: PickerSession = load_json(store, PICKER_USER)?.ok_or_else(|| {
        ProviderError::new(
            "picker_required",
            "No hay archivos seleccionados; ejecutá google pick",
        )
    })?;
    let token = picker_access_token(store, &mut picker)?;
    let files = fetch_selected(&token, &picker.picked_file_ids)?;
    Ok(serde_json::json!({
        "picked_file_ids": picker.picked_file_ids,
        "files": files,
    }))
}
pub fn preview(store: &dyn CredentialStore, file_id: &str) -> Result<Value> {
    let id = file_id.trim();
    if id.is_empty() || id.len() > 2048 {
        return Err(ProviderError::validation("file_id inválido"));
    }
    let id = id.to_owned();
    let mut picker: PickerSession = load_json(store, PICKER_USER)?.ok_or_else(|| {
        ProviderError::new(
            "picker_required",
            "No hay una selección Picker; ejecutá google pick",
        )
    })?;
    let token = picker_access_token(store, &mut picker)?;
    let base = api_base()?;
    let metadata = get_json(
        &format!(
            "{base}/drive/v3/files/{}?fields=id,name,mimeType,webViewLink",
            url_encode(&id)
        ),
        Some(&token),
        Duration::from_secs(30),
    )?;
    let mime = metadata["mimeType"].as_str().unwrap_or_default();
    let export = preview_export_mime(mime).ok_or_else(|| {
        ProviderError::new(
            "preview_unsupported",
            "Este tipo de archivo no tiene preview embebido; abrila en Google Drive.",
        )
    })?;
    let text = get_text(
        &format!(
            "{base}/drive/v3/files/{}/export?mimeType={}",
            url_encode(&id),
            url_encode(export)
        ),
        Some(&token),
        Duration::from_secs(60),
        MAX_PREVIEW_BYTES,
    )?;
    Ok(serde_json::json!({
        "id": id,
        "name": metadata["name"].as_str().unwrap_or_default(),
        "mime_type": mime,
        "preview_mime": export,
        "text": text,
    }))
}
fn preview_export_mime(mime: &str) -> Option<&'static str> {
    match mime {
        "application/vnd.google-apps.document" => Some("text/plain"),
        "application/vnd.google-apps.spreadsheet" => Some("text/csv"),
        _ => None,
    }
}
fn exchange_code(
    client_id: &str,
    client_secret: &str,
    redirect: &str,
    verifier: &str,
    code: &str,
) -> Result<TokenReply> {
    let mut form = vec![
        ("grant_type", "authorization_code"),
        ("code", code),
        ("client_id", client_id),
        ("redirect_uri", redirect),
        ("code_verifier", verifier),
    ];
    if !client_secret.is_empty() {
        form.push(("client_secret", client_secret));
    }
    exchange(&token_base()?, &form)
}
fn exchange(url: &str, form: &[(&str, &str)]) -> Result<TokenReply> {
    let value = post_form(url, form, Duration::from_secs(30))?;
    serde_json::from_value(value)
        .map_err(|e| ProviderError::new("auth_http", format!("Token inválido: {e}")))
}
fn enrich(mut oauth: OAuthSession) -> Result<Session> {
    let value = get_json(
        &format!("{}/oauth2/v3/userinfo", api_base()?),
        Some(&oauth.access_token),
        Duration::from_secs(20),
    )?;
    oauth.subject = value["sub"].as_str().unwrap_or_default().to_owned();
    oauth.email = value["email"].as_str().map(str::to_owned);
    if oauth.subject.is_empty() {
        return Err(ProviderError::new(
            "google_identity",
            "Google no devolvió sub",
        ));
    }
    Ok(Session { oauth })
}
pub fn status(store: &dyn CredentialStore) -> Result<Value> {
    match load_json::<Session>(store, USER)? {
        None => Ok(serde_json::json!({"provider":"google","state":"sin_sesion"})),
        Some(s) => {
            let picked_count = load_json::<PickerSession>(store, PICKER_USER)?
                .map(|picker| picker.picked_file_ids.len())
                .unwrap_or(0);
            Ok(serde_json::json!({
                "provider":"google",
                "state":"conectado",
                "subject":s.oauth.subject,
                "email":s.oauth.email,
                "expires_at":s.oauth.expires_at,
                "expired":s.oauth.expired(),
                "scopes":s.oauth.scopes,
                "picked_count":picked_count,
            }))
        }
    }
}
pub fn logout(store: &dyn CredentialStore) -> Result<()> {
    store.delete(PICKER_USER)?;
    store.delete(USER)
}
pub fn access_token(store: &dyn CredentialStore) -> Result<String> {
    let mut session: Session = load_json(store, USER)?
        .ok_or_else(|| ProviderError::new("auth_required", "Iniciá sesión con Google"))?;
    if !session.oauth.expired() {
        return Ok(session.oauth.access_token);
    }
    let refresh = session.oauth.refresh_token.clone().ok_or_else(|| {
        ProviderError::new(
            "auth_required",
            "La sesión Google expiró; volvé a iniciar sesión",
        )
    })?;
    let (client_id, client_secret) = client()?;
    let mut form = vec![
        ("grant_type", "refresh_token"),
        ("client_id", client_id.as_str()),
        ("refresh_token", refresh.as_str()),
    ];
    if !client_secret.is_empty() {
        form.push(("client_secret", client_secret.as_str()));
    }
    let token = exchange(&token_base()?, &form)?;
    session.oauth.access_token = token.access_token;
    if let Some(r) = token.refresh_token {
        session.oauth.refresh_token = Some(r);
    }
    session.oauth.expires_at = now() + token.expires_in.unwrap_or(3600);
    store_json(store, USER, &session)?;
    Ok(session.oauth.access_token)
}
fn picker_access_token(store: &dyn CredentialStore, picker: &mut PickerSession) -> Result<String> {
    if !picker.oauth.expired() {
        return Ok(picker.oauth.access_token.clone());
    }
    let refresh = picker.oauth.refresh_token.clone().ok_or_else(|| {
        ProviderError::new(
            "auth_required",
            "La selección de Drive expiró; volvé a ejecutar google pick",
        )
    })?;
    let (client_id, client_secret) = client()?;
    let mut form = vec![
        ("grant_type", "refresh_token"),
        ("client_id", client_id.as_str()),
        ("refresh_token", refresh.as_str()),
    ];
    if !client_secret.is_empty() {
        form.push(("client_secret", client_secret.as_str()));
    }
    let token = exchange(&token_base()?, &form)?;
    picker.oauth.access_token = token.access_token;
    if let Some(r) = token.refresh_token {
        picker.oauth.refresh_token = Some(r);
    }
    picker.oauth.expires_at = now() + token.expires_in.unwrap_or(3600);
    store_json(store, PICKER_USER, picker)?;
    Ok(picker.oauth.access_token.clone())
}
fn fetch_selected(token: &str, ids: &[String]) -> Result<Vec<Value>> {
    let base = api_base()?;
    ids.iter()
        .map(|id| {
            get_json(
                &format!(
                    "{base}/drive/v3/files/{}?fields=id,name,mimeType,modifiedTime,webViewLink",
                    url_encode(id)
                ),
                Some(token),
                Duration::from_secs(30),
            )
        })
        .collect()
}
pub fn files(store: &dyn CredentialStore) -> Result<Value> {
    let token = access_token(store)?;
    get_json(
        &format!(
            "{}/drive/v3/files?pageSize=20&fields=files(id,name,mimeType,modifiedTime,webViewLink),nextPageToken",
            api_base()?
        ),
        Some(&token),
        Duration::from_secs(30),
    )
}
fn url_encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picker_url_uses_only_drive_file_and_triggers_onepick() {
        let url = oauth_url(
            "https://accounts.google.com/o/oauth2/v2/auth",
            "client.apps.googleusercontent.com",
            "http://127.0.0.1:1234/auth/callback",
            PICKER_SCOPE,
            "state123",
            "challenge123",
            true,
        );
        let query = parse_query(url.split_once('?').unwrap().1);
        assert_eq!(query["scope"], PICKER_SCOPE);
        assert!(!query["scope"].contains("openid"));
        assert_eq!(query["trigger_onepick"], "true");
        assert_eq!(query["allow_multiple"], "true");
        assert_eq!(query["prompt"], "consent");
        assert_eq!(query["state"], "state123");
        assert_eq!(query["code_challenge"], "challenge123");
        assert!(!query.contains_key("include_granted_scopes"));
    }

    #[test]
    fn login_url_keeps_identity_scopes() {
        let url = oauth_url(
            "https://accounts.google.com/o/oauth2/v2/auth",
            "client",
            "http://127.0.0.1:1/auth/callback",
            SCOPES,
            "s",
            "c",
            false,
        );
        let query = parse_query(url.split_once('?').unwrap().1);
        assert_eq!(query["scope"], SCOPES);
        assert!(!query.contains_key("trigger_onepick"));
        assert_eq!(query["include_granted_scopes"], "true");
    }

    #[test]
    fn preview_supports_only_docs_and_sheets() {
        assert_eq!(
            preview_export_mime("application/vnd.google-apps.document"),
            Some("text/plain")
        );
        assert_eq!(
            preview_export_mime("application/vnd.google-apps.spreadsheet"),
            Some("text/csv")
        );
        assert_eq!(preview_export_mime("application/pdf"), None);
    }

    #[test]
    fn selected_without_picker_is_explicit() {
        let store = MemoryStore::default();
        let error = selected(&store).unwrap_err();
        assert_eq!(error.code, "picker_required");
    }
}
