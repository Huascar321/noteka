use crate::{
    batch_bindings, batch_plan, batch_storage as store,
    contracts::*,
    database as db, entities,
    error::*,
    formulas,
    mutations::{self, MutationContext},
};
use rusqlite::{Connection, TransactionBehavior};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
pub(crate) fn query_slots() -> Value {
    json!({"collection_id":["/collection_id"],"view_id":["/view_id"],"property_id":["/config/columns/<index>","/config/filters/<index>/property_id","/config/sorts/<index>/property_id","/config/group_by","/config/calendar/property_id","/config/filter_tree/<recognized condition>/filter/property_id"],"entity_id":["/config/<recognized relation filter>/value/<index>"],"conditions":"Índices decimales canónicos; filter_tree sólo condition/all/any/not. Valores de relación requieren propiedad relation existente en la conexión."})
}
pub(crate) fn catalog(inputs: &Value) -> Value {
    use BatchOperation::*;
    let mut out = serde_json::Map::new();
    for op in [
        EntityCreate,
        EntityUpdate,
        CollectionCreate,
        PropertyDefine,
        FormulaSet,
        RecordCreate,
        RecordAttach,
        RecordUpdate,
        ViewSave,
    ] {
        let slots = match op {
            EntityCreate => {
                json!({"entity_id":["/context/target_id","/record/values/<property>/<index>"],"collection_id":["/record/collection_id","/document/<recognized collection_view>/attrs/collection_id"],"view_id":["/document/<recognized collection_view>/attrs/view_id"]})
            }
            EntityUpdate => {
                json!({"entity_id":["/id"],"entity_revision":["/expected_revision"],"collection_id":["/document/<recognized collection_view>/attrs/collection_id"],"view_id":["/document/<recognized collection_view>/attrs/view_id"]})
            }
            CollectionCreate => json!({}),
            PropertyDefine => {
                json!({"collection_id":["/collection_id","/spec/target_collection_id"],"collection_revision":["/expected_revision"]})
            }
            FormulaSet => {
                json!({"collection_id":["/collection_id"],"collection_revision":["/expected_revision"],"property_id":["/property_id","/expression/<recognized AST>/property_id","/expression/<recognized AST>/relation_id","/expression/<recognized allocate>/cost_id","/expression/<recognized allocate>/weight_id"]})
            }
            RecordCreate => {
                json!({"collection_id":["/collection_id","/document/<recognized collection_view>/attrs/collection_id"],"view_id":["/document/<recognized collection_view>/attrs/view_id"],"entity_id":["/values/<relation property>/<index>"]})
            }
            RecordAttach | RecordUpdate => {
                json!({"collection_id":["/collection_id"],"entity_id":["/entity_id","/values/<relation property>/<index>"],"entity_revision":["/expected_revision"]})
            }
            ViewSave => {
                let mut v = query_slots();
                v.as_object_mut().unwrap().remove("view_id");
                v["view_id"] = json!(["/id"]);
                v["view_revision"] = json!(["/expected_revision"]);
                v
            }
        };
        out.insert(op.name().into(),json!({"resolved_input_schema":inputs[op.name()],"exports":batch_bindings::exports_for(op),"value_slots":slots,"property_key_slots":match op {EntityCreate=>vec!["/record/values/<placeholder>"],RecordCreate|RecordAttach|RecordUpdate=>vec!["/values/<placeholder>"],_=>vec![]}}));
    }
    Value::Object(out)
}

fn context(
    mut e: CoreError,
    phase: &str,
    key: Option<&str>,
    index: Option<usize>,
    operation: Option<&str>,
) -> CoreError {
    if e.details.get("phase").is_none() {
        e.details["phase"] = json!(phase);
    }
    if let Some(key) = key {
        e.details["step_key"] = json!(key);
    }
    if let Some(index) = index {
        e.details["step_index"] = json!(index);
    }
    if let Some(operation) = operation {
        e.details["operation"] = json!(operation);
    }
    e
}
fn conflict(reason: &str) -> CoreError {
    let mut e = CoreError::new(
        ErrorCode::RevisionConflict,
        "El estado o efecto ya no coincide con el preview.",
    );
    e.details = json!({"reason":reason});
    e
}
#[derive(serde::Serialize)]
struct Evaluation {
    steps: Vec<BatchStepResult>,
    checks: Vec<BatchCheckResult>,
    diff: Vec<BatchDiff>,
    history_summary: Vec<HistorySummary>,
}
pub(crate) fn validate_shape(op: Option<BatchOperation>, input: Value) -> Result<()> {
    use BatchOperation::*;
    match op {
        Some(EntityCreate) => {
            let _: CreateEntity = serde_json::from_value(input)?;
        }
        Some(EntityUpdate) => {
            let _: UpdateEntity = serde_json::from_value(input)?;
        }
        Some(CollectionCreate) => {
            let _: CreateCollection = serde_json::from_value(input)?;
        }
        Some(PropertyDefine) => {
            let _: DefineProperty = serde_json::from_value(input)?;
        }
        Some(FormulaSet) => {
            let _: crate::contracts::FormulaSet = serde_json::from_value(input)?;
        }
        Some(RecordCreate) => {
            let _: CreateRecord = serde_json::from_value(input)?;
        }
        Some(RecordAttach | RecordUpdate) => {
            let _: UpdateRecord = serde_json::from_value(input)?;
        }
        Some(ViewSave) => {
            let _: SaveView = serde_json::from_value(input)?;
        }
        None => {
            let _: DatabaseQuery = serde_json::from_value(input)?;
        }
    }
    Ok(())
}
fn execute_step(
    conn: &Connection,
    root: &Path,
    op: BatchOperation,
    input: Value,
    c: &mut MutationContext<'_>,
) -> Result<(Value, BTreeMap<BatchExport, ExportValue>)> {
    use BatchExport::*;
    use BatchOperation::*;
    let mut exports = BTreeMap::new();
    let mut id = |key, value: String| {
        exports.insert(key, ExportValue::Id(value));
    };
    let data = match op {
        EntityCreate | EntityUpdate => {
            let e = if op == EntityCreate {
                entities::create_in_transaction(conn, root, serde_json::from_value(input)?, c)?.0
            } else {
                entities::update_in_transaction(conn, root, serde_json::from_value(input)?, c)?
            };
            id(EntityId, e.id.clone());
            exports.insert(EntityRevision, ExportValue::Revision(e.revision));
            serde_json::to_value(e)?
        }
        CollectionCreate => {
            let b = db::collection_create_in_transaction(conn, serde_json::from_value(input)?, c)?;
            id(CollectionId, b.collection.id.clone());
            id(
                TitlePropertyId,
                b.properties
                    .iter()
                    .find(|p| p.spec.kind == PropertyKind::Title)
                    .ok_or_else(|| batch_plan::invalid("missing_title_property"))?
                    .id
                    .clone(),
            );
            exports.insert(
                CollectionRevision,
                ExportValue::Revision(b.collection.revision),
            );
            serde_json::to_value(b)?
        }
        PropertyDefine => {
            let p = db::property_define_in_transaction(conn, serde_json::from_value(input)?, c)?;
            id(PropertyId, p.id.clone());
            id(CollectionId, p.collection_id.clone());
            exports.insert(
                CollectionRevision,
                ExportValue::Revision(db::collection(conn, &p.collection_id)?.revision),
            );
            serde_json::to_value(p)?
        }
        FormulaSet => {
            let input: crate::contracts::FormulaSet = serde_json::from_value(input)?;
            let old = input.property_id.clone();
            let name = input.name.trim().to_string();
            let b = formulas::set_in_transaction(conn, input, c)?;
            let p = b
                .properties
                .iter()
                .find(|p| old.as_ref().map_or(p.spec.name == name, |id| id == &p.id))
                .ok_or_else(|| batch_plan::invalid("missing_formula_property"))?;
            id(PropertyId, p.id.clone());
            id(CollectionId, b.collection.id.clone());
            exports.insert(
                CollectionRevision,
                ExportValue::Revision(b.collection.revision),
            );
            serde_json::to_value(b)?
        }
        RecordCreate | RecordAttach | RecordUpdate => {
            let collection = input["collection_id"]
                .as_str()
                .ok_or_else(|| batch_plan::invalid("collection_id_required"))?
                .to_string();
            let r = if op == RecordCreate {
                db::record_create_in_transaction(conn, root, serde_json::from_value(input)?, c)?
            } else {
                db::record_update_in_transaction(
                    conn,
                    root,
                    serde_json::from_value(input)?,
                    op == RecordAttach,
                    c,
                )?
            };
            id(EntityId, r.entity.id.clone());
            id(CollectionId, collection);
            exports.insert(EntityRevision, ExportValue::Revision(r.entity.revision));
            serde_json::to_value(r)?
        }
        ViewSave => {
            let v = db::view_save_in_transaction(conn, root, serde_json::from_value(input)?, c)?;
            id(ViewId, v.id.clone());
            id(CollectionId, v.collection_id.clone());
            exports.insert(ViewRevision, ExportValue::Revision(v.revision));
            serde_json::to_value(v)?
        }
    };
    Ok((data, exports))
}
fn evaluate(
    conn: &Connection,
    root: &Path,
    plan: &BatchPlan,
    c: &mut MutationContext<'_>,
    budget: &store::Budget<'_>,
) -> Result<Evaluation> {
    let mut out = Evaluation {
        steps: vec![],
        checks: vec![],
        diff: vec![],
        history_summary: vec![],
    };
    let mut exports = BTreeMap::new();
    for (i, step) in plan.steps.iter().enumerate() {
        budget.check()?;
        c.step(&step.key);
        let result = (|| {
            let resolved = batch_bindings::resolve(
                conn,
                Some(step.operation),
                &step.input,
                &step.bindings,
                &exports,
            )?;
            execute_step(conn, root, step.operation, resolved, c)
        })();
        let (data, fields) = result.map_err(|e| {
            context(
                budget.map(e),
                "step",
                Some(&step.key),
                Some(i),
                Some(step.operation.name()),
            )
        })?;
        exports.insert(step.key.clone(), fields.clone());
        out.steps.push(BatchStepResult {
            key: step.key.clone(),
            operation: step.operation,
            data,
            exports: fields,
        });
    }
    for (i, check) in plan.checks.iter().enumerate() {
        budget.check()?;
        let result = (|| {
            let input: DatabaseQuery = serde_json::from_value(batch_bindings::resolve(
                conn,
                None,
                &check.input,
                &check.bindings,
                &exports,
            )?)?;
            let has_cursor = input.cursor.is_some();
            if check.require_complete && has_cursor {
                return Err(batch_plan::invalid("incomplete_check"));
            }
            let page = db::query_in_connection(conn, root, input)?;
            if check.require_complete
                && (has_cursor
                    || page.records.next_cursor.is_some()
                    || page.matching_count as usize != page.records.items.len())
            {
                return Err(batch_plan::invalid("incomplete_check"));
            }
            if check.require_no_calculation_errors
                && let Some(record) = page
                    .records
                    .items
                    .iter()
                    .find(|r| !r.calculation_errors.is_empty())
            {
                let mut e = batch_plan::invalid("calculation_errors");
                e.details["entity_id"] = json!(record.entity.id);
                e.details["calculation_errors"] = json!(record.calculation_errors);
                return Err(e);
            }
            Ok(page)
        })();
        out.checks.push(BatchCheckResult {
            key: check.key.clone(),
            data: result.map_err(|e| {
                context(
                    budget.map(e),
                    "check",
                    Some(&check.key),
                    Some(i),
                    Some("database.query"),
                )
            })?,
        });
    }
    for ((kind, key), before) in &c.before {
        budget.check()?;
        let after = mutations::object(conn, *kind, key)?;
        if before != &after {
            out.diff.push(BatchDiff {
                kind: kind.name().into(),
                key: key.clone(),
                before: before.clone(),
                after,
            });
        }
    }
    out.history_summary = c
        .history
        .iter()
        .map(|h| serde_json::from_value(h.clone()).map_err(CoreError::from))
        .collect::<Result<_>>()?;
    budget.check()?;
    Ok(out)
}
fn entity_clock(v: &mut Value) {
    for k in ["created_at", "updated_at"] {
        if let Some(v) = v.get_mut(k) {
            *v = Value::Null;
        }
    }
}
fn record_clock(v: &mut Value, time_properties: &BTreeSet<String>) {
    if let Some(e) = v.get_mut("entity") {
        entity_clock(e);
    }
    if let Some(values) = v.get_mut("values").and_then(Value::as_object_mut) {
        for (id, value) in values {
            if time_properties.contains(id) {
                *value = Value::Null;
            }
        }
    }
}
fn effect_hash(conn: &Connection, evaluation: &Evaluation) -> Result<String> {
    let mut time_properties = BTreeSet::new();
    let mut stmt = conn.prepare("SELECT id,spec FROM properties")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let raw: String = row.get(1)?;
        // Formatos futuros no se normalizan por nombres parecidos.
        if let Ok(spec) = serde_json::from_str::<PropertySpec>(&raw)
            && matches!(spec.kind, PropertyKind::CreatedAt | PropertyKind::UpdatedAt)
        {
            time_properties.insert(row.get::<_, String>(0)?);
        }
    }
    let mut value = serde_json::to_value(evaluation)?;
    for step in value["steps"].as_array_mut().unwrap() {
        match step["operation"].as_str() {
            Some("entity.create" | "entity.update") => entity_clock(&mut step["data"]),
            Some("record.create" | "record.attach" | "record.update") => {
                record_clock(&mut step["data"], &time_properties)
            }
            _ => {}
        }
    }
    for check in value["checks"].as_array_mut().unwrap() {
        let data = &mut check["data"];
        for r in data["records"]["items"].as_array_mut().unwrap() {
            record_clock(r, &time_properties);
        }
        if data["config"]["group_by"]
            .as_str()
            .is_some_and(|id| time_properties.contains(id))
        {
            for group in data["groups"].as_array_mut().unwrap() {
                group["key"] = Value::Null;
                group["label"] = Value::Null;
            }
        }
    }
    for diff in value["diff"].as_array_mut().unwrap() {
        if diff["kind"] == "entity" {
            for side in ["before", "after"] {
                entity_clock(&mut diff[side]);
            }
        }
    }
    store::json_hash("noteka.batch.effect.v1", &value)
}
fn response_size<T: serde::Serialize>(value: &T, request: &str) -> Result<()> {
    let envelope = json!({"api_version":1,"request_id":request,"ok":true,"data":value});
    if serde_json::to_vec(&envelope)?.len() > batch_plan::MAX_RESPONSE_BYTES {
        return Err(batch_plan::invalid("response_size_limit"));
    }
    Ok(())
}
pub(crate) fn preview(
    path: &Path,
    plan: BatchPlan,
    origin: &str,
    request: &str,
) -> Result<BatchPreview> {
    preview_scoped(path, plan, origin, request, None)
}
pub(crate) fn preview_scoped(
    path: &Path,
    plan: BatchPlan,
    origin: &str,
    request: &str,
    scope: Option<&crate::scoped::LocalScope>,
) -> Result<BatchPreview> {
    let seed = batch_plan::validate(&plan)?;
    let (source, root) = store::open(path, false)?;
    if let Some(scope) = scope {
        scope.check(&source)?;
    }
    let mut conn = store::snapshot(&source)?;
    drop(source);
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(scope) = scope {
        scope.check(&tx)?;
    }
    let budget = store::Budget::new(&tx)?;
    let result = (|| {
        let workspace_id: String =
            tx.query_row("SELECT id FROM workspace_meta", [], |r| r.get(0))?;
        let base = store::base_hash(&tx, &budget)?;
        let mut c = MutationContext::batch(origin, request, seed);
        let e = evaluate(&tx, &root, &plan, &mut c, &budget)?;
        let receipt = BatchReceipt {
            version: 1,
            workspace_id,
            path_hash: store::path_hash(&root),
            plan_hash: store::json_hash("noteka.batch.plan.v1", &serde_json::to_value(&plan)?)?,
            base_hash: base,
            effect_hash: effect_hash(&tx, &e)?,
        };
        let out = BatchPreview {
            applied: false,
            batch_id: plan.batch_id.clone(),
            receipt,
            evaluated_at: c.now(),
            steps: e.steps,
            checks: e.checks,
            diff: e.diff,
            history_summary: e.history_summary,
        };
        response_size(&out, request)?;
        budget.check()?;
        Ok(out)
    })();
    result.map_err(|e| context(budget.map(e), "preview", None, None, None))
}
pub(crate) fn apply(
    path: &Path,
    input: ApplyBatch,
    origin: &str,
    request: &str,
) -> Result<BatchApplied> {
    apply_scoped(path, input, origin, request, None)
}
pub(crate) fn apply_scoped(
    path: &Path,
    input: ApplyBatch,
    origin: &str,
    request: &str,
    scope: Option<&crate::scoped::LocalScope>,
) -> Result<BatchApplied> {
    apply_inner(
        path,
        input,
        origin,
        request,
        scope,
        #[cfg(test)]
        None,
    )
}
#[cfg(test)]
type BeforeCommit<'a> = Option<&'a dyn Fn(&Connection) -> Result<()>>;
#[cfg(test)]
pub(crate) fn apply_injected(
    path: &Path,
    input: ApplyBatch,
    hook: &dyn Fn(&Connection) -> Result<()>,
) -> Result<BatchApplied> {
    apply_inner(path, input, "cli", "injected", None, Some(hook))
}
fn apply_inner(
    path: &Path,
    input: ApplyBatch,
    origin: &str,
    request: &str,
    scope: Option<&crate::scoped::LocalScope>,
    #[cfg(test)] before_commit: BeforeCommit<'_>,
) -> Result<BatchApplied> {
    let seed = batch_plan::validate(&input.plan)?;
    entities::validate_request_limits(&serde_json::to_value(&input)?)?;
    let expected = &input.expected;
    if expected.version != 1
        || uuid::Uuid::parse_str(&expected.workspace_id).is_err()
        || [
            &expected.path_hash,
            &expected.plan_hash,
            &expected.base_hash,
            &expected.effect_hash,
        ]
        .iter()
        .any(|h| {
            h.len() != 64
                || !h
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        })
    {
        return Err(batch_plan::invalid("invalid_receipt"));
    }
    if expected.plan_hash
        != store::json_hash("noteka.batch.plan.v1", &serde_json::to_value(&input.plan)?)?
    {
        return Err(batch_plan::invalid("plan_hash_mismatch"));
    }
    let (mut conn, root) = store::open(path, true)?;
    if let Some(scope) = scope {
        scope.check(&conn)?;
    }
    if expected.path_hash != store::path_hash(&root) {
        return Err(batch_plan::invalid("receipt_scope_mismatch"));
    }
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(scope) = scope {
        scope.check(&tx)?;
    }
    let budget = store::Budget::new(&tx)?;
    let result = (|| {
        store::structure(&tx)?;
        store::size(&tx)?;
        budget.check()?;
        let workspace_id: String =
            tx.query_row("SELECT id FROM workspace_meta", [], |r| r.get(0))?;
        if workspace_id != expected.workspace_id {
            return Err(batch_plan::invalid("receipt_scope_mismatch"));
        }
        if store::base_hash(&tx, &budget)? != expected.base_hash {
            return Err(conflict("base_hash_mismatch"));
        }
        let mut c = MutationContext::batch(origin, request, seed);
        let e = evaluate(&tx, &root, &input.plan, &mut c, &budget)?;
        if effect_hash(&tx, &e)? != expected.effect_hash {
            return Err(conflict("effect_hash_mismatch"));
        }
        let out = BatchApplied {
            applied: true,
            batch_id: input.plan.batch_id.clone(),
            executed_at: c.now(),
            steps: e.steps,
            checks: e.checks,
            diff: e.diff,
            history_summary: e.history_summary,
        };
        response_size(&out, request)?;
        #[cfg(test)]
        if let Some(hook) = before_commit {
            hook(&tx)?;
        }
        budget.check()?;
        Ok(out)
    })()
    .map_err(|e| context(budget.map(e), "apply", None, None, None));
    drop(budget);
    let out = result?;
    tx.commit()?;
    Ok(out)
}

#[cfg(test)]
mod measurements {
    use super::*;
    use std::time::Instant;
    #[test]
    fn response_envelope_at_two_mib_is_accepted_one_byte_more_is_rejected() {
        let request = "00000000-0000-0000-0000-000000000000";
        let empty = json!({"padding":""});
        let overhead = serde_json::to_vec(
            &json!({"api_version":1,"request_id":request,"ok":true,"data":empty}),
        )
        .unwrap()
        .len();
        let exact = json!({"padding":"x".repeat(batch_plan::MAX_RESPONSE_BYTES-overhead)});
        assert!(response_size(&exact, request).is_ok());
        let excessive = json!({"padding":"x".repeat(batch_plan::MAX_RESPONSE_BYTES-overhead+1)});
        assert!(response_size(&excessive, request).is_err());
    }
    #[test]
    #[ignore = "Medición release explícita, no cambia datos del usuario"]
    fn release_batch_resources() {
        let temp = tempfile::tempdir().unwrap();
        let mut w = crate::workspace::Workspace::create(&temp.path().join("w"), Some("UTC".into()))
            .unwrap();
        let tx = w.conn.transaction().unwrap();
        let timestamp = crate::workspace::now();
        for i in 0..10000 {
            let id = format!("{:08x}-0000-4000-8000-000000000000", i);
            tx.execute(
                "INSERT INTO entities VALUES (?1,?2,?3,1,NULL,?4,?4,?5)",
                rusqlite::params![
                    id,
                    if i < 1000 { "tag" } else { "note" },
                    format!("Synthetic {i}"),
                    timestamp,
                    if i < 1000 {
                        r#"{"classification":"topic"}"#
                    } else {
                        r#"{"inbox":false}"#
                    }
                ],
            )
            .unwrap();
            if i >= 1000 {
                tx.execute("INSERT INTO documents VALUES (?1,1,?2)",rusqlite::params![id,serde_json::to_string(&json!({"schema_version":1,"blocks":[{"id":format!("b{i}"),"type":"paragraph","content":[{"type":"text","text":"synthetic ".repeat(40)}]}]})).unwrap()]).unwrap();
            }
        }
        for i in 0..20000 {
            tx.execute(
                "INSERT INTO relations VALUES (?1,?2,?3,'tag')",
                rusqlite::params![
                    format!("relation-{i}"),
                    format!("{:08x}-0000-4000-8000-000000000000", 1000 + i % 9000),
                    format!(
                        "{:08x}-0000-4000-8000-000000000000",
                        (i % 1000 + i / 9000) % 1000
                    )
                ],
            )
            .unwrap();
        }
        crate::search::rebuild(&tx).unwrap();
        tx.commit().unwrap();
        let mut samples = vec![];
        for iteration in 0..25 {
            let plan:BatchPlan=serde_json::from_value(json!({"version":1,"batch_id":uuid::Uuid::new_v4(),"steps":[{"key":"note","operation":"entity.create","input":{"kind":"note","title":"Benchmark note"}}]})).unwrap();
            let start = Instant::now();
            let (source, root) = store::open(&w.path, false).unwrap();
            let preflight = start.elapsed().as_secs_f64() * 1000.;
            let start = Instant::now();
            let mut snapshot = store::snapshot(&source).unwrap();
            let snapshot_ms = start.elapsed().as_secs_f64() * 1000.;
            drop(source);
            let preview_tx = snapshot.transaction().unwrap();
            let budget = store::Budget::new(&preview_tx).unwrap();
            let start = Instant::now();
            let base = store::base_hash(&preview_tx, &budget).unwrap();
            let hash_ms = start.elapsed().as_secs_f64() * 1000.;
            let start = Instant::now();
            let mut c = MutationContext::batch(
                "cli",
                "benchmark",
                uuid::Uuid::parse_str(&plan.batch_id).unwrap(),
            );
            let e = evaluate(&preview_tx, &root, &plan, &mut c, &budget).unwrap();
            let effect = effect_hash(&preview_tx, &e).unwrap();
            let simulation = start.elapsed().as_secs_f64() * 1000.;
            let output_bytes = serde_json::to_vec(&e).unwrap().len();
            drop(budget);
            drop(preview_tx);
            let start = Instant::now();
            let (mut conn, root) = store::open(&w.path, true).unwrap();
            let apply_open = start.elapsed().as_secs_f64() * 1000.;
            let wait = Instant::now();
            let tx = conn
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .unwrap();
            let lock_wait = wait.elapsed().as_secs_f64() * 1000.;
            let lock_start = Instant::now();
            let budget = store::Budget::new(&tx).unwrap();
            store::structure(&tx).unwrap();
            store::size(&tx).unwrap();
            assert_eq!(store::base_hash(&tx, &budget).unwrap(), base);
            let mut c = MutationContext::batch(
                "cli",
                "benchmark-real",
                uuid::Uuid::parse_str(&plan.batch_id).unwrap(),
            );
            let actual = evaluate(&tx, &root, &plan, &mut c, &budget).unwrap();
            assert_eq!(effect_hash(&tx, &actual).unwrap(), effect);
            response_size(&actual, "benchmark-real").unwrap();
            budget.check().unwrap();
            drop(budget);
            let start = Instant::now();
            tx.commit().unwrap();
            let commit = start.elapsed().as_secs_f64() * 1000.;
            let lock_hold = lock_start.elapsed().as_secs_f64() * 1000.;
            if iteration >= 5 {
                samples.push(json!({"preflight_ms":preflight,"snapshot_ms":snapshot_ms,"hash_ms":hash_ms,"simulation_ms":simulation,"apply_open_ms":apply_open,"lock_wait_ms":lock_wait,"lock_hold_ms":lock_hold,"commit_ms":commit,"output_bytes":output_bytes}));
            }
        }
        let pages: u32 = w
            .conn
            .pragma_query_value(None, "page_count", |r| r.get(0))
            .unwrap();
        let page_size: u32 = w
            .conn
            .pragma_query_value(None, "page_size", |r| r.get(0))
            .unwrap();
        println!(
            "BATCH_BENCHMARK {}",
            json!({"fixture_entities":10000,"fixture_relations":20000,"steps":1,"checks":0,"warmups":5,"repetitions":20,"snapshot_bytes":u64::from(pages)*u64::from(page_size),"samples":samples})
        );
    }
}
