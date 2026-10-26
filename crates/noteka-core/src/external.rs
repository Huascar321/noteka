//! Asociaciones genéricas entre entidades y recursos externos.
use crate::{
    contracts::*,
    entities,
    error::*,
    workspace::{self, Workspace},
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde_json::{Value, json};

pub const MAX_METADATA_BYTES: usize = 65_536;

fn field(label: &str, value: &str, max: usize) -> Result<String> {
    let value = value.trim();
    if value.is_empty() || value.len() > max {
        return Err(CoreError::validation(format!(
            "{label} debe tener entre 1 y {max} bytes."
        )));
    }
    Ok(value.to_owned())
}
fn metadata(value: &Value) -> Result<String> {
    let value = if value.is_null() {
        json!({})
    } else {
        value.clone()
    };
    let raw = serde_json::to_string(&value)?;
    if raw.len() < 2 || raw.len() > MAX_METADATA_BYTES {
        return Err(CoreError::validation(format!(
            "metadata debe ocupar entre 2 y {MAX_METADATA_BYTES} bytes."
        )));
    }
    Ok(raw)
}
type RawResource = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
);
fn resource_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawResource> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
        row.get(7)?,
    ))
}
fn decode_resource(raw: RawResource) -> Result<ExternalResource> {
    Ok(ExternalResource {
        id: raw.0,
        provider: raw.1,
        account_id: raw.2,
        resource_type: raw.3,
        resource_id: raw.4,
        metadata: serde_json::from_str(&raw.5)
            .map_err(|_| CoreError::new(ErrorCode::StorageError, "metadata externa inválida."))?,
        created_at: raw.6,
        updated_at: raw.7,
    })
}
fn get_resource(conn: &Connection, id: &str) -> Result<ExternalResource> {
    decode_resource(
        conn.query_row(
            "SELECT id,provider,account_id,resource_type,resource_id,metadata,created_at,updated_at FROM external_resources WHERE id=?1",
            [id],
            resource_row,
        )?,
    )
}

impl Workspace {
    pub fn external_link(
        &mut self,
        input: LinkExternalResource,
        origin: &str,
        request: &str,
    ) -> Result<ExternalResourceResult> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let entity = entities::get(&tx, &input.entity_id)?;
        entities::expect(&entity, input.expected_revision)?;
        let provider = field("provider", &input.provider, 64)?;
        let account_id = field("account_id", &input.account_id, 256)?;
        let resource_type = field("resource_type", &input.resource_type, 64)?;
        let resource_id = field("resource_id", &input.resource_id, 2048)?;
        let metadata = metadata(&input.metadata)?;
        let existing: Option<(String, String)> = tx
            .query_row(
                "SELECT id,metadata FROM external_resources WHERE provider=?1 AND account_id=?2 AND resource_type=?3 AND resource_id=?4",
                params![provider, account_id, resource_type, resource_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let timestamp = workspace::now();
        let (external_id, resource_changed) = match existing {
            Some((id, previous)) => {
                let changed = previous != metadata;
                if changed {
                    tx.execute(
                        "UPDATE external_resources SET metadata=?2,updated_at=?3 WHERE id=?1",
                        params![id, metadata, timestamp],
                    )?;
                }
                (id, changed)
            }
            None => {
                let id = workspace::id();
                tx.execute(
                    "INSERT INTO external_resources VALUES (?1,?2,?3,?4,?5,?6,?7,?7)",
                    params![
                        id,
                        provider,
                        account_id,
                        resource_type,
                        resource_id,
                        metadata,
                        timestamp
                    ],
                )?;
                (id, true)
            }
        };
        let linked = tx
            .query_row(
                "SELECT 1 FROM entity_external_resources WHERE entity_id=?1 AND external_resource_id=?2",
                params![input.entity_id, external_id],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if !linked {
            tx.execute(
                "INSERT INTO entity_external_resources VALUES (?1,?2,?3)",
                params![input.entity_id, external_id, timestamp],
            )?;
        }
        if resource_changed || !linked {
            entities::bump(&tx, &entity, "external.link", origin, request)?;
        }
        let entity = entities::get(&tx, &input.entity_id)?;
        let resource = get_resource(&tx, &external_id)?;
        tx.commit()?;
        Ok(ExternalResourceResult {
            resource,
            entity,
            changed: resource_changed || !linked,
        })
    }

    pub fn external_unlink(
        &mut self,
        input: UnlinkExternalResource,
        origin: &str,
        request: &str,
    ) -> Result<ExternalResourceResult> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let entity = entities::get(&tx, &input.entity_id)?;
        entities::expect(&entity, input.expected_revision)?;
        let exists: Option<String> = tx
            .query_row(
                "SELECT external_resource_id FROM entity_external_resources WHERE entity_id=?1 AND external_resource_id=?2",
                params![input.entity_id, input.id],
                |row| row.get(0),
            )
            .optional()?;
        if exists.is_none() {
            return Err(CoreError::new(
                ErrorCode::NotFound,
                "La entidad no tiene asociado ese recurso externo.",
            ));
        }
        tx.execute(
            "DELETE FROM entity_external_resources WHERE entity_id=?1 AND external_resource_id=?2",
            params![input.entity_id, input.id],
        )?;
        entities::bump(&tx, &entity, "external.unlink", origin, request)?;
        let entity = entities::get(&tx, &input.entity_id)?;
        let resource = get_resource(&tx, &input.id)?;
        tx.commit()?;
        Ok(ExternalResourceResult {
            resource,
            entity,
            changed: true,
        })
    }

    pub fn external_list(&self, input: ListExternalResources) -> Result<Page<ExternalResource>> {
        entities::get(&self.conn, &input.entity_id)?;
        let n = entities::limit(input.limit)?;
        let key = input
            .cursor
            .map(|value| serde_json::from_str::<String>(&value))
            .transpose()?
            .unwrap_or_default();
        let mut stmt = self.conn.prepare(
            "SELECT r.id,r.provider,r.account_id,r.resource_type,r.resource_id,r.metadata,r.created_at,r.updated_at
             FROM external_resources r
             JOIN entity_external_resources e ON e.external_resource_id=r.id
             WHERE e.entity_id=?1 AND r.id>?2 ORDER BY r.id LIMIT ?3",
        )?;
        let items = stmt
            .query_map(params![input.entity_id, key, n + 1], resource_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?
            .into_iter()
            .map(decode_resource)
            .collect::<Result<Vec<_>>>()?;
        Ok(entities::page(items, n, |resource| {
            json!(resource.id).to_string()
        }))
    }
}
