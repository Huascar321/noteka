//! Contexto prestado: nunca abre, confirma ni revierte transacciones.
use crate::{error::*, workspace};
use rusqlite::{Connection, OptionalExtension, params_from_iter};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ObjectKind {
    Entity,
    Document,
    Relation,
    Collection,
    Property,
    Membership,
    Value,
    Calculation,
    View,
    Change,
}
impl ObjectKind {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Entity => "entity",
            Self::Document => "document",
            Self::Relation => "relation",
            Self::Collection => "collection",
            Self::Property => "property",
            Self::Membership => "membership",
            Self::Value => "value",
            Self::Calculation => "calculation",
            Self::View => "view",
            Self::Change => "change",
        }
    }
    fn storage(self) -> (&'static str, &'static [&'static str]) {
        match self {
            Self::Entity => ("entities", &["id"]),
            Self::Document => ("documents", &["entity_id"]),
            Self::Relation => ("relations", &["id"]),
            Self::Collection => ("collections", &["id"]),
            Self::Property => ("properties", &["id"]),
            Self::Membership => ("memberships", &["collection_id", "entity_id"]),
            Self::Value => ("property_values", &["entity_id", "property_id"]),
            Self::Calculation => ("calculations", &["property_id"]),
            Self::View => ("views", &["id"]),
            Self::Change => ("changes", &["id"]),
        }
    }
}
pub(crate) fn object(
    conn: &Connection,
    kind: ObjectKind,
    keys: &[String],
) -> Result<Option<Value>> {
    let (table, columns) = kind.storage();
    if columns.len() != keys.len() {
        return Err(CoreError::validation("Clave interna incompatible."));
    }
    let predicate = columns
        .iter()
        .enumerate()
        .map(|(i, c)| format!("{c}=?{}", i + 1))
        .collect::<Vec<_>>()
        .join(" AND ");
    let mut stmt = conn.prepare(&format!("SELECT * FROM {table} WHERE {predicate}"))?;
    let names = stmt
        .column_names()
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let row = stmt
        .query_row(params_from_iter(keys), |row| {
            let mut out = serde_json::Map::new();
            for (i, name) in names.iter().enumerate() {
                let v = match row.get_ref(i)? {
                    rusqlite::types::ValueRef::Null => Value::Null,
                    rusqlite::types::ValueRef::Integer(v) => json!(v),
                    rusqlite::types::ValueRef::Real(v) => json!(v),
                    rusqlite::types::ValueRef::Text(_) => json!(row.get::<_, String>(i)?),
                    rusqlite::types::ValueRef::Blob(v) => json!({"blob":v}),
                };
                out.insert(name.clone(), v);
            }
            Ok(Value::Object(out))
        })
        .optional()?;
    row.map(|mut row| {
        let field = match kind {
            ObjectKind::Entity => Some("payload"),
            ObjectKind::Document | ObjectKind::Value | ObjectKind::View => Some("json"),
            ObjectKind::Property => Some("spec"),
            ObjectKind::Calculation => Some("expression"),
            _ => None,
        };
        if let Some(field) = field
            && let Some(raw) = row[field].as_str()
        {
            let known = match kind {
                ObjectKind::Document => serde_json::from_str::<crate::contracts::Document>(raw)
                    .ok()
                    .is_some_and(|d| crate::documents::validate(&d).is_ok()),
                ObjectKind::Property => {
                    serde_json::from_str::<crate::contracts::PropertySpec>(raw).is_ok()
                }
                ObjectKind::Calculation => {
                    serde_json::from_str::<crate::contracts::Expression>(raw).is_ok()
                }
                ObjectKind::View => {
                    serde_json::from_str::<crate::contracts::CollectionView>(raw).is_ok()
                }
                _ => true,
            };
            if known {
                row[field] = serde_json::from_str(raw)?;
            }
        }
        Ok(row)
    })
    .transpose()
}

pub(crate) struct MutationContext<'a> {
    pub origin: &'a str,
    pub request: &'a str,
    seed: Option<Uuid>,
    step: String,
    timestamp: Option<String>,
    ordinals: BTreeMap<&'static str, u64>,
    pub before: BTreeMap<(ObjectKind, Vec<String>), Option<Value>>,
    pub history: Vec<Value>,
}
impl<'a> MutationContext<'a> {
    pub fn individual(origin: &'a str, request: &'a str) -> Self {
        Self {
            origin,
            request,
            seed: None,
            step: String::new(),
            timestamp: None,
            ordinals: BTreeMap::new(),
            before: BTreeMap::new(),
            history: vec![],
        }
    }
    pub fn batch(origin: &'a str, request: &'a str, seed: Uuid) -> Self {
        let mut c = Self::individual(origin, request);
        c.seed = Some(seed);
        c.timestamp = Some(workspace::now());
        c
    }
    pub fn step(&mut self, key: &str) {
        self.step = key.into();
        self.ordinals.clear();
    }
    pub fn now(&self) -> String {
        self.timestamp.clone().unwrap_or_else(workspace::now)
    }
    pub fn id(&mut self, role: &'static str) -> String {
        let Some(seed) = self.seed else {
            return workspace::id();
        };
        let ordinal = self.ordinals.entry(role).or_default();
        let mut h = Sha256::new();
        for part in [
            b"noteka.batch.ids.v1".as_slice(),
            seed.as_bytes(),
            self.step.as_bytes(),
            role.as_bytes(),
            &ordinal.to_be_bytes(),
        ] {
            h.update((part.len() as u64).to_be_bytes());
            h.update(part);
        }
        *ordinal += 1;
        let mut bytes = [0; 16];
        bytes.copy_from_slice(&h.finalize()[..16]);
        bytes[6] = (bytes[6] & 0x0f) | 0x80;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        Uuid::from_bytes(bytes).to_string()
    }
    pub fn new_id(
        &mut self,
        conn: &Connection,
        role: &'static str,
        kind: ObjectKind,
    ) -> Result<String> {
        let id = self.id(role);
        if self.seed.is_some() && object(conn, kind, std::slice::from_ref(&id))?.is_some() {
            let mut error =
                CoreError::new(ErrorCode::RevisionConflict, "El ID reservado ya existe.");
            error.details = json!({"reason":"reserved_id_collision","kind":kind.name(),"id":id});
            return Err(error);
        }
        Ok(id)
    }
    pub fn observe(&mut self, conn: &Connection, kind: ObjectKind, keys: &[&str]) -> Result<()> {
        if self.seed.is_none() {
            return Ok(());
        }
        let keys = keys.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let entry = (kind, keys);
        if !self.before.contains_key(&entry) {
            self.before
                .insert(entry.clone(), object(conn, kind, &entry.1)?);
        }
        Ok(())
    }
    pub fn record_history(&mut self, entity_id: &str, operation: &str, previous: u32, next: u32) {
        if self.seed.is_some() {
            self.history.push(json!({"entity_id":entity_id,"operation":operation,"previous_revision":previous,"new_revision":next}));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{contracts::*, entities, workspace::Workspace};
    use rusqlite::TransactionBehavior;

    #[test]
    fn borrowed_schema_record_view_and_query_helpers_never_commit() {
        use crate::{database as db, formulas};
        let temp = tempfile::tempdir().unwrap();
        let mut w = Workspace::create(&temp.path().join("workspace"), Some("UTC".into())).unwrap();
        let original = w
            .entity_create(
                serde_json::from_value(json!({"kind":"note","title":"Compartida"})).unwrap(),
                "cli",
                "before",
            )
            .unwrap();
        let mut context = MutationContext::batch("cli", "batch", Uuid::nil());
        {
            let tx = w
                .conn
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .unwrap();
            context.step("collection");
            let c = db::collection_create_in_transaction(
                &tx,
                CreateCollection {
                    name: "Members".into(),
                },
                &mut context,
            )
            .unwrap();
            context.step("property");
            let p=db::property_define_in_transaction(&tx,serde_json::from_value(json!({"collection_id":c.collection.id,"expected_revision":1,"spec":{"name":"Peso","type":"number"}})).unwrap(),&mut context).unwrap();
            context.step("formula");
            let expression = json!({"op":"binary","operator":"mul","left":{"op":"property","property_id":p.id},"right":{"op":"number","value":2}});
            let bundle=formulas::set_in_transaction(&tx,serde_json::from_value(json!({"collection_id":c.collection.id,"expected_revision":2,"name":"Doble","expression":expression})).unwrap(),&mut context).unwrap();
            let computed = bundle.properties.last().unwrap().id.clone();
            context.step("record");
            let r=db::record_create_in_transaction(&tx,&w.path,serde_json::from_value(json!({"collection_id":c.collection.id,"title":"Registro","values":{p.id.clone():3}})).unwrap(),&mut context).unwrap();
            assert_eq!(r.values[&computed].as_f64(), Some(6.0));
            context.step("attach");
            let attached=db::record_update_in_transaction(&tx,&w.path,serde_json::from_value(json!({"collection_id":c.collection.id,"entity_id":original.id,"expected_revision":1,"values":{p.id.clone():1}})).unwrap(),true,&mut context).unwrap();
            assert_eq!(attached.entity.id, original.id);
            context.step("update");
            let updated=db::record_update_in_transaction(&tx,&w.path,serde_json::from_value(json!({"collection_id":c.collection.id,"entity_id":original.id,"expected_revision":2,"values":{p.id.clone():2}})).unwrap(),false,&mut context).unwrap();
            assert_eq!(updated.values[&computed].as_f64(), Some(4.0));
            context.step("view");
            let view=db::view_save_in_transaction(&tx,&w.path,serde_json::from_value(json!({"collection_id":c.collection.id,"name":"Orden","layout":"table","config":{"sorts":[{"property_id":computed,"descending":true}]}})).unwrap(),&mut context).unwrap();
            let page = db::query_in_connection(
                &tx,
                &w.path,
                serde_json::from_value(json!({"collection_id":c.collection.id,"view_id":view.id}))
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(page.matching_count, 2);
            assert_eq!(page.records.items[0].entity.id, r.entity.id);
            for kind in [
                ObjectKind::Entity,
                ObjectKind::Document,
                ObjectKind::Collection,
                ObjectKind::Property,
                ObjectKind::Membership,
                ObjectKind::Value,
                ObjectKind::Calculation,
                ObjectKind::View,
            ] {
                assert!(context.before.keys().any(|(k, _)| *k == kind), "{kind:?}");
            }
            context.step("invalid");
            let error=formulas::set_in_transaction(&tx,serde_json::from_value(json!({"collection_id":c.collection.id,"expected_revision":3,"property_id":computed,"name":"Doble","expression":{"op":"property","property_id":computed}})).unwrap(),&mut context).unwrap_err();
            assert!(matches!(error.code, ErrorCode::ValidationError));
        }
        assert_eq!(
            w.conn
                .query_row("SELECT count(*) FROM collections", [], |r| r
                    .get::<_, u32>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            w.conn
                .query_row("SELECT count(*) FROM memberships", [], |r| r
                    .get::<_, u32>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            w.conn
                .query_row("SELECT count(*) FROM changes", [], |r| r.get::<_, u32>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            w.entity_get(&IdInput { id: original.id }).unwrap().revision,
            1
        );
    }

    #[test]
    fn borrowed_entity_helpers_share_rollback_and_preserve_unknown_documents() {
        let temp = tempfile::tempdir().unwrap();
        let mut w = Workspace::create(&temp.path().join("workspace"), Some("UTC".into())).unwrap();
        let input = |v| serde_json::from_value::<CreateEntity>(v).unwrap();
        let original = w
            .entity_create(
                input(json!({"kind":"note","title":"Original"})),
                "cli",
                "before",
            )
            .unwrap();
        let future =
            r#"{"schema_version":99,"blocks":[{"id":"future","type":"future","unknown":true}]}"#;
        w.conn
            .execute(
                "UPDATE documents SET schema_version=99,json=?2 WHERE entity_id=?1",
                rusqlite::params![original.id, future],
            )
            .unwrap();
        let mut context = MutationContext::batch("desktop", "one-request", Uuid::nil());
        context.step("project");
        {
            let tx = w
                .conn
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .unwrap();
            let project = entities::create_in_transaction(
                &tx,
                &w.path,
                input(json!({"kind":"project","title":"Proyecto"})),
                &mut context,
            )
            .unwrap()
            .0;
            context.step("note");
            let note=entities::create_in_transaction(&tx,&w.path,input(json!({"kind":"note","title":"Nueva","context":{"kind":"project","target_id":project.id}})),&mut context).unwrap().0;
            assert_eq!(note.created_at, project.created_at);
            context.step("update");
            let updated = entities::update_in_transaction(
                &tx,
                &w.path,
                serde_json::from_value(
                    json!({"id":original.id,"expected_revision":1,"title":"Nuevo título"}),
                )
                .unwrap(),
                &mut context,
            )
            .unwrap();
            assert!(!updated.document_editable);
            assert_eq!(updated.document.unwrap()["schema_version"], 99);
            let error=entities::update_in_transaction(&tx,&w.path,serde_json::from_value(json!({"id":original.id,"expected_revision":2,"document":{"schema_version":1,"blocks":[]}})).unwrap(),&mut context).unwrap_err();
            assert!(matches!(error.code, ErrorCode::UnsupportedDocumentFormat));
            assert_eq!(context.history.len(), 3);
            assert_eq!(tx.query_row("SELECT count(*) FROM changes WHERE request_id='one-request' AND origin='desktop'",[],|r|r.get::<_,u32>(0)).unwrap(),3);
            assert!(
                context
                    .before
                    .keys()
                    .any(|(kind, _)| *kind == ObjectKind::Relation)
            );
            // Ningún helper confirma: abandonar la transacción revierte los tres pasos.
        }
        let saved = w
            .entity_get(&IdInput {
                id: original.id.clone(),
            })
            .unwrap();
        assert_eq!((saved.title.as_str(), saved.revision), ("Original", 1));
        assert_eq!(
            w.conn
                .query_row(
                    "SELECT json FROM documents WHERE entity_id=?1",
                    [original.id],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
            future
        );
        assert_eq!(
            w.conn
                .query_row("SELECT count(*) FROM entities", [], |r| r.get::<_, u32>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            w.conn
                .query_row("SELECT count(*) FROM changes", [], |r| r.get::<_, u32>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            w.conn
                .query_row(
                    "SELECT count(*) FROM search_fts WHERE search_fts MATCH 'Nueva'",
                    [],
                    |r| r.get::<_, u32>(0)
                )
                .unwrap(),
            0
        );
    }
}
