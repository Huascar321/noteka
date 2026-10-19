use crate::{
    contracts::*,
    error::*,
    mutations::{MutationContext, ObjectKind},
    workspace::{Workspace, id, now},
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde_json::{Value, json};
use std::path::Path;

pub const MAX_DOCUMENT_BYTES: usize = 1_048_576;
pub const MAX_BLOCKS: usize = 10_000;
pub const MAX_DEPTH: usize = 32;

pub(crate) fn validate_document(document: &Document) -> Result<Value> {
    let value = serde_json::to_value(document)?;
    validate_json_limits(&value)?;
    crate::documents::validate(document)?;
    Ok(value)
}
pub(crate) fn validate_json_limits(value: &Value) -> Result<()> {
    validate_limits(value, MAX_DOCUMENT_BYTES)
}
pub(crate) fn validate_request_limits(value: &Value) -> Result<()> {
    validate_limits(value, 2 * MAX_DOCUMENT_BYTES)
}
fn validate_limits(value: &Value, max_bytes: usize) -> Result<()> {
    fn depth(v: &Value, d: usize) -> bool {
        if d > MAX_DEPTH {
            return false;
        }
        match v {
            Value::Array(a) => a.iter().all(|x| depth(x, d + 1)),
            Value::Object(o) => o.values().all(|x| depth(x, d + 1)),
            _ => true,
        }
    }
    if !depth(value, 1) || serde_json::to_vec(value)?.len() > max_bytes {
        return Err(CoreError::validation(
            "JSON supera el límite de bytes o profundidad 32.",
        ));
    }
    Ok(())
}
pub fn document_editable(value: &Value) -> bool {
    serde_json::from_value::<Document>(value.clone())
        .ok()
        .and_then(|d| validate_document(&d).ok())
        .is_some()
}
pub(crate) fn title(input: &str) -> Result<String> {
    let t = input.trim();
    if t.is_empty() || t.len() > 4096 {
        return Err(CoreError::validation(
            "El título debe tener entre 1 y 4.096 bytes.",
        ));
    }
    Ok(t.into())
}
fn payload(kind: &EntityKind, value: Value) -> Result<Value> {
    crate::productivity::payload(kind, value)
}
fn enum_string<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_value(v).unwrap().as_str().unwrap().into()
}
pub(crate) fn get(conn: &Connection, entity_id: &str) -> Result<Entity> {
    let raw: (String,String,u32,Option<String>,String,String,String,Option<String>) = conn.query_row(
        "SELECT e.kind,e.title,e.revision,e.archived_at,e.created_at,e.updated_at,e.payload,d.json FROM entities e LEFT JOIN documents d ON d.entity_id=e.id WHERE e.id=?1",[entity_id],
        |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?)))?;
    let document = raw
        .7
        .map(|s| serde_json::from_str::<Value>(&s))
        .transpose()
        .map_err(|_| CoreError::new(ErrorCode::StorageError, "Documento almacenado inválido."))?;
    Ok(Entity {
        id: entity_id.into(),
        kind: serde_json::from_value(json!(raw.0))?,
        title: raw.1,
        revision: raw.2,
        archived_at: raw.3,
        created_at: raw.4,
        updated_at: raw.5,
        payload: serde_json::from_str(&raw.6)?,
        document_editable: document.as_ref().is_some_and(document_editable),
        document,
    })
}
pub(crate) fn expect(entity: &Entity, revision: u32) -> Result<()> {
    if revision == 0 {
        return Err(CoreError::validation(
            "expected_revision debe ser mayor que cero.",
        ));
    }
    if revision != entity.revision {
        let mut e = CoreError::new(
            ErrorCode::RevisionConflict,
            "La entidad cambió; tu buffer no se sobrescribió.",
        );
        e.details = json!({"entity_id":entity.id,"expected_revision":revision,"current_revision":entity.revision});
        return Err(e);
    }
    Ok(())
}
fn change(
    conn: &Connection,
    entity_id: &str,
    operation: &str,
    previous: u32,
    next: u32,
    context: &mut MutationContext<'_>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO changes VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            context.new_id(conn, "change", ObjectKind::Change)?,
            context.request,
            entity_id,
            operation,
            context.origin,
            previous,
            next,
            context.now()
        ],
    )?;
    context.record_history(entity_id, operation, previous, next);
    Ok(())
}
pub(crate) fn bump(
    conn: &Connection,
    entity: &Entity,
    operation: &str,
    origin: &str,
    request: &str,
) -> Result<()> {
    bump_in_transaction(
        conn,
        entity,
        operation,
        &mut MutationContext::individual(origin, request),
    )
}
pub(crate) fn bump_in_transaction(
    conn: &Connection,
    entity: &Entity,
    operation: &str,
    context: &mut MutationContext<'_>,
) -> Result<()> {
    if entity.revision == u32::MAX {
        return Err(CoreError::validation("Se alcanzó el límite de revisión."));
    }
    conn.execute(
        "UPDATE entities SET revision=revision+1,updated_at=?2 WHERE id=?1",
        params![entity.id, context.now()],
    )?;
    change(
        conn,
        &entity.id,
        operation,
        entity.revision,
        entity.revision + 1,
        context,
    )?;
    crate::search::reindex(conn, &get(conn, &entity.id)?)?;
    Ok(())
}
pub(crate) fn limit(value: Option<u32>) -> Result<u32> {
    let n = value.unwrap_or(50);
    if !(1..=200).contains(&n) {
        return Err(CoreError::validation("limit debe estar entre 1 y 200."));
    }
    Ok(n)
}
pub(crate) fn cursor(value: Option<&str>) -> Result<(String, String)> {
    value
        .map(|s| serde_json::from_str(s).map_err(CoreError::from))
        .unwrap_or_else(|| Ok((String::new(), String::new())))
}
pub(crate) fn page<T>(mut items: Vec<T>, n: u32, cursor_of: impl FnOnce(&T) -> String) -> Page<T> {
    let next_cursor = if items.len() > n as usize {
        items.pop();
        items.last().map(cursor_of)
    } else {
        None
    };
    Page { items, next_cursor }
}

// El llamador es dueño del commit; también se usa al instanciar una plantilla.
pub(crate) fn create_in_transaction(
    conn: &Connection,
    root: &Path,
    input: CreateEntity,
    context: &mut MutationContext<'_>,
) -> Result<(Entity, Option<RecordData>)> {
    let title = title(&input.title)?;
    let payload = payload(&input.kind, input.payload)?;
    if input.kind != EntityKind::Note && input.document.is_some() {
        return Err(CoreError::validation("Sólo las notas tienen documento."));
    }
    let document = if input.kind == EntityKind::Note {
        Some(validate_document(&input.document.unwrap_or_default())?)
    } else {
        None
    };
    let entity_id = context.new_id(conn, "entity", ObjectKind::Entity)?;
    let timestamp = context.now();
    context.observe(conn, ObjectKind::Entity, &[&entity_id])?;
    conn.execute(
        "INSERT INTO entities VALUES (?1,?2,?3,1,NULL,?4,?4,?5)",
        params![
            entity_id,
            enum_string(&input.kind),
            title,
            timestamp,
            payload.to_string()
        ],
    )?;
    if let Some(d) = document {
        context.observe(conn, ObjectKind::Document, &[&entity_id])?;
        conn.execute(
            "INSERT INTO documents VALUES (?1,?2,?3)",
            params![
                entity_id,
                d["schema_version"].as_u64().unwrap() as u32,
                d.to_string()
            ],
        )?;
        crate::documents::validate_references(conn, root, &d)?;
    }
    if let Some(link) = input.context {
        let source = get(conn, &entity_id)?;
        let target = get(conn, &link.target_id)?;
        validate_relation(&source, &target, &link.kind)?;
        let relation_id = context.new_id(conn, "relation", ObjectKind::Relation)?;
        context.observe(conn, ObjectKind::Relation, &[&relation_id])?;
        conn.execute(
            "INSERT INTO relations VALUES (?1,?2,?3,?4)",
            params![relation_id, entity_id, target.id, enum_string(&link.kind)],
        )?;
    }
    let record_collection = input.record.as_ref().map(|r| r.collection_id.clone());
    if let Some(seed) = input.record {
        crate::database::observe_record(conn, &entity_id, &seed, context)?;
        crate::database::attach(conn, root, &entity_id, &seed)?;
    }
    change(conn, &entity_id, "entity.create", 0, 1, context)?;
    let entity = get(conn, &entity_id)?;
    crate::search::reindex(conn, &entity)?;
    let record = record_collection
        .as_ref()
        .map(|c| crate::database::record(conn, c, &entity_id))
        .transpose()?;
    Ok((entity, record))
}

impl Workspace {
    pub fn entity_get(&self, input: &IdInput) -> Result<Entity> {
        get(&self.conn, &input.id)
    }
    pub fn entity_create(
        &mut self,
        input: CreateEntity,
        origin: &str,
        request: &str,
    ) -> Result<Entity> {
        Ok(self.entity_create_confirmed(input, origin, request)?.0)
    }
    pub(crate) fn entity_create_confirmed(
        &mut self,
        input: CreateEntity,
        origin: &str,
        request: &str,
    ) -> Result<(Entity, Option<RecordData>)> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let result = create_in_transaction(
            &tx,
            &self.path,
            input,
            &mut MutationContext::individual(origin, request),
        )?;
        tx.commit()?;
        Ok(result)
    }
    pub fn entity_update(
        &mut self,
        input: UpdateEntity,
        origin: &str,
        request: &str,
    ) -> Result<Entity> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let entity = update_in_transaction(
            &tx,
            &self.path,
            input,
            &mut MutationContext::individual(origin, request),
        )?;
        tx.commit()?;
        Ok(entity)
    }
}
pub(crate) fn update_in_transaction(
    conn: &Connection,
    root: &Path,
    input: UpdateEntity,
    context: &mut MutationContext<'_>,
) -> Result<Entity> {
    let current = get(conn, &input.id)?;
    expect(&current, input.expected_revision)?;
    context.observe(conn, ObjectKind::Entity, &[&input.id])?;
    let title = input
        .title
        .map(|t| title(&t))
        .transpose()?
        .unwrap_or(current.title.clone());
    let changes_payload = input.payload.is_some();
    let payload = input
        .payload
        .map(|v| {
            // Completar una tarea no elimina fechas; un payload futuro no se
            // reemplaza por uno menor. La validación se hace sobre la fusión.
            let mut merged = current.payload.as_object().cloned().ok_or_else(|| {
                CoreError::validation(
                    "Payload original no reconocido; recuperalo sin reemplazarlo.",
                )
            })?;
            let patch = v.as_object().ok_or_else(|| {
                CoreError::validation("payload debe ser un objeto de campos explícitos.")
            })?;
            merged.extend(patch.clone());
            payload(&current.kind, Value::Object(merged))
        })
        .transpose()?
        .unwrap_or(current.payload.clone());
    if let Some(d) = input.document {
        if current.kind != EntityKind::Note {
            return Err(CoreError::validation("Sólo las notas tienen documento."));
        }
        if !current.document_editable {
            return Err(CoreError::new(
                ErrorCode::UnsupportedDocumentFormat,
                "El documento original no es editable; recuperá su JSON.",
            ));
        }
        let d = validate_document(&d)?;
        crate::documents::validate_references(conn, root, &d)?;
        context.observe(conn, ObjectKind::Document, &[&input.id])?;
        conn.execute(
            "UPDATE documents SET schema_version=?3,json=?2 WHERE entity_id=?1",
            params![
                input.id,
                d.to_string(),
                d["schema_version"].as_u64().unwrap() as u32
            ],
        )?;
    }
    if changes_payload {
        conn.execute(
            "UPDATE entities SET title=?2,payload=?3 WHERE id=?1",
            params![input.id, title, payload.to_string()],
        )?;
    } else {
        // Un cambio de título/documento no reserializa datos omitidos, incluidos
        // campos futuros y su representación original en SQLite.
        conn.execute(
            "UPDATE entities SET title=?2 WHERE id=?1",
            params![input.id, title],
        )?;
    }
    bump_in_transaction(conn, &current, "entity.update", context)?;
    get(conn, &input.id)
}
impl Workspace {
    pub fn entity_archive(
        &mut self,
        input: RevisionInput,
        archive: bool,
        origin: &str,
        request: &str,
    ) -> Result<Entity> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current = get(&tx, &input.id)?;
        expect(&current, input.expected_revision)?;
        if archive != current.archived_at.is_some() {
            tx.execute(
                "UPDATE entities SET archived_at=?2 WHERE id=?1",
                params![input.id, archive.then(now)],
            )?;
            bump(
                &tx,
                &current,
                if archive {
                    "entity.archive"
                } else {
                    "entity.restore"
                },
                origin,
                request,
            )?;
        }
        let entity = get(&tx, &input.id)?;
        tx.commit()?;
        Ok(entity)
    }
    pub fn entity_list(&self, input: ListEntities) -> Result<Page<Entity>> {
        let n = limit(input.limit)?;
        let (time, key) = cursor(input.cursor.as_deref())?;
        if let Some(ref classification) = input.classification
            && !["area", "resource", "topic"].contains(&classification.as_str())
        {
            return Err(CoreError::validation("classification inválida."));
        }
        if let Some(ref state) = input.state
            && !["todo", "done", "planned", "doing", "ongoing", "on_hold"].contains(&state.as_str())
        {
            return Err(CoreError::validation("state inválido."));
        }
        let kind = input.kind.as_ref().map(enum_string);
        let mut stmt=self.conn.prepare("SELECT id FROM entities WHERE (archived_at IS NOT NULL)=?1 AND (?2 IS NULL OR kind=?2) AND (?3 IS NULL OR json_extract(payload,'$.state')=?3) AND (?4 IS NULL OR json_extract(payload,'$.classification')=?4) AND (created_at,id)>(?5,?6) ORDER BY created_at,id LIMIT ?7")?;
        let ids = stmt
            .query_map(
                params![
                    input.archived,
                    kind,
                    input.state,
                    input.classification,
                    time,
                    key,
                    n + 1
                ],
                |r| r.get::<_, String>(0),
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let items = ids
            .iter()
            .map(|i| get(&self.conn, i))
            .collect::<Result<Vec<_>>>()?;
        Ok(page(items, n, |e| json!([e.created_at, e.id]).to_string()))
    }
    pub fn relation_add(
        &mut self,
        input: AddRelation,
        origin: &str,
        request: &str,
    ) -> Result<RelationResult> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let source = get(&tx, &input.source_id)?;
        expect(&source, input.expected_revision)?;
        let target = get(&tx, &input.target_id)?;
        validate_relation(&source, &target, &input.kind)?;
        let existing: Option<String> = tx
            .query_row(
                "SELECT id FROM relations WHERE source_id=?1 AND target_id=?2 AND kind=?3",
                params![source.id, target.id, enum_string(&input.kind)],
                |r| r.get(0),
            )
            .optional()?;
        let changed = existing.is_none();
        let relation_id = existing.unwrap_or_else(id);
        if changed {
            tx.execute(
                "INSERT INTO relations VALUES (?1,?2,?3,?4)",
                params![relation_id, source.id, target.id, enum_string(&input.kind)],
            )?;
            bump(&tx, &source, "relation.add", origin, request)?;
        }
        let entity = get(&tx, &source.id)?;
        let relation = Relation {
            id: relation_id,
            source_id: source.id,
            target_id: target.id,
            kind: input.kind,
        };
        tx.commit()?;
        Ok(RelationResult {
            relation,
            entity,
            changed,
        })
    }
    pub fn relation_remove(
        &mut self,
        input: RemoveRelation,
        origin: &str,
        request: &str,
    ) -> Result<RelationResult> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let relation = tx.query_row(
            "SELECT id,source_id,target_id,kind FROM relations WHERE id=?1",
            [&input.id],
            relation_row,
        )?;
        let source = get(&tx, &relation.source_id)?;
        expect(&source, input.expected_revision)?;
        tx.execute("DELETE FROM relations WHERE id=?1", [&input.id])?;
        bump(&tx, &source, "relation.remove", origin, request)?;
        let entity = get(&tx, &source.id)?;
        tx.commit()?;
        Ok(RelationResult {
            relation,
            entity,
            changed: true,
        })
    }
    pub fn relation_list(&self, input: ListRelations) -> Result<Page<Relation>> {
        get(&self.conn, &input.entity_id)?;
        let n = limit(input.limit)?;
        let key = input
            .cursor
            .map(|s| serde_json::from_str::<String>(&s))
            .transpose()?
            .unwrap_or_default();
        let mut stmt=self.conn.prepare("SELECT id,source_id,target_id,kind FROM relations WHERE (source_id=?1 OR target_id=?1) AND id>?2 ORDER BY id LIMIT ?3")?;
        let items = stmt
            .query_map(params![input.entity_id, key, n + 1], relation_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(page(items, n, |r| json!(r.id).to_string()))
    }
    pub fn changes_list(&self, input: ListChanges) -> Result<Page<Change>> {
        let n = limit(input.limit)?;
        let (time, key) = cursor(input.cursor.as_deref())?;
        let mut stmt=self.conn.prepare("SELECT id,request_id,entity_id,operation,origin,previous_revision,new_revision,created_at FROM changes WHERE (?1 IS NULL OR entity_id=?1) AND (created_at,id)>(?2,?3) ORDER BY created_at,id LIMIT ?4")?;
        let items = stmt
            .query_map(params![input.entity_id, time, key, n + 1], |r| {
                Ok(Change {
                    id: r.get(0)?,
                    request_id: r.get(1)?,
                    entity_id: r.get(2)?,
                    operation: r.get(3)?,
                    origin: r.get(4)?,
                    previous_revision: r.get(5)?,
                    new_revision: r.get(6)?,
                    created_at: r.get(7)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(page(items, n, |c| json!([c.created_at, c.id]).to_string()))
    }
}
fn validate_relation(source: &Entity, target: &Entity, kind: &RelationKind) -> Result<()> {
    let compatible = source.id != target.id
        && match kind {
            RelationKind::Project => {
                matches!(source.kind, EntityKind::Note | EntityKind::Task)
                    && target.kind == EntityKind::Project
            }
            RelationKind::Tag => source.kind != EntityKind::Tag && target.kind == EntityKind::Tag,
            RelationKind::Person => {
                source.kind != EntityKind::Person && target.kind == EntityKind::Person
            }
        };
    if !compatible {
        return Err(CoreError::validation(
            "Extremos incompatibles para esta relación.",
        ));
    }
    Ok(())
}
fn relation_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Relation> {
    let kind: String = r.get(3)?;
    let kind = match kind.as_str() {
        "project" => RelationKind::Project,
        "tag" => RelationKind::Tag,
        "person" => RelationKind::Person,
        _ => return Err(rusqlite::Error::InvalidQuery),
    };
    Ok(Relation {
        id: r.get(0)?,
        source_id: r.get(1)?,
        target_id: r.get(2)?,
        kind,
    })
}
