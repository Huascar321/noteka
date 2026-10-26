//! Persistencia operativa del harness. Las trazas no son fuente de verdad y se
//! pueden borrar sin afectar entidades, documentos o índices.
use crate::{
    agent_contracts::*,
    error::*,
    workspace::{self, DB_NAME, SCHEMA_VERSION},
};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use serde_json::json;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

pub(crate) fn open(path: &Path, write: bool) -> Result<(Connection, PathBuf)> {
    let v = workspace::preflight(path)?;
    if v != SCHEMA_VERSION {
        let mut e = agent_error("migration_required");
        e.code = ErrorCode::UnsupportedWorkspaceVersion;
        e.details = json!({"current_version":v,"supported_version":SCHEMA_VERSION,"migration_required":v<SCHEMA_VERSION});
        return Err(e);
    }
    let root = std::fs::canonicalize(path)?;
    let c = Connection::open_with_flags(
        root.join(DB_NAME),
        (if write {
            OpenFlags::SQLITE_OPEN_READ_WRITE
        } else {
            OpenFlags::SQLITE_OPEN_READ_ONLY
        }) | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    c.busy_timeout(Duration::from_millis(workspace::BUSY_TIMEOUT_MS))?;
    if write {
        c.pragma_update(None, "foreign_keys", true)?;
        c.pragma_update(None, "synchronous", "FULL")?;
    }
    Ok((c, root))
}

fn timestamp(value: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|v| v.with_timezone(&Utc))
        .map_err(|_| agent_error("invalid_agent_timestamp"))
}
fn lease_expired(state: &str, lease: &str) -> bool {
    state == "running" && timestamp(lease).is_ok_and(|v| v < Utc::now())
}
fn state(value: &str) -> Result<AgentRunState> {
    match value {
        "running" => Ok(AgentRunState::Running),
        "succeeded" => Ok(AgentRunState::Succeeded),
        "failed" => Ok(AgentRunState::Failed),
        "cancelled" => Ok(AgentRunState::Cancelled),
        "limited" => Ok(AgentRunState::Limited),
        _ => Err(agent_error("invalid_agent_state")),
    }
}

type Row = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    Option<String>,
    String,
    i64,
    i64,
    Option<String>,
    Option<String>,
    String,
    Option<String>,
);
fn row_to_summary(row: Row) -> Result<AgentRunSummary> {
    let (
        id,
        state_text,
        provider,
        model,
        endpoint,
        question,
        started,
        finished,
        lease,
        cancel,
        steps,
        answer,
        citations,
        trace,
        error,
    ) = row;
    timestamp(&started)?;
    if let Some(finished) = &finished {
        timestamp(finished)?;
    }
    timestamp(&lease)?;
    if !(0..=64).contains(&steps) || !(0..=1).contains(&cancel) {
        return Err(agent_error("invalid_agent_run"));
    }
    if trace.len() > MAX_TRACE_BYTES || trace.len() < 2 {
        return Err(agent_error("invalid_agent_trace"));
    }
    let parsed: AgentStepTrace = serde_json::from_str(&trace)?;
    let citations: Vec<AgentCitation> = citations
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?
        .unwrap_or_default();
    if parsed.steps.len() != steps as usize {
        return Err(agent_error("invalid_agent_trace"));
    }
    Ok(AgentRunSummary {
        id,
        state: state(&state_text)?,
        provider,
        model,
        endpoint,
        question,
        started_at: started,
        finished_at: finished,
        lease_expired: lease_expired(&state_text, &lease),
        cancel_requested: cancel == 1,
        step_count: steps as u32,
        answer,
        citations,
        error,
    })
}

const COLUMNS: &str = "id,state,provider,model,endpoint,question,started_at,finished_at,lease_until,cancel_requested,step_count,answer,citations_json,trace_json,error";

pub(crate) fn get(c: &Connection, id: &str) -> Result<AgentTrace> {
    let row: Option<Row> = c
        .query_row(
            &format!("SELECT {COLUMNS} FROM agent_runs WHERE id=?1"),
            [id],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get(7)?,
                    r.get(8)?,
                    r.get(9)?,
                    r.get(10)?,
                    r.get(11)?,
                    r.get(12)?,
                    r.get(13)?,
                    r.get(14)?,
                ))
            },
        )
        .optional()?;
    let row = row.ok_or_else(|| agent_error("agent_not_found"))?;
    let trace: AgentStepTrace = serde_json::from_str(&row.13)?;
    let run = row_to_summary(row)?;
    Ok(AgentTrace {
        run,
        steps: trace.steps,
    })
}

pub(crate) fn list(c: &Connection, limit: u32, cursor: Option<&str>) -> Result<AgentTracePage> {
    let offset: u32 = cursor
        .map(serde_json::from_str)
        .transpose()
        .map_err(|_| agent_invalid("invalid_agent_cursor"))?
        .unwrap_or(0);
    if offset > 10_000 {
        return Err(agent_invalid("invalid_agent_cursor"));
    }
    let mut stmt = c.prepare(&format!(
        "SELECT {COLUMNS} FROM agent_runs ORDER BY started_at DESC,id LIMIT ?1 OFFSET ?2"
    ))?;
    let rows = stmt
        .query_map(params![limit + 1, offset], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
                r.get(7)?,
                r.get(8)?,
                r.get(9)?,
                r.get(10)?,
                r.get(11)?,
                r.get(12)?,
                r.get(13)?,
                r.get(14)?,
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut items = rows
        .into_iter()
        .map(row_to_summary)
        .collect::<Result<Vec<_>>>()?;
    for item in &mut items {
        item.citations.clear();
    }
    let next = if items.len() > limit as usize {
        items.pop();
        Some(serde_json::to_string(&(offset + limit)).expect("cursor"))
    } else {
        None
    };
    Ok(AgentTracePage {
        items,
        next_cursor: next,
    })
}

pub(crate) fn insert(
    c: &Connection,
    id: &str,
    provider: &AgentProviderConfig,
    question: &str,
    lease_until: &str,
) -> Result<()> {
    let trace = serde_json::to_string(&AgentStepTrace { steps: vec![] })?;
    c.execute(
        "INSERT INTO agent_runs(id,version,state,provider,model,endpoint,question,started_at,lease_until,trace_json) VALUES (?1,1,'running',?2,?3,?4,?5,?6,?7,?8)",
        params![
            id,
            provider.runtime,
            provider.model,
            provider.endpoint,
            question,
            workspace::now(),
            lease_until,
            trace
        ],
    )?;
    Ok(())
}

pub(crate) fn save_trace(c: &Connection, id: &str, steps: &[AgentStep]) -> Result<()> {
    let trace = serde_json::to_string(&AgentStepTrace {
        steps: steps.to_vec(),
    })?;
    if trace.len() > MAX_TRACE_BYTES {
        return Err(agent_error("agent_trace_size_limit"));
    }
    c.execute(
        "UPDATE agent_runs SET trace_json=?2,step_count=?3 WHERE id=?1 AND state='running'",
        params![id, trace, steps.len() as u32],
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn finish(
    c: &Connection,
    id: &str,
    state: AgentRunState,
    answer: Option<&str>,
    citations: &[AgentCitation],
    error: Option<&str>,
    steps: &[AgentStep],
) -> Result<AgentRunSummary> {
    let state = match state {
        AgentRunState::Running => return Err(agent_invalid("agent_state_running")),
        AgentRunState::Succeeded => "succeeded",
        AgentRunState::Failed => "failed",
        AgentRunState::Cancelled => "cancelled",
        AgentRunState::Limited => "limited",
    };
    let trace = serde_json::to_string(&AgentStepTrace {
        steps: steps.to_vec(),
    })?;
    if trace.len() > MAX_TRACE_BYTES {
        return Err(agent_error("agent_trace_size_limit"));
    }
    let citations_json = serde_json::to_string(citations)?;
    if citations_json.len() > 262_144 {
        return Err(agent_error("agent_citations_size_limit"));
    }
    let at = workspace::now();
    let changed = c.execute(
        "UPDATE agent_runs SET state=?2,finished_at=?3,answer=?4,citations_json=?5,error=?6,trace_json=?7,step_count=?8 WHERE id=?1 AND state='running'",
        params![
            id,
            state,
            at,
            answer,
            citations_json,
            error.map(|e| e.chars().take(4096).collect::<String>()),
            trace,
            steps.len() as u32
        ],
    )?;
    if changed == 0 {
        return Err(agent_invalid("agent_not_running"));
    }
    get(c, id).map(|trace| trace.run)
}

pub(crate) fn cancel(c: &Connection, id: &str) -> Result<AgentRunSummary> {
    let changed = c.execute(
        "UPDATE agent_runs SET cancel_requested=1 WHERE id=?1 AND state='running'",
        [id],
    )?;
    if changed == 0 {
        let exists: bool = c.query_row(
            "SELECT EXISTS(SELECT 1 FROM agent_runs WHERE id=?1)",
            [id],
            |r| r.get(0),
        )?;
        return Err(if exists {
            agent_invalid("agent_not_running")
        } else {
            agent_error("agent_not_found")
        });
    }
    get(c, id).map(|trace| trace.run)
}

pub(crate) fn cancel_requested(c: &Connection, id: &str) -> Result<bool> {
    c.query_row(
        "SELECT cancel_requested FROM agent_runs WHERE id=?1",
        [id],
        |r| r.get::<_, i64>(0),
    )
    .optional()?
    .map(|v| Ok(v == 1))
    .unwrap_or_else(|| Err(agent_error("agent_not_found")))
}

pub(crate) fn active_lease(c: &Connection) -> Result<Option<String>> {
    let now = workspace::now();
    Ok(c.query_row(
        "SELECT id FROM agent_runs WHERE state='running' AND lease_until>?1 ORDER BY started_at LIMIT 1",
        [now],
        |r| r.get(0),
    )
    .optional()?)
}

pub(crate) fn expire_leases(c: &Connection) -> Result<u32> {
    let now = workspace::now();
    Ok(c.execute(
        "UPDATE agent_runs SET state='failed',finished_at=?1,error='lease_expired' WHERE state='running' AND lease_until<=?1",
        [now],
    )? as u32)
}

pub(crate) fn prune(c: &Connection) -> Result<()> {
    c.execute(
        "DELETE FROM agent_runs WHERE id IN (SELECT id FROM agent_runs WHERE state<>'running' ORDER BY started_at DESC LIMIT -1 OFFSET ?1)",
        [AGENT_RETENTION],
    )?;
    Ok(())
}

pub(crate) fn integrity(c: &Connection) -> Result<()> {
    let rows = c
        .prepare(&format!("SELECT {COLUMNS} FROM agent_runs"))?
        .query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
                r.get(7)?,
                r.get(8)?,
                r.get(9)?,
                r.get(10)?,
                r.get(11)?,
                r.get(12)?,
                r.get(13)?,
                r.get(14)?,
            ))
        })?
        .collect::<std::result::Result<Vec<Row>, _>>()?;
    for row in rows {
        let trace_text = row.13.clone();
        let summary = row_to_summary(row)?;
        if summary.question.is_empty()
            || summary.question.len() > MAX_QUESTION_BYTES
            || summary
                .answer
                .as_ref()
                .is_some_and(|a| a.len() > MAX_ANSWER_BYTES)
            || summary.error.as_ref().is_some_and(|e| e.len() > 4096)
        {
            return Err(agent_error("invalid_agent_run"));
        }
        let _: AgentStepTrace = serde_json::from_str(&trace_text)?;
        if summary.state == AgentRunState::Succeeded && summary.answer.is_none() {
            return Err(agent_error("invalid_agent_run"));
        }
    }
    Ok(())
}
