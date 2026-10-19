use crate::{batch_contracts::*, entities, error::*};
use serde_json::json;
use std::collections::HashSet;
use uuid::Uuid;
pub const MAX_STEPS: usize = 100;
pub const MAX_CHECKS: usize = 10;
pub const MAX_BINDINGS: usize = 512;
pub const MAX_RESPONSE_BYTES: usize = 2_097_152;

pub(crate) fn invalid(reason: &str) -> CoreError {
    let mut e = CoreError::validation("Plan de cambios inválido.");
    e.details = json!({"reason":reason});
    e
}
fn key(s: &str) -> bool {
    s.len() <= 64
        && s.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
pub(crate) fn pointer(s: &str) -> Result<Vec<String>> {
    if s.is_empty() || s.len() > 512 || !s.starts_with('/') {
        return Err(invalid("invalid_pointer"));
    }
    let parts = s[1..]
        .split('/')
        .map(|part| {
            let mut out = String::new();
            let mut chars = part.chars();
            while let Some(c) = chars.next() {
                if c == '~' {
                    out.push(match chars.next() {
                        Some('0') => '~',
                        Some('1') => '/',
                        _ => return Err(invalid("invalid_pointer_escape")),
                    });
                } else {
                    out.push(c);
                }
            }
            Ok(out)
        })
        .collect::<Result<Vec<_>>>()?;
    let canonical = format!(
        "/{}",
        parts
            .iter()
            .map(|s| s.replace('~', "~0").replace('/', "~1"))
            .collect::<Vec<_>>()
            .join("/")
    );
    if canonical != s {
        return Err(invalid("noncanonical_pointer"));
    }
    Ok(parts)
}
pub(crate) fn validate(plan: &BatchPlan) -> Result<Uuid> {
    validate_inner(plan).map_err(|mut e| {
        e.details["phase"] = json!("plan");
        e
    })
}
fn at_step(mut e: CoreError, key: &str, index: usize, op: Option<BatchOperation>) -> CoreError {
    e.details["step_key"] = json!(key);
    e.details["step_index"] = json!(index);
    e.details["operation"] = json!(op.map_or("database.query", BatchOperation::name));
    e
}
fn validate_inner(plan: &BatchPlan) -> Result<Uuid> {
    entities::validate_request_limits(&serde_json::to_value(plan)?)?;
    if plan.version != 1 {
        return Err(invalid("unsupported_plan_version"));
    }
    let seed = Uuid::parse_str(&plan.batch_id).map_err(|_| invalid("invalid_batch_id"))?;
    if seed.to_string() != plan.batch_id {
        return Err(invalid("noncanonical_batch_id"));
    }
    if plan.steps.is_empty() || plan.steps.len() > MAX_STEPS || plan.checks.len() > MAX_CHECKS {
        return Err(invalid("step_or_check_limit"));
    }
    let count = plan
        .steps
        .iter()
        .map(|s| s.bindings.len())
        .chain(plan.checks.iter().map(|s| s.bindings.len()))
        .sum::<usize>();
    if count > MAX_BINDINGS {
        return Err(invalid("binding_limit"));
    }
    let mut seen = HashSet::new();
    let mut previous: std::collections::BTreeMap<String, Vec<BatchExport>> =
        std::collections::BTreeMap::new();
    for (index, (name, input, bindings, op)) in plan
        .steps
        .iter()
        .map(|s| (&s.key, &s.input, &s.bindings, Some(s.operation)))
        .chain(
            plan.checks
                .iter()
                .map(|s| (&s.key, &s.input, &s.bindings, None)),
        )
        .enumerate()
    {
        if !key(name) || !seen.insert(name) {
            return Err(at_step(
                invalid("invalid_or_duplicate_key"),
                name,
                index,
                op,
            ));
        }
        if !input.is_object() {
            return Err(at_step(invalid("input_requires_object"), name, index, op));
        }
        let mut targets = HashSet::new();
        for (i, b) in bindings.iter().enumerate() {
            let result = (|| {
                pointer(&b.target)?;
                let fields = previous
                    .get(&b.source.step)
                    .ok_or_else(|| invalid("non_previous_reference"))?;
                if !key(&b.source.step) || !fields.contains(&b.source.export) {
                    return Err(invalid("unavailable_export"));
                }
                if input.pointer(&b.target).is_none() {
                    return Err(invalid("missing_binding_target"));
                }
                if !targets.insert((b.kind as u8, &b.target)) {
                    return Err(invalid("duplicate_binding_target"));
                }
                Ok(())
            })();
            result.map_err(|mut e: CoreError| {
                e.details["binding_index"] = json!(i);
                at_step(e, name, index, op)
            })?;
        }
        let values = bindings
            .iter()
            .enumerate()
            .filter(|(_, b)| b.kind == BindingKind::Value)
            .map(|(i, b)| (i, &b.target))
            .collect::<Vec<_>>();
        for (i, (binding_index, a)) in values.iter().enumerate() {
            if values[i + 1..]
                .iter()
                .any(|(_, b)| a.starts_with(&format!("{b}/")) || b.starts_with(&format!("{a}/")))
            {
                let mut e = invalid("overlapping_binding_targets");
                e.details["binding_index"] = json!(binding_index);
                return Err(at_step(e, name, index, op));
            }
        }
        let mut shape = input.clone();
        for b in bindings.iter().filter(|b| b.kind == BindingKind::Value) {
            *shape
                .pointer_mut(&b.target)
                .ok_or_else(|| invalid("missing_binding_target"))? = match b.source.export {
                BatchExport::EntityRevision
                | BatchExport::CollectionRevision
                | BatchExport::ViewRevision => json!(1),
                _ => json!(Uuid::nil().to_string()),
            };
        }
        crate::batches::validate_shape(op, shape).map_err(|e| at_step(e, name, index, op))?;
        if let Some(op) = op {
            previous.insert(name.clone(), crate::batch_bindings::exports_for(op));
        }
    }
    Ok(seed)
}
