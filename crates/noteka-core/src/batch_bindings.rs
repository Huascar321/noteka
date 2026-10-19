//! Registry de slots semánticos. No interpola strings ni crea rutas.
use crate::{
    batch_plan::{invalid, pointer},
    contracts::*,
    error::*,
};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::collections::BTreeMap;
type Exports = BTreeMap<String, BTreeMap<BatchExport, ExportValue>>;
type Slots = BTreeMap<String, BatchExport>;
fn escape(s: &str) -> String {
    s.replace('~', "~0").replace('/', "~1")
}
fn add(slots: &mut Slots, root: &str, field: &str, kind: BatchExport) {
    slots.insert(format!("{root}/{field}"), kind);
}
fn expression(v: &Value, p: &str, s: &mut Slots) {
    use BatchExport::PropertyId;
    match v["op"].as_str() {
        Some("property") => add(s, p, "property_id", PropertyId),
        Some("rollup") => {
            add(s, p, "relation_id", PropertyId);
            add(s, p, "property_id", PropertyId);
        }
        Some("allocate") => {
            for k in ["relation_id", "cost_id", "weight_id"] {
                add(s, p, k, PropertyId);
            }
        }
        Some("binary") => {
            for k in ["left", "right"] {
                expression(&v[k], &format!("{p}/{k}"), s);
            }
        }
        Some("not") => expression(&v["value"], &format!("{p}/value"), s),
        Some("if") => {
            for k in ["condition", "then", "otherwise"] {
                expression(&v[k], &format!("{p}/{k}"), s);
            }
        }
        _ => {}
    }
}
fn blocks(v: &Value, p: &str, s: &mut Slots) {
    if let Some(items) = v.as_array() {
        for (i, b) in items.iter().enumerate() {
            let p = format!("{p}/{i}");
            if b["type"] == "collection_view" {
                add(
                    s,
                    &format!("{p}/attrs"),
                    "collection_id",
                    BatchExport::CollectionId,
                );
                add(s, &format!("{p}/attrs"), "view_id", BatchExport::ViewId);
            }
            blocks(&b["children"], &format!("{p}/children"), s);
        }
    }
}
fn source<'a>(exports: &'a Exports, b: &Binding) -> Result<&'a ExportValue> {
    exports
        .get(&b.source.step)
        .and_then(|s| s.get(&b.source.export))
        .ok_or_else(|| invalid("unavailable_export"))
}
fn compatible(expected: BatchExport, actual: BatchExport) -> bool {
    expected == actual
        || expected == BatchExport::PropertyId && actual == BatchExport::TitlePropertyId
}
fn relation_slots(
    conn: &Connection,
    v: &Value,
    p: &str,
    property: &str,
    s: &mut Slots,
) -> Result<()> {
    let raw: Option<String> = conn
        .query_row("SELECT spec FROM properties WHERE id=?1", [property], |r| {
            r.get(0)
        })
        .optional()?;
    if let Some(raw) = raw {
        let spec: PropertySpec = serde_json::from_str(&raw)?;
        if spec.kind == PropertyKind::Relation
            && let Some(items) = v.as_array()
        {
            for i in 0..items.len() {
                s.insert(format!("{p}/{i}"), BatchExport::EntityId);
            }
        }
    }
    Ok(())
}
use rusqlite::OptionalExtension;
fn filter(conn: &Connection, v: &Value, p: &str, s: &mut Slots) -> Result<()> {
    add(s, p, "property_id", BatchExport::PropertyId);
    if let Some(id) = v["property_id"].as_str() {
        relation_slots(conn, &v["value"], &format!("{p}/value"), id, s)?;
    }
    Ok(())
}
fn tree(conn: &Connection, v: &Value, p: &str, s: &mut Slots) -> Result<()> {
    match v["op"].as_str() {
        Some("condition") => filter(conn, &v["filter"], &format!("{p}/filter"), s)?,
        Some("all" | "any") => {
            if let Some(a) = v["children"].as_array() {
                for (i, c) in a.iter().enumerate() {
                    tree(conn, c, &format!("{p}/children/{i}"), s)?;
                }
            }
        }
        Some("not") => tree(conn, &v["child"], &format!("{p}/child"), s)?,
        _ => {}
    }
    Ok(())
}
fn config(conn: &Connection, v: &Value, s: &mut Slots) -> Result<()> {
    add(s, "/config", "group_by", BatchExport::PropertyId);
    add(
        s,
        "/config/calendar",
        "property_id",
        BatchExport::PropertyId,
    );
    if let Some(a) = v["config"]["columns"].as_array() {
        for i in 0..a.len() {
            s.insert(format!("/config/columns/{i}"), BatchExport::PropertyId);
        }
    }
    if let Some(a) = v["config"]["sorts"].as_array() {
        for i in 0..a.len() {
            add(
                s,
                &format!("/config/sorts/{i}"),
                "property_id",
                BatchExport::PropertyId,
            );
        }
    }
    if let Some(a) = v["config"]["filters"].as_array() {
        for (i, f) in a.iter().enumerate() {
            filter(conn, f, &format!("/config/filters/{i}"), s)?;
        }
    }
    tree(conn, &v["config"]["filter_tree"], "/config/filter_tree", s)
}
fn slots(
    conn: &Connection,
    op: Option<BatchOperation>,
    v: &Value,
    bindings: &[Binding],
    exports: &Exports,
) -> Result<Slots> {
    use BatchExport::*;
    use BatchOperation::*;
    let mut s = Slots::new();
    match op {
        Some(EntityCreate) => {
            add(&mut s, "/context", "target_id", EntityId);
            add(&mut s, "/record", "collection_id", CollectionId);
        }
        Some(EntityUpdate) => {
            add(&mut s, "", "id", EntityId);
            add(&mut s, "", "expected_revision", EntityRevision);
        }
        Some(CollectionCreate) => {}
        Some(PropertyDefine) => {
            add(&mut s, "", "collection_id", CollectionId);
            add(&mut s, "", "expected_revision", CollectionRevision);
            add(&mut s, "/spec", "target_collection_id", CollectionId);
        }
        Some(FormulaSet) => {
            add(&mut s, "", "collection_id", CollectionId);
            add(&mut s, "", "expected_revision", CollectionRevision);
            add(&mut s, "", "property_id", PropertyId);
            expression(&v["expression"], "/expression", &mut s);
        }
        Some(RecordCreate | RecordAttach | RecordUpdate) => {
            add(&mut s, "", "collection_id", CollectionId);
            if op != Some(RecordCreate) {
                add(&mut s, "", "entity_id", EntityId);
                add(&mut s, "", "expected_revision", EntityRevision);
            }
        }
        Some(ViewSave) => {
            add(&mut s, "", "collection_id", CollectionId);
            add(&mut s, "", "id", ViewId);
            add(&mut s, "", "expected_revision", ViewRevision);
            config(conn, v, &mut s)?;
        }
        None => {
            add(&mut s, "", "collection_id", CollectionId);
            add(&mut s, "", "view_id", ViewId);
            config(conn, v, &mut s)?;
        }
    }
    if matches!(op, Some(EntityCreate | EntityUpdate | RecordCreate)) {
        blocks(&v["document"]["blocks"], "/document/blocks", &mut s);
    }
    let root = match op {
        Some(EntityCreate) => Some("/record/values"),
        Some(RecordCreate | RecordAttach | RecordUpdate) => Some("/values"),
        _ => None,
    };
    if let Some(root) = root
        && let Some(values) = v.pointer(root).and_then(Value::as_object)
    {
        for (key, value) in values {
            let p = format!("{root}/{}", escape(key));
            let id = if let Some((i, b)) = bindings
                .iter()
                .enumerate()
                .find(|(_, b)| b.kind == BindingKind::PropertyKey && b.target == p)
            {
                if b.source.export != PropertyId {
                    let mut e = invalid("property_key_requires_property_id");
                    e.details["binding_index"] = json!(i);
                    return Err(e);
                }
                match source(exports, b)? {
                    ExportValue::Id(id) => id.as_str(),
                    _ => return Err(invalid("export_type_mismatch")),
                }
            } else {
                key.as_str()
            };
            relation_slots(conn, value, &p, id, &mut s)?;
        }
    }
    Ok(s)
}
pub(crate) fn resolve(
    conn: &Connection,
    op: Option<BatchOperation>,
    input: &Value,
    bindings: &[Binding],
    exports: &Exports,
) -> Result<Value> {
    let mut out = input.clone();
    // Resolver primero IDs de propiedad de filtros para conocer el tipo de sus valores.
    for (i, b) in bindings
        .iter()
        .enumerate()
        .filter(|(_, b)| b.kind == BindingKind::Value)
    {
        let value = serde_json::to_value(source(exports, b).map_err(|mut e| {
            e.details["binding_index"] = json!(i);
            e
        })?)?;
        *out.pointer_mut(&b.target)
            .ok_or_else(|| invalid("missing_binding_target"))? = value;
    }
    let allowed = slots(conn, op, &out, bindings, exports)?;
    for (i, b) in bindings.iter().enumerate() {
        let result = (|| {
            pointer(&b.target)?;
            match b.kind {
                BindingKind::Value => {
                    let expected = allowed
                        .get(&b.target)
                        .ok_or_else(|| invalid("forbidden_binding_slot"))?;
                    if !compatible(*expected, b.source.export) {
                        return Err(invalid("export_type_mismatch"));
                    }
                }
                BindingKind::PropertyKey => {
                    let parts = pointer(&b.target)?;
                    let valid = matches!(
                        op,
                        Some(
                            BatchOperation::RecordCreate
                                | BatchOperation::RecordAttach
                                | BatchOperation::RecordUpdate
                        )
                    ) && parts.len() == 2
                        && parts[0] == "values"
                        || op == Some(BatchOperation::EntityCreate)
                            && parts.len() == 3
                            && parts[0] == "record"
                            && parts[1] == "values";
                    if !valid || b.source.export != BatchExport::PropertyId {
                        return Err(invalid("forbidden_property_key_slot"));
                    }
                    let ExportValue::Id(id) = source(exports, b)? else {
                        return Err(invalid("export_type_mismatch"));
                    };
                    let parent = b.target.rsplit_once('/').unwrap().0;
                    let map = out
                        .pointer_mut(parent)
                        .and_then(Value::as_object_mut)
                        .ok_or_else(|| invalid("missing_binding_target"))?;
                    if map.contains_key(id)
                        || input
                            .pointer(parent)
                            .and_then(Value::as_object)
                            .is_some_and(|m| m.contains_key(id))
                    {
                        return Err(invalid("property_key_collision"));
                    }
                    let old = parts.last().unwrap();
                    let value = map
                        .remove(old)
                        .ok_or_else(|| invalid("missing_binding_target"))?;
                    map.insert(id.clone(), value);
                }
            }
            Ok(())
        })();
        result.map_err(|mut e: CoreError| {
            e.details["binding_index"] = json!(i);
            e
        })?;
    }
    Ok(out)
}

pub(crate) fn exports_for(op: BatchOperation) -> Vec<BatchExport> {
    use BatchExport::*;
    use BatchOperation::*;
    match op {
        EntityCreate | EntityUpdate => vec![EntityId, EntityRevision],
        CollectionCreate => vec![CollectionId, CollectionRevision, TitlePropertyId],
        PropertyDefine | FormulaSet => vec![PropertyId, CollectionId, CollectionRevision],
        RecordCreate | RecordAttach | RecordUpdate => vec![EntityId, EntityRevision, CollectionId],
        ViewSave => vec![ViewId, ViewRevision, CollectionId],
    }
}
