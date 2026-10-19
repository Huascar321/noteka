use crate::{
    contracts::*,
    entities::{bump_in_transaction, expect, get, limit, page},
    error::*,
    mutations::{MutationContext, ObjectKind},
    productivity::civil_date,
    workspace::Workspace,
};
use rusqlite::{Connection, TransactionBehavior, params};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashSet},
    path::Path,
};

pub(crate) fn name(s: &str) -> Result<String> {
    let s = s.trim();
    if s.is_empty() || s.len() > 256 {
        return Err(CoreError::validation("Nombre vacío o mayor que 256 bytes."));
    }
    Ok(s.into())
}
pub(crate) fn revision(id: &str, current: u32, expected: u32) -> Result<()> {
    if expected == 0 {
        return Err(CoreError::validation(
            "expected_revision debe ser positivo.",
        ));
    }
    if current != expected {
        let mut e = CoreError::new(
            ErrorCode::RevisionConflict,
            "El esquema o la vista cambió; no se reemplazó el buffer.",
        );
        e.details = json!({"id":id,"current_revision":current,"expected_revision":expected});
        return Err(e);
    }
    Ok(())
}
pub(crate) fn collection(conn: &Connection, id: &str) -> Result<Collection> {
    Ok(conn.query_row(
        "SELECT name,revision FROM collections WHERE id=?1",
        [id],
        |r| {
            Ok(Collection {
                id: id.into(),
                name: r.get(0)?,
                revision: r.get(1)?,
            })
        },
    )?)
}
pub(crate) fn properties(conn: &Connection, id: &str) -> Result<Vec<Property>> {
    collection(conn, id)?;
    let mut stmt =
        conn.prepare("SELECT id,spec FROM properties WHERE collection_id=?1 ORDER BY rowid")?;
    let raw = stmt
        .query_map([id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    raw.into_iter()
        .map(|(key, s)| {
            Ok(Property {
                id: key,
                collection_id: id.into(),
                spec: serde_json::from_str(&s).map_err(|_| {
                    CoreError::new(
                        ErrorCode::StorageError,
                        "Esquema de propiedad desconocido; no se convirtió.",
                    )
                })?,
            })
        })
        .collect()
}
pub(crate) fn property(conn: &Connection, id: &str) -> Result<Property> {
    let (c, s): (String, String) = conn.query_row(
        "SELECT collection_id,spec FROM properties WHERE id=?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    Ok(Property {
        id: id.into(),
        collection_id: c,
        spec: serde_json::from_str(&s)?,
    })
}
pub(crate) fn member(conn: &Connection, c: &str, e: &str) -> Result<()> {
    if !conn
        .prepare("SELECT 1 FROM memberships WHERE collection_id=?1 AND entity_id=?2")?
        .exists(params![c, e])?
    {
        return Err(CoreError::validation(
            "La entidad no pertenece a la colección declarada.",
        ));
    }
    Ok(())
}
fn validate_spec(conn: &Connection, s: &PropertySpec) -> Result<()> {
    if s.calculation_type.is_some()
        || matches!(s.kind, PropertyKind::Formula | PropertyKind::Rollup)
    {
        return Err(CoreError::validation(
            "Calculadas se definen por formula.set, no como valores editables.",
        ));
    }
    name(&s.name)?;
    if s.kind == PropertyKind::Money {
        let c = s.currency.as_deref().unwrap_or("");
        if c.len() != 3 || !c.bytes().all(|b| b.is_ascii_uppercase()) {
            return Err(CoreError::validation(
                "Moneda debe ser un código de tres mayúsculas explícito.",
            ));
        }
    } else if s.currency.is_some() {
        return Err(CoreError::validation("Sólo money admite currency."));
    }
    if s.kind == PropertyKind::Relation {
        collection(
            conn,
            s.target_collection_id
                .as_deref()
                .ok_or_else(|| CoreError::validation("Relación requiere target_collection_id."))?,
        )?;
    } else if s.target_collection_id.is_some() {
        return Err(CoreError::validation(
            "Sólo relation admite colección destino.",
        ));
    }
    if matches!(s.kind, PropertyKind::Select | PropertyKind::MultiSelect) {
        if s.options.is_empty()
            || s.options.len() > 100
            || s.options
                .iter()
                .any(|s| s.trim().is_empty() || s.len() > 256)
            || s.options.iter().collect::<HashSet<_>>().len() != s.options.len()
        {
            return Err(CoreError::validation(
                "Selección requiere 1–100 opciones únicas no vacías.",
            ));
        }
    } else if !s.options.is_empty() {
        return Err(CoreError::validation(
            "Opciones sólo corresponden a selección.",
        ));
    }
    Ok(())
}
// Escala común 9, hasta 28 dígitos originales: <=37 dígitos, dentro de i128.
pub(crate) fn decimal(s: &str) -> Result<i128> {
    let (negative, s) = s.strip_prefix('-').map_or((false, s), |s| (true, s));
    let parts = s.split('.').collect::<Vec<_>>();
    let whole = parts[0];
    let fraction = parts.get(1).copied().unwrap_or("");
    if parts.len() > 2
        || whole.is_empty()
        || (whole.len() > 1 && whole.starts_with('0'))
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || (!fraction.is_empty() && !fraction.bytes().all(|b| b.is_ascii_digit()))
        || (parts.len() == 2 && fraction.is_empty())
        || fraction.len() > 9
        || whole.len() + fraction.len() > 28
    {
        return Err(CoreError::validation(
            "Decimal: string sin exponente, hasta 28 dígitos/9 decimales, sin ceros iniciales.",
        ));
    }
    let mantissa = format!("{whole}{fraction}")
        .parse::<i128>()
        .map_err(|_| CoreError::validation("Decimal fuera de rango."))?;
    if negative && mantissa == 0 {
        return Err(CoreError::validation("Decimal no admite cero negativo."));
    }
    let v = mantissa * 10_i128.pow(9 - fraction.len() as u32);
    Ok(if negative { -v } else { v })
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CivilRange {
    start: String,
    end: String,
}
fn strings(v: &Value) -> Result<Vec<&str>> {
    v.as_array()
        .ok_or_else(|| CoreError::validation("Se requiere un array de IDs/opciones."))?
        .iter()
        .map(|x| {
            x.as_str()
                .ok_or_else(|| CoreError::validation("Array requiere texto."))
        })
        .collect()
}
fn validate_value(
    conn: &Connection,
    root: &Path,
    s: &PropertySpec,
    v: &Value,
    write: bool,
) -> Result<()> {
    use PropertyKind::*;
    if matches!(s.kind, Formula | Rollup) {
        if write {
            return Err(CoreError::validation(
                "Propiedad calculada: no admite escritura de valores.",
            ));
        }
        return crate::formulas::validate_scalar(
            s.calculation_type
                .as_ref()
                .ok_or_else(|| CoreError::validation("Calculada sin tipo."))?,
            v,
        )
        .map_err(crate::formulas::core_error);
    }
    if write && matches!(s.kind, Title | CreatedAt | UpdatedAt) {
        return Err(CoreError::validation(
            "Título/timestamps son proyecciones; editá el título por entity.update.",
        ));
    }
    if v.is_null() {
        return Ok(());
    }
    let invalid =
        || CoreError::validation(format!("Valor incompatible con {} ({:?}).", s.name, s.kind));
    match s.kind {
        Title | Text => {
            if !v.is_string() || v.as_str().unwrap().len() > 16_384 {
                return Err(invalid());
            }
        }
        Number => {
            if !v
                .as_f64()
                .is_some_and(|n| n.is_finite() && n.abs() <= 9_007_199_254_740_991.)
            {
                return Err(invalid());
            }
        }
        Decimal | Money => {
            decimal(v.as_str().ok_or_else(invalid)?)?;
        }
        Checkbox => {
            if !v.is_boolean() {
                return Err(invalid());
            }
        }
        Date => {
            civil_date(v.as_str().ok_or_else(invalid)?)?;
        }
        DateRange => {
            let r: CivilRange = serde_json::from_value(v.clone())?;
            civil_date(&r.start)?;
            civil_date(&r.end)?;
            if r.end < r.start {
                return Err(invalid());
            }
        }
        Url => {
            let s = v.as_str().ok_or_else(invalid)?;
            if !(s.starts_with("https://") || s.starts_with("http://"))
                || s.len() > 4096
                || s.bytes()
                    .any(|b| b.is_ascii_whitespace() || b.is_ascii_control())
                || s.split("://").nth(1).unwrap_or("").is_empty()
            {
                return Err(invalid());
            }
        }
        Email => {
            let s = v.as_str().ok_or_else(invalid)?;
            let parts = s.split('@').collect::<Vec<_>>();
            if parts.len() != 2
                || parts[0].is_empty()
                || !parts[1].contains('.')
                || parts[1].starts_with('.')
                || parts[1].ends_with('.')
                || s.len() > 320
                || s.bytes()
                    .any(|b| b.is_ascii_whitespace() || b.is_ascii_control())
            {
                return Err(invalid());
            }
        }
        Select => {
            if !s.options.iter().any(|s| Some(s.as_str()) == v.as_str()) {
                return Err(invalid());
            }
        }
        MultiSelect | Files | Relation => {
            let values = strings(v)?;
            if values.len() > 100 || values.iter().collect::<HashSet<_>>().len() != values.len() {
                return Err(invalid());
            }
            for value in values {
                match s.kind {
                    MultiSelect => {
                        if !s.options.iter().any(|s| s == value) {
                            return Err(invalid());
                        }
                    }
                    Files => {
                        let a = crate::attachments::metadata(conn, value).map_err(|_| invalid())?;
                        if write {
                            crate::attachments::verify(root, &a)?;
                        }
                    }
                    Relation => member(
                        conn,
                        s.target_collection_id.as_deref().ok_or_else(invalid)?,
                        value,
                    )?,
                    _ => unreachable!(),
                }
            }
        }
        CreatedAt | UpdatedAt => {
            chrono::DateTime::parse_from_rfc3339(v.as_str().ok_or_else(invalid)?)
                .map_err(|_| invalid())?;
        }
        Formula | Rollup => unreachable!(),
    }
    Ok(())
}
pub(crate) fn validate_values(
    conn: &Connection,
    root: &Path,
    c: &str,
    values: &Values,
    write: bool,
) -> Result<()> {
    let props = properties(conn, c)?;
    if values.len() > 128 {
        return Err(CoreError::validation("Patch supera 128 propiedades."));
    }
    for (key, value) in values {
        let p = props.iter().find(|p| &p.id == key).ok_or_else(|| {
            CoreError::validation("Propiedad ajena/desconocida; no se escribieron valores.")
        })?;
        // Las definiciones/defaults sólo admiten propiedades editables incluso al leer.
        if matches!(
            p.spec.kind,
            PropertyKind::Title
                | PropertyKind::CreatedAt
                | PropertyKind::UpdatedAt
                | PropertyKind::Formula
                | PropertyKind::Rollup
        ) {
            return Err(CoreError::validation(
                "Defaults/patch sólo admiten propiedades editables.",
            ));
        }
        validate_value(conn, root, &p.spec, value, write)?;
    }
    Ok(())
}
fn write_values(conn: &Connection, root: &Path, c: &str, e: &str, values: &Values) -> Result<()> {
    validate_values(conn, root, c, values, true)?;
    let props = properties(conn, c)?;
    for (key, value) in values {
        let p = props.iter().find(|p| &p.id == key).expect("Mapa validado");
        let mut old =
            conn.prepare("SELECT json FROM property_values WHERE entity_id=?1 AND property_id=?2")?;
        let mut rows = old.query(params![e, key])?;
        if let Some(row) = rows.next()? {
            let raw: String = row.get(0)?;
            let original: Value = serde_json::from_str(&raw)?;
            validate_value(conn, root, &p.spec, &original, false).map_err(|_| {
                CoreError::validation(
                    "El valor original tiene formato no reconocido; no se reemplazó ni descartó.",
                )
            })?;
        }
        if value.is_null() {
            conn.execute(
                "DELETE FROM property_values WHERE entity_id=?1 AND property_id=?2",
                params![e, key],
            )?;
        } else {
            conn.execute("INSERT INTO property_values VALUES (?1,?2,?3) ON CONFLICT(entity_id,property_id) DO UPDATE SET json=excluded.json",params![e,key,value.to_string()])?;
        }
    }
    Ok(())
}
pub(crate) fn attach(conn: &Connection, root: &Path, e: &str, seed: &RecordSeed) -> Result<()> {
    collection(conn, &seed.collection_id)?;
    if !conn
        .prepare("SELECT 1 FROM memberships WHERE collection_id=?1 AND entity_id=?2")?
        .exists(params![seed.collection_id, e])?
    {
        let count: u32 = conn.query_row(
            "SELECT count(*) FROM memberships WHERE collection_id=?1",
            [&seed.collection_id],
            |r| r.get(0),
        )?;
        if count >= 10_000 {
            return Err(CoreError::validation(
                "Límite inicial: 10.000 registros por colección.",
            ));
        }
        conn.execute(
            "INSERT INTO memberships VALUES (?1,?2)",
            params![seed.collection_id, e],
        )?;
    }
    write_values(conn, root, &seed.collection_id, e, &seed.values)
}
pub(crate) fn observe_record(
    conn: &Connection,
    e: &str,
    seed: &RecordSeed,
    context: &mut crate::mutations::MutationContext<'_>,
) -> Result<()> {
    use crate::mutations::ObjectKind;
    context.observe(conn, ObjectKind::Membership, &[&seed.collection_id, e])?;
    for key in seed.values.keys() {
        context.observe(conn, ObjectKind::Value, &[e, key])?;
    }
    Ok(())
}
pub(crate) fn record(conn: &Connection, c: &str, e: &str) -> Result<RecordData> {
    let mut r = record_raw(conn, c, e)?;
    let mut evaluator = crate::formulas::Evaluator::new(conn);
    evaluator.hydrate(&properties(conn, c)?, &mut r)?;
    Ok(r)
}
fn record_raw(conn: &Connection, c: &str, e: &str) -> Result<RecordData> {
    member(conn, c, e)?;
    let entity = get(conn, e)?;
    let mut values = Values::new();
    let mut stmt=conn.prepare("SELECT pv.property_id,pv.json FROM property_values pv JOIN properties p ON p.id=pv.property_id WHERE pv.entity_id=?1 AND p.collection_id=?2 ORDER BY pv.property_id")?;
    for row in stmt.query_map(params![e, c], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
    })? {
        let (k, v) = row?;
        values.insert(k, serde_json::from_str(&v)?);
    }
    for p in properties(conn, c)? {
        let v = match p.spec.kind {
            PropertyKind::Title => json!(entity.title),
            PropertyKind::CreatedAt => json!(entity.created_at),
            PropertyKind::UpdatedAt => json!(entity.updated_at),
            _ => continue,
        };
        values.insert(p.id, v);
    }
    let mut stmt = conn.prepare(
        "SELECT collection_id FROM memberships WHERE entity_id=?1 ORDER BY collection_id",
    )?;
    let collection_ids = stmt
        .query_map([e], |r| r.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(RecordData {
        entity,
        values,
        collection_ids,
        calculation_errors: BTreeMap::new(),
    })
}
fn views(conn: &Connection, c: &str) -> Result<Vec<CollectionView>> {
    let mut stmt = conn.prepare("SELECT json FROM views WHERE collection_id=?1 ORDER BY id")?;
    let raw = stmt
        .query_map([c], |r| r.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    raw.iter()
        .map(|s| serde_json::from_str(s).map_err(CoreError::from))
        .collect()
}
pub(crate) fn bundle(conn: &Connection, c: &str) -> Result<CollectionBundle> {
    Ok(CollectionBundle {
        collection: collection(conn, c)?,
        properties: properties(conn, c)?,
        views: views(conn, c)?,
        calculations: crate::formulas::definitions(conn, c)?,
    })
}
fn config(conn: &Connection, root: &Path, props: &[Property], cfg: &ViewConfig) -> Result<()> {
    if cfg.columns.len() > 128 || cfg.filters.len() > 16 || cfg.sorts.len() > 4 {
        return Err(CoreError::validation(
            "Vista excede columnas/filtros/ordenamientos permitidos.",
        ));
    }
    let find = |id: &str| {
        props
            .iter()
            .find(|p| p.id == id)
            .ok_or_else(|| CoreError::validation("Vista refiere propiedad ajena o desconocida."))
    };
    if cfg.columns.iter().collect::<HashSet<_>>().len() != cfg.columns.len() {
        return Err(CoreError::validation("Columnas duplicadas."));
    }
    for id in &cfg.columns {
        find(id)?;
    }
    for s in &cfg.sorts {
        find(&s.property_id)?;
    }
    for f in crate::database_filters::conditions(cfg)? {
        let p = find(&f.property_id)?;
        match f.op {
            FilterOp::Eq | FilterOp::Ne => validate_value(conn, root, &p.spec, &f.value, false)?,
            FilterOp::Lt | FilterOp::Le | FilterOp::Gt | FilterOp::Ge => {
                if f.value.is_null() || !crate::database_filters::ordered(p) {
                    return Err(CoreError::validation(
                        "Comparador ordenado incompatible o value null.",
                    ));
                }
                validate_value(conn, root, &p.spec, &f.value, false)?;
            }
            FilterOp::IsEmpty => {
                if !f.value.is_null() {
                    return Err(CoreError::validation("is_empty no admite value."));
                }
            }
            FilterOp::Contains => {
                if !(p
                    .spec
                    .calculation_type
                    .as_ref()
                    .is_some_and(|t| t.kind == ScalarKind::Text)
                    || matches!(
                        p.spec.kind,
                        PropertyKind::Text
                            | PropertyKind::Title
                            | PropertyKind::Url
                            | PropertyKind::Email
                    ))
                    || !f.value.is_string()
                {
                    return Err(CoreError::validation("contains sólo admite texto."));
                }
            }
        }
    }
    Ok(())
}
pub(crate) fn compare(kind: &PropertyKind, a: &Value, b: &Value) -> Ordering {
    if a.is_null() || b.is_null() {
        return match (a.is_null(), b.is_null()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
            _ => Ordering::Less,
        };
    }
    match kind {
        PropertyKind::Number => a
            .as_f64()
            .unwrap_or(0.)
            .partial_cmp(&b.as_f64().unwrap_or(0.))
            .unwrap_or(Ordering::Equal),
        PropertyKind::Decimal | PropertyKind::Money => decimal(a.as_str().unwrap_or(""))
            .unwrap_or(0)
            .cmp(&decimal(b.as_str().unwrap_or("")).unwrap_or(0)),
        _ if a.is_string() && b.is_string() => a.as_str().unwrap().cmp(b.as_str().unwrap()),
        _ => a.to_string().cmp(&b.to_string()),
    }
}
pub(crate) fn sort_kind(p: &Property) -> PropertyKind {
    match p.spec.calculation_type.as_ref().map(|t| &t.kind) {
        Some(ScalarKind::Number) => PropertyKind::Number,
        Some(ScalarKind::Decimal) => PropertyKind::Decimal,
        _ => p.spec.kind.clone(),
    }
}
pub(crate) fn empty(v: &Value) -> bool {
    v.is_null() || v.as_str().is_some_and(str::is_empty) || v.as_array().is_some_and(Vec::is_empty)
}

pub(crate) fn collection_create_in_transaction(
    tx: &Connection,
    input: CreateCollection,
    context: &mut crate::mutations::MutationContext<'_>,
) -> Result<CollectionBundle> {
    use crate::mutations::ObjectKind;
    let name = name(&input.name)?;
    let c = context.new_id(tx, "collection", ObjectKind::Collection)?;
    let title_id = context.new_id(tx, "property", ObjectKind::Property)?;
    context.observe(tx, ObjectKind::Collection, &[&c])?;
    context.observe(tx, ObjectKind::Property, &[&title_id])?;
    tx.execute("INSERT INTO collections VALUES (?1,?2,1)", params![c, name])?;
    let spec = PropertySpec {
        name: "Título".into(),
        kind: PropertyKind::Title,
        options: vec![],
        currency: None,
        target_collection_id: None,
        calculation_type: None,
    };
    tx.execute(
        "INSERT INTO properties VALUES (?1,?2,?3,?4)",
        params![title_id, c, spec.name, serde_json::to_string(&spec)?],
    )?;
    bundle(tx, &c)
}
impl Workspace {
    pub fn collection_create(&mut self, input: CreateCollection) -> Result<CollectionBundle> {
        name(&input.name)?; // Preserva prioridad de validación individual previa al lock.
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let result = collection_create_in_transaction(
            &tx,
            input,
            &mut crate::mutations::MutationContext::individual("internal", ""),
        )?;
        tx.commit()?;
        Ok(result)
    }
    pub fn collection_get(&self, id: &str) -> Result<CollectionBundle> {
        let tx = self.conn.unchecked_transaction()?;
        bundle(&tx, id)
    }
    pub fn collection_list(&self, input: ListCollections) -> Result<Page<Collection>> {
        let n = limit(input.limit)?;
        let mut stmt = self
            .conn
            .prepare("SELECT id,name,revision FROM collections WHERE id>?1 ORDER BY id LIMIT ?2")?;
        let items = stmt
            .query_map(params![input.cursor.unwrap_or_default(), n + 1], |r| {
                Ok(Collection {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    revision: r.get(2)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(page(items, n, |c| c.id.clone()))
    }
}
pub(crate) fn property_define_in_transaction(
    tx: &Connection,
    mut input: DefineProperty,
    context: &mut crate::mutations::MutationContext<'_>,
) -> Result<Property> {
    use crate::mutations::ObjectKind;
    input.spec.name = name(&input.spec.name)?;
    let c = collection(tx, &input.collection_id)?;
    revision(&c.id, c.revision, input.expected_revision)?;
    validate_spec(tx, &input.spec)?;
    let existing = properties(tx, &c.id)?;
    if input.spec.kind == PropertyKind::Title
        || existing.len() >= 128
        || existing.iter().any(|p| p.spec.name == input.spec.name)
    {
        return Err(CoreError::validation(
            "Título duplicado, nombre existente o más de 128 propiedades.",
        ));
    }
    if c.revision == u32::MAX {
        return Err(CoreError::validation("Límite de revisión."));
    }
    let p = Property {
        id: context.new_id(tx, "property", ObjectKind::Property)?,
        collection_id: c.id,
        spec: input.spec,
    };
    context.observe(tx, ObjectKind::Collection, &[&p.collection_id])?;
    context.observe(tx, ObjectKind::Property, &[&p.id])?;
    tx.execute(
        "INSERT INTO properties VALUES (?1,?2,?3,?4)",
        params![
            p.id,
            p.collection_id,
            p.spec.name,
            serde_json::to_string(&p.spec)?
        ],
    )?;
    tx.execute(
        "UPDATE collections SET revision=revision+1 WHERE id=?1",
        [&p.collection_id],
    )?;
    Ok(p)
}
impl Workspace {
    pub fn property_define(&mut self, input: DefineProperty) -> Result<Property> {
        name(&input.spec.name)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let result = property_define_in_transaction(
            &tx,
            input,
            &mut crate::mutations::MutationContext::individual("internal", ""),
        )?;
        tx.commit()?;
        Ok(result)
    }
    pub fn record_create(
        &mut self,
        input: CreateRecord,
        origin: &str,
        request: &str,
    ) -> Result<RecordData> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let result = record_create_in_transaction(
            &tx,
            &self.path,
            input,
            &mut MutationContext::individual(origin, request),
        )?;
        tx.commit()?;
        Ok(result)
    }
    pub fn record_get(&self, input: RecordInput) -> Result<RecordData> {
        let tx = self.conn.unchecked_transaction()?;
        record(&tx, &input.collection_id, &input.entity_id)
    }
}
pub(crate) fn record_update_in_transaction(
    tx: &Connection,
    root: &Path,
    input: UpdateRecord,
    add: bool,
    context: &mut MutationContext<'_>,
) -> Result<RecordData> {
    let e = get(tx, &input.entity_id)?;
    expect(&e, input.expected_revision)?;
    context.observe(tx, ObjectKind::Entity, &[&e.id])?;
    let seed = RecordSeed {
        collection_id: input.collection_id.clone(),
        values: input.values,
    };
    observe_record(tx, &e.id, &seed, context)?;
    if add {
        attach(tx, root, &e.id, &seed)?;
    } else {
        member(tx, &input.collection_id, &e.id)?;
        write_values(tx, root, &input.collection_id, &e.id, &seed.values)?;
    }
    bump_in_transaction(
        tx,
        &e,
        if add {
            "record.attach"
        } else {
            "record.update"
        },
        context,
    )?;
    record(tx, &input.collection_id, &e.id)
}
impl Workspace {
    pub fn record_update(
        &mut self,
        input: UpdateRecord,
        add: bool,
        origin: &str,
        request: &str,
    ) -> Result<RecordData> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let result = record_update_in_transaction(
            &tx,
            &self.path,
            input,
            add,
            &mut MutationContext::individual(origin, request),
        )?;
        tx.commit()?;
        Ok(result)
    }
    pub fn record_inverse(&self, input: InverseQuery) -> Result<Page<RecordData>> {
        let tx = self.conn.unchecked_transaction()?;
        let p = property(&tx, &input.property_id)?;
        if p.spec.kind != PropertyKind::Relation {
            return Err(CoreError::validation(
                "Inversa requiere propiedad relation.",
            ));
        }
        member(
            &tx,
            p.spec.target_collection_id.as_deref().unwrap_or(""),
            &input.target_id,
        )?;
        let n = limit(input.limit)?;
        let mut stmt=tx.prepare("SELECT pv.entity_id FROM property_values pv WHERE pv.property_id=?1 AND pv.entity_id>?2 AND EXISTS(SELECT 1 FROM json_each(pv.json) WHERE value=?3) ORDER BY pv.entity_id LIMIT ?4")?;
        let ids = stmt
            .query_map(
                params![
                    p.id,
                    input.cursor.unwrap_or_default(),
                    input.target_id,
                    n + 1
                ],
                |r| r.get::<_, String>(0),
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let records = ids
            .iter()
            .map(|id| record(&tx, &p.collection_id, id))
            .collect::<Result<Vec<_>>>()?;
        Ok(page(records, n, |r| r.entity.id.clone()))
    }
}
pub(crate) fn view_save_in_transaction(
    tx: &Connection,
    root: &Path,
    input: SaveView,
    context: &mut MutationContext<'_>,
) -> Result<CollectionView> {
    let props = properties(tx, &input.collection_id)?;
    config(tx, root, &props, &input.config)?;
    crate::collection_views::validate(&input.layout, &props, &input.config)?;
    let revision = if let Some(ref id) = input.id {
        let old: CollectionView = serde_json::from_str(&tx.query_row(
            "SELECT json FROM views WHERE id=?1 AND collection_id=?2",
            params![id, input.collection_id],
            |r| r.get::<_, String>(0),
        )?)?;
        revision(
            id,
            old.revision,
            input
                .expected_revision
                .ok_or_else(|| CoreError::validation("Actualizar vista requiere revisión."))?,
        )?;
        old.revision
            .checked_add(1)
            .ok_or_else(|| CoreError::validation("Límite de revisión."))?
    } else {
        if input.expected_revision.is_some() {
            return Err(CoreError::validation(
                "Nueva vista no lleva expected_revision.",
            ));
        }
        1
    };
    let key = match input.id {
        Some(id) => id,
        None => context.new_id(tx, "view", ObjectKind::View)?,
    };
    context.observe(tx, ObjectKind::View, &[&key])?;
    let v = CollectionView {
        id: key,
        collection_id: input.collection_id,
        revision,
        name: name(&input.name)?,
        layout: input.layout,
        config: input.config,
    };
    tx.execute("INSERT INTO views VALUES (?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET revision=excluded.revision,json=excluded.json",params![v.id,v.collection_id,v.revision,serde_json::to_string(&v)?])?;
    Ok(v)
}
impl Workspace {
    pub fn view_save(&mut self, input: SaveView) -> Result<CollectionView> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let result = view_save_in_transaction(
            &tx,
            &self.path,
            input,
            &mut MutationContext::individual("internal", ""),
        )?;
        tx.commit()?;
        Ok(result)
    }
    pub fn database_query(&self, input: DatabaseQuery) -> Result<DatabasePage> {
        let tx = self.conn.unchecked_transaction()?;
        query_in_connection(&tx, &self.path, input)
    }
}
pub(crate) fn query_in_connection(
    tx: &Connection,
    root: &Path,
    input: DatabaseQuery,
) -> Result<DatabasePage> {
    let n = input.limit.unwrap_or(50);
    if !(1..=100).contains(&n) {
        return Err(CoreError::validation("limit debe ser 1–100."));
    }
    let offset: u32 = input
        .cursor
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?
        .unwrap_or(0);
    if offset > 10_000 {
        return Err(CoreError::validation("Cursor fuera de límite."));
    }
    let props = properties(tx, &input.collection_id)?;
    let (layout, cfg) = if let Some(id) = input.view_id {
        if input.config.is_some() || input.layout.is_some() {
            return Err(CoreError::validation("view_id excluye config/layout."));
        }
        let view = views(tx, &input.collection_id)?
            .into_iter()
            .find(|v| v.id == id)
            .ok_or_else(|| CoreError::validation("Vista ajena/inexistente."))?;
        (view.layout, view.config)
    } else {
        (
            input.layout.unwrap_or(ViewLayout::Table),
            input.config.unwrap_or_default(),
        )
    };
    config(tx, root, &props, &cfg)?;
    crate::collection_views::validate(&layout, &props, &cfg)?;
    let conditions = crate::database_filters::conditions(&cfg)?;
    let mut stmt=tx.prepare("SELECT m.entity_id FROM memberships m JOIN entities e ON e.id=m.entity_id WHERE m.collection_id=?1 AND (e.archived_at IS NOT NULL)=?2 ORDER BY m.entity_id LIMIT 10001")?;
    let ids = stmt
        .query_map(params![input.collection_id, cfg.archived], |r| {
            r.get::<_, String>(0)
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if ids.len() > 10_000 {
        return Err(CoreError::validation(
            "Colección excede límite de consulta inicial.",
        ));
    }
    let mut records = ids
        .iter()
        .map(|id| record_raw(tx, &input.collection_id, id))
        .collect::<Result<Vec<_>>>()?;
    let mut evaluator = crate::formulas::Evaluator::new(tx);
    for r in &mut records {
        evaluator.hydrate(&props, r)?;
        for id in conditions
            .iter()
            .map(|f| &f.property_id)
            .chain(cfg.sorts.iter().map(|s| &s.property_id))
        {
            if let Some(e) = r.calculation_errors.get(id) {
                return Err(crate::formulas::core_error(e.clone()));
            }
        }
    }
    // No interpretar valores futuros/corruptos como ceros al filtrar/ordenar.
    for r in &records {
        for p in &props {
            if let Some(v) = r.values.get(&p.id) {
                validate_value(tx, root, &p.spec, v, false)?;
            }
        }
    }
    records.retain(|r| crate::database_filters::matches(&cfg, r, &props));
    if let Some(calendar) = &cfg.calendar {
        let (start, end) = crate::collection_views::month(&calendar.month)?;
        records.retain(|r| {
            crate::collection_views::in_month(r, calendar, &start.to_string(), &end.to_string())
        });
    }
    records.sort_by(|a, b| {
        for s in &cfg.sorts {
            let x = a.values.get(&s.property_id).unwrap_or(&Value::Null);
            let y = b.values.get(&s.property_id).unwrap_or(&Value::Null);
            let mut order = compare(
                &sort_kind(props.iter().find(|p| p.id == s.property_id).unwrap()),
                x,
                y,
            );
            if s.descending && !x.is_null() && !y.is_null() {
                order = order.reverse();
            }
            if order != Ordering::Equal {
                return order;
            }
        }
        a.entity.id.cmp(&b.entity.id)
    });
    let matching_count = records.len() as u32;
    let items = records
        .into_iter()
        .skip(offset as usize)
        .take(n as usize + 1)
        .collect::<Vec<_>>();
    let next_cursor = if items.len() > n as usize {
        Some((offset + n).to_string())
    } else {
        None
    };
    let mut items = items;
    if next_cursor.is_some() {
        items.pop();
    }
    Ok(DatabasePage {
        groups: crate::collection_views::groups(&layout, &props, &cfg, &items)?,
        layout,
        matching_count,
        properties: props,
        config: cfg,
        records: Page { items, next_cursor },
    })
}

pub(crate) fn record_create_in_transaction(
    conn: &Connection,
    root: &Path,
    input: CreateRecord,
    context: &mut MutationContext<'_>,
) -> Result<RecordData> {
    let (_, record) = crate::entities::create_in_transaction(
        conn,
        root,
        CreateEntity {
            kind: EntityKind::Note,
            title: input.title,
            payload: json!({"inbox":false}),
            document: input.document,
            context: None,
            record: Some(RecordSeed {
                collection_id: input.collection_id,
                values: input.values,
            }),
        },
        context,
    )?;
    record.ok_or_else(|| {
        CoreError::new(
            ErrorCode::StorageError,
            "La creación no devolvió la membresía confirmada.",
        )
    })
}
