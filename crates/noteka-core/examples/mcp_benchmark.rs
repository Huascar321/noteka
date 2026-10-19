//! Companion to scripts/benchmark-mcp.py. Synthetic current workspace only.
use noteka_core::{
    execute_json,
    scoped::{LocalScope, Policy},
};
use serde_json::{Value, json};
use std::{path::Path, time::Instant};
fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    assert_eq!(args.len(), 3);
    let scope = LocalScope::new(Path::new(&args[1]), Policy::new(false, false).unwrap()).unwrap();
    let cases = [
        ("entity.get", json!({"id":args[2]})),
        ("entity.list", json!({})),
        (
            "home.query",
            json!({"view":"notes_all","as_of":"2026-10-15"}),
        ),
    ];
    let mut results = serde_json::Map::new();
    for (op, input) in cases {
        for scoped in [false, true] {
            let mut samples = Vec::new();
            for i in 0..25 {
                let command = json!({"operation":op,"input":input});
                let start = Instant::now();
                let r = if scoped {
                    scope.execute_json(command, "benchmark")
                } else {
                    execute_json(json!({"workspace":args[1],"command":command}), "benchmark")
                };
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                assert!(r.ok, "{:?}", r.error);
                if i >= 5 {
                    samples.push(elapsed);
                }
            }
            results.insert(
                format!(
                    "{op}.{}",
                    if scoped {
                        "scoped_core"
                    } else {
                        "historical_core"
                    }
                ),
                json!(samples),
            );
        }
    }
    println!("{}", Value::Object(results));
}
