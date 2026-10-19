//! Corpus sintético fuera de Git. Cargo run --release -p noteka-core --example search_benchmark
use noteka_core::{contracts::*, execute_json, workspace::Workspace};
use serde_json::{Value, json};
use std::time::Instant;
fn percentile(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    v[((v.len() as f64 * 0.95).ceil() as usize - 1).min(v.len() - 1)]
}
fn main() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("w");
    let mut w = Workspace::create(&path, Some("UTC".into())).unwrap();
    for i in 0..10_000 {
        let input:CreateEntity=serde_json::from_value(json!({"kind":"note","title":format!("Acuerdo proyecto {i}"),"document":{"schema_version":2,"blocks":[{"id":"block","type":"paragraph","content":[{"type":"text","text":format!("CRE tema{} acuerdo seguimiento {}",i%50,"texto local ".repeat(32))}]}]}})).unwrap();
        w.entity_create(input, "benchmark", "synthetic").unwrap();
    }
    let queries = ["CRE", "tema7", "acuerdo seguimiento", "ausentezyx"];
    let mut direct = vec![];
    let mut operation = vec![];
    for i in 0..80 {
        let input = json!({"query":queries[i%queries.len()],"limit":30});
        let start = Instant::now();
        w.search_query(serde_json::from_value(input.clone()).unwrap())
            .unwrap();
        direct.push(start.elapsed().as_secs_f64() * 1000.);
        let start = Instant::now();
        let r = execute_json(
            json!({"workspace":path,"command":{"operation":"search.query","input":input}}),
            "benchmark",
        );
        assert!(r.ok, "{:?}", r.error);
        operation.push(start.elapsed().as_secs_f64() * 1000.);
    }
    let bytes = std::fs::metadata(path.join("noteka.sqlite")).unwrap().len();
    let report: Value = json!({"notes":10_000,"samples_each":80,"queries":queries,"direct_p95_ms":percentile(&mut direct),"operation_with_workspace_preflight_p95_ms":percentile(&mut operation),"main_sqlite_bytes_excludes_wal":bytes,"profile":"release","runtime_arch":std::env::consts::ARCH,"corpus":"synthetic paragraph ~400 bytes; title+body indexed; hot caches; no UI, LLM or attachments"});
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
