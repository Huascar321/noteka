use crate::{contracts::*, error::*, productivity::civil_date, workspace::Workspace};
use chrono::{DateTime, Datelike, Months, NaiveDate, Utc};
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
const REF_FIELDS: &str = "e.id,e.kind,e.title,e.revision,e.created_at,e.updated_at,e.archived_at,json_extract(e.payload,'$.icon'),json_extract(e.payload,'$.icon_color')";
// -> conserva booleanos y distingue SQL NULL (omisión) del texto JSON null.
const META_FIELDS: &str = "json_type(e.payload),e.payload -> '$.state',e.payload -> '$.inbox',e.payload -> '$.priority',e.payload -> '$.due_date',e.payload -> '$.planned_date',e.payload -> '$.classification',e.payload";

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
    source_archived: bool,
    project_order: Option<u32>,
    activity_at: Option<i64>,
    person_library: bool,
}
struct Edge {
    source: EntityRef,
    target: EntityRef,
    kind: String,
    source_is_goal: bool,
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
        icon: r.get(start + 7)?,
        icon_color: r.get(start + 8)?,
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
    let mut source_archived = false;
    let mut project_order = None;
    let mut activity_at = None;
    let mut person_library = false;
    let object: String = r.get(9)?;
    if object != "object" {
        meta.issues.push("payload".into());
    } else {
        match entity.kind {
            EntityKind::Task | EntityKind::Project => {
                let task = entity.kind == EntityKind::Task;
                meta.state = enum_field(
                    r.get(10)?,
                    Some(if task { "todo" } else { "planned" }),
                    if task {
                        &["todo", "doing", "done"]
                    } else {
                        &["planned", "doing", "ongoing", "done", "on_hold"]
                    },
                    "state",
                    &mut meta.issues,
                );
                if task {
                    meta.priority = enum_field(
                        r.get(12)?,
                        Some("normal"),
                        &["low", "normal", "high"],
                        "priority",
                        &mut meta.issues,
                    );
                    meta.due_date = date_field(r.get(13)?, "due_date", &mut meta.issues);
                    meta.planned_date = date_field(r.get(14)?, "planned_date", &mut meta.issues);
                }
            }
            EntityKind::Tag => {
                meta.classification = enum_field(
                    r.get(15)?,
                    None,
                    &["area", "resource", "entity", "topic"],
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
            let raw: Option<String> = r.get(11)?;
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
    if entity.kind == EntityKind::Project {
        let raw: String = r.get(16)?;
        match serde_json::from_str::<ProjectFields>(&raw)
            .ok()
            .filter(|f| {
                f.board_order.is_none_or(|order| order <= 10_000)
                    && f.source.as_ref().is_none_or(|source| {
                        crate::productivity::validate_note_source(source).is_ok()
                    })
            }) {
            Some(fields) => {
                source_archived = fields.source.is_some_and(|source| source.archived);
                project_order = fields.board_order;
                meta.project_end_date = fields.end_date.filter(|date| {
                    if civil_date(date).is_err() {
                        meta.issues.push("end_date".into());
                        false
                    } else {
                        true
                    }
                });
            }
            None => meta.issues.push("project_metadata".into()),
        }
    }
    if entity.kind == EntityKind::Tag {
        let raw: String = r.get(16)?;
        match serde_json::from_str::<TagFields>(&raw).ok().filter(|f| {
            f.source
                .as_ref()
                .is_none_or(|source| crate::productivity::validate_note_source(source).is_ok())
                && f.source_activity_at.as_ref().is_none_or(|activity| {
                    crate::productivity::validate_tag_activity(activity).is_ok()
                })
        }) {
            Some(fields) => {
                meta.favorite = Some(fields.favorite.unwrap_or(false));
                source_archived = fields.source.as_ref().is_some_and(|source| source.archived);
                activity_at = fields
                    .source_activity_at
                    .as_deref()
                    .or(fields
                        .source
                        .as_ref()
                        .and_then(|source| source.updated_at.as_deref()))
                    .or(Some(&entity.updated_at))
                    .and_then(instant);
            }
            None => meta.issues.push("tag_metadata".into()),
        }
    }
    if entity.kind == EntityKind::Note {
        let raw: String = r.get(16)?;
        match serde_json::from_str::<NoteFields>(&raw)
            .ok()
            .filter(|f| crate::productivity::validate_note_fields(f).is_ok())
        {
            Some(fields) => {
                source_archived = fields.source.as_ref().is_some_and(|source| source.archived);
                meta.note_type = fields.note_type.as_ref().map(|t| t.as_str().to_owned());
                meta.favorite = Some(fields.favorite.unwrap_or(false));
                meta.note_library = Some(
                    fields.goal.is_none()
                        && fields.milestone.is_none()
                        && fields
                            .source
                            .as_ref()
                            .is_none_or(|source| source.collection == "Notes" && !source.archived),
                );
                meta.source_updated_at = fields.source.and_then(|source| source.updated_at);
                meta.source_updated_days_ago = meta
                    .source_updated_at
                    .as_ref()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|d| {
                        (civil_date(day).expect("día validado") - d.with_timezone(&tz).date_naive())
                            .num_days()
                    });
            }
            None => meta.issues.push("note_metadata".into()),
        }
    }
    if entity.kind == EntityKind::Person {
        let raw: String = r.get(16)?;
        match serde_json::from_str::<PersonFields>(&raw)
            .ok()
            .filter(|f| crate::productivity::validate_person_fields(f).is_ok())
        {
            Some(fields) => {
                source_archived = fields.source.as_ref().is_some_and(|s| s.archived);
                person_library = fields
                    .source
                    .as_ref()
                    .is_none_or(|s| s.collection == "People" && !s.archived);
                let day = civil_date(day).expect("día validado");
                meta.birthday_occurrence = fields
                    .birthday
                    .as_deref()
                    .and_then(|s| civil_date(s).ok())
                    .and_then(|birthday| {
                        let occurrence =
                            NaiveDate::from_ymd_opt(day.year(), birthday.month(), birthday.day())
                                .or_else(|| {
                                    (birthday.month() == 2 && birthday.day() == 29)
                                        .then(|| NaiveDate::from_ymd_opt(day.year(), 2, 28))
                                        .flatten()
                                });
                        occurrence
                            .filter(|d| d.month() == day.month())
                            .map(|d| d.to_string())
                    });
                meta.person = Some(fields);
            }
            None => meta.issues.push("person_metadata".into()),
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
        source_archived,
        project_order,
        activity_at,
        person_library,
    })
}
fn bad(m: &HomeMetadata, field: &str) -> bool {
    m.issues.iter().any(|i| i == "payload" || i == field)
}
fn matches_view(row: &ReadRow, view: HomeView, day: &str, timezone: chrono_tz::Tz) -> Option<bool> {
    use HomeView::*;
    let m = &row.meta;
    if matches!(
        view,
        ProjectsActive
            | ProjectsPlanned
            | ProjectsBoard
            | ProjectsDeadlines
            | ProjectsCompleted
            | ProjectsOnHold
            | ProjectsByTag
            | ProjectsByPeople
            | PeopleActiveProjects
    ) {
        if bad(m, "project_metadata") {
            return None;
        }
        if row.source_archived {
            return Some(false);
        }
    }
    if matches!(
        view,
        PeopleContacts
            | PeopleNew
            | PeopleUpdated
            | PeopleCheckIns
            | PeopleBirthdays
            | PeoplePipeline
    ) {
        if bad(m, "person_metadata") {
            return None;
        }
        if !row.person_library {
            return Some(false);
        }
    }
    if matches!(
        view,
        TagsRecent | TagsFavorites | TagsAlphabetical | TagsTypes
    ) {
        if bad(m, "tag_metadata") {
            return None;
        }
        if row.source_archived {
            return Some(false);
        }
    }
    let state = |states: &[&str]| {
        if bad(m, "state") {
            None
        } else {
            Some(m.state.as_deref().is_some_and(|s| states.contains(&s)))
        }
    };
    match view {
        TasksToday => {
            if state(&["todo", "doing"])? {
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
            if state(&["todo", "doing"])? {
                m.inbox
            } else {
                Some(false)
            }
        }
        TasksDone => state(&["done"]),
        TasksActiveProjects | TasksProjectsInProgress => state(&["todo", "doing"]),
        TasksWeek | TasksMonth | TasksScheduled | TasksNoDue => {
            if !state(&["todo", "doing"])? {
                return Some(false);
            }
            if bad(m, "due_date") {
                return None;
            }
            if view == TasksNoDue {
                return Some(m.due_date.is_none());
            }
            let Some(due) = m.due_date.as_deref() else {
                return Some(false);
            };
            if view == TasksScheduled {
                return Some(true);
            }
            let due = NaiveDate::parse_from_str(due, "%Y-%m-%d").ok()?;
            let day = NaiveDate::parse_from_str(day, "%Y-%m-%d").ok()?;
            Some(if view == TasksWeek {
                due.iso_week() == day.iso_week()
            } else {
                due.year() == day.year() && due.month() == day.month()
            })
        }
        NotesLibrary | NotesFavorites | NotesClips | NotesVoice | NotesJournal | NotesMeetings => {
            if !m.note_library? {
                return Some(false);
            }
            Some(match view {
                NotesLibrary => !matches!(
                    m.note_type.as_deref(),
                    Some("web_clip" | "voice" | "journal" | "meeting")
                ),
                NotesFavorites => m.favorite?,
                NotesClips => m.note_type.as_deref() == Some("web_clip"),
                NotesVoice => m.note_type.as_deref() == Some("voice"),
                NotesJournal => m.note_type.as_deref() == Some("journal"),
                NotesMeetings => m.note_type.as_deref() == Some("meeting"),
                _ => unreachable!(),
            })
        }
        NotesInbox => m.inbox,
        TagsFavorites => m.favorite,
        ProjectsActive | PeopleActiveProjects => state(&["doing", "ongoing"]),
        ProjectsPlanned => state(&["planned"]),
        ProjectsCompleted => state(&["done"]),
        ProjectsOnHold => state(&["on_hold"]),
        ProjectsByTag | ProjectsByPeople => {
            state(&["planned", "doing", "ongoing", "done", "on_hold"])
        }
        ProjectsDeadlines => {
            if state(&["done"])? {
                Some(false)
            } else if bad(m, "end_date") {
                None
            } else {
                Some(m.project_end_date.is_some())
            }
        }
        PeopleNew | PeopleUpdated => {
            let person = m.person.as_ref()?;
            let timestamp = if view == PeopleNew {
                person
                    .source
                    .as_ref()
                    .and_then(|s| s.created_at.as_deref())
                    .unwrap_or(&row.entity.created_at)
            } else {
                person
                    .source
                    .as_ref()
                    .and_then(|s| s.updated_at.as_deref())
                    .unwrap_or(&row.entity.updated_at)
            };
            let timestamp = DateTime::parse_from_rfc3339(timestamp).ok()?;
            let date = civil_date(day).ok()?;
            let start = date.checked_sub_months(Months::new(1))?;
            Some(timestamp.with_timezone(&timezone).date_naive() >= start)
        }
        PeopleCheckIns => {
            let until = civil_date(day).ok()?.checked_add_months(Months::new(1))?;
            Some(
                m.person
                    .as_ref()?
                    .check_in
                    .as_deref()
                    .and_then(|date| civil_date(date).ok())
                    .is_some_and(|date| date <= until),
            )
        }
        PeopleBirthdays => Some(m.birthday_occurrence.is_some()),
        _ => Some(true),
    }
}
fn kind(view: HomeView) -> &'static str {
    use HomeView::*;
    match view {
        TasksToday
        | TasksInbox
        | TasksAll
        | TasksDone
        | TasksWeek
        | TasksMonth
        | TasksScheduled
        | TasksNoDue
        | TasksActiveProjects
        | TasksProjectsInProgress => "task",
        NotesAll | NotesInbox | NotesLibrary | NotesFavorites | NotesClips | NotesVoice
        | NotesJournal | NotesMeetings | GoalsActivity | GoalsDeadline | GoalsByTag
        | GoalsAchieved | MilestonesCompleted | MilestonesAll => "note",
        ProjectsActive | ProjectsPlanned | ProjectsBoard | ProjectsDeadlines
        | ProjectsCompleted | ProjectsOnHold | ProjectsByTag | ProjectsByPeople
        | PeopleActiveProjects => "project",
        TagsRecent | TagsFavorites | TagsAlphabetical | TagsTypes => "tag",
        PeopleContacts | PeopleNew | PeopleUpdated | PeopleCheckIns | PeopleBirthdays
        | PeoplePipeline => "person",
    }
}
fn instant(value: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|d| d.timestamp_millis())
}
fn modified_at(row: &ReadRow) -> Option<i64> {
    instant(
        row.meta
            .source_updated_at
            .as_deref()
            .unwrap_or(&row.entity.updated_at),
    )
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
        TasksToday | TasksWeek | TasksMonth | TasksScheduled => a
            .meta
            .due_date
            .cmp(&b.meta.due_date)
            .then(a.entity.created_at.cmp(&b.entity.created_at)),
        TasksInbox | TasksAll | TasksNoDue | NotesInbox => {
            a.entity.created_at.cmp(&b.entity.created_at)
        }
        TasksActiveProjects | TasksProjectsInProgress => {
            b.entity.created_at.cmp(&a.entity.created_at)
        }
        TasksDone | NotesAll => b.entity.updated_at.cmp(&a.entity.updated_at),
        NotesLibrary | NotesFavorites | NotesClips | NotesVoice | NotesJournal | NotesMeetings => {
            let modified = |row: &ReadRow| {
                DateTime::parse_from_rfc3339(
                    row.meta
                        .source_updated_at
                        .as_deref()
                        .unwrap_or(&row.entity.updated_at),
                )
                .map(|d| d.timestamp_millis())
                .unwrap_or(i64::MIN)
            };
            modified(b).cmp(&modified(a))
        }
        TagsRecent => b.activity_at.cmp(&a.activity_at),
        PeopleCheckIns => a
            .meta
            .person
            .as_ref()
            .and_then(|p| p.check_in.as_ref())
            .cmp(&b.meta.person.as_ref().and_then(|p| p.check_in.as_ref()))
            .then_with(title),
        PeopleContacts | PeopleNew | PeopleUpdated | PeoplePipeline | PeopleBirthdays => a
            .meta
            .person
            .as_ref()
            .and_then(|p| p.gallery_order)
            .unwrap_or(u32::MAX)
            .cmp(
                &b.meta
                    .person
                    .as_ref()
                    .and_then(|p| p.gallery_order)
                    .unwrap_or(u32::MAX),
            )
            .then_with(title),
        ProjectsDeadlines => a
            .meta
            .project_end_date
            .cmp(&b.meta.project_end_date)
            .then_with(title),
        ProjectsActive | ProjectsBoard | ProjectsCompleted | ProjectsOnHold | ProjectsByTag
        | ProjectsByPeople => rank(
            a.meta.state.as_deref(),
            &["planned", "doing", "ongoing", "done", "on_hold"],
        )
        .cmp(&rank(
            b.meta.state.as_deref(),
            &["planned", "doing", "ongoing", "done", "on_hold"],
        ))
        .then(
            a.project_order
                .unwrap_or(u32::MAX)
                .cmp(&b.project_order.unwrap_or(u32::MAX)),
        )
        .then_with(title),
        TagsTypes => rank(
            a.meta.classification.as_deref(),
            &["area", "resource", "entity", "topic"],
        )
        .cmp(&rank(
            b.meta.classification.as_deref(),
            &["area", "resource", "entity", "topic"],
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
        related_entity: None,
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
        if crate::goals::is_goal_view(input.view) {
            return self.goal_home_query(input);
        }
        if input.goal_id.is_some() {
            return Err(CoreError::validation(
                "goal_id sólo se admite en milestones_all.",
            ));
        }
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
        let mut rows = stmt
            .query_map(params![kind(input.view), (CANDIDATES + 1) as u32], |r| {
                read_row(r, &date, tz)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if rows.len() > CANDIDATES {
            return Err(exceeded("candidates", CANDIDATES));
        }
        if input.view == HomeView::TagsRecent {
            // La actividad de notas se obtiene antes de ordenar/paginar tags,
            // con los mismos límites de relaciones y fuentes de Home.
            let tag_ids =
                serde_json::to_string(&rows.iter().map(|r| &r.entity.id).collect::<Vec<_>>())?;
            let mut statement = tx.prepare(&format!("SELECT {REF_FIELDS},{META_FIELDS},r.target_id FROM relations r JOIN entities e ON e.id=r.source_id WHERE r.kind='tag' AND e.kind='note' AND e.archived_at IS NULL AND r.target_id IN (SELECT value FROM json_each(?1)) LIMIT ?2"))?;
            let note_links = statement.query_map(params![tag_ids, (LINKS + 1) as u32], |r| {
                Ok((r.get::<_, String>(17)?, read_row(r, &date, tz)?))
            })?;
            let mut source_ids = BTreeSet::new();
            let mut activity = BTreeMap::<String, i64>::new();
            for (index, link) in note_links.enumerate() {
                if index >= LINKS {
                    return Err(exceeded("links", LINKS));
                }
                let (tag_id, note) = link?;
                source_ids.insert(note.entity.id.clone());
                if source_ids.len() > SOURCES {
                    return Err(exceeded("summary_sources", SOURCES));
                }
                if !note.source_archived && !bad(&note.meta, "note_metadata") {
                    if let Some(time) = modified_at(&note) {
                        activity
                            .entry(tag_id)
                            .and_modify(|current| *current = (*current).max(time))
                            .or_insert(time);
                    }
                }
            }
            for row in &mut rows {
                if let Some(time) = activity.get(&row.entity.id) {
                    row.activity_at =
                        Some(row.activity_at.map_or(*time, |current| current.max(*time)));
                }
            }
        }
        let by_project = matches!(
            input.view,
            HomeView::TasksActiveProjects | HomeView::TasksProjectsInProgress
        );
        let mut eligible_projects = BTreeSet::<String>::new();
        let mut project_membership = BTreeMap::<String, (bool, bool)>::new();
        if by_project {
            let candidates =
                serde_json::to_string(&rows.iter().map(|r| &r.entity.id).collect::<Vec<_>>())?;
            let mut stmt = tx.prepare(&format!("SELECT {REF_FIELDS},{META_FIELDS},r.source_id FROM relations r JOIN entities e ON e.id=r.target_id WHERE r.kind='project' AND r.source_id IN (SELECT value FROM json_each(?1)) LIMIT ?2"))?;
            let links = stmt
                .query_map(params![candidates, (LINKS + 1) as u32], |r| {
                    Ok((r.get::<_, String>(17)?, read_row(r, &date, tz)?))
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            if links.len() > LINKS {
                return Err(exceeded("links", LINKS));
            }
            for (task_id, project) in links {
                if project.entity.kind != EntityKind::Project {
                    return Err(CoreError::new(
                        ErrorCode::StorageError,
                        "Relación de proyecto incompatible; Home no la convierte.",
                    ));
                }
                if project.entity.archived_at.is_some() || project.source_archived {
                    continue;
                }
                let membership = project_membership.entry(task_id).or_default();
                if bad(&project.meta, "state") || bad(&project.meta, "project_metadata") {
                    membership.1 = true;
                    continue;
                }
                let states: &[&str] = if input.view == HomeView::TasksActiveProjects {
                    &["planned", "doing", "ongoing"]
                } else {
                    &["doing", "ongoing"]
                };
                if project
                    .meta
                    .state
                    .as_deref()
                    .is_some_and(|state| states.contains(&state))
                {
                    membership.0 = true;
                    eligible_projects.insert(project.entity.id);
                }
            }
        }
        let mut people_membership = BTreeMap::<String, BTreeMap<String, EntityRef>>::new();
        let mut unavailable_people_projects = BTreeSet::new();
        if input.view == HomeView::PeopleActiveProjects {
            let candidates =
                serde_json::to_string(&rows.iter().map(|r| &r.entity.id).collect::<Vec<_>>())?;
            let mut statement = tx.prepare(&format!("SELECT {REF_FIELDS},{META_FIELDS},r.source_id FROM relations r JOIN entities e ON e.id=r.target_id WHERE r.kind='person' AND r.source_id IN (SELECT value FROM json_each(?1)) LIMIT ?2"))?;
            let links = statement.query_map(params![candidates, (LINKS + 1) as u32], |r| {
                Ok((r.get::<_, String>(17)?, read_row(r, &date, tz)?))
            })?;
            let mut people_ids = BTreeSet::new();
            for (index, link) in links.enumerate() {
                if index >= LINKS {
                    return Err(exceeded("links", LINKS));
                }
                let (project_id, person) = link?;
                if person.entity.kind != EntityKind::Person {
                    return Err(CoreError::new(
                        ErrorCode::StorageError,
                        "Relación de persona incompatible.",
                    ));
                }
                people_ids.insert(person.entity.id.clone());
                if people_ids.len() > SOURCES {
                    return Err(exceeded("summary_sources", SOURCES));
                }
                if person.entity.archived_at.is_some() {
                    continue;
                }
                if bad(&person.meta, "person_metadata") {
                    unavailable_people_projects.insert(project_id);
                } else if person.person_library {
                    people_membership
                        .entry(project_id)
                        .or_default()
                        .insert(person.entity.id.clone(), person.entity);
                }
            }
        }
        let mut unavailable_count = 0;
        let mut selected = rows
            .into_iter()
            .filter(|r| match matches_view(r, input.view, &date, tz) {
                Some(true) if by_project => {
                    let (eligible, unknown) = project_membership
                        .get(&r.entity.id)
                        .copied()
                        .unwrap_or_default();
                    if !eligible && unknown {
                        unavailable_count += 1;
                    }
                    eligible
                }
                Some(true) if input.view == HomeView::PeopleActiveProjects => {
                    let eligible = people_membership
                        .get(&r.entity.id)
                        .is_some_and(|p| !p.is_empty());
                    if !eligible && unavailable_people_projects.contains(&r.entity.id) {
                        unavailable_count += 1;
                    }
                    eligible
                }
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
        let mut stmt=tx.prepare(&format!("SELECT r.kind,{refs_s},{refs_t},s.payload FROM relations r JOIN entities s ON s.id=r.source_id JOIN entities t ON t.id=r.target_id WHERE r.source_id IN (SELECT value FROM json_each(?1)) OR r.target_id IN (SELECT value FROM json_each(?1)) LIMIT ?2"))?;
        let edges = stmt
            .query_map(params![id_json, (LINKS + 1) as u32], |r| {
                Ok(Edge {
                    kind: r.get(0)?,
                    source: reference(r, 1)?,
                    target: reference(r, 10)?,
                    source_is_goal: serde_json::from_str::<NoteFields>(&r.get::<_, String>(19)?)
                        .ok()
                        .is_some_and(|fields| {
                            crate::productivity::validate_note_fields(&fields).is_ok()
                                && fields.source.as_ref().is_none_or(|source| !source.archived)
                                && (fields.goal.is_some()
                                    || fields
                                        .source
                                        .as_ref()
                                        .is_some_and(|source| source.collection == "Goals"))
                        }),
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
                    && (matches!(
                        other.kind,
                        EntityKind::Project | EntityKind::Tag | EntityKind::Person
                    ) || (me.kind == EntityKind::Project
                        && other.id == e.source.id
                        && other.kind == EntityKind::Note
                        && e.source_is_goal))
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
        if matches!(
            input.view,
            TasksInbox | TasksAll | TasksDone | TasksActiveProjects | TasksProjectsInProgress
        ) {
            if !by_project {
                groups.insert("none".into(), group("none", "Sin proyecto"));
            }
        }
        if matches!(
            input.view,
            ProjectsActive | ProjectsBoard | ProjectsCompleted | ProjectsOnHold
        ) {
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
                if input.view == ProjectsCompleted && state != "done" {
                    continue;
                }
                if input.view == ProjectsOnHold && state != "on_hold" {
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
                ("entity", "Entity"),
                ("topic", "Topic"),
            ] {
                let mut g = group(t, label);
                g.classification = Some(serde_json::from_value(Value::String(t.into()))?);
                groups.insert(t.into(), g);
            }
        }
        if input.view == PeoplePipeline {
            for (state, label) in [
                ("prospect", "Prospect"),
                ("contacted", "Contacted"),
                ("negotiating", "Negotiating"),
                ("rejected", "Rejected"),
                ("closed", "Closed"),
            ] {
                groups.insert(state.into(), group(state, label));
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
            if matches!(
                input.view,
                TasksInbox | TasksAll | TasksDone | TasksActiveProjects | TasksProjectsInProgress
            ) {
                let projects = ctx
                    .iter()
                    .filter(|e| {
                        e.kind == EntityKind::Project
                            && (!by_project || eligible_projects.contains(&e.id))
                    })
                    .collect::<Vec<_>>();
                if projects.is_empty() && !by_project {
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
            } else if input.view == PeopleActiveProjects {
                for person in people_membership
                    .get(id)
                    .into_iter()
                    .flat_map(|p| p.values())
                {
                    let key = format!("person:{}", person.id);
                    groups
                        .entry(key.clone())
                        .or_insert_with(|| group(&key, &person.title))
                        .entity_ids
                        .push(id.clone());
                }
            } else if matches!(input.view, ProjectsByTag | ProjectsByPeople) {
                let (related_kind, prefix, empty_label) = if input.view == ProjectsByTag {
                    (EntityKind::Tag, "tag", "Sin tag")
                } else {
                    (EntityKind::Person, "person", "Sin persona")
                };
                let mut has_relation = false;
                for related in ctx.iter().filter(|entity| entity.kind == related_kind) {
                    has_relation = true;
                    let key = format!("{prefix}:{}", related.id);
                    groups
                        .entry(key.clone())
                        .or_insert_with(|| {
                            let mut g = group(&key, &related.title);
                            g.related_entity = Some(related.clone());
                            g
                        })
                        .entity_ids
                        .push(id.clone());
                }
                if !has_relation {
                    groups
                        .entry("none".into())
                        .or_insert_with(|| group("none", empty_label))
                        .entity_ids
                        .push(id.clone());
                }
            } else if input.view == PeoplePipeline {
                let key = row
                    .meta
                    .person
                    .as_ref()
                    .and_then(|p| p.pipeline_status.as_ref())
                    .and_then(|s| serde_json::to_value(s).ok())
                    .and_then(|v| v.as_str().map(str::to_owned));
                let key = key.as_deref().unwrap_or("none");
                groups
                    .entry(key.into())
                    .or_insert_with(|| group(key, "Sin estado"))
                    .entity_ids
                    .push(id.clone());
            } else if matches!(
                input.view,
                ProjectsActive | ProjectsBoard | ProjectsCompleted | ProjectsOnHold | TagsTypes
            ) {
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
                        progress_tenths: (available && total > 0)
                            .then(|| (done * 1000 + total / 2) / total),
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
                    let refs = related
                        .get(id)
                        .cloned()
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|key| !sources[key].source_archived)
                        .collect::<BTreeSet<_>>();
                    let available = refs.iter().all(|key| {
                        !bad(&sources[key].meta, "updated_at")
                            && !bad(&sources[key].meta, "note_metadata")
                    });
                    let mut notes = refs
                        .iter()
                        .map(|id| source_refs[id].clone())
                        .collect::<Vec<_>>();
                    notes.sort_by(|a, b| {
                        modified_at(&sources[&b.id])
                            .cmp(&modified_at(&sources[&a.id]))
                            .then(a.id.cmp(&b.id))
                    });
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
        if input.view == TagsTypes && groups.get("topic").is_some_and(|g| g.entity_ids.is_empty()) {
            groups.remove("topic");
        }
        if groups.len() > GROUPS {
            return Err(exceeded("groups", GROUPS));
        }
        let mut groups = groups.into_values().collect::<Vec<_>>();
        groups.sort_by(|a, b| {
            if matches!(
                input.view,
                ProjectsActive | ProjectsBoard | ProjectsCompleted | ProjectsOnHold
            ) {
                rank(
                    Some(&a.key),
                    &["planned", "doing", "ongoing", "done", "on_hold"],
                )
                .cmp(&rank(
                    Some(&b.key),
                    &["planned", "doing", "ongoing", "done", "on_hold"],
                ))
            } else if matches!(input.view, ProjectsByTag | ProjectsByPeople) {
                a.related_entity
                    .is_none()
                    .cmp(&b.related_entity.is_none())
                    .then(a.label.to_lowercase().cmp(&b.label.to_lowercase()))
                    .then(a.key.cmp(&b.key))
            } else if input.view == PeoplePipeline {
                rank(
                    Some(&a.key),
                    &[
                        "prospect",
                        "contacted",
                        "negotiating",
                        "rejected",
                        "closed",
                        "none",
                    ],
                )
                .cmp(&rank(
                    Some(&b.key),
                    &[
                        "prospect",
                        "contacted",
                        "negotiating",
                        "rejected",
                        "closed",
                        "none",
                    ],
                ))
            } else if input.view == TagsTypes {
                rank(Some(&a.key), &["area", "resource", "entity", "topic"]).cmp(&rank(
                    Some(&b.key),
                    &["area", "resource", "entity", "topic"],
                ))
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
            goal_id: None,
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
