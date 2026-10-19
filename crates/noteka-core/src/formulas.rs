use crate::{
    contracts::*,
    database as db,
    error::{CoreError, ErrorCode, Result},
    mutations::{MutationContext, ObjectKind},
    workspace::Workspace,
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap, HashSet};
const UNIT: i128 = 1_000_000_000;
type Calc<T> = std::result::Result<T, CalculationError>;
fn fail(code: &str, message: &str) -> CalculationError {
    CalculationError {
        code: code.into(),
        message: message.into(),
    }
}
pub(crate) fn core_error(e: CalculationError) -> CoreError {
    let mut error = CoreError::new(
        ErrorCode::ValidationError,
        format!("{}: {}", e.code, e.message),
    );
    error.details = json!({"calculation_error":e});
    error
}
fn storage(e: impl std::fmt::Debug) -> CalculationError {
    fail("invalid_value", &format!("{e:?}"))
}
fn typ(kind: ScalarKind, currency: Option<String>) -> CalculationType {
    CalculationType { kind, currency }
}
fn scalar(p: &Property) -> Calc<CalculationType> {
    use PropertyKind::*;
    Ok(match p.spec.kind {
        Formula | Rollup => p
            .spec
            .calculation_type
            .clone()
            .ok_or_else(|| fail("type_error", "Calculada sin tipo reconocido."))?,
        Number => typ(ScalarKind::Number, None),
        Decimal | Money => typ(ScalarKind::Decimal, p.spec.currency.clone()),
        Checkbox => typ(ScalarKind::Boolean, None),
        Title | Text | Url | Email | Select | Date | CreatedAt | UpdatedAt => {
            typ(ScalarKind::Text, None)
        }
        _ => {
            return Err(fail(
                "type_error",
                "Este AST no admite listas/rangos como escalares.",
            ));
        }
    })
}
pub(crate) fn validate_scalar(t: &CalculationType, v: &Value) -> Calc<()> {
    if v.is_null() {
        return Ok(());
    }
    let valid = match t.kind {
        ScalarKind::Number => v.as_f64().is_some_and(f64::is_finite),
        ScalarKind::Decimal => v.as_str().is_some_and(|s| db::decimal(s).is_ok()),
        ScalarKind::Boolean => v.is_boolean(),
        ScalarKind::Text => v.as_str().is_some_and(|s| s.len() <= 16_384),
    };
    if valid {
        Ok(())
    } else {
        Err(fail(
            "invalid_value",
            "El valor fuente no corresponde al tipo calculado.",
        ))
    }
}
fn prop(conn: &Connection, id: &str, c: Option<&str>) -> Calc<Property> {
    let p = db::property(conn, id)
        .map_err(|_| fail("missing_reference", "Propiedad ausente o no reconocida."))?;
    if c.is_some_and(|c| c != p.collection_id) {
        return Err(fail(
            "missing_reference",
            "Propiedad ajena a la colección de evaluación.",
        ));
    }
    Ok(p)
}
fn relation(conn: &Connection, id: &str, c: &str, inverse: bool) -> Calc<(Property, String)> {
    let p = prop(conn, id, None)?;
    if p.spec.kind != PropertyKind::Relation {
        return Err(fail("type_error", "Se requiere una propiedad relation."));
    }
    let target = p
        .spec
        .target_collection_id
        .clone()
        .ok_or_else(|| fail("missing_reference", "Relación sin destino."))?;
    let other = if inverse {
        if target != c {
            return Err(fail(
                "missing_reference",
                "Inversa no apunta a esta colección.",
            ));
        }
        p.collection_id.clone()
    } else {
        if p.collection_id != c {
            return Err(fail(
                "missing_reference",
                "Relación ajena a esta colección.",
            ));
        }
        target
    };
    Ok((p, other))
}
fn infer(
    conn: &Connection,
    c: &str,
    e: &Expression,
    depth: u32,
    nodes: &mut u32,
    deps: &mut HashSet<String>,
) -> Calc<CalculationType> {
    *nodes += 1;
    if depth > 32 || *nodes > 256 {
        return Err(fail("complexity_limit", "AST supera 32 niveles/256 nodos."));
    }
    use Expression::*;
    match e {
        Number { value } => {
            if !value.is_finite() {
                return Err(fail("invalid_value", "Literal number no finito."));
            }
            Ok(typ(ScalarKind::Number, None))
        }
        Decimal { value, currency } => {
            db::decimal(value).map_err(storage)?;
            if currency
                .as_ref()
                .is_some_and(|s| s.len() != 3 || !s.bytes().all(|b| b.is_ascii_uppercase()))
            {
                return Err(fail("type_error", "Moneda debe tener tres mayúsculas."));
            }
            Ok(typ(ScalarKind::Decimal, currency.clone()))
        }
        Boolean { .. } => Ok(typ(ScalarKind::Boolean, None)),
        Text { value } => {
            if value.len() > 16_384 {
                return Err(fail("complexity_limit", "Texto literal excesivo."));
            }
            Ok(typ(ScalarKind::Text, None))
        }
        Property { property_id } => {
            deps.insert(property_id.clone());
            scalar(&prop(conn, property_id, Some(c))?)
        }
        Not { value } => {
            let t = infer(conn, c, value, depth + 1, nodes, deps)?;
            if t.kind != ScalarKind::Boolean {
                return Err(fail("type_error", "not requiere boolean."));
            }
            Ok(t)
        }
        If {
            condition,
            then,
            otherwise,
        } => {
            let cond = infer(conn, c, condition, depth + 1, nodes, deps)?;
            let a = infer(conn, c, then, depth + 1, nodes, deps)?;
            let b = infer(conn, c, otherwise, depth + 1, nodes, deps)?;
            if cond.kind != ScalarKind::Boolean || a != b {
                return Err(fail(
                    "type_error",
                    "if exige condición boolean y ramas del mismo tipo/moneda.",
                ));
            }
            Ok(a)
        }
        Binary {
            operator,
            left,
            right,
        } => {
            let a = infer(conn, c, left, depth + 1, nodes, deps)?;
            let b = infer(conn, c, right, depth + 1, nodes, deps)?;
            binary_type(operator, &a, &b)
        }
        Rollup {
            relation_id,
            property_id,
            aggregate,
            inverse,
        } => {
            deps.insert(relation_id.clone());
            let (_, other) = relation(conn, relation_id, c, *inverse)?;
            if *aggregate == Aggregate::Count {
                if property_id.is_some() {
                    return Err(fail(
                        "type_error",
                        "count cuenta relaciones y no lleva property_id.",
                    ));
                }
                return Ok(typ(ScalarKind::Number, None));
            }
            let id = property_id
                .as_deref()
                .ok_or_else(|| fail("missing_reference", "Rollup requiere propiedad fuente."))?;
            deps.insert(id.into());
            let t = scalar(&prop(conn, id, Some(&other))?)?;
            if !matches!(t.kind, ScalarKind::Number | ScalarKind::Decimal) {
                return Err(fail("type_error", "Agregación requiere number o decimal."));
            }
            Ok(t)
        }
        Allocate {
            relation_id,
            cost_id,
            weight_id,
            scale,
        } => {
            if *scale > 9 {
                return Err(fail("precision_loss", "Unidad mínima admite scale 0–9."));
            }
            let (_, other) = relation(conn, relation_id, c, false)?;
            let t = scalar(&prop(conn, cost_id, Some(&other))?)?;
            let w = scalar(&prop(conn, weight_id, Some(c))?)?;
            if t.kind != ScalarKind::Decimal || w.kind != ScalarKind::Number {
                return Err(fail(
                    "type_error",
                    "allocate exige costo decimal/money y peso number.",
                ));
            }
            deps.extend([relation_id.clone(), cost_id.clone(), weight_id.clone()]);
            Ok(t)
        }
    }
}
fn binary_type(op: &BinaryOp, a: &CalculationType, b: &CalculationType) -> Calc<CalculationType> {
    use BinaryOp::*;
    let invalid = || fail("type_error", "Operandos/tipos/monedas incompatibles.");
    match op {
        And | Or => {
            if a.kind == ScalarKind::Boolean && a == b {
                Ok(a.clone())
            } else {
                Err(invalid())
            }
        }
        Eq | Lt => {
            if a == b {
                Ok(typ(ScalarKind::Boolean, None))
            } else {
                Err(invalid())
            }
        }
        Add | Sub => {
            if a == b && matches!(a.kind, ScalarKind::Number | ScalarKind::Decimal) {
                Ok(a.clone())
            } else {
                Err(invalid())
            }
        }
        Mul | Div => {
            if a.kind != b.kind || !matches!(a.kind, ScalarKind::Number | ScalarKind::Decimal) {
                return Err(invalid());
            }
            if matches!(op, Div) && a.currency.is_some() && a.currency == b.currency {
                return Ok(typ(a.kind.clone(), None));
            }
            if b.currency.is_none() {
                return Ok(a.clone());
            }
            if matches!(op, Mul) && a.currency.is_none() {
                return Ok(b.clone());
            }
            Err(invalid())
        }
    }
}
pub(crate) fn definitions(conn: &Connection, c: &str) -> Result<BTreeMap<String, Expression>> {
    let mut stmt=conn.prepare("SELECT f.property_id,f.expression FROM calculations f JOIN properties p ON p.id=f.property_id WHERE p.collection_id=?1 ORDER BY f.property_id")?;
    let rows = stmt
        .query_map([c], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    rows.into_iter()
        .map(|(id, s)| {
            Ok((
                id,
                serde_json::from_str(&s).map_err(|_| {
                    CoreError::new(
                        ErrorCode::StorageError,
                        "Definición calculada desconocida; no se convirtió.",
                    )
                })?,
            ))
        })
        .collect()
}
fn graph(conn: &Connection) -> Calc<()> {
    let mut stmt=conn.prepare("SELECT p.id,p.collection_id,f.expression FROM calculations f JOIN properties p ON p.id=f.property_id").map_err(storage)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .map_err(storage)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(storage)?;
    let mut edges = HashMap::new();
    for (id, c, s) in rows {
        let e: Expression = serde_json::from_str(&s)
            .map_err(|_| fail("invalid_syntax", "Definición almacenada no reconocida."))?;
        let mut deps = HashSet::new();
        let inferred = infer(conn, &c, &e, 0, &mut 0, &mut deps)?;
        if scalar(&prop(conn, &id, Some(&c))?)? != inferred {
            return Err(fail("type_error", "Metadata calculada difiere de su AST."));
        }
        edges.insert(id, deps);
    }
    fn visit(
        id: &str,
        edges: &HashMap<String, HashSet<String>>,
        active: &mut HashSet<String>,
        heights: &mut HashMap<String, u32>,
        depth: u32,
    ) -> Calc<u32> {
        if depth > 128 {
            return Err(fail("complexity_limit", "Grafo supera 128 niveles."));
        }
        if let Some(height) = heights.get(id) {
            if depth + height > 128 {
                return Err(fail("complexity_limit", "Grafo supera 128 niveles."));
            }
            return Ok(*height);
        }
        if !active.insert(id.into()) {
            return Err(fail("cycle", "Dependencia calculada cíclica."));
        }
        let mut height = 0;
        if let Some(deps) = edges.get(id) {
            for dep in deps {
                height = height.max(1 + visit(dep, edges, active, heights, depth + 1)?);
            }
        }
        active.remove(id);
        heights.insert(id.into(), height);
        Ok(height)
    }
    let mut heights = HashMap::new();
    for id in edges.keys() {
        visit(id, &edges, &mut HashSet::new(), &mut heights, 0)?;
    }
    Ok(())
}
pub(crate) fn set_in_transaction(
    tx: &Connection,
    input: FormulaSet,
    context: &mut MutationContext<'_>,
) -> Result<CollectionBundle> {
    let name = db::name(&input.name)?;
    let c = db::collection(tx, &input.collection_id)?;
    db::revision(&c.id, c.revision, input.expected_revision)?;
    let existing = db::properties(tx, &c.id)?;
    let old = input
        .property_id
        .as_ref()
        .map(|id| prop(tx, id, Some(&c.id)).map_err(core_error))
        .transpose()?;
    if let Some(p) = &old {
        if !matches!(p.spec.kind, PropertyKind::Formula | PropertyKind::Rollup)
            || p.spec.name != name
        {
            return Err(CoreError::validation(
                "Actualizar cálculo conserva nombre y tipo de propiedad.",
            ));
        }
        // Un AST desconocido jamás se reemplaza para hacerlo legible.
        let raw: String = tx.query_row(
            "SELECT expression FROM calculations WHERE property_id=?1",
            [&p.id],
            |r| r.get(0),
        )?;
        serde_json::from_str::<Expression>(&raw).map_err(|_| {
            core_error(fail(
                "invalid_syntax",
                "AST original desconocido; se conservó.",
            ))
        })?;
    } else if existing.len() >= 128 || existing.iter().any(|p| p.spec.name == name) {
        return Err(CoreError::validation(
            "Nombre existente o límite de propiedades.",
        ));
    }
    let t =
        infer(tx, &c.id, &input.expression, 0, &mut 0, &mut HashSet::new()).map_err(core_error)?;
    if old
        .as_ref()
        .is_some_and(|p| p.spec.calculation_type.as_ref() != Some(&t))
    {
        return Err(core_error(fail(
            "type_error",
            "No se cambia el tipo/moneda de una calculada existente.",
        )));
    }
    let key = match input.property_id {
        Some(id) => id,
        None => context.new_id(tx, "property", ObjectKind::Property)?,
    };
    let kind = old.map_or_else(
        || {
            if matches!(input.expression, Expression::Rollup { .. }) {
                PropertyKind::Rollup
            } else {
                PropertyKind::Formula
            }
        },
        |p| p.spec.kind,
    );
    let spec = PropertySpec {
        name: name.clone(),
        kind,
        options: vec![],
        currency: None,
        target_collection_id: None,
        calculation_type: Some(t),
    };
    context.observe(tx, ObjectKind::Collection, &[&c.id])?;
    context.observe(tx, ObjectKind::Property, &[&key])?;
    context.observe(tx, ObjectKind::Calculation, &[&key])?;
    tx.execute("INSERT INTO properties VALUES (?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET spec=excluded.spec",params![key,c.id,name,serde_json::to_string(&spec)?])?;
    tx.execute("INSERT INTO calculations VALUES (?1,?2) ON CONFLICT(property_id) DO UPDATE SET expression=excluded.expression",params![key,serde_json::to_string(&input.expression)?])?;
    graph(tx).map_err(core_error)?;
    let rev = c
        .revision
        .checked_add(1)
        .ok_or_else(|| CoreError::validation("Límite de revisión."))?;
    tx.execute(
        "UPDATE collections SET revision=?2 WHERE id=?1",
        params![c.id, rev],
    )?;
    db::bundle(tx, &c.id)
}
impl Workspace {
    pub fn formula_set(&mut self, input: FormulaSet) -> Result<CollectionBundle> {
        db::name(&input.name)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let result =
            set_in_transaction(&tx, input, &mut MutationContext::individual("internal", ""))?;
        tx.commit()?;
        Ok(result)
    }
}

fn dec(v: &Value) -> Calc<i128> {
    db::decimal(
        v.as_str()
            .ok_or_else(|| fail("invalid_value", "Se esperaba decimal string."))?,
    )
    .map_err(storage)
}
fn exact_div(a: i128, b: i128) -> Calc<i128> {
    if b == 0 {
        return Err(fail("division_by_zero", "Divisor cero."));
    }
    if a.checked_rem(b)
        .ok_or_else(|| fail("overflow", "Overflow decimal."))?
        != 0
    {
        return Err(fail(
            "precision_loss",
            "Resultado no representable con escala nueve; no se redondeó.",
        ));
    }
    a.checked_div(b)
        .ok_or_else(|| fail("overflow", "Overflow decimal."))
}
fn scaled_ratio(mut a: i128, mut b: i128, mut denominator: i128) -> Calc<i128> {
    if denominator == 0 {
        return Err(fail("division_by_zero", "Divisor cero."));
    }
    fn gcd(mut a: i128, mut b: i128) -> i128 {
        while b != 0 {
            let r = a % b;
            a = b;
            b = r;
        }
        a
    }
    let x = gcd(
        a.checked_abs()
            .ok_or_else(|| fail("overflow", "Overflow decimal."))?,
        denominator
            .checked_abs()
            .ok_or_else(|| fail("overflow", "Overflow decimal."))?,
    );
    a /= x;
    denominator /= x;
    let y = gcd(
        b.checked_abs()
            .ok_or_else(|| fail("overflow", "Overflow decimal."))?,
        denominator.abs(),
    );
    b /= y;
    denominator /= y;
    if denominator.abs() != 1 {
        return Err(fail(
            "precision_loss",
            "Resultado no representable con escala nueve.",
        ));
    }
    a.checked_mul(b)
        .and_then(|n| n.checked_div(denominator))
        .ok_or_else(|| fail("overflow", "Overflow decimal."))
}
fn decimal_json(n: i128) -> Calc<Value> {
    let negative = n < 0;
    let abs = n
        .checked_abs()
        .ok_or_else(|| fail("overflow", "Decimal fuera de rango."))?;
    let whole = abs / UNIT;
    let fraction = format!("{:09}", abs % UNIT);
    let fraction = fraction.trim_end_matches('0');
    let s = format!(
        "{}{}{}{}",
        if negative { "-" } else { "" },
        whole,
        if fraction.is_empty() { "" } else { "." },
        fraction
    );
    db::decimal(&s)
        .map_err(|_| fail("overflow", "Resultado excede precisión decimal soportada."))?;
    Ok(json!(s))
}
fn numeric(op: &BinaryOp, a: &Value, b: &Value, t: &CalculationType) -> Calc<Value> {
    use BinaryOp::*;
    if a.is_null() || b.is_null() {
        return Ok(Value::Null);
    }
    validate_scalar(t, a)?;
    // t es tipo izquierdo; moneda no cambia representación JSON.
    validate_scalar(t, b)?;
    match t.kind {
        ScalarKind::Number => {
            let x = a.as_f64().unwrap();
            let y = b.as_f64().unwrap();
            let n = match op {
                Add => x + y,
                Sub => x - y,
                Mul => x * y,
                Div => {
                    if y == 0. {
                        return Err(fail("division_by_zero", "Divisor cero."));
                    }
                    x / y
                }
                Eq => return Ok(json!(x == y)),
                Lt => return Ok(json!(x < y)),
                _ => return Err(fail("type_error", "Operador no numérico.")),
            };
            if !n.is_finite() {
                return Err(fail("overflow", "Número no finito."));
            }
            Ok(json!(n))
        }
        ScalarKind::Decimal => {
            let x = dec(a)?;
            let y = dec(b)?;
            let overflow = || fail("overflow", "Overflow decimal intermedio.");
            let n = match op {
                Add => x.checked_add(y).ok_or_else(overflow)?,
                Sub => x.checked_sub(y).ok_or_else(overflow)?,
                Mul => scaled_ratio(x, y, UNIT)?,
                Div => scaled_ratio(x, UNIT, y)?,
                Eq => return Ok(json!(x == y)),
                Lt => return Ok(json!(x < y)),
                _ => return Err(fail("type_error", "Operador no decimal.")),
            };
            decimal_json(n)
        }
        ScalarKind::Boolean | ScalarKind::Text => match op {
            Eq => Ok(json!(a == b)),
            Lt => Ok(json!(if t.kind == ScalarKind::Text {
                a.as_str().unwrap() < b.as_str().unwrap()
            } else {
                !a.as_bool().unwrap() && b.as_bool().unwrap()
            })),
            _ => Err(fail("type_error", "Operador incompatible con escalar.")),
        },
    }
}
type AllocationKey = (String, String, String, String, u32);
pub(crate) struct Evaluator<'a> {
    conn: &'a Connection,
    memo: HashMap<(String, String), Calc<Value>>,
    active: HashSet<(String, String)>,
    allocations: HashMap<AllocationKey, BTreeMap<String, i128>>,
    steps: u32,
}
impl<'a> Evaluator<'a> {
    pub(crate) fn new(conn: &'a Connection) -> Self {
        Self {
            conn,
            memo: HashMap::new(),
            active: HashSet::new(),
            allocations: HashMap::new(),
            steps: 0,
        }
    }
    fn step(&mut self) -> Calc<()> {
        self.steps += 1;
        if self.steps > 1_000_000 {
            return Err(fail(
                "complexity_limit",
                "Evaluación supera un millón de pasos.",
            ));
        }
        Ok(())
    }
    pub(crate) fn hydrate(&mut self, props: &[Property], r: &mut RecordData) -> Result<()> {
        for p in props
            .iter()
            .filter(|p| matches!(p.spec.kind, PropertyKind::Formula | PropertyKind::Rollup))
        {
            match self.value(&r.entity.id, &p.id) {
                Ok(value) => {
                    r.values.insert(p.id.clone(), value);
                }
                Err(e) => {
                    r.values.remove(&p.id);
                    r.calculation_errors.insert(p.id.clone(), e);
                }
            }
        }
        Ok(())
    }
    fn value(&mut self, e: &str, id: &str) -> Calc<Value> {
        let key = (e.into(), id.into());
        if let Some(v) = self.memo.get(&key) {
            return v.clone();
        }
        self.step()?;
        if self.active.len() > 128 {
            return Err(fail(
                "complexity_limit",
                "Dependencias exceden profundidad.",
            ));
        }
        if !self.active.insert(key.clone()) {
            return Err(fail("cycle", "Ciclo de evaluación."));
        }
        let result = self.read_value(e, id);
        self.active.remove(&key);
        self.memo.insert(key, result.clone());
        result
    }
    fn read_value(&mut self, e: &str, id: &str) -> Calc<Value> {
        let p = prop(self.conn, id, None)?;
        db::member(self.conn, &p.collection_id, e)
            .map_err(|_| fail("missing_reference", "Fuente no pertenece a colección."))?;
        let t = scalar(&p)?;
        let v = if matches!(p.spec.kind, PropertyKind::Formula | PropertyKind::Rollup) {
            let s: String = self
                .conn
                .query_row(
                    "SELECT expression FROM calculations WHERE property_id=?1",
                    [id],
                    |r| r.get(0),
                )
                .map_err(storage)?;
            let expr: Expression = serde_json::from_str(&s)
                .map_err(|_| fail("invalid_syntax", "AST almacenado no reconocido."))?;
            let inferred = infer(
                self.conn,
                &p.collection_id,
                &expr,
                0,
                &mut 0,
                &mut HashSet::new(),
            )?;
            if inferred != t {
                return Err(fail(
                    "type_error",
                    "Tipo/moneda almacenados no corresponden al AST.",
                ));
            }
            self.eval(e, &p.collection_id, &expr, 0)?
        } else {
            match p.spec.kind {
                PropertyKind::Title | PropertyKind::CreatedAt | PropertyKind::UpdatedAt => {
                    let entity = crate::entities::get(self.conn, e).map_err(storage)?;
                    json!(match p.spec.kind {
                        PropertyKind::Title => entity.title,
                        PropertyKind::CreatedAt => entity.created_at,
                        _ => entity.updated_at,
                    })
                }
                _ => {
                    let raw:Option<String>=self.conn.query_row("SELECT json FROM property_values WHERE entity_id=?1 AND property_id=?2",params![e,id],|r|r.get(0)).optional().map_err(storage)?;
                    raw.map(|s| serde_json::from_str(&s).map_err(storage))
                        .transpose()?
                        .unwrap_or(Value::Null)
                }
            }
        };
        validate_scalar(&t, &v)?;
        Ok(v)
    }
    fn related(&mut self, e: &str, c: &str, id: &str, inverse: bool) -> Calc<Vec<String>> {
        self.step()?;
        let (p, other) = relation(self.conn, id, c, inverse)?;
        if inverse {
            let mut stmt=self.conn.prepare("SELECT pv.entity_id FROM property_values pv JOIN memberships m ON m.entity_id=pv.entity_id AND m.collection_id=?1 WHERE pv.property_id=?2 AND EXISTS(SELECT 1 FROM json_each(pv.json) WHERE value=?3) ORDER BY pv.entity_id LIMIT 10001").map_err(storage)?;
            let ids = stmt
                .query_map(params![other, id, e], |r| r.get::<_, String>(0))
                .map_err(storage)?
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(storage)?;
            if ids.len() > 10_000 {
                return Err(fail("complexity_limit", "Inversa supera 10.000 registros."));
            }
            for id_source in &ids {
                self.related(id_source, &other, id, false)?;
            }
            Ok(ids)
        } else {
            db::member(self.conn, &p.collection_id, e).map_err(storage)?;
            let raw: Option<String> = self
                .conn
                .query_row(
                    "SELECT json FROM property_values WHERE entity_id=?1 AND property_id=?2",
                    params![e, id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(storage)?;
            let v: Value = raw
                .map(|s| serde_json::from_str(&s).map_err(storage))
                .transpose()?
                .unwrap_or(json!([]));
            let array = v
                .as_array()
                .ok_or_else(|| fail("invalid_value", "Relación no es array."))?;
            if array.len() > 100 {
                return Err(fail("complexity_limit", "Relación supera 100 IDs."));
            }
            let mut ids = vec![];
            for value in array {
                self.step()?;
                let id = value
                    .as_str()
                    .ok_or_else(|| fail("invalid_value", "Relación con ID no textual."))?;
                if ids.iter().any(|s| s == id) {
                    return Err(fail("invalid_value", "Relación duplicada."));
                }
                db::member(self.conn, &other, id)
                    .map_err(|_| fail("missing_reference", "Destino no pertenece a colección."))?;
                ids.push(id.into());
            }
            Ok(ids)
        }
    }
    fn eval(&mut self, e: &str, c: &str, expr: &Expression, depth: u32) -> Calc<Value> {
        self.step()?;
        if depth > 32 {
            return Err(fail("complexity_limit", "AST excede profundidad."));
        }
        use Expression::*;
        match expr {
            Number { value } => Ok(json!(value)),
            Decimal { value, .. } | Text { value } => Ok(json!(value)),
            Boolean { value } => Ok(json!(value)),
            Property { property_id } => self.value(e, property_id),
            Not { value } => {
                let v = self.eval(e, c, value, depth + 1)?;
                Ok(v.as_bool().map(|v| json!(!v)).unwrap_or(Value::Null))
            }
            If {
                condition,
                then,
                otherwise,
            } => {
                let cond = self.eval(e, c, condition, depth + 1)?;
                match cond.as_bool() {
                    Some(true) => self.eval(e, c, then, depth + 1),
                    Some(false) => self.eval(e, c, otherwise, depth + 1),
                    None => Ok(Value::Null),
                }
            }
            Binary {
                operator,
                left,
                right,
            } => {
                let a = self.eval(e, c, left, depth + 1)?;
                if matches!(operator, BinaryOp::And | BinaryOp::Or) {
                    if a.is_null() {
                        return Ok(Value::Null);
                    }
                    let x = a
                        .as_bool()
                        .ok_or_else(|| fail("type_error", "Booleano requerido."))?;
                    if (matches!(operator, BinaryOp::And) && !x)
                        || (matches!(operator, BinaryOp::Or) && x)
                    {
                        return Ok(json!(x));
                    }
                    return self.eval(e, c, right, depth + 1);
                }
                let b = self.eval(e, c, right, depth + 1)?;
                let t = infer(self.conn, c, left, 0, &mut 0, &mut HashSet::new())?;
                numeric(operator, &a, &b, &t)
            }
            Rollup {
                relation_id,
                property_id,
                aggregate,
                inverse,
            } => {
                let ids = self.related(e, c, relation_id, *inverse)?;
                if *aggregate == Aggregate::Count {
                    return Ok(json!(ids.len()));
                }
                let id = property_id
                    .as_deref()
                    .ok_or_else(|| fail("missing_reference", "Rollup sin propiedad."))?;
                let t = scalar(&prop(self.conn, id, None)?)?;
                let mut values = vec![];
                for id_entity in ids {
                    let v = self.value(&id_entity, id)?;
                    if !v.is_null() {
                        values.push(v);
                    }
                }
                if values.is_empty() {
                    return if *aggregate == Aggregate::Sum {
                        Ok(if t.kind == ScalarKind::Decimal {
                            json!("0")
                        } else {
                            json!(0)
                        })
                    } else {
                        Ok(Value::Null)
                    };
                }
                let mut result = values[0].clone();
                for v in &values[1..] {
                    result = match aggregate {
                        Aggregate::Sum | Aggregate::Avg => numeric(&BinaryOp::Add, &result, v, &t)?,
                        Aggregate::Min => {
                            if numeric(&BinaryOp::Lt, v, &result, &t)? == json!(true) {
                                v.clone()
                            } else {
                                result
                            }
                        }
                        Aggregate::Max => {
                            if numeric(&BinaryOp::Lt, &result, v, &t)? == json!(true) {
                                v.clone()
                            } else {
                                result
                            }
                        }
                        _ => unreachable!(),
                    };
                }
                if *aggregate == Aggregate::Avg {
                    let divisor = if t.kind == ScalarKind::Decimal {
                        json!(values.len().to_string())
                    } else {
                        json!(values.len())
                    };
                    result = numeric(&BinaryOp::Div, &result, &divisor, &t)?;
                }
                Ok(result)
            }
            Allocate {
                relation_id,
                cost_id,
                weight_id,
                scale,
            } => self.allocate(e, c, relation_id, cost_id, weight_id, *scale),
        }
    }
    fn allocate(
        &mut self,
        e: &str,
        c: &str,
        relation_id: &str,
        cost_id: &str,
        weight_id: &str,
        scale: u32,
    ) -> Calc<Value> {
        let targets = self.related(e, c, relation_id, false)?;
        if targets.len() != 1 {
            return Err(fail(
                "invalid_value",
                "Reparto exige exactamente un destino.",
            ));
        }
        let target = &targets[0];
        let key = (
            relation_id.into(),
            target.clone(),
            cost_id.into(),
            weight_id.into(),
            scale,
        );
        if !self.allocations.contains_key(&key) {
            let unit = 10_i128.pow(9 - scale);
            let cost = dec(&self.value(target, cost_id)?)?;
            if cost < 0 {
                return Err(fail(
                    "invalid_value",
                    "Costo negativo no admitido en reparto.",
                ));
            }
            let minor = exact_div(cost, unit)?;
            let (_, other) = relation(self.conn, relation_id, c, false)?;
            let ids = self.related(target, &other, relation_id, true)?;
            let mut weights = vec![];
            let mut total = 0_i128;
            for id in ids {
                let links = self.related(&id, c, relation_id, false)?;
                if links.len() != 1 {
                    return Err(fail(
                        "invalid_value",
                        "Un participante tiene más de un destino.",
                    ));
                }
                let v = self.value(&id, weight_id)?;
                let w = v
                    .as_f64()
                    .filter(|w| w.is_finite() && *w > 0. && *w <= 1_000_000. && w.fract() == 0.)
                    .ok_or_else(|| {
                        fail(
                            "invalid_value",
                            "Peso debe ser entero positivo hasta 1.000.000.",
                        )
                    })? as i128;
                total += w;
                weights.push((id, w));
            }
            if total == 0 {
                return Err(fail("division_by_zero", "Suma de pesos cero."));
            }
            let mut shares = BTreeMap::new();
            let mut remainders = vec![];
            let mut assigned = 0;
            for (id, w) in weights {
                let n = minor
                    .checked_mul(w)
                    .ok_or_else(|| fail("overflow", "Overflow de reparto."))?;
                let base = n / total;
                assigned += base;
                shares.insert(id.clone(), base);
                remainders.push((id, n % total));
            }
            remainders.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            for (id, _) in remainders.into_iter().take((minor - assigned) as usize) {
                *shares.get_mut(&id).unwrap() += 1;
            }
            for v in shares.values_mut() {
                *v = v
                    .checked_mul(unit)
                    .ok_or_else(|| fail("overflow", "Overflow de unidad monetaria."))?;
            }
            self.allocations.insert(key.clone(), shares);
        }
        decimal_json(
            *self.allocations[&key]
                .get(e)
                .ok_or_else(|| fail("missing_reference", "Miembro no figura en reparto."))?,
        )
    }
}
