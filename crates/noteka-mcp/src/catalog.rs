use noteka_core::scoped::{self, Access, Policy};
use rmcp::{
    ErrorData,
    model::{PaginatedRequestParams, Tool},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const PAGE: usize = 8;
pub const INPUT_CAP: usize = 4 * 1024 * 1024;
pub const ENVELOPE_CAP: usize = 2 * 1024 * 1024;
pub const OUTPUT_CAP: usize = 16 * 1024 * 1024;
pub const URI: &str = "noteka://catalog";

pub struct Catalog {
    pub tools: Vec<Tool>,
    pub names: BTreeMap<String, String>,
    pub resource: String,
    fingerprint: String,
}
fn rebase(v: &mut Value, prefix: &str) {
    match v {
        Value::Object(o) => {
            for (key, v) in o {
                if key == "$ref" {
                    if let Some(suffix) = v.as_str().and_then(|s| s.strip_prefix('#')) {
                        *v = json!(format!("{prefix}{suffix}"));
                    }
                } else {
                    rebase(v, prefix);
                }
            }
        }
        Value::Array(a) => {
            for v in a {
                rebase(v, prefix);
            }
        }
        _ => (),
    }
}
fn output_schema(core: &Value, operation: &str) -> Value {
    let mut base = core["contract_schema"]["$defs"]["Response"].clone();
    let mut domain = core["outputs"][operation].clone();
    let mut error = core["error_schema"].clone();
    rebase(&mut domain, "#/$defs/domain");
    rebase(&mut error, "#/$defs/error");
    base["$schema"] = json!("https://json-schema.org/draft/2020-12/schema");
    base["properties"]["error"] = json!({"anyOf":[{"$ref":"#/$defs/error"},{"type":"null"}]});
    base["$defs"] = json!({"domain":domain,"error":error});
    base["oneOf"] = json!([
        {"properties":{"ok":{"const":true},"data":{"$ref":"#/$defs/domain"},"error":{"type":"null"}},"required":["data"]},
        {"properties":{"ok":{"const":false},"data":{"type":"null"},"error":{"$ref":"#/$defs/error"}},"required":["error"]}
    ]);
    base
}
impl Catalog {
    pub fn new(policy: Policy) -> Result<Self, ErrorData> {
        let core = noteka_core::describe();
        let mut names = BTreeMap::new();
        let mut tools = Vec::new();
        let mut enabled = Vec::new();
        for operation in core["inputs"].as_object().expect("core catalog").keys() {
            let name = format!("noteka_{}", operation.replace('.', "_"));
            if !name.is_ascii()
                || name.len() > 128
                || names.insert(name.clone(), operation.clone()).is_some()
            {
                return Err(ErrorData::internal_error("Catálogo ambiguo.", None));
            }
            let access = scoped::named_access(operation, policy)
                .ok_or_else(|| ErrorData::internal_error("Operación sin política.", None))?;
            if !policy.allows(access) {
                continue;
            }
            let read = access == Access::Read;
            let mut input = core["inputs"][operation].clone();
            input["type"] = json!("object");
            let mixed = matches!(
                operation.as_str(),
                "extraction.control" | "extraction.index"
            );
            if mixed && !policy.allow_write() {
                input["oneOf"]
                    .as_array_mut()
                    .expect("mixed enum")
                    .retain(|v| v["properties"]["action"]["const"] == "get");
            }
            let annotations = json!({"readOnlyHint":read,"destructiveHint":!read,"idempotentHint":read,"openWorldHint":false});
            let tool: Tool = serde_json::from_value(json!({"name":name,"description":format!("Operación local {operation}; validación y permisos del núcleo."),"inputSchema":input,"outputSchema":output_schema(&core,operation),"annotations":annotations}))
                .map_err(|_| ErrorData::internal_error("Schema de catálogo inválido.",None))?;
            let actions: Vec<&str> = if operation == "extraction.control" {
                if read {
                    vec!["get"]
                } else {
                    vec!["get", "set"]
                }
            } else if operation == "extraction.index" {
                if read {
                    vec!["get"]
                } else {
                    vec!["get", "put", "reset"]
                }
            } else {
                vec![]
            };
            enabled.push(json!({"name":name,"operation":operation,"access":access,"actions":actions,"annotations":annotations}));
            tools.push(tool);
        }
        tools.sort_by(|a, b| a.name.cmp(&b.name));
        let resource = serde_json::to_string(&json!({
            "core_catalog":core,"tools":enabled,"excluded_operations":scoped::EXCLUDED,
            "policy":{"allow_write":policy.allow_write(),"allow_run":policy.allow_run()},
            "protocol_versions":["2026-07-28","2025-11-25"],
            "limits":{"input_bytes":INPUT_CAP,"envelope_bytes":ENVELOPE_CAP,"output_bytes":OUTPUT_CAP,"page_size":PAGE,"cursor_bytes":2048,"protocol_requests":4,"core_active":1,"core_pending":0,"input_channel":4,"output_channel":4,"stdout_timeout_ms":5000,"shutdown_ms":30000}
        })).expect("JSON catalog");
        let fingerprint = format!("{:x}", Sha256::digest(resource.as_bytes()));
        Ok(Self {
            tools,
            names,
            resource,
            fingerprint,
        })
    }
    pub fn page(
        &self,
        request: Option<PaginatedRequestParams>,
    ) -> Result<(Vec<Tool>, Option<String>), ErrorData> {
        let cursor = request.and_then(|p| p.cursor);
        let offset = if let Some(cursor) = cursor {
            if cursor.len() > 2048 {
                return Err(bad_cursor());
            }
            let prefix = format!("v1.{}.", self.fingerprint);
            let s = cursor.strip_prefix(&prefix).ok_or_else(bad_cursor)?;
            let n: usize = s.parse().map_err(|_| bad_cursor())?;
            if n == 0 || n >= self.tools.len() || !n.is_multiple_of(PAGE) || s != n.to_string() {
                return Err(bad_cursor());
            }
            n
        } else {
            0
        };
        let end = (offset + PAGE).min(self.tools.len());
        Ok((
            self.tools[offset..end].to_vec(),
            (end < self.tools.len()).then(|| format!("v1.{}.{end}", self.fingerprint)),
        ))
    }
}
fn bad_cursor() -> ErrorData {
    ErrorData::invalid_params("Cursor de catálogo inválido.", None)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn refs(v: &Value, root: &Value) {
        match v {
            Value::Object(o) => {
                for (k, v) in o {
                    if k == "$ref" {
                        assert!(
                            root.pointer(v.as_str().unwrap().strip_prefix('#').unwrap())
                                .is_some(),
                            "{v}"
                        );
                    } else {
                        refs(v, root);
                    }
                }
            }
            Value::Array(a) => {
                for v in a {
                    refs(v, root);
                }
            }
            _ => (),
        }
    }
    #[test]
    fn complete_schemas_pages_and_scoped_cursors() {
        let mut cursors = Vec::new();
        for (w, r, n) in [(false, false, 25), (true, false, 45), (true, true, 46)] {
            let c = Catalog::new(Policy::new(w, r).unwrap()).unwrap();
            assert_eq!(c.names.len(), 53);
            assert_eq!(c.tools.len(), n);
            for t in &c.tools {
                for s in [
                    serde_json::to_value(&t.input_schema).unwrap(),
                    serde_json::to_value(&t.output_schema).unwrap(),
                ] {
                    assert_eq!(s["type"], "object");
                    refs(&s, &s);
                }
            }
            let mut p = None;
            let mut all = Vec::new();
            loop {
                let (tools, next) = c.page(p).unwrap();
                assert!(tools.len() <= 8);
                all.extend(tools);
                match next {
                    Some(cursor) => {
                        cursors.push(cursor.clone());
                        p = Some(serde_json::from_value(json!({"cursor":cursor})).unwrap());
                    }
                    None => break,
                }
            }
            assert_eq!(all, c.tools);
            for cursor in cursors
                .iter()
                .filter(|s| !s.contains(&c.fingerprint))
                .chain(
                    [
                        "bad".to_string(),
                        "a".repeat(2049),
                        format!("v1.{}.0", c.fingerprint),
                    ]
                    .iter(),
                )
            {
                assert!(
                    c.page(Some(
                        serde_json::from_value(json!({"cursor":cursor})).unwrap()
                    ))
                    .is_err()
                );
            }
        }
    }
}
