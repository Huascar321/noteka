use crate::{contracts::*, error::*, productivity::civil_date, workspace::Workspace};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, Row, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

const CANDIDATES: usize = 10_000;
const SOURCES: usize = 10_000;
const LINKS: usize = 50_000;
const GROUPS: usize = 200;
const REF_FIELDS: &str = "e.id,e.kind,e.title,e.revision,e.created_at,e.updated_at,e.archived_at";
// -> conserva booleanos y distingue SQL NULL (omisión) del texto JSON null.
const META_FIELDS: &str = "json_type(e.payload),e.payload -> '$.state',e.payload -> '$.inbox',e.payload -> '$.priority',e.payload -> '$.due_date',e.payload -> '$.planned_date',e.payload -> '$.classification'";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cursor {
    version: u32,
    scope: String,
    view: HomeView,
    date: String,
    offset: u32,
    limit: u32,
}
struct ReadRow {
    entity: EntityRef,
    meta: HomeMetadata,
    title_key: String,
}
struct Edge {
    source: EntityRef,
    target: EntityRef,
    kind: String,
}
fn exceeded(name: &str, maximum: usize) -> CoreError {
    let mut e = CoreError::validation(
        "Consulta Home excede el límite; acotá el conjunto antes de consultar.",
    );
    e.details = serde_json::json!({"limit":name,"maximum":maximum});
    e
}
fn reference(r: &Row<'_>, start: usize) -> rusqlite::Result<EntityRef> {
    let kind: String = r.get(start + 1)?;
    let kind = match kind.as_str() {
        "note" => EntityKind::Note,
        "task" => EntityKind::Task,
        "project" => EntityKind::Project,
        "tag" => EntityKind::Tag,
        "person" => EntityKind::Person,
        _ => return Err(rusqlite::Error::InvalidQuery),
    };
    Ok(EntityRef {
        id: r.get(start)?,
        kind,
        title: r.get(start + 2)?,
        revision: r.get(start + 3)?,
        created_at: r.get(start + 4)?,
        updated_at: r.get(start + 5)?,
        archived_at: r.get(start + 6)?,
    })
}
fn field(raw: Option<String>) -> Option<Value> {
    raw.map(|s| serde_json::from_str(&s).unwrap_or(Value::Null))
}
fn enum_field(
    raw: Option<String>,
    default: Option<&str>,
    allowed: &[&str],
    name: &str,
    issues: &mut Vec<String>,
) -> Option<String> {
    if raw.is_none() {
        return default.map(str::to_owned);
    }
    let v = field(raw);
    let text = v.as_ref().and_then(Value::as_str);
    if let Some(text) = text.filter(|s| allowed.contains(s)) {
        Some(text.into())
    } else {
        issues.push(name.into());
        None
    }
}
fn date_field(raw: Option<String>, name: &str, issues: &mut Vec<String>) -> Option<String> {
    let v = field(raw)?;
    if v.is_null() {
        return None;
    }
    if let Some(s) = v.as_str().filter(|s| civil_date(s).is_ok()) {
        Some(s.into())
    } else {
        issues.push(name.into());
        None
    }
}
fn read_row(r: &Row<'_>, day: &str, tz: chrono_tz::Tz) -> rusqlite::Result<ReadRow> {
    let entity = reference(r, 0)?;
    let mut meta = HomeMetadata::default();
    let object: String = r.get(7)?;
    if object != "object" {
        meta.issues.push("payload".into());
    } else {
        match entity.kind {
            EntityKind::Task | EntityKind::Project => {
                let task = entity.kind == EntityKind::Task;
                meta.state = enum_field(
                    r.get(8)?,
                    Some(if task { "todo" } else { "planned" }),
                    if task {
                        &["todo", "done"]
                    } else {
                        &["planned", "doing", "ongoing", "done", "on_hold"]
                    },
                    "state",
                    &mut meta.issues,
                );
                if task {
                    meta.priority = enum_field(
                        r.get(10)?,
                        Some("normal"),
                        &["low", "normal", "high"],
                        "priority",
                        &mut meta.issues,
                    );
                    meta.due_date = date_field(r.get(11)?, "due_date", &mut meta.issues);
                    meta.planned_date = date_field(r.get(12)?, "planned_date", &mut meta.issues);
                }
            }
            EntityKind::Tag => {
                meta.classification = enum_field(
                    r.get(13)?,
                    None,
                    &["area", "resource", "topic"],
                    "classification",
                    &mut meta.issues,
                );
                if meta.classification.is_none()
                    && !meta.issues.iter().any(|i| i == "classification")
                {
                    meta.issues.push("classification".into());
                }
            }
            _ => {}
        }
        if matches!(entity.kind, EntityKind::Task | EntityKind::Note) {
            let raw: Option<String> = r.get(9)?;
            meta.inbox = if raw.is_none() {
                Some(true)
            } else {
                field(raw).and_then(|v| v.as_bool())
            };
            if meta.inbox.is_none() {
                meta.issues.push("inbox".into());
            }
        }
    }
    meta.updated_days_ago = DateTime::parse_from_rfc3339(&entity.updated_at)
        .ok()
        .map(|d| {
            (civil_date(day).expect("día validado") - d.with_timezone(&tz).date_naive()).num_days()
        });
    if meta.updated_days_ago.is_none() {
        meta.issues.push("updated_at".into());
    }
    Ok(ReadRow {
        title_key: entity.title.to_lowercase(),
        entity,
        meta,
    })
}
fn bad(m: &HomeMetadata, field: &str) -> bool {
    m.issues.iter().any(|i| i == "payload" || i == field)
}
fn matches_view(row: &ReadRow, view: HomeView, day: &str) -> Option<bool> {
    use HomeView::*;
    let m = &row.meta;
    let state = |states: &[&str]| {
        if bad(m, "state") {
            None
        } else {
            Some(m.state.as_deref().is_some_and(|s| states.contains(&s)))
        }
    };
    match view {
        TasksToday => {
            if state(&["todo"])? {
                if bad(m, "due_date") {
                    None
                } else {
                    Some(m.due_date.as_deref().is_some_and(|d| d <= day))
                }
            } else {
                Some(false)
            }
        }
        TasksInbox => {
            if state(&["todo"])? {
                m.inbox
            } else {
                Some(false)
            }
        }
        TasksDone => state(&["done"]),
        NotesInbox => m.inbox,
        ProjectsActive => state(&["doing", "ongoing"]),
        ProjectsPlanned => state(&["planned"]),
        _ => Some(true),
    }
}
fn kind(view: HomeView) -> &'static str {
    use HomeView::*;
    match view {
        TasksToday | TasksInbox | TasksAll | TasksDone => "task",
        NotesAll | NotesInbox => "note",
        ProjectsActive | ProjectsPlanned | ProjectsBoard => "project",
        TagsRecent | TagsAlphabetical | TagsTypes => "tag",
        PeopleContacts | PeopleNew | PeopleUpdated => "person",
    }
}
fn rank(value: Option<&str>, options: &[&str]) -> usize {
    options
        .iter()
        .position(|v| Some(*v) == value)
        .unwrap_or(options.len())
}
fn today_at(now: chrono::DateTime<Utc>, timezone: chrono_tz::Tz) -> String {
    now.with_timezone(&timezone).date_naive().to_string()
}
fn compare(a: &ReadRow, b: &ReadRow, view: HomeView) -> Ordering {
    use HomeView::*;
    let title = || a.title_key.cmp(&b.title_key);
    let order = match view {
        TasksToday => a
            .meta
            .due_date
            .cmp(&b.meta.due_date)
            .then(a.entity.created_at.cmp(&b.entity.created_at)),
        TasksInbox | TasksAll | NotesInbox => a.entity.created_at.cmp(&b.entity.created_at),
        TasksDone | NotesAll | TagsRecent | PeopleUpdated => {
            b.entity.updated_at.cmp(&a.entity.updated_at)
        }
        PeopleNew => b.entity.created_at.cmp(&a.entity.created_at),
        ProjectsActive | ProjectsBoard => rank(
            a.meta.state.as_deref(),
            &["planned", "doing", "ongoing", "done", "on_hold"],
        )
        .cmp(&rank(
            b.meta.state.as_deref(),
            &["planned", "doing", "ongoing", "done", "on_hold"],
        ))
        .then_with(title),
        TagsTypes => rank(
            a.meta.classification.as_deref(),
            &["area", "resource", "topic"],
        )
        .cmp(&rank(
            b.meta.classification.as_deref(),
            &["area", "resource", "topic"],
        ))
        .then_with(title),
        _ => title(),
    };
    order.then(a.entity.id.cmp(&b.entity.id))
}
fn group(key: &str, label: &str) -> HomeGroup {
    HomeGroup {
        key: key.into(),
        label: label.into(),
        entity_ids: vec![],
        project_id: None,
        project_archived: false,
        project_state: None,
        classification: None,
    }
}
fn compatible(edge: &Edge) -> bool {
    match edge.kind.as_str() {
        "project" => {
            matches!(edge.source.kind, EntityKind::Note | EntityKind::Task)
                && edge.target.kind == EntityKind::Project
        }
        "tag" => edge.source.kind != EntityKind::Tag && edge.target.kind == EntityKind::Tag,
        "person" => {
            edge.source.kind != EntityKind::Person && edge.target.kind == EntityKind::Person
        }
        _ => false,
    }
}
impl Workspace {
    pub fn home_query(&self, input: HomeQuery) -> Result<HomePage> {
        self.home_read(input, |_| Ok(()))
    }
    fn home_read(
        &self,
        input: HomeQuery,
        after_snapshot: impl FnOnce(&Connection) -> Result<()>,
    ) -> Result<HomePage> {
        let limit = input.limit.unwrap_or(50);
        if !(1..=100).contains(&limit) {
            return Err(CoreError::validation(
                "Home limit debe estar entre 1 y 100.",
            ));
        }
        let tx = self.conn.unchecked_transaction()?;
        let (workspace_id, timezone): (String, String) =
            tx.query_row("SELECT id,timezone FROM workspace_meta", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })?;
        after_snapshot(&tx)?;
        let tz: chrono_tz::Tz = timezone
            .parse()
            .map_err(|_| CoreError::validation("Zona de workspace inválida."))?;
        let scope = format!(
            "{workspace_id}:{:x}",
            Sha256::digest(
                std::fs::canonicalize(&self.path)?
                    .to_string_lossy()
                    .as_bytes()
            )
        );
        let cursor: Option<Cursor> = input
            .cursor
            .as_ref()
            .map(|s| {
                if s.len() > 2048 {
                    return Err(CoreError::validation("Cursor Home excede 2 KiB."));
                }
                Ok(serde_json::from_str::<Cursor>(s)?)
            })
            .transpose()?;
        let date = input
            .as_of
            .or_else(|| cursor.as_ref().map(|c| c.date.clone()))
            .unwrap_or_else(|| today_at(Utc::now(), tz));
        civil_date(&date)?;
        let offset = if let Some(c) = cursor {
            if c.version != 1
                || c.scope != scope
                || c.view != input.view
                || c.date != date
                || c.limit != limit
                || c.offset == 0
                || c.offset as usize > CANDIDATES
                || c.offset % limit != 0
            {
                return Err(CoreError::validation(
                    "Cursor Home incompatible con workspace, vista, día o página.",
                ));
            }
            c.offset
        } else {
            0
        };
        let mut stmt=tx.prepare(&format!("SELECT {REF_FIELDS},{META_FIELDS} FROM entities e WHERE e.kind=?1 AND e.archived_at IS NULL ORDER BY e.id LIMIT ?2"))?;
        let rows = stmt
            .query_map(params![kind(input.view), (CANDIDATES + 1) as u32], |r| {
                read_row(r, &date, tz)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if rows.len() > CANDIDATES {
            return Err(exceeded("candidates", CANDIDATES));
        }
        let mut unavailable_count = 0;
        let mut selected = rows
            .into_iter()
            .filter(|r| match matches_view(r, input.view, &date) {
                Some(v) => v,
                None => {
                    unavailable_count += 1;
                    false
                }
            })
            .collect::<Vec<_>>();
        selected.sort_by(|a, b| compare(a, b, input.view));
        let matching_count = selected.len() as u32;
        let page = selected
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect::<Vec<_>>();
        let ids: BTreeSet<String> = page.iter().map(|r| r.entity.id.clone()).collect();
        let id_json = serde_json::to_string(&ids)?;
        let refs_s = REF_FIELDS.replace("e.", "s.");
        let refs_t = REF_FIELDS.replace("e.", "t.");
        let mut stmt=tx.prepare(&format!("SELECT r.kind,{refs_s},{refs_t} FROM relations r JOIN entities s ON s.id=r.source_id JOIN entities t ON t.id=r.target_id WHERE r.source_id IN (SELECT value FROM json_each(?1)) OR r.target_id IN (SELECT value FROM json_each(?1)) LIMIT ?2"))?;
        let edges = stmt
            .query_map(params![id_json, (LINKS + 1) as u32], |r| {
                Ok(Edge {
                    kind: r.get(0)?,
                    source: reference(r, 1)?,
                    target: reference(r, 8)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if edges.len() > LINKS {
            return Err(exceeded("links", LINKS));
        }
        if edges.iter().any(|e| !compatible(e)) {
            return Err(CoreError::new(
                ErrorCode::StorageError,
                "Relación almacenada incompatible; Home no la convierte.",
            ));
        }
        let mut context: BTreeMap<String, BTreeMap<String, EntityRef>> = BTreeMap::new();
        let mut related: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut source_refs = BTreeMap::new();
        for e in &edges {
            for (me, other) in [(&e.source, &e.target), (&e.target, &e.source)] {
                if ids.contains(&me.id)
                    && matches!(
                        other.kind,
                        EntityKind::Project | EntityKind::Tag | EntityKind::Person
                    )
                {
                    context
                        .entry(me.id.clone())
                        .or_default()
                        .insert(other.id.clone(), other.clone());
                }
            }
            if ids.contains(&e.target.id)
                && e.source.archived_at.is_none()
                && ((e.kind == "project" && e.source.kind == EntityKind::Task)
                    || (e.kind == "tag" && e.source.kind == EntityKind::Note))
            {
                related
                    .entry(e.target.id.clone())
                    .or_default()
                    .insert(e.source.id.clone());
                source_refs.insert(e.source.id.clone(), e.source.clone());
            }
        }
        if source_refs.len() > SOURCES {
            return Err(exceeded("summary_sources", SOURCES));
        }
        let source_ids = serde_json::to_string(&source_refs.keys().collect::<Vec<_>>())?;
        let mut stmt=tx.prepare(&format!("SELECT {REF_FIELDS},{META_FIELDS} FROM entities e WHERE e.id IN (SELECT value FROM json_each(?1)) LIMIT ?2"))?;
        let sources = stmt
            .query_map(params![source_ids, (SOURCES + 1) as u32], |r| {
                read_row(r, &date, tz)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?
            .into_iter()
            .map(|r| (r.entity.id.clone(), r))
            .collect::<BTreeMap<_, _>>();
        let mut groups = BTreeMap::<String, HomeGroup>::new();
        use HomeView::*;
        if matches!(input.view, TasksInbox | TasksAll | TasksDone) {
            groups.insert("none".into(), group("none", "Sin proyecto"));
        }
        if matches!(input.view, ProjectsActive | ProjectsBoard) {
            for (state, label) in [
                ("planned", "Planned"),
                ("doing", "Doing"),
                ("ongoing", "Ongoing"),
                ("done", "Done"),
                ("on_hold", "On Hold"),
            ] {
                if input.view == ProjectsActive && !matches!(state, "doing" | "ongoing") {
                    continue;
                }
                let mut g = group(state, label);
                g.project_state = Some(serde_json::from_value(Value::String(state.into()))?);
                groups.insert(state.into(), g);
            }
        }
        if input.view == TagsTypes {
            for (t, label) in [
                ("area", "Area"),
                ("resource", "Resource"),
                ("topic", "Topic"),
            ] {
                let mut g = group(t, label);
                g.classification = Some(serde_json::from_value(Value::String(t.into()))?);
                groups.insert(t.into(), g);
            }
        }
        let mut items = vec![];
        for row in page {
            let id = &row.entity.id;
            let mut ctx = context
                .remove(id)
                .unwrap_or_default()
                .into_values()
                .collect::<Vec<_>>();
            ctx.sort_by(|a, b| {
                format!("{:?}", a.kind)
                    .cmp(&format!("{:?}", b.kind))
                    .then(a.title.to_lowercase().cmp(&b.title.to_lowercase()))
                    .then(a.id.cmp(&b.id))
            });
            if matches!(input.view, TasksInbox | TasksAll | TasksDone) {
                let projects = ctx
                    .iter()
                    .filter(|e| e.kind == EntityKind::Project)
                    .collect::<Vec<_>>();
                if projects.is_empty() {
                    groups.get_mut("none").unwrap().entity_ids.push(id.clone());
                }
                for p in projects {
                    let key = format!("project:{}", p.id);
                    let g = groups.entry(key.clone()).or_insert_with(|| {
                        let mut g = group(&key, &p.title);
                        g.project_id = Some(p.id.clone());
                        g.project_archived = p.archived_at.is_some();
                        g
                    });
                    g.entity_ids.push(id.clone());
                }
            } else if matches!(input.view, ProjectsActive | ProjectsBoard | TagsTypes) {
                let key = if input.view == TagsTypes {
                    row.meta.classification.as_deref()
                } else {
                    row.meta.state.as_deref()
                }
                .unwrap_or("unavailable");
                groups
                    .entry(key.into())
                    .or_insert_with(|| group(key, "No disponible"))
                    .entity_ids
                    .push(id.clone());
            }
            let summary = match row.entity.kind {
                EntityKind::Project => {
                    let refs = related.get(id).cloned().unwrap_or_default();
                    let total = refs.len() as u32;
                    let (mut pending, mut done, mut overdue) = (0, 0, 0);
                    let mut available = true;
                    for key in refs {
                        let s = &sources[&key];
                        if bad(&s.meta, "state") || bad(&s.meta, "due_date") {
                            available = false;
                            continue;
                        }
                        if s.meta.state.as_deref() == Some("done") {
                            done += 1;
                        } else {
                            pending += 1;
                            if s.meta
                                .due_date
                                .as_deref()
                                .is_some_and(|d| d < date.as_str())
                            {
                                overdue += 1;
                            }
                        }
                    }
                    Some(HomeSummary::Project {
                        available,
                        total: available.then_some(total),
                        pending: available.then_some(pending),
                        done: available.then_some(done),
                        overdue: available.then_some(overdue),
                        progress_percent: if available && total > 0 {
                            Some(
                                done.checked_mul(100).ok_or_else(|| {
                                    CoreError::validation("Progreso fuera de rango.")
                                })? / total,
                            )
                        } else {
                            None
                        },
                    })
                }
                EntityKind::Tag => {
                    let refs = related.get(id).cloned().unwrap_or_default();
                    // El conteo/última nota dependen de identidad y timestamp, no de Inbox/payload.
                    let available = refs
                        .iter()
                        .all(|id| !sources[id].meta.issues.iter().any(|i| i == "updated_at"));
                    let mut notes = refs
                        .iter()
                        .map(|id| source_refs[id].clone())
                        .collect::<Vec<_>>();
                    notes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at).then(a.id.cmp(&b.id)));
                    Some(HomeSummary::Tag {
                        available,
                        note_count: available.then_some(notes.len() as u32),
                        last_note: if available {
                            notes.into_iter().next()
                        } else {
                            None
                        },
                    })
                }
                _ => None,
            };
            let context_total = ctx.len() as u32;
            ctx.truncate(50);
            items.push(HomeItem {
                entity: row.entity,
                metadata: row.meta,
                context: ctx,
                context_total,
                summary,
            });
        }
        if groups.len() > GROUPS {
            return Err(exceeded("groups", GROUPS));
        }
        let mut groups = groups.into_values().collect::<Vec<_>>();
        groups.sort_by(|a, b| {
            if matches!(input.view, ProjectsActive | ProjectsBoard) {
                rank(
                    Some(&a.key),
                    &["planned", "doing", "ongoing", "done", "on_hold"],
                )
                .cmp(&rank(
                    Some(&b.key),
                    &["planned", "doing", "ongoing", "done", "on_hold"],
                ))
            } else if input.view == TagsTypes {
                rank(Some(&a.key), &["area", "resource", "topic"])
                    .cmp(&rank(Some(&b.key), &["area", "resource", "topic"]))
            } else {
                a.label
                    .to_lowercase()
                    .cmp(&b.label.to_lowercase())
                    .then(a.key.cmp(&b.key))
            }
        });
        let next = offset + items.len() as u32;
        let next_cursor = if next < matching_count {
            Some(serde_json::to_string(&Cursor {
                version: 1,
                scope,
                view: input.view,
                date: date.clone(),
                offset: next,
                limit,
            })?)
        } else {
            None
        };
        Ok(HomePage {
            view: input.view,
            date,
            timezone,
            items,
            groups,
            matching_count,
            unavailable_count,
            offset,
            next_cursor,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deterministic_civil_day_differs_from_utc_without_converting_due_dates() {
        let now = DateTime::parse_from_rfc3339("2026-10-11T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(today_at(now, chrono_tz::UTC), "2026-10-11");
        assert_eq!(today_at(now, chrono_tz::Pacific::Kiritimati), "2026-10-12");
    }
    #[test]
    fn real_wal_writer_cannot_mix_snapshot_selection_context_and_summary() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("w");
        let call = |op: &str, input: Value| {
            let response = crate::execute_json(
                serde_json::json!({"workspace":path,"command":{"operation":op,"input":input}}),
                "test",
            );
            assert!(response.ok, "{:?}", response.error);
            response.data.unwrap()
        };
        call("workspace.create", serde_json::json!({}));
        let project = call(
            "entity.create",
            serde_json::json!({"kind":"project","title":"Snapshot","payload":{"state":"doing"}}),
        );
        let task = call(
            "entity.create",
            serde_json::json!({"kind":"task","title":"Source"}),
        );
        call(
            "relation.add",
            serde_json::json!({"source_id":task["id"],"target_id":project["id"],"kind":"project","expected_revision":1}),
        );
        let w = Workspace::open(&path).unwrap();
        let query = || HomeQuery {
            view: HomeView::ProjectsActive,
            as_of: Some("2026-10-12".into()),
            limit: None,
            cursor: None,
        };
        let old = w.home_read(query(), |_| {
            call("entity.update", serde_json::json!({"id":task["id"],"expected_revision":2,"payload":{"state":"done"}}));
            call("entity.update", serde_json::json!({"id":project["id"],"expected_revision":1,"title":"Changed"}));
            Ok(())
        }).unwrap();
        assert_eq!(old.items[0].entity.title, "Snapshot");
        assert!(matches!(
            old.items[0].summary,
            Some(HomeSummary::Project {
                pending: Some(1),
                done: Some(0),
                ..
            })
        ));
        let new = w.home_query(query()).unwrap();
        assert_eq!(new.items[0].entity.title, "Changed");
        assert!(matches!(
            new.items[0].summary,
            Some(HomeSummary::Project {
                pending: Some(0),
                done: Some(1),
                ..
            })
        ));
    }
}
