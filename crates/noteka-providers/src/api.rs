//! Proveedores con API key: OpenAI Responses y Gemini generateContent.
use crate::*;
use serde_json::Value;
use std::time::Duration;

pub const OPENAI_USER: &str = "openai-api-key";
pub const GEMINI_USER: &str = "gemini-api-key";

pub fn set_key(store: &dyn CredentialStore, provider: &str, key: &str) -> Result<()> {
    if key.trim().is_empty() || key.len() > 512 || key.chars().any(char::is_control) {
        return Err(ProviderError::validation("API key inválida"));
    }
    store.set(user(provider)?, key.trim())
}
pub fn delete_key(store: &dyn CredentialStore, provider: &str) -> Result<()> {
    store.delete(user(provider)?)
}
pub fn key_status(store: &dyn CredentialStore, provider: &str) -> Result<Value> {
    let configured = key(store, provider)?.is_some();
    Ok(serde_json::json!({"provider":provider,"api_key_configured":configured}))
}
fn user(provider: &str) -> Result<&'static str> {
    match provider {
        "openai" => Ok(OPENAI_USER),
        "gemini" => Ok(GEMINI_USER),
        _ => Err(ProviderError::validation(
            "Proveedor de API key desconocido",
        )),
    }
}
pub fn key(store: &dyn CredentialStore, provider: &str) -> Result<Option<String>> {
    let user = user(provider)?;
    if let Some(value) = store.get(user)? {
        return Ok(Some(value));
    }
    let env = match provider {
        "openai" => std::env::var("NOTEKA_OPENAI_API_KEY").ok(),
        "gemini" => std::env::var("NOTEKA_GEMINI_API_KEY").ok(),
        _ => None,
    };
    Ok(env.filter(|v| !v.trim().is_empty()))
}
pub fn openai_chat(
    store: &dyn CredentialStore,
    model: &str,
    prompt: &str,
    max_tokens: u32,
) -> Result<String> {
    let key = key(store, "openai")?
        .ok_or_else(|| ProviderError::new("auth_required", "Configurá una API key de OpenAI"))?;
    let base = https_endpoint(
        std::env::var("NOTEKA_OPENAI_API_BASE").ok().as_deref(),
        "https://api.openai.com/v1",
    )?;
    post_stream_text(
        &format!("{base}/responses"),
        Some(&key),
        &serde_json::json!({"model":model,"input":[{"role":"user","content":prompt}],"max_output_tokens":max_tokens,"store":false,"stream":true}),
        Duration::from_secs(120),
    )
}

pub fn gemini_chat(store: &dyn CredentialStore, model: &str, prompt: &str) -> Result<Value> {
    let key = key(store, "gemini")?
        .ok_or_else(|| ProviderError::new("auth_required", "Configurá una API key de Gemini"))?;
    let base = https_endpoint(
        std::env::var("NOTEKA_GEMINI_API_BASE").ok().as_deref(),
        "https://generativelanguage.googleapis.com/v1beta",
    )?;
    let url = format!(
        "{base}/models/{model}:generateContent?key={}",
        url_encode(&key)
    );
    post_json(
        &url,
        None,
        &serde_json::json!({"contents":[{"parts":[{"text":prompt}]}]}),
        Duration::from_secs(120),
    )
}
pub fn gemini_text(value: &Value) -> Option<String> {
    let mut out = String::new();
    for part in value
        .get("candidates")?
        .as_array()?
        .first()?
        .get("content")?
        .get("parts")?
        .as_array()?
    {
        if let Some(text) = part.get("text").and_then(Value::as_str) {
            out.push_str(text);
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
    fn api_key_store_status() {
        let store = MemoryStore::default();
        assert!(
            !key_status(&store, "openai").unwrap()["api_key_configured"]
                .as_bool()
                .unwrap()
        );
        set_key(&store, "openai", "sk-test").unwrap();
        assert!(
            key_status(&store, "openai").unwrap()["api_key_configured"]
                .as_bool()
                .unwrap()
        );
        delete_key(&store, "openai").unwrap();
        assert!(key(&store, "openai").unwrap().is_none());
    }
}

#[cfg(test)]
mod missing_key_tests {
    use super::*;

    #[test]
    fn unknown_provider_is_rejected_before_storage() {
        let error = set_key(&MemoryStore::default(), "otro", "value").unwrap_err();
        assert_eq!(error.code, "validation_error");
    }

    #[test]
    fn missing_keys_fail_explicitly_when_env_is_absent() {
        let store = MemoryStore::default();
        if std::env::var("NOTEKA_OPENAI_API_KEY").is_err() {
            let error = openai_chat(&store, "model", "hola", 16).unwrap_err();
            assert_eq!(error.code, "auth_required");
        }
        if std::env::var("NOTEKA_GEMINI_API_KEY").is_err() {
            let error = gemini_chat(&store, "model", "hola").unwrap_err();
            assert_eq!(error.code, "auth_required");
        }
    }
}
