use crate::{
    contracts::*,
    entities::{cursor, get, limit, page},
    error::*,
    workspace::Workspace,
};
use chrono::{Datelike, NaiveDate, Utc};
use rusqlite::params;
use serde_json::{Value, json};

pub(crate) fn civil_date(value: &str) -> Result<NaiveDate> {
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| CoreError::validation("Fecha civil inválida; usá YYYY-MM-DD."))?;
    if value.len() != 10 || date.to_string() != value || !(1..=9999).contains(&date.year()) {
        return Err(CoreError::validation(
            "Fecha civil debe ser canónica, entre 0001 y 9999.",
        ));
    }
    Ok(date)
}
fn optional_date(value: &Option<String>) -> Result<()> {
    if let Some(s) = value {
        civil_date(s)?;
    }
    Ok(())
}
pub(crate) fn payload(kind: &EntityKind, value: Value) -> Result<Value> {
    let v = if value.is_null() { json!({}) } else { value };
    Ok(match kind {
        EntityKind::Note => serde_json::to_value(serde_json::from_value::<NoteFields>(v)?)?,
        EntityKind::Task => {
            let f: TaskFields = serde_json::from_value(v)?;
            optional_date(&f.due_date)?;
            optional_date(&f.planned_date)?;
            serde_json::to_value(f)?
        }
        EntityKind::Project => {
            let f: ProjectFields = serde_json::from_value(v)?;
            optional_date(&f.start_date)?;
            optional_date(&f.end_date)?;
            if f.start_date
                .as_ref()
                .zip(f.end_date.as_ref())
                .is_some_and(|(s, e)| e < s)
            {
                return Err(CoreError::validation(
                    "El fin del proyecto precede al inicio.",
                ));
            }
            if f.purpose.len() > 16_384 {
                return Err(CoreError::validation("Propósito supera 16 KiB."));
            }
            serde_json::to_value(f)?
        }
        EntityKind::Tag => serde_json::to_value(serde_json::from_value::<TagFields>(v)?)?,
        EntityKind::Person => serde_json::to_value(serde_json::from_value::<Empty>(v)?)?,
    })
}

impl Workspace {
    pub fn workflow_query(&self, input: WorkflowQuery) -> Result<WorkflowPage> {
        let date = match &input.as_of {
            Some(s) => civil_date(s)?.to_string(),
            None => {
                let tz: chrono_tz::Tz = self
                    .info()?
                    .timezone
                    .parse()
                    .map_err(|_| CoreError::validation("Zona de workspace inválida."))?;
                Utc::now().with_timezone(&tz).date_naive().to_string()
            }
        };
        let n = limit(input.limit)?;
        let (time, key) = cursor(input.cursor.as_deref())?;
        if (input.view == WorkflowView::Context) != input.related_id.is_some() {
            return Err(CoreError::validation("Sólo context requiere related_id."));
        }
        let tx = self.conn.unchecked_transaction()?;
        if let Some(ref id) = input.related_id {
            let e = get(&tx, id)?;
            if !matches!(
                e.kind,
                EntityKind::Project | EntityKind::Tag | EntityKind::Person
            ) {
                return Err(CoreError::validation(
                    "Contexto debe ser proyecto, tag o persona.",
                ));
            }
        }
        let filter = match input.view {
            WorkflowView::Inbox => {
                "kind IN ('note','task') AND coalesce(json_extract(payload,'$.inbox'),1)=1 AND (kind='note' OR coalesce(json_extract(payload,'$.state'),'todo')='todo')"
            }
            WorkflowView::MyDay => {
                "kind='task' AND coalesce(json_extract(payload,'$.state'),'todo')='todo' AND json_extract(payload,'$.planned_date')=?4"
            }
            WorkflowView::Pending => {
                "kind='task' AND coalesce(json_extract(payload,'$.state'),'todo')='todo'"
            }
            WorkflowView::Overdue => {
                "kind='task' AND coalesce(json_extract(payload,'$.state'),'todo')='todo' AND json_extract(payload,'$.due_date')<?4"
            }
            WorkflowView::ActiveProjects => {
                "kind='project' AND coalesce(json_extract(payload,'$.state'),'planned') IN ('planned','doing','ongoing')"
            }
            WorkflowView::Context => {
                "EXISTS (SELECT 1 FROM relations r WHERE (r.source_id=entities.id AND r.target_id=?3) OR (r.target_id=entities.id AND r.source_id=?3))"
            }
        };
        // Sólo fragmentos estáticos del enum; nunca SQL proporcionado por el cliente.
        let sql = format!(
            "SELECT id FROM entities WHERE archived_at IS NULL AND (created_at,id)>(?1,?2) AND ({filter}) ORDER BY created_at,id LIMIT ?5"
        );
        let mut stmt = tx.prepare(&sql)?;
        let ids = stmt
            .query_map(params![time, key, input.related_id, date, n + 1], |r| {
                r.get::<_, String>(0)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let items = ids
            .iter()
            .map(|id| get(&tx, id))
            .collect::<Result<Vec<_>>>()?;
        let result = page(items, n, |e| json!([e.created_at, e.id]).to_string());
        Ok(WorkflowPage {
            items: result.items,
            next_cursor: result.next_cursor,
            date,
        })
    }
}
