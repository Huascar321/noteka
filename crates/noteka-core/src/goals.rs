//! Goals are typed pages; reads never expand or replace their documents.
use crate::{
    contracts::*,
    error::*,
    mutations::{MutationContext, ObjectKind},
    productivity::{civil_date, validate_note_fields},
    workspace::Workspace,
};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const CANDIDATES: usize = 10_000;
const SOURCES: usize = 10_000;
const LINKS: usize = 50_000;
const GROUPS: usize = 200;
const CONTEXT: usize = 8;

pub(crate) fn validate_note_links(
    conn: &Connection,
    entity_id: Option<&str>,
    value: &Value,
    previous: Option<&Value>,
    context: &mut MutationContext<'_>,
) -> Result<()> {
    let fields: NoteFields = serde_json::from_value(value.clone())?;
    if let Some(id) = entity_id {
        if fields.goal.is_some() {
            let tags: u32 = conn.query_row(
                "SELECT count(*) FROM relations WHERE source_id=?1 AND kind='tag'",
                [id],
                |r| r.get(0),
            )?;
            if tags > 1 {
                return Err(CoreError::validation("Un Goal admite un solo tag directo."));
            }
        } else if previous
            .and_then(|p| p.get("goal"))
            .is_some_and(|g| !g.is_null())
        {
            let linked: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM entities WHERE kind='note' AND json_extract(payload,'$.milestone.goal_id')=?1)", [id], |r| r.get(0))?;
            if linked {
                return Err(CoreError::validation(
                    "El Goal tiene hitos vinculados; no se puede quitar su tipo.",
                ));
            }
        }
    }
    if let Some(goal_id) = fields.milestone.as_ref().and_then(|m| m.goal_id.as_ref()) {
        let target: Option<(String, String, Option<String>)> = conn
            .query_row(
                "SELECT kind,payload,archived_at FROM entities WHERE id=?1",
                [goal_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        let Some((kind, payload, archived)) = target else {
            return Err(CoreError::validation(
                "El Goal del hito no existe en este workspace.",
            ));
        };
        let goal: NoteFields = serde_json::from_str(&payload)
            .map_err(|_| CoreError::validation("El destino no es un Goal interpretable."))?;
        if kind != "note" || goal.goal.is_none() || validate_note_fields(&goal).is_err() {
            return Err(CoreError::validation(
                "El destino del hito debe ser un Goal.",
            ));
        }
        let old_id = previous
            .and_then(|p| p.pointer("/milestone/goal_id"))
            .and_then(Value::as_str);
        if old_id != Some(goal_id)
            && (archived.is_some() || goal.source.as_ref().is_some_and(|s| s.archived))
        {
            return Err(CoreError::validation(
                "No se puede asignar un hito nuevo a un Goal archivado.",
            ));
        }
        context.observe(conn, ObjectKind::Entity, &[goal_id])?;
    }
    Ok(())
}

pub(crate) fn validate_tag_link(
    conn: &Connection,
    source: &Entity,
    target: &Entity,
    kind: &RelationKind,
) -> Result<()> {
    if *kind == RelationKind::Tag
        && source.kind == EntityKind::Note
        && source.payload.get("goal").is_some_and(Value::is_object)
    {
        let other: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM relations WHERE source_id=?1 AND kind='tag' AND target_id<>?2)", params![source.id,target.id], |r| r.get(0))?;
        if other {
            return Err(CoreError::validation("Un Goal admite un solo tag directo."));
        }
    }
    Ok(())
}

pub(crate) fn is_goal_view(view: HomeView) -> bool {
    matches!(
        view,
        HomeView::GoalsActivity
            | HomeView::GoalsDeadline
            | HomeView::GoalsByTag
            | HomeView::GoalsAchieved
            | HomeView::MilestonesCompleted
            | HomeView::MilestonesAll
    )
}
fn exceeded(name: &str, maximum: usize) -> CoreError {
    let mut error = CoreError::validation("Consulta Goals excede el límite; acotá el conjunto.");
    error.details = serde_json::json!({"limit":name,"maximum":maximum});
    error
}
fn instant(value: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|d| d.timestamp_millis())
}
fn state_key(state: &GoalState) -> &'static str {
    match state {
        GoalState::Active => "active",
        GoalState::Dream => "dream",
        GoalState::Achieved => "achieved",
    }
}

#[derive(Clone)]
struct Source {
    reference: EntityRef,
    payload: Value,
    note: Option<NoteFields>,
    source_archived: bool,
}
fn source(row: &rusqlite::Row<'_>) -> rusqlite::Result<Source> {
    let payload: String = row.get(7)?;
    let payload =
        serde_json::from_str::<Value>(&payload).map_err(|_| rusqlite::Error::InvalidQuery)?;
    let kind: String = row.get(1)?;
    let kind = serde_json::from_value::<EntityKind>(Value::String(kind))
        .map_err(|_| rusqlite::Error::InvalidQuery)?;
    let note = if kind == EntityKind::Note {
        serde_json::from_value::<NoteFields>(payload.clone())
            .ok()
            .filter(|f| validate_note_fields(f).is_ok())
    } else {
        None
    };
    let source_archived = payload
        .pointer("/source/archived")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let reference = EntityRef {
        id: row.get(0)?,
        kind,
        title: row.get(2)?,
        revision: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
        archived_at: row.get(6)?,
        icon: payload
            .get("icon")
            .and_then(Value::as_str)
            .map(str::to_owned),
        icon_color: payload
            .get("icon_color")
            .and_then(Value::as_str)
            .map(str::to_owned),
    };
    Ok(Source {
        reference,
        payload,
        note,
        source_archived,
    })
}
const FIELDS: &str = "id,kind,title,revision,created_at,updated_at,archived_at,payload";
fn live(source: &Source) -> bool {
    source.reference.archived_at.is_none() && !source.source_archived
}
fn load_sources(
    conn: &Connection,
    ids: &BTreeSet<String>,
    sources: &mut BTreeMap<String, Source>,
) -> Result<()> {
    if ids.len() > SOURCES {
        return Err(exceeded("summary_sources", SOURCES));
    }
    let json = serde_json::to_string(ids)?;
    let mut statement = conn.prepare(&format!(
        "SELECT {FIELDS} FROM entities WHERE id IN (SELECT value FROM json_each(?1))"
    ))?;
    for row in statement.query_map([json], source)? {
        let row = row?;
        sources.insert(row.reference.id.clone(), row);
    }
    Ok(())
}
#[derive(Clone)]
struct Edge {
    source: String,
    target: String,
    kind: String,
}
fn load_edges(conn: &Connection, ids: &BTreeSet<String>) -> Result<Vec<Edge>> {
    let json = serde_json::to_string(ids)?;
    let mut statement = conn.prepare("SELECT source_id,target_id,kind FROM relations WHERE source_id IN (SELECT value FROM json_each(?1)) OR (kind='project' AND target_id IN (SELECT value FROM json_each(?1))) ORDER BY id LIMIT ?2")?;
    let edges = statement
        .query_map(params![json, (LINKS + 1) as u32], |r| {
            Ok(Edge {
                source: r.get(0)?,
                target: r.get(1)?,
                kind: r.get(2)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if edges.len() > LINKS {
        return Err(exceeded("links", LINKS));
    }
    Ok(edges)
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cursor {
    version: u32,
    scope: String,
    view: HomeView,
    date: String,
    goal_id: Option<String>,
    offset: u32,
    limit: u32,
}
struct GoalRow {
    item: HomeItem,
    activity: Option<i64>,
    title_key: String,
}

fn activity(
    goal: &Source,
    fields: &GoalFields,
    related: &BTreeSet<String>,
    sources: &BTreeMap<String, Source>,
) -> Option<String> {
    let snapshot = fields.activity_snapshot.as_ref();
    let mut dates = Vec::<(i64, String)>::new();
    if let Some(s) = snapshot {
        if let Some(at) = instant(&s.at) {
            dates.push((at, s.at.clone()));
        }
    }
    for id in std::iter::once(&goal.reference.id).chain(related.iter()) {
        let Some(entity) = sources.get(id) else {
            continue;
        };
        if snapshot.is_some_and(|s| s.revisions.get(id) == Some(&entity.reference.revision)) {
            continue;
        }
        let value = if snapshot.is_some() {
            Some(entity.reference.updated_at.as_str())
        } else {
            entity
                .payload
                .pointer("/source/updated_at")
                .and_then(Value::as_str)
                .or(Some(entity.reference.updated_at.as_str()))
        };
        if let Some(value) = value {
            if let Some(at) = instant(value) {
                dates.push((at, value.to_owned()));
            }
        }
    }
    dates.sort();
    dates.pop().map(|(_, value)| value)
}

impl Workspace {
    pub(crate) fn goal_home_query(&self, input: HomeQuery) -> Result<HomePage> {
        let limit = input.limit.unwrap_or(50);
        if !(1..=100).contains(&limit) {
            return Err(CoreError::validation(
                "Home limit debe estar entre 1 y 100.",
            ));
        }
        if input.goal_id.is_some() && input.view != HomeView::MilestonesAll {
            return Err(CoreError::validation(
                "goal_id sólo se admite en milestones_all.",
            ));
        }
        let tx = self.conn.unchecked_transaction()?;
        let (workspace_id, timezone): (String, String) =
            tx.query_row("SELECT id,timezone FROM workspace_meta", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })?;
        let tz: chrono_tz::Tz = timezone
            .parse()
            .map_err(|_| CoreError::validation("Zona de workspace inválida."))?;
        let scope = format!(
            "goals-v1:{workspace_id}:{:x}",
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
                Ok(serde_json::from_str(s)?)
            })
            .transpose()?;
        let date = input
            .as_of
            .clone()
            .or_else(|| cursor.as_ref().map(|c| c.date.clone()))
            .unwrap_or_else(|| Utc::now().with_timezone(&tz).date_naive().to_string());
        civil_date(&date)?;
        let offset = if let Some(cursor) = cursor {
            if cursor.version != 1
                || cursor.scope != scope
                || cursor.view != input.view
                || cursor.date != date
                || cursor.goal_id != input.goal_id
                || cursor.limit != limit
                || cursor.offset == 0
                || cursor.offset as usize > CANDIDATES
                || cursor.offset % limit != 0
            {
                return Err(CoreError::validation(
                    "Cursor Goals incompatible con workspace, vista, día, Goal o página.",
                ));
            }
            cursor.offset
        } else {
            0
        };
        let mut statement=tx.prepare(&format!("SELECT {FIELDS} FROM entities WHERE kind='note' AND (archived_at IS NULL OR json_type(payload,'$.milestone')='object') ORDER BY id LIMIT ?1"))?;
        let notes = statement
            .query_map([(CANDIDATES + 1) as u32], source)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if notes.len() > CANDIDATES {
            return Err(exceeded("candidates", CANDIDATES));
        }
        let mut sources = notes
            .into_iter()
            .map(|s| (s.reference.id.clone(), s))
            .collect::<BTreeMap<_, _>>();
        let mut referenced_goals = sources
            .values()
            .filter_map(|s| {
                s.note
                    .as_ref()
                    .and_then(|n| n.milestone.as_ref())
                    .and_then(|m| m.goal_id.clone())
            })
            .collect::<BTreeSet<_>>();
        if let Some(id) = &input.goal_id {
            if uuid::Uuid::parse_str(id).is_err() {
                return Err(CoreError::validation("Goal inválido."));
            }
            referenced_goals.insert(id.clone());
        }
        load_sources(&tx, &referenced_goals, &mut sources)?;
        if let Some(id) = &input.goal_id {
            if sources
                .get(id)
                .and_then(|s| s.note.as_ref())
                .and_then(|n| n.goal.as_ref())
                .is_none()
            {
                return Err(CoreError::validation(
                    "El filtro requiere un Goal existente.",
                ));
            }
        }
        let goal_ids = sources
            .values()
            .filter(|s| s.note.as_ref().is_some_and(|n| n.goal.is_some()))
            .map(|s| s.reference.id.clone())
            .collect::<BTreeSet<_>>();
        let mut edges = load_edges(&tx, &goal_ids)?;
        let project_ids = edges
            .iter()
            .filter(|e| e.kind == "project" && goal_ids.contains(&e.source))
            .map(|e| e.target.clone())
            .collect::<BTreeSet<_>>();
        let project_edges = load_edges(&tx, &project_ids)?;
        let mut unique = edges
            .iter()
            .map(|e| (e.source.clone(), e.target.clone(), e.kind.clone()))
            .collect::<BTreeSet<_>>();
        for edge in project_edges {
            if unique.insert((edge.source.clone(), edge.target.clone(), edge.kind.clone())) {
                edges.push(edge);
            }
        }
        if edges.len() > LINKS {
            return Err(exceeded("links", LINKS));
        }
        let mut ids = edges
            .iter()
            .flat_map(|e| [e.source.clone(), e.target.clone()])
            .chain(goal_ids.iter().cloned())
            .collect::<BTreeSet<_>>();
        ids.extend(referenced_goals);
        ids.extend(
            sources
                .values()
                .filter(|source| {
                    source
                        .note
                        .as_ref()
                        .is_some_and(|note| note.milestone.is_some())
                })
                .map(|source| source.reference.id.clone()),
        );
        load_sources(&tx, &ids, &mut sources)?;
        let mut goal_projects = BTreeMap::<String, BTreeSet<String>>::new();
        let mut goal_tags = BTreeMap::<String, BTreeSet<String>>::new();
        let mut project_sources = BTreeMap::<String, BTreeSet<String>>::new();
        for edge in &edges {
            let (Some(from), Some(to)) = (sources.get(&edge.source), sources.get(&edge.target))
            else {
                return Err(CoreError::validation(
                    "Relación Goals sin extremo recuperable.",
                ));
            };
            if edge.kind == "project" {
                if !matches!(from.reference.kind, EntityKind::Note | EntityKind::Task)
                    || to.reference.kind != EntityKind::Project
                {
                    return Err(CoreError::validation("Relación de proyecto incompatible."));
                }
                if goal_ids.contains(&edge.source) {
                    goal_projects
                        .entry(edge.source.clone())
                        .or_default()
                        .insert(edge.target.clone());
                }
                let ordinary = from.reference.kind == EntityKind::Task
                    || from
                        .note
                        .as_ref()
                        .is_some_and(|n| n.goal.is_none() && n.milestone.is_none());
                if ordinary {
                    project_sources
                        .entry(edge.target.clone())
                        .or_default()
                        .insert(edge.source.clone());
                }
            } else if edge.kind == "tag" && goal_ids.contains(&edge.source) {
                if to.reference.kind != EntityKind::Tag {
                    return Err(CoreError::validation("Relación de tag incompatible."));
                }
                goal_tags
                    .entry(edge.source.clone())
                    .or_default()
                    .insert(edge.target.clone());
            }
        }
        let mut goal_milestones = BTreeMap::<String, Vec<&Source>>::new();
        let mut bad_milestones = BTreeSet::new();
        let mut activity_milestones = BTreeMap::<String, BTreeSet<String>>::new();
        let mut unavailable_count = 0;
        for source in sources.values() {
            if let Some(goal_id) = source
                .note
                .as_ref()
                .and_then(|n| n.milestone.as_ref())
                .and_then(|m| m.goal_id.as_ref())
            {
                activity_milestones
                    .entry(goal_id.clone())
                    .or_default()
                    .insert(source.reference.id.clone());
            }
        }
        for source in sources
            .values()
            .filter(|s| s.reference.kind == EntityKind::Note && live(s))
        {
            match source.note.as_ref() {
                Some(note) => {
                    if let Some(goal_id) = note.milestone.as_ref().and_then(|m| m.goal_id.as_ref())
                    {
                        goal_milestones
                            .entry(goal_id.clone())
                            .or_default()
                            .push(source);
                    }
                }
                None => {
                    if let Some(goal_id) = source
                        .payload
                        .pointer("/milestone/goal_id")
                        .and_then(Value::as_str)
                    {
                        bad_milestones.insert(goal_id.to_owned());
                    }
                    let relevant = if matches!(
                        input.view,
                        HomeView::MilestonesCompleted | HomeView::MilestonesAll
                    ) {
                        source
                            .payload
                            .get("milestone")
                            .is_some_and(|m| !m.is_null())
                    } else {
                        source.payload.get("goal").is_some_and(|g| !g.is_null())
                    };
                    if relevant {
                        unavailable_count += 1;
                    }
                }
            }
        }
        let mut rows = Vec::<GoalRow>::new();
        let milestone_view = matches!(
            input.view,
            HomeView::MilestonesCompleted | HomeView::MilestonesAll
        );
        for source in sources
            .values()
            .filter(|s| s.reference.kind == EntityKind::Note && live(s))
        {
            let Some(note) = &source.note else { continue };
            let mut metadata = HomeMetadata::default();
            let mut context = BTreeMap::<String, EntityRef>::new();
            let mut summary = None;
            let mut activity_at = None;
            if milestone_view {
                let Some(milestone) = &note.milestone else {
                    continue;
                };
                if input.view == HomeView::MilestonesCompleted && milestone.completed_on.is_none() {
                    continue;
                }
                if input
                    .goal_id
                    .as_ref()
                    .is_some_and(|id| milestone.goal_id.as_ref() != Some(id))
                {
                    continue;
                }
                if let Some(id) = &milestone.goal_id {
                    if let Some(goal) = sources
                        .get(id)
                        .filter(|g| g.note.as_ref().is_some_and(|n| n.goal.is_some()))
                    {
                        context.insert(id.clone(), goal.reference.clone());
                        for tag in goal_tags.get(id).into_iter().flatten() {
                            if let Some(tag) = sources.get(tag) {
                                context.insert(tag.reference.id.clone(), tag.reference.clone());
                            }
                        }
                    } else {
                        metadata.issues.push("goal_unavailable".into());
                    }
                }
                metadata.milestone = Some(milestone.clone());
            } else {
                let Some(goal) = &note.goal else { continue };
                if input.view == HomeView::GoalsAchieved && goal.state != GoalState::Achieved {
                    continue;
                }
                if input.view == HomeView::GoalsByTag && goal.state != GoalState::Active {
                    continue;
                }
                let milestones = goal_milestones.get(&source.reference.id);
                let total = milestones.map_or(0, |m| m.len() as u32);
                let done = milestones.map_or(0, |m| {
                    m.iter()
                        .filter(|m| {
                            m.note
                                .as_ref()
                                .and_then(|n| n.milestone.as_ref())
                                .is_some_and(|m| m.completed_on.is_some())
                        })
                        .count() as u32
                });
                let available = !bad_milestones.contains(&source.reference.id);
                summary = Some(HomeSummary::Goal {
                    available,
                    total: available.then_some(total),
                    done: available.then_some(done),
                    progress_tenths: if available && total > 0 {
                        Some((done * 1000 + total / 2) / total)
                    } else {
                        None
                    },
                });
                if !available {
                    metadata.issues.push("milestone_metadata".into());
                }
                let mut related = BTreeSet::new();
                for project_id in goal_projects
                    .get(&source.reference.id)
                    .into_iter()
                    .flatten()
                {
                    related.insert(project_id.clone());
                    if let Some(project) = sources.get(project_id) {
                        context.insert(project_id.clone(), project.reference.clone());
                    }
                    related.extend(
                        project_sources
                            .get(project_id)
                            .into_iter()
                            .flatten()
                            .cloned(),
                    );
                }
                for tag_id in goal_tags.get(&source.reference.id).into_iter().flatten() {
                    if let Some(tag) = sources.get(tag_id) {
                        context.insert(tag_id.clone(), tag.reference.clone());
                    }
                }
                related.extend(
                    activity_milestones
                        .get(&source.reference.id)
                        .into_iter()
                        .flatten()
                        .cloned(),
                );
                let value = activity(source, goal, &related, &sources);
                activity_at = value.as_deref().and_then(instant);
                metadata.goal_activity_at = value;
                metadata.state = Some(state_key(&goal.state).to_owned());
                let mut visible = goal.clone();
                visible.activity_snapshot = None;
                metadata.goal = Some(visible);
            }
            let context_total = context.len() as u32;
            let mut context = context.into_values().collect::<Vec<_>>();
            context.sort_by(|a, b| {
                (a.kind != EntityKind::Tag)
                    .cmp(&(b.kind != EntityKind::Tag))
                    .then_with(|| a.id.cmp(&b.id))
            });
            rows.push(GoalRow {
                item: HomeItem {
                    entity: source.reference.clone(),
                    metadata,
                    context: context.into_iter().take(CONTEXT).collect(),
                    context_total,
                    summary,
                },
                activity: activity_at,
                title_key: source.reference.title.to_lowercase(),
            });
        }
        rows.sort_by(|a, b| {
            let order = match input.view {
                HomeView::GoalsActivity => b.activity.cmp(&a.activity),
                HomeView::GoalsDeadline => {
                    let left = a
                        .item
                        .metadata
                        .goal
                        .as_ref()
                        .and_then(|g| g.target_deadline.as_ref());
                    let right = b
                        .item
                        .metadata
                        .goal
                        .as_ref()
                        .and_then(|g| g.target_deadline.as_ref());
                    left.is_none()
                        .cmp(&right.is_none())
                        .then_with(|| left.cmp(&right))
                }
                HomeView::GoalsAchieved => b
                    .item
                    .metadata
                    .goal
                    .as_ref()
                    .and_then(|g| g.achieved_on.as_ref())
                    .cmp(
                        &a.item
                            .metadata
                            .goal
                            .as_ref()
                            .and_then(|g| g.achieved_on.as_ref()),
                    ),
                HomeView::MilestonesCompleted => b
                    .item
                    .metadata
                    .milestone
                    .as_ref()
                    .and_then(|m| m.completed_on.as_ref())
                    .cmp(
                        &a.item
                            .metadata
                            .milestone
                            .as_ref()
                            .and_then(|m| m.completed_on.as_ref()),
                    ),
                _ => std::cmp::Ordering::Equal,
            };
            order
                .then_with(|| a.title_key.cmp(&b.title_key))
                .then_with(|| a.item.entity.id.cmp(&b.item.entity.id))
        });
        let matching_count = rows.len() as u32;
        let items = rows
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .map(|r| r.item)
            .collect::<Vec<_>>();
        let mut groups = Vec::new();
        let group = |key: &str,
                     label: &str,
                     entity_ids: Vec<String>,
                     related_entity: Option<EntityRef>| HomeGroup {
            key: key.into(),
            label: label.into(),
            entity_ids,
            project_id: None,
            project_archived: false,
            project_state: None,
            classification: None,
            related_entity,
        };
        if !milestone_view && input.view != HomeView::GoalsByTag {
            for (key, label) in [
                ("active", "Active"),
                ("dream", "Dream"),
                ("achieved", "Achieved"),
            ] {
                if input.view == HomeView::GoalsAchieved && key != "achieved" {
                    continue;
                }
                groups.push(group(
                    key,
                    label,
                    items
                        .iter()
                        .filter(|i| i.metadata.state.as_deref() == Some(key))
                        .map(|i| i.entity.id.clone())
                        .collect(),
                    None,
                ));
            }
        } else if input.view == HomeView::GoalsByTag {
            let mut tags = BTreeMap::<String, Vec<String>>::new();
            for item in &items {
                match goal_tags
                    .get(&item.entity.id)
                    .filter(|tags| !tags.is_empty())
                {
                    Some(ids) => {
                        for id in ids {
                            tags.entry(id.clone())
                                .or_default()
                                .push(item.entity.id.clone());
                        }
                    }
                    None => {
                        tags.entry(String::new())
                            .or_default()
                            .push(item.entity.id.clone());
                    }
                }
            }
            if tags.len() > GROUPS {
                return Err(exceeded("groups", GROUPS));
            }
            let mut ordered = tags
                .into_iter()
                .map(|(id, ids)| {
                    let tag = sources.get(&id).map(|s| s.reference.clone());
                    let label = tag
                        .as_ref()
                        .map_or("Sin tag", |t| t.title.as_str())
                        .to_owned();
                    group(&format!("tag:{id}"), &label, ids, tag)
                })
                .collect::<Vec<_>>();
            ordered.sort_by(|a, b| {
                a.label
                    .to_lowercase()
                    .cmp(&b.label.to_lowercase())
                    .then_with(|| a.key.cmp(&b.key))
            });
            groups = ordered;
        }
        let next_offset = offset + items.len() as u32;
        let next_cursor = if next_offset < matching_count {
            Some(serde_json::to_string(&Cursor {
                version: 1,
                scope,
                view: input.view,
                date: date.clone(),
                goal_id: input.goal_id,
                offset: next_offset,
                limit,
            })?)
        } else {
            None
        };
        drop(statement);
        tx.commit()?;
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
