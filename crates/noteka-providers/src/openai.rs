//! Sign in with ChatGPT (OpenID Connect + PKCE) y Responses API.
use crate::*;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header, jwk::JwkSet};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

pub const USER: &str = "openai-chatgpt";
pub const CLIENT_DYNAMIC: &str = "dynamic_agent_client";
pub const SCOPES: &str =
    "openid profile email offline_access resource.invoke chatgpt.tokens.use.direct";
pub const RESOURCE: &str = "https://api.openai.com/v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    #[serde(flatten)]
    pub oauth: OAuthSession,
    pub id_token: Option<String>,
}
#[derive(Debug, Deserialize)]
struct TokenReply {
    access_token: String,
    refresh_token: Option<String>,
    id_token: Option<String>,
    token_type: String,
    expires_in: Option<u64>,
    scope: Option<String>,
}
#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    email: Option<String>,
    nonce: Option<String>,
    aud: Value,
    exp: u64,
}
fn auth_base() -> Result<String> {
    https_endpoint(
        std::env::var("NOTEKA_OPENAI_AUTH_BASE").ok().as_deref(),
        "https://auth.openai.com",
    )
}
fn api_base() -> Result<String> {
    https_endpoint(
        std::env::var("NOTEKA_OPENAI_API_BASE").ok().as_deref(),
        "https://api.openai.com/v1",
    )
}
fn callback_port() -> Result<u16> {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0))
        .map_err(|e| ProviderError::new("callback_error", format!("{e}")))?;
    Ok(listener.local_addr().unwrap().port())
}

pub fn login(store: &dyn CredentialStore) -> Result<Session> {
    let existing: Option<Session> = load_json(store, USER)?;
    let host = host_id()?;
    let state = random_state();
    let nonce = random_state();
    let verifier = pkce_verifier();
    let challenge = pkce_challenge(&verifier);
    let port = callback_port()?;
    let redirect = format!("http://127.0.0.1:{port}/auth/callback");
    let base = auth_base()?;
    let mut query = vec![
        ("response_type", "code"),
        ("redirect_uri", redirect.as_str()),
        ("scope", SCOPES),
        ("resource", RESOURCE),
        ("state", state.as_str()),
        ("nonce", nonce.as_str()),
        ("code_challenge_method", "S256"),
        ("code_challenge", challenge.as_str()),
    ];
    let issued = existing.as_ref().map(|s| s.oauth.client_id.clone());
    let client = issued.as_deref().unwrap_or(CLIENT_DYNAMIC);
    query.push(("client_id", client));
    if let Some(session) = &existing {
        if let Some(token) = &session.id_token {
            query.push(("id_token_hint", token));
        }
    } else {
        query.push(("agent_name_hint", "Noteka"));
        query.push(("ext_agent_host_id", host.as_str()));
    }
    let url = format!(
        "{base}/api/accounts/authorize?{}",
        query
            .iter()
            .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
            .collect::<Vec<_>>()
            .join("&")
    );
    eprintln!("Abrí esta URL para autorizar Noteka:\n{url}");
    let _ = open_browser(&url);
    let callback = wait_callback(port, "/auth/callback", &state, Duration::from_secs(900))?;
    let issued_client = callback
        .issued_client_id
        .or(issued)
        .unwrap_or_else(|| CLIENT_DYNAMIC.to_owned());
    if issued_client == CLIENT_DYNAMIC {
        return Err(ProviderError::new(
            "registration_incomplete",
            "El callback no entregó el client_id emitido",
        ));
    }
    let token = exchange(
        &base,
        &[
            ("grant_type", "authorization_code"),
            ("client_id", &issued_client),
            ("code", &callback.code),
            ("code_verifier", &verifier),
            ("redirect_uri", &redirect),
            ("resource", RESOURCE),
        ],
    )?;
    let id_token = token
        .id_token
        .clone()
        .ok_or_else(|| ProviderError::new("invalid_id_token", "Falta id_token"))?;
    let claims = verify_id_token(&base, &id_token, &issued_client, &nonce)?;
    let now = now();
    let session = Session {
        oauth: OAuthSession {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            token_type: token.token_type,
            scopes: parse_scopes(token.scope.as_deref()),
            expires_at: now + token.expires_in.unwrap_or(3600),
            client_id: issued_client,
            subject: claims.sub,
            email: claims.email,
        },
        id_token: Some(id_token),
    };
    store_json(store, USER, &session)?;
    Ok(session)
}

fn exchange(base: &str, form: &[(&str, &str)]) -> Result<TokenReply> {
    let value = post_form(
        &format!("{base}/api/accounts/oauth/token"),
        form,
        Duration::from_secs(30),
    )?;
    serde_json::from_value(value)
        .map_err(|e| ProviderError::new("auth_http", format!("Token inválido: {e}")))
}
fn verify_id_token(base: &str, token: &str, client_id: &str, nonce: &str) -> Result<Claims> {
    let discovery = get_json(
        &format!("{base}/.well-known/openid-configuration"),
        None,
        Duration::from_secs(20),
    )?;
    let jwks_url = discovery["jwks_uri"]
        .as_str()
        .ok_or_else(|| ProviderError::new("invalid_id_token", "Discovery sin jwks_uri"))?;
    let jwks: JwkSet =
        serde_json::from_value(get_json(jwks_url, None, Duration::from_secs(20))?)
            .map_err(|e| ProviderError::new("invalid_id_token", format!("JWKS inválido: {e}")))?;
    let header = decode_header(token)
        .map_err(|_| ProviderError::new("invalid_id_token", "Header JWT inválido"))?;
    let kid = header
        .kid
        .ok_or_else(|| ProviderError::new("invalid_id_token", "JWT sin kid"))?;
    let jwk = jwks
        .find(&kid)
        .ok_or_else(|| ProviderError::new("invalid_id_token", "kid desconocido"))?;
    let key = DecodingKey::from_jwk(jwk)
        .map_err(|_| ProviderError::new("invalid_id_token", "JWK inválida"))?;
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[base]);
    validation.set_audience(&[client_id]);
    validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
    validation.leeway = 60;
    let claims = decode::<Claims>(token, &key, &validation)
        .map_err(|e| ProviderError::new("invalid_id_token", format!("JWT inválido: {e}")))?
        .claims;
    if claims.nonce.as_deref() != Some(nonce) || claims.exp <= now() {
        return Err(ProviderError::new(
            "invalid_id_token",
            "Nonce/expiración inválidos",
        ));
    }
    if let Value::Array(aud) = &claims.aud
        && !aud.iter().any(|v| v == client_id)
    {
        return Err(ProviderError::new("invalid_id_token", "Audiencia inválida"));
    }
    Ok(claims)
}

pub fn status(store: &dyn CredentialStore) -> Result<Value> {
    match load_json::<Session>(store, USER)? {
        None => Ok(serde_json::json!({"provider":"chatgpt","state":"sin_sesion"})),
        Some(s) => Ok(serde_json::json!({
            "provider":"chatgpt",
            "state":"conectado",
            "subject":s.oauth.subject,
            "email":s.oauth.email,
            "expires_at":s.oauth.expires_at,
            "expired":s.oauth.expired(),
            "scopes":s.oauth.scopes,
            "chatgpt_plan":s.oauth.scopes.iter().any(|x| x=="chatgpt.tokens.use.direct"),
        })),
    }
}
pub fn logout(store: &dyn CredentialStore) -> Result<()> {
    store.delete(USER)
}
pub fn access_token(store: &dyn CredentialStore) -> Result<(Session, String)> {
    let mut session: Session = load_json(store, USER)?
        .ok_or_else(|| ProviderError::new("auth_required", "Iniciá sesión con ChatGPT"))?;
    if !session.oauth.expired() {
        return Ok((session.clone(), session.oauth.access_token));
    }
    let refresh = session.oauth.refresh_token.clone().ok_or_else(|| {
        ProviderError::new("auth_required", "La sesión expiró; volvé a iniciar sesión")
    })?;
    let base = auth_base()?;
    let token = exchange(
        &base,
        &[
            ("grant_type", "refresh_token"),
            ("client_id", &session.oauth.client_id),
            ("refresh_token", &refresh),
            ("resource", RESOURCE),
        ],
    )?;
    session.oauth.access_token = token.access_token;
    if let Some(r) = token.refresh_token {
        session.oauth.refresh_token = Some(r);
    }
    session.oauth.expires_at = now() + token.expires_in.unwrap_or(3600);
    if let Some(scope) = token.scope {
        session.oauth.scopes = parse_scopes(Some(&scope));
    }
    store_json(store, USER, &session)?;
    Ok((session.clone(), session.oauth.access_token))
}
pub fn chat(
    store: &dyn CredentialStore,
    model: &str,
    prompt: &str,
    max_tokens: u32,
) -> Result<String> {
    let (_, token) = access_token(store)?;
    let base = api_base()?;
    post_stream_text(
        &format!("{base}/responses"),
        Some(&token),
        &serde_json::json!({"model":model,"input":[{"role":"user","content":prompt}],"max_output_tokens":max_tokens,"store":false,"stream":true}),
        Duration::from_secs(120),
    )
}

pub fn response_text(value: &Value) -> Option<String> {
    let mut out = String::new();
    for item in value.get("output")?.as_array()? {
        for content in item.get("content")?.as_array()? {
            if content.get("type").and_then(Value::as_str) == Some("output_text")
                && let Some(text) = content.get("text").and_then(Value::as_str)
            {
                out.push_str(text);
            }
        }
    }
    (!out.is_empty()).then_some(out)
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
    fn status_never_exposes_tokens_and_reports_plan() {
        let store = MemoryStore::default();
        let session = Session {
            oauth: OAuthSession {
                access_token: "secret-access".into(),
                refresh_token: Some("secret-refresh".into()),
                token_type: "Bearer".into(),
                scopes: vec!["chatgpt.tokens.use.direct".into()],
                expires_at: now() + 3600,
                client_id: "client".into(),
                subject: "subject".into(),
                email: None,
            },
            id_token: Some("secret-id".into()),
        };
        store_json(&store, USER, &session).unwrap();
        let value = status(&store).unwrap().to_string();
        assert!(!value.contains("secret-access"));
        assert!(!value.contains("secret-refresh"));
        assert!(!value.contains("secret-id"));
        assert!(value.contains("\"chatgpt_plan\":true"));
    }

    #[test]
    fn chat_without_session_is_explicitly_auth_required() {
        let error = chat(&MemoryStore::default(), "model", "hola", 16).unwrap_err();
        assert_eq!(error.code, "auth_required");
    }
}
