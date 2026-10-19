use crate::{contracts::*, database as db, error::*};
use serde_json::Value;
use std::cmp::Ordering;

// También inspecciona ramas que no llegarían a ejecutarse.
pub(crate) fn conditions(cfg: &ViewConfig) -> Result<Vec<&PropertyFilter>> {
    fn collect<'a>(
        e: &'a FilterExpression,
        depth: u32,
        nodes: &mut u32,
        out: &mut Vec<&'a PropertyFilter>,
    ) -> Result<()> {
        *nodes += 1;
        if depth > 8 || *nodes > 64 {
            return Err(CoreError::validation(
                "Filtro supera profundidad 8/64 nodos.",
            ));
        }
        match e {
            FilterExpression::Condition { filter } => out.push(filter),
            FilterExpression::Not { child } => collect(child, depth + 1, nodes, out)?,
            FilterExpression::All { children } | FilterExpression::Any { children } => {
                if children.is_empty() {
                    return Err(CoreError::validation("Grupo de filtro vacío."));
                }
                for child in children {
                    collect(child, depth + 1, nodes, out)?;
                }
            }
        }
        if out.len() > 32 {
            return Err(CoreError::validation(
                "Vista supera 32 condiciones combinadas.",
            ));
        }
        Ok(())
    }
    let mut out = cfg.filters.iter().collect::<Vec<_>>();
    if let Some(tree) = &cfg.filter_tree {
        collect(tree, 0, &mut 0, &mut out)?;
    }
    Ok(out)
}
pub(crate) fn ordered(p: &Property) -> bool {
    if let Some(t) = &p.spec.calculation_type {
        return matches!(
            t.kind,
            ScalarKind::Number | ScalarKind::Decimal | ScalarKind::Text
        );
    }
    matches!(
        p.spec.kind,
        PropertyKind::Number
            | PropertyKind::Decimal
            | PropertyKind::Money
            | PropertyKind::Title
            | PropertyKind::Text
            | PropertyKind::Url
            | PropertyKind::Email
            | PropertyKind::Select
            | PropertyKind::Date
            | PropertyKind::CreatedAt
            | PropertyKind::UpdatedAt
    )
}
fn condition(f: &PropertyFilter, r: &RecordData, props: &[Property]) -> bool {
    let value = r.values.get(&f.property_id).unwrap_or(&Value::Null);
    let compare = || {
        db::compare(
            &db::sort_kind(props.iter().find(|p| p.id == f.property_id).unwrap()),
            value,
            &f.value,
        )
    };
    match f.op {
        FilterOp::Eq => compare() == Ordering::Equal,
        FilterOp::Ne => compare() != Ordering::Equal,
        FilterOp::Contains => value
            .as_str()
            .is_some_and(|s| s.contains(f.value.as_str().unwrap())),
        FilterOp::IsEmpty => db::empty(value),
        _ if value.is_null() => false,
        FilterOp::Lt => compare() == Ordering::Less,
        FilterOp::Le => compare() != Ordering::Greater,
        FilterOp::Gt => compare() == Ordering::Greater,
        FilterOp::Ge => compare() != Ordering::Less,
    }
}
fn tree(e: &FilterExpression, r: &RecordData, props: &[Property]) -> bool {
    match e {
        FilterExpression::Condition { filter } => condition(filter, r, props),
        FilterExpression::All { children } => children.iter().all(|e| tree(e, r, props)),
        FilterExpression::Any { children } => children.iter().any(|e| tree(e, r, props)),
        FilterExpression::Not { child } => !tree(child, r, props),
    }
}
pub(crate) fn matches(cfg: &ViewConfig, r: &RecordData, props: &[Property]) -> bool {
    cfg.filters.iter().all(|f| condition(f, r, props))
        && cfg.filter_tree.as_ref().is_none_or(|e| tree(e, r, props))
}
