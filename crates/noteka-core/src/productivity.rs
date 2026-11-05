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
fn visual(icon: &Option<String>, color: &Option<String>) -> Result<()> {
    if let Some(icon) = icon {
        if icon.is_empty()
            || icon.len() > 40
            || !icon
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err(CoreError::validation(
                "Icono inválido; usá una clave Lucide o image-UUID local en minúsculas.",
            ));
        }
    }
    if let Some(color) = color {
        let valid = color.len() == 7
            && color.starts_with('#')
            && color[1..].bytes().all(|b| b.is_ascii_hexdigit());
        if !valid {
            return Err(CoreError::validation(
                "Color de icono inválido; usá #RRGGBB.",
            ));
        }
    }
    Ok(())
}
fn optional_date(value: &Option<String>) -> Result<()> {
    if let Some(s) = value {
        civil_date(s)?;
    }
    Ok(())
}
pub(crate) fn validate_note_source(source: &NoteSource) -> Result<()> {
    if source.provider.is_empty()
        || source.provider.len() > 64
        || !source
            .provider
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
        || source.collection.trim().is_empty()
        || source.collection.len() > 512
        || source.collection.chars().any(char::is_control)
        || source
            .record_id
            .as_ref()
            .is_some_and(|s| s.is_empty() || s.len() > 128 || s.chars().any(char::is_control))
    {
        return Err(CoreError::validation("Procedencia de nota inválida."));
    }
    for date in [&source.created_at, &source.updated_at]
        .into_iter()
        .flatten()
    {
        if date.len() > 64 || chrono::DateTime::parse_from_rfc3339(date).is_err() {
            return Err(CoreError::validation(
                "Timestamp original de nota inválido.",
            ));
        }
    }
    Ok(())
}
pub(crate) fn validate_tag_activity(value: &str) -> Result<()> {
    if value.len() > 64 || chrono::DateTime::parse_from_rfc3339(value).is_err() {
        return Err(CoreError::validation(
            "Timestamp de actividad de tag inválido.",
        ));
    }
    Ok(())
}
pub(crate) fn validate_note_fields(fields: &NoteFields) -> Result<()> {
    visual(&fields.icon, &fields.icon_color)?;
    if let Some(source) = &fields.source {
        validate_note_source(source)?;
    }
    if fields.goal.is_some() && fields.milestone.is_some() {
        return Err(CoreError::validation(
            "Una página no puede ser Goal y Milestone a la vez.",
        ));
    }
    if let Some(goal) = &fields.goal {
        for date in [&goal.goal_set, &goal.target_deadline, &goal.achieved_on] {
            optional_date(date)?;
        }
        if let Some(snapshot) = &goal.activity_snapshot {
            validate_tag_activity(&snapshot.at)?;
            if snapshot.revisions.len() > 10_000
                || snapshot
                    .revisions
                    .iter()
                    .any(|(id, revision)| *revision == 0 || uuid::Uuid::parse_str(id).is_err())
            {
                return Err(CoreError::validation(
                    "Snapshot de actividad inválido o excesivo.",
                ));
            }
        }
    }
    if let Some(milestone) = &fields.milestone {
        optional_date(&milestone.target_deadline)?;
        optional_date(&milestone.completed_on)?;
        if milestone.goal_id.as_ref().is_some_and(|id| {
            uuid::Uuid::parse_str(id).is_err()
                || uuid::Uuid::parse_str(id).is_ok_and(|v| v.to_string() != *id)
        }) {
            return Err(CoreError::validation(
                "Goal de Milestone debe ser un ID canónico.",
            ));
        }
    }
    Ok(())
}
pub(crate) fn validate_person_fields(fields: &PersonFields) -> Result<()> {
    visual(&fields.icon, &fields.icon_color)?;
    for date in [&fields.birthday, &fields.check_in, &fields.last_check_in] {
        optional_date(date)?;
    }
    if let Some(source) = &fields.source {
        validate_note_source(source)?;
    }
    let invalid_labels = |labels: &Vec<String>| {
        labels.len() > 32
            || labels
                .iter()
                .any(|s| s.trim().is_empty() || s.len() > 128 || s.chars().any(char::is_control))
            || labels
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != labels.len()
    };
    let invalid_contact = [
        &fields.job_title,
        &fields.location,
        &fields.phone,
        &fields.email,
        &fields.instagram,
        &fields.linkedin,
        &fields.twitter,
        &fields.website,
    ]
    .iter()
    .any(|value| {
        value
            .as_ref()
            .is_some_and(|s| s.len() > 2048 || s.chars().any(char::is_control))
    });
    if invalid_contact
        || fields.interests.as_ref().is_some_and(invalid_labels)
        || fields
            .company
            .as_ref()
            .is_some_and(|s| s.len() > 512 || s.chars().any(char::is_control))
        || fields.relationships.as_ref().is_some_and(invalid_labels)
        || fields
            .photo
            .as_ref()
            .is_some_and(|s| uuid::Uuid::parse_str(s).is_err())
        || fields.photo_position.is_some_and(|position| position > 100)
        || fields.gallery_order.is_some_and(|order| order > 10_000)
    {
        return Err(CoreError::validation(
            "Metadatos de persona inválidos o excesivos.",
        ));
    }
    Ok(())
}
pub(crate) fn payload(kind: &EntityKind, value: Value) -> Result<Value> {
    let v = if value.is_null() { json!({}) } else { value };
    Ok(match kind {
        EntityKind::Note => {
            let f: NoteFields = serde_json::from_value(v)?;
            validate_note_fields(&f)?;
            serde_json::to_value(f)?
        }
        EntityKind::Task => {
            let f: TaskFields = serde_json::from_value(v)?;
            visual(&f.icon, &f.icon_color)?;
            optional_date(&f.due_date)?;
            optional_date(&f.planned_date)?;
            serde_json::to_value(f)?
        }
        EntityKind::Project => {
            let f: ProjectFields = serde_json::from_value(v)?;
            visual(&f.icon, &f.icon_color)?;
            if let Some(source) = &f.source {
                validate_note_source(source)?;
            }
            if f.board_order.is_some_and(|order| order > 10_000) {
                return Err(CoreError::validation("Orden de proyecto supera 10000."));
            }
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
        EntityKind::Tag => {
            let f: TagFields = serde_json::from_value(v)?;
            visual(&f.icon, &f.icon_color)?;
            if let Some(source) = &f.source {
                validate_note_source(source)?;
            }
            if let Some(activity) = &f.source_activity_at {
                validate_tag_activity(activity)?;
            }
            serde_json::to_value(f)?
        }
        EntityKind::Person => {
            let f: PersonFields = serde_json::from_value(v)?;
            validate_person_fields(&f)?;
            serde_json::to_value(f)?
        }
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
                // Derivado de relaciones: notas activas y tareas pendientes sin
                // relación Project. payload.inbox queda como dato heredado.
                "kind IN ('note','task') AND (kind='note' OR coalesce(json_extract(payload,'$.state'),'todo') IN ('todo','doing')) AND NOT EXISTS (SELECT 1 FROM relations r JOIN entities t ON t.id=r.target_id WHERE r.source_id=entities.id AND r.kind='project' AND t.kind='project')"
            }
            WorkflowView::MyDay => {
                "kind='task' AND coalesce(json_extract(payload,'$.state'),'todo') IN ('todo','doing') AND json_extract(payload,'$.planned_date')=?4"
            }
            WorkflowView::Pending => {
                "kind='task' AND coalesce(json_extract(payload,'$.state'),'todo') IN ('todo','doing')"
            }
            WorkflowView::Overdue => {
                "kind='task' AND coalesce(json_extract(payload,'$.state'),'todo') IN ('todo','doing') AND json_extract(payload,'$.due_date')<?4"
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
