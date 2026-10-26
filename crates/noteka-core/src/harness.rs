//! Bucle acotado del harness: proveedor local, herramientas públicas, citas y
//! trazas. No hay acceso genérico a SQL, shell, archivos o workspace.
use crate::{
    agent_provider::{self, ChatMessage},
    agent_storage,
    contracts::*,
    error::*,
    scoped, semantic,
    workspace::{self, Workspace},
};
use chrono::{Duration as ChronoDuration, Utc};
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    path::Path,
    time::{Duration, Instant},
};

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum ModelReply {
    ToolCall {
        tool: String,
        input: Value,
    },
    Final {
        answer: String,
        #[serde(default)]
        citations: Vec<CitationInput>,
        #[serde(default)]
        insufficient_evidence: bool,
    },
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CitationInput {
    job_id: String,
    generation_id: String,
    ordinal: u32,
    quote: String,
}

enum Tool {
    Retrieval,
    EntityGet,
    DatabaseQuery,
    ChangesPreview,
    ChangesApply,
}
impl Tool {
    fn parse(name: &str, allow_apply: bool) -> Result<Self> {
        match name {
            "retrieval.search" => Ok(Self::Retrieval),
            "entity.get" => Ok(Self::EntityGet),
            "database.query" => Ok(Self::DatabaseQuery),
            "changes.preview" => Ok(Self::ChangesPreview),
            "changes.apply" if allow_apply => Ok(Self::ChangesApply),
            _ => Err(agent_invalid("agent_tool_not_allowed")),
        }
    }
    fn name(&self) -> &'static str {
        match self {
            Self::Retrieval => "retrieval.search",
            Self::EntityGet => "entity.get",
            Self::DatabaseQuery => "database.query",
            Self::ChangesPreview => "changes.preview",
            Self::ChangesApply => "changes.apply",
        }
    }
}

fn truncate(value: &str, max: usize) -> String {
    if value.len() <= max {
        return value.to_string();
    }
    let mut end = max;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_string()
}
fn normalize_ws(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn ask(
    path: &Path,
    input: AgentAskInput,
    scope: Option<&scoped::LocalScope>,
) -> Result<AgentRunResult> {
    if input.question.is_empty()
        || input.question.len() > MAX_QUESTION_BYTES
        || input.question.trim().is_empty()
        || input.question.contains('\0')
    {
        return Err(agent_invalid("agent_question_limit"));
    }
    input.provider.validate()?;
    input.limits.validate()?;
    let (c, _) = agent_storage::open(path, true)?;
    if let Some(scope) = scope {
        scope.check(&c)?;
    }
    let id = workspace::id();
    let lease = (Utc::now()
        + ChronoDuration::milliseconds(
            (input.limits.deadline_ms + 5_000).min(i64::MAX as u64) as i64
        ))
    .to_rfc3339_opts(chrono::SecondsFormat::Micros, true);
    {
        let tx = c.unchecked_transaction()?;
        agent_storage::expire_leases(&tx)?;
        agent_storage::prune(&tx)?;
        if agent_storage::active_lease(&tx)?.is_some() {
            let mut e = CoreError::new(
                ErrorCode::WorkspaceBusy,
                "Ya hay una ejecución del harness en curso.",
            );
            e.details = json!({"reason":"agent_busy"});
            return Err(e);
        }
        agent_storage::insert(&tx, &id, &input.provider, &input.question, &lease)?;
        tx.commit()?;
    }
    let result = run_loop(path, &c, &id, &input, scope);
    match result {
        Ok(result) => Ok(result),
        Err(error) => {
            let reason = error
                .details
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("agent_failed")
                .to_owned();
            let steps = vec![AgentStep {
                index: 0,
                kind: AgentStepKind::Model,
                tool: None,
                duration_ms: 0,
                input_bytes: 0,
                output_bytes: 0,
                status: "error".into(),
                error: Some(reason.clone()),
                summary: None,
            }];
            let run = agent_storage::finish(
                &c,
                &id,
                AgentRunState::Failed,
                None,
                &[],
                Some(&reason),
                &steps,
            )?;
            Ok(AgentRunResult {
                run,
                steps,
                citations: vec![],
            })
        }
    }
}

fn run_loop(
    path: &Path,
    c: &Connection,
    id: &str,
    input: &AgentAskInput,
    scope: Option<&scoped::LocalScope>,
) -> Result<AgentRunResult> {
    let started = Instant::now();
    let deadline = started + Duration::from_millis(input.limits.deadline_ms);
    let mut steps = Vec::<AgentStep>::new();
    let mut hits = HashMap::<(String, String, u32), RetrievalHit>::new();
    let mut previews = HashMap::<(String, String), BatchReceipt>::new();
    let system = system_prompt(input.allow_apply)?;
    let mut messages = vec![
        ChatMessage {
            role: "system",
            content: system,
        },
        ChatMessage {
            role: "user",
            content: input.question.clone(),
        },
    ];
    for turn in 0..input.limits.max_steps {
        if Instant::now() >= deadline {
            return finish(
                c,
                id,
                AgentRunState::Limited,
                None,
                &[],
                Some("deadline_exceeded"),
                &steps,
            );
        }
        if agent_storage::cancel_requested(c, id)? {
            return finish(
                c,
                id,
                AgentRunState::Cancelled,
                None,
                &[],
                Some("cancelled"),
                &steps,
            );
        }
        scoped::check_shutdown()?;
        let context: usize = messages.iter().map(|m| m.content.len()).sum();
        if context > input.limits.max_context_bytes as usize {
            return finish(
                c,
                id,
                AgentRunState::Limited,
                None,
                &[],
                Some("context_limit"),
                &steps,
            );
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return finish(
                c,
                id,
                AgentRunState::Limited,
                None,
                &[],
                Some("deadline_exceeded"),
                &steps,
            );
        }
        let timeout = remaining.min(Duration::from_millis(input.provider.timeout_ms));
        let model_started = Instant::now();
        let content = match agent_provider::chat(&input.provider, &messages, timeout) {
            Ok(content) => content,
            Err(error) => {
                let reason = error
                    .details
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("provider_error")
                    .to_owned();
                steps.push(AgentStep {
                    index: steps.len() as u32,
                    kind: AgentStepKind::Model,
                    tool: None,
                    duration_ms: model_started.elapsed().as_millis() as u64,
                    input_bytes: context.min(u32::MAX as usize) as u32,
                    output_bytes: 0,
                    status: "error".into(),
                    error: Some(reason.clone()),
                    summary: None,
                });
                let _ = agent_storage::save_trace(c, id, &steps);
                return finish(
                    c,
                    id,
                    AgentRunState::Failed,
                    None,
                    &[],
                    Some(&reason),
                    &steps,
                );
            }
        };
        steps.push(AgentStep {
            index: steps.len() as u32,
            kind: AgentStepKind::Model,
            tool: None,
            duration_ms: model_started.elapsed().as_millis() as u64,
            input_bytes: context.min(u32::MAX as usize) as u32,
            output_bytes: content.len().min(u32::MAX as usize) as u32,
            status: "ok".into(),
            error: None,
            summary: Some(truncate(&content, MAX_STEP_SUMMARY_BYTES)),
        });
        agent_storage::save_trace(c, id, &steps)?;
        let reply: ModelReply = match serde_json::from_str(&content) {
            Ok(reply) => reply,
            Err(_) => {
                return finish(
                    c,
                    id,
                    AgentRunState::Failed,
                    None,
                    &[],
                    Some("agent_protocol_invalid"),
                    &steps,
                );
            }
        };
        match reply {
            ModelReply::ToolCall { tool, input: value } => {
                if agent_storage::cancel_requested(c, id)? {
                    return finish(
                        c,
                        id,
                        AgentRunState::Cancelled,
                        None,
                        &[],
                        Some("cancelled"),
                        &steps,
                    );
                }
                let tool = match Tool::parse(&tool, input.allow_apply) {
                    Ok(tool) => tool,
                    Err(error) => {
                        let reason = error
                            .details
                            .get("reason")
                            .and_then(|v| v.as_str())
                            .unwrap_or("agent_tool_not_allowed")
                            .to_owned();
                        steps.push(AgentStep {
                            index: steps.len() as u32,
                            kind: AgentStepKind::Tool,
                            tool: Some(tool),
                            duration_ms: 0,
                            input_bytes: 0,
                            output_bytes: 0,
                            status: "error".into(),
                            error: Some(reason.clone()),
                            summary: None,
                        });
                        let _ = agent_storage::save_trace(c, id, &steps);
                        return finish(
                            c,
                            id,
                            AgentRunState::Failed,
                            None,
                            &[],
                            Some(&reason),
                            &steps,
                        );
                    }
                };
                let tool_started = Instant::now();
                let input_bytes = serde_json::to_vec(&value)?.len();
                let outcome = dispatch(
                    path,
                    &tool,
                    value.clone(),
                    input.allow_apply,
                    &mut hits,
                    &mut previews,
                    scope,
                    id,
                );
                match outcome {
                    Ok(result) => {
                        let result_bytes = serde_json::to_vec(&result)?.len();
                        if result_bytes > input.limits.max_tool_bytes as usize {
                            steps.push(AgentStep {
                                index: steps.len() as u32,
                                kind: AgentStepKind::Tool,
                                tool: Some(tool.name().into()),
                                duration_ms: tool_started.elapsed().as_millis() as u64,
                                input_bytes: input_bytes.min(u32::MAX as usize) as u32,
                                output_bytes: result_bytes.min(u32::MAX as usize) as u32,
                                status: "error".into(),
                                error: Some("tool_result_too_large".into()),
                                summary: None,
                            });
                            let _ = agent_storage::save_trace(c, id, &steps);
                            return finish(
                                c,
                                id,
                                AgentRunState::Limited,
                                None,
                                &[],
                                Some("tool_result_too_large"),
                                &steps,
                            );
                        }
                        let summary = truncate(
                            &String::from_utf8_lossy(&serde_json::to_vec(&result)?),
                            MAX_STEP_SUMMARY_BYTES,
                        );
                        steps.push(AgentStep {
                            index: steps.len() as u32,
                            kind: AgentStepKind::Tool,
                            tool: Some(tool.name().into()),
                            duration_ms: tool_started.elapsed().as_millis() as u64,
                            input_bytes: input_bytes.min(u32::MAX as usize) as u32,
                            output_bytes: result_bytes.min(u32::MAX as usize) as u32,
                            status: "ok".into(),
                            error: None,
                            summary: Some(summary),
                        });
                        let result_message = json!({"tool":tool.name(),"result":result});
                        let text = serde_json::to_string(&result_message)?;
                        let next_context = messages.iter().map(|m| m.content.len()).sum::<usize>()
                            + content.len()
                            + text.len();
                        if next_context > input.limits.max_context_bytes as usize {
                            let _ = agent_storage::save_trace(c, id, &steps);
                            return finish(
                                c,
                                id,
                                AgentRunState::Limited,
                                None,
                                &[],
                                Some("context_limit"),
                                &steps,
                            );
                        }
                        messages.push(ChatMessage {
                            role: "assistant",
                            content,
                        });
                        messages.push(ChatMessage {
                            role: "user",
                            content: text,
                        });
                        agent_storage::save_trace(c, id, &steps)?;
                    }
                    Err(error) => {
                        let reason = error
                            .details
                            .get("reason")
                            .and_then(|v| v.as_str())
                            .unwrap_or("agent_tool_failed")
                            .to_owned();
                        steps.push(AgentStep {
                            index: steps.len() as u32,
                            kind: AgentStepKind::Tool,
                            tool: Some(tool.name().into()),
                            duration_ms: tool_started.elapsed().as_millis() as u64,
                            input_bytes: input_bytes.min(u32::MAX as usize) as u32,
                            output_bytes: 0,
                            status: "error".into(),
                            error: Some(reason.clone()),
                            summary: None,
                        });
                        let _ = agent_storage::save_trace(c, id, &steps);
                        return finish(
                            c,
                            id,
                            AgentRunState::Failed,
                            None,
                            &[],
                            Some(&reason),
                            &steps,
                        );
                    }
                }
                if turn + 1 == input.limits.max_steps {
                    return finish(
                        c,
                        id,
                        AgentRunState::Limited,
                        None,
                        &[],
                        Some("steps_limit"),
                        &steps,
                    );
                }
            }
            ModelReply::Final {
                answer,
                citations,
                insufficient_evidence,
            } => {
                if agent_storage::cancel_requested(c, id)? {
                    return finish(
                        c,
                        id,
                        AgentRunState::Cancelled,
                        None,
                        &[],
                        Some("cancelled"),
                        &steps,
                    );
                }
                if answer.is_empty() || answer.len() > MAX_ANSWER_BYTES {
                    return finish(
                        c,
                        id,
                        AgentRunState::Failed,
                        None,
                        &[],
                        Some("agent_answer_limit"),
                        &steps,
                    );
                }
                let verified = match verify_citations(&citations, insufficient_evidence, &hits) {
                    Ok(citations) => citations,
                    Err(error) => {
                        let reason = error
                            .details
                            .get("reason")
                            .and_then(|v| v.as_str())
                            .unwrap_or("citation_unverified")
                            .to_owned();
                        return finish(
                            c,
                            id,
                            AgentRunState::Failed,
                            None,
                            &[],
                            Some(&reason),
                            &steps,
                        );
                    }
                };
                return finish(
                    c,
                    id,
                    AgentRunState::Succeeded,
                    Some(&answer),
                    &verified,
                    None,
                    &steps,
                );
            }
        }
    }
    finish(
        c,
        id,
        AgentRunState::Limited,
        None,
        &[],
        Some("steps_limit"),
        &steps,
    )
}

fn finish(
    c: &Connection,
    id: &str,
    state: AgentRunState,
    answer: Option<&str>,
    citations: &[AgentCitation],
    error: Option<&str>,
    steps: &[AgentStep],
) -> Result<AgentRunResult> {
    let run = agent_storage::finish(c, id, state, answer, citations, error, steps)?;
    Ok(AgentRunResult {
        run,
        steps: steps.to_vec(),
        citations: citations.to_vec(),
    })
}

fn system_prompt(allow_apply: bool) -> Result<String> {
    let mut tools = vec![
        "retrieval.search: busca fragmentos locales con fuentes. Input: {query, mode?: hybrid|semantic|lexical, limit?, budget_bytes?, job_id?, attachment_id?, extractor?, include_stale?}.",
        "entity.get: lee una entidad por id. Input: {id}.",
        "database.query: consulta una colección. Input: el DTO DatabaseQuery del catálogo.",
        "changes.preview: valida un BatchPlan y devuelve un recibo. Input: BatchPlan v1.",
    ];
    if allow_apply {
        tools.push("changes.apply: aplica un plan SÓLO con plan y recibo exactos de un preview de esta ejecución. Input: {plan, expected}.");
    }
    let catalog = crate::describe();
    let schema = |name: &str| {
        serde_json::to_string(&catalog["inputs"][name])
            .unwrap_or_else(|_| "\"no disponible\"".into())
    };
    Ok(format!(
        "Eres el harness local de Noteka. Respondés en español y sólo con JSON estricto, sin markdown ni texto adicional.\n\
         Cada turno debe ser una de estas dos formas:\n\
         {{\"type\":\"tool_call\",\"tool\":\"<nombre>\",\"input\":<objeto>}}\n\
         {{\"type\":\"final\",\"answer\":\"<texto>\",\"citations\":[{{\"job_id\":\"...\",\"generation_id\":\"...\",\"ordinal\":1,\"quote\":\"...\"}}],\"insufficient_evidence\":false}}\n\
         Reglas: una sola herramienta por turno; no inventes hechos ni fuentes; si no hay evidencia suficiente usá insufficient_evidence=true y citations=[]; cada cita debe copiar texto exacto de un fragmento devuelto por retrieval.search. No tenés SQL, shell, archivos ni operaciones fuera de la lista.\n\
         Herramientas permitidas:\n- {}\n\
         Esquemas de entrada disponibles (referencia): entity.get={}\n database.query={}\n changes.preview={}\n\
         Ejemplo mínimo de changes.preview: {{\"version\":1,\"batch_id\":\"<uuid>\",\"steps\":[{{\"key\":\"one\",\"operation\":\"entity.create\",\"input\":{{\"kind\":\"note\",\"title\":\"Nota\"}},\"bindings\":[]}}],\"checks\":[]}}. Para aplicar, usá el plan y el recibo exactos del preview.",
        tools.join("\n- "),
        schema("entity.get"),
        schema("database.query"),
        schema("changes.preview")
    ))
}

#[allow(clippy::too_many_arguments)]
fn dispatch(
    path: &Path,
    tool: &Tool,
    value: Value,
    allow_apply: bool,
    hits: &mut HashMap<(String, String, u32), RetrievalHit>,
    previews: &mut HashMap<(String, String), BatchReceipt>,
    scope: Option<&scoped::LocalScope>,
    run_id: &str,
) -> Result<Value> {
    match tool {
        Tool::Retrieval => {
            let input: RetrievalInput = serde_json::from_value(value)?;
            let page = semantic::search(path, input, scope)?;
            for hit in &page.items {
                hits.insert(
                    (
                        hit.job_id.clone(),
                        hit.generation_id.clone(),
                        hit.fragment_ordinal,
                    ),
                    hit.clone(),
                );
            }
            Ok(serde_json::to_value(page)?)
        }
        Tool::EntityGet => {
            let input: IdInput = serde_json::from_value(value)?;
            let w = open_read(path, scope)?;
            Ok(serde_json::to_value(w.entity_get(&input)?)?)
        }
        Tool::DatabaseQuery => {
            let input: DatabaseQuery = serde_json::from_value(value)?;
            let w = open_read(path, scope)?;
            Ok(serde_json::to_value(w.database_query(input)?)?)
        }
        Tool::ChangesPreview => {
            let plan: BatchPlan = serde_json::from_value(value)?;
            let preview = crate::batches::preview(path, plan, "agent", run_id)?;
            previews.insert(
                (
                    preview.receipt.plan_hash.clone(),
                    preview.receipt.base_hash.clone(),
                ),
                preview.receipt.clone(),
            );
            Ok(serde_json::to_value(preview)?)
        }
        Tool::ChangesApply => {
            if !allow_apply {
                return Err(agent_invalid("agent_apply_not_allowed"));
            }
            let apply: ApplyBatch = serde_json::from_value(value)?;
            let key = (
                apply.expected.plan_hash.clone(),
                apply.expected.base_hash.clone(),
            );
            if !previews.contains_key(&key) {
                return Err(agent_invalid("agent_apply_without_preview"));
            }
            let applied = crate::batches::apply(path, apply, "agent", run_id)?;
            previews.clear();
            Ok(serde_json::to_value(applied)?)
        }
    }
}

fn open_read(path: &Path, scope: Option<&scoped::LocalScope>) -> Result<Workspace> {
    match scope {
        Some(scope) => scope.open(false),
        None => Workspace::open(path),
    }
}

fn verify_citations(
    citations: &[CitationInput],
    insufficient: bool,
    hits: &HashMap<(String, String, u32), RetrievalHit>,
) -> Result<Vec<AgentCitation>> {
    if insufficient {
        if !citations.is_empty() {
            return Err(agent_invalid("citation_contradiction"));
        }
        return Ok(vec![]);
    }
    if citations.is_empty() {
        return Err(agent_invalid("citation_required"));
    }
    let mut out = Vec::with_capacity(citations.len());
    for citation in citations {
        if citation.quote.is_empty() || citation.quote.len() > MAX_QUOTE_BYTES {
            return Err(agent_invalid("citation_quote_limit"));
        }
        let hit = hits
            .get(&(
                citation.job_id.clone(),
                citation.generation_id.clone(),
                citation.ordinal,
            ))
            .ok_or_else(|| agent_invalid("citation_unverified"))?;
        let quote = normalize_ws(&citation.quote);
        if quote.is_empty() || !normalize_ws(&hit.text).contains(&quote) {
            return Err(agent_invalid("citation_unverified"));
        }
        out.push(AgentCitation {
            job_id: hit.job_id.clone(),
            attachment_id: hit.attachment_id.clone(),
            generation_id: hit.generation_id.clone(),
            extractor: hit.extractor.clone(),
            ordinal: hit.fragment_ordinal,
            quote: citation.quote.clone(),
            locator: hit.locator.clone(),
            text_sha256: hit.text_sha256.clone(),
            verified: true,
        });
    }
    Ok(out)
}

pub(crate) fn cancel(
    path: &Path,
    input: AgentCancelInput,
    scope: Option<&scoped::LocalScope>,
) -> Result<AgentCancelResult> {
    if uuid::Uuid::parse_str(&input.id).is_err() {
        return Err(agent_invalid("invalid_agent_id"));
    }
    let (c, _) = agent_storage::open(path, true)?;
    if let Some(scope) = scope {
        scope.check(&c)?;
    }
    Ok(AgentCancelResult {
        run: agent_storage::cancel(&c, &input.id)?,
    })
}

pub(crate) fn trace(
    path: &Path,
    input: AgentTraceInput,
    scope: Option<&scoped::LocalScope>,
) -> Result<AgentTraceResult> {
    let (c, _) = agent_storage::open(path, false)?;
    if let Some(scope) = scope {
        scope.check(&c)?;
    }
    match input {
        AgentTraceInput::Get { id } => {
            if uuid::Uuid::parse_str(&id).is_err() {
                return Err(agent_invalid("invalid_agent_id"));
            }
            Ok(AgentTraceResult::Get(agent_storage::get(&c, &id)?))
        }
        AgentTraceInput::List { limit, cursor } => {
            let limit = limit.unwrap_or(20);
            if limit == 0 || limit > 50 {
                return Err(agent_invalid("agent_page_limit"));
            }
            if cursor.as_ref().is_some_and(|s| s.len() > 2048) {
                return Err(agent_invalid("invalid_agent_cursor"));
            }
            Ok(AgentTraceResult::List(agent_storage::list(
                &c,
                limit,
                cursor.as_deref(),
            )?))
        }
    }
}

#[cfg(test)]
#[path = "agent_tests.rs"]
mod tests;
