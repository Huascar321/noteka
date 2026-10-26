//! Cliente de chat local OpenAI-compatible. Sin credenciales, sin TLS y sólo
//! loopback; el endpoint lo configura explícitamente el usuario.
use crate::{agent_contracts::*, error::Result, local_http};
use serde::Serialize;
use serde_json::json;
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ChatMessage {
    pub role: &'static str,
    pub content: String,
}

pub(crate) fn chat(
    config: &AgentProviderConfig,
    messages: &[ChatMessage],
    timeout: Duration,
) -> Result<String> {
    config.validate()?;
    let body = json!({
        "model": config.model,
        "messages": messages,
        "temperature": config.temperature,
        "max_tokens": config.max_tokens,
        "stream": false
    });
    let response = local_http::post_json(&config.endpoint, &body, 2 * 1024 * 1024, timeout)?;
    let choice = response
        .get("choices")
        .and_then(|v| v.as_array())
        .and_then(|v| v.first())
        .ok_or_else(|| agent_error("provider_response_invalid"))?;
    let content = choice
        .get("message")
        .and_then(|v| v.get("content"))
        .and_then(|v| v.as_str())
        .or_else(|| choice.get("text").and_then(|v| v.as_str()))
        .ok_or_else(|| agent_error("provider_response_invalid"))?;
    if content.is_empty() || content.len() > 256 * 1024 {
        return Err(agent_error("provider_response_invalid"));
    }
    Ok(content.to_string())
}
