use crate::{
    contracts::*,
    database as db, entities,
    error::*,
    mutations::MutationContext,
    workspace::{Workspace, id},
};
use rusqlite::{Connection, TransactionBehavior, params};
use serde_json::{Value, json};
use std::path::Path;

fn unsupported() -> CoreError {
    CoreError::new(
        ErrorCode::UnsupportedTemplateFormat,
        "Definición de plantilla no reconocida; recuperá su JSON sin reemplazarlo.",
    )
}
fn structure(d: &TemplateDefinition) -> Result<()> {
    if d.schema_version != 1 {
        return Err(CoreError::validation("Versión de plantilla no admitida."));
    }
    entities::title(&d.title)?;
    entities::validate_document(&d.document)?;
    if d.values.len() > 128 {
        return Err(CoreError::validation("Defaults superan 128 propiedades."));
    }
    Ok(())
}
fn parse(value: &Value) -> Result<TemplateDefinition> {
    entities::validate_request_limits(value).map_err(|_| unsupported())?;
    let d: TemplateDefinition = serde_json::from_value(value.clone()).map_err(|_| unsupported())?;
    structure(&d).map_err(|_| unsupported())?;
    Ok(d)
}
fn get(conn: &Connection, key: &str) -> Result<Template> {
    let (collection_id, name, revision, raw): (String, String, u32, String) = conn.query_row(
        "SELECT collection_id,name,revision,definition FROM templates WHERE id=?1",
        [key],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    )?;
    let definition: Value = serde_json::from_str(&raw).map_err(|_| {
        CoreError::new(
            ErrorCode::StorageError,
            "JSON de plantilla almacenado corrupto; no se reparó.",
        )
    })?;
    let editable = parse(&definition).is_ok();
    Ok(Template {
        id: key.into(),
        collection_id,
        name,
        revision,
        definition,
        editable,
    })
}
fn validate(conn: &Connection, root: &Path, c: &str, d: &TemplateDefinition) -> Result<()> {
    structure(d)?;
    crate::documents::validate_references(conn, root, &serde_json::to_value(&d.document)?)?;
    db::validate_values(conn, root, c, &d.values, true)
}
impl Workspace {
    pub fn template_get(&self, key: &str) -> Result<Template> {
        get(&self.conn, key)
    }
    pub fn template_list(&self, input: ListTemplates) -> Result<Page<Template>> {
        let tx = self.conn.unchecked_transaction()?;
        db::collection(&tx, &input.collection_id)?;
        let n = entities::limit(input.limit)?;
        let mut stmt = tx.prepare(
            "SELECT id FROM templates WHERE collection_id=?1 AND id>?2 ORDER BY id LIMIT ?3",
        )?;
        let ids = stmt
            .query_map(
                params![input.collection_id, input.cursor.unwrap_or_default(), n + 1],
                |r| r.get::<_, String>(0),
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let items = ids
            .iter()
            .map(|id| get(&tx, id))
            .collect::<Result<Vec<_>>>()?;
        Ok(entities::page(items, n, |t| t.id.clone()))
    }
    pub fn template_save(&mut self, input: SaveTemplate) -> Result<Template> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let c = db::collection(&tx, &input.collection_id)?;
        db::revision(&c.id, c.revision, input.expected_collection_revision)?;
        let name = db::name(&input.name)?;
        let (key, revision) = match (&input.id, input.expected_revision) {
            (None, None) => (id(), 1),
            (Some(key), Some(expected)) => {
                let original = get(&tx, key)?;
                if original.collection_id != c.id {
                    return Err(CoreError::validation(
                        "La plantilla pertenece a otra colección.",
                    ));
                }
                db::revision(key, original.revision, expected)?;
                let old = parse(&original.definition)?;
                // Un formato futuro en defaults tampoco se reemplaza por uno menor.
                db::validate_values(&tx, &self.path, &c.id, &old.values, false)
                    .map_err(|_| unsupported())?;
                let next = original
                    .revision
                    .checked_add(1)
                    .ok_or_else(|| CoreError::validation("Límite de revisión."))?;
                (key.clone(), next)
            }
            _ => {
                return Err(CoreError::validation(
                    "Actualizar requiere id y expected_revision; crear no los admite.",
                ));
            }
        };
        validate(&tx, &self.path, &c.id, &input.definition)?;
        tx.execute("INSERT INTO templates VALUES (?1,?2,?3,?4,?5) ON CONFLICT(id) DO UPDATE SET name=excluded.name,revision=excluded.revision,definition=excluded.definition",params![key,c.id,name,revision,serde_json::to_string(&input.definition)?])?;
        let result = get(&tx, &key)?;
        tx.commit()?;
        Ok(result)
    }
    pub fn template_instantiate(
        &mut self,
        input: InstantiateTemplate,
        origin: &str,
        request: &str,
    ) -> Result<RecordData> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let t = get(&tx, &input.id)?;
        db::revision(&t.id, t.revision, input.expected_template_revision)?;
        let c = db::collection(&tx, &t.collection_id)?;
        db::revision(&c.id, c.revision, input.expected_collection_revision)?;
        let mut d = parse(&t.definition)?;
        validate(&tx, &self.path, &c.id, &d)?;
        db::validate_values(&tx, &self.path, &c.id, &input.values, true)?;
        d.values.extend(input.values);
        fn reidentify(blocks: &mut [Block]) {
            for b in blocks {
                b.id = id();
                reidentify(&mut b.children);
            }
        }
        reidentify(&mut d.document.blocks);
        let (_, record) = entities::create_in_transaction(
            &tx,
            &self.path,
            CreateEntity {
                kind: EntityKind::Note,
                title: input.title.unwrap_or(d.title),
                payload: json!({"inbox":false}),
                document: Some(d.document),
                context: None,
                record: Some(RecordSeed {
                    collection_id: c.id,
                    values: d.values,
                }),
            },
            &mut MutationContext::individual(origin, request),
        )?;
        let record = record.ok_or_else(|| {
            CoreError::new(
                ErrorCode::StorageError,
                "Instanciación sin pertenencia confirmada.",
            )
        })?;
        tx.commit()?;
        Ok(record)
    }
}
