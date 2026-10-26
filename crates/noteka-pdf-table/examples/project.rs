//! Sólo desarrollo: JSON por stdin, sin rutas, workspace, helpers ni red.
use noteka_pdf_table::{Input, MAX_WIRE, TableError, project};
use std::io::{Read, Write};
fn main() {
    let mut raw = Vec::new();
    let result = std::io::stdin()
        .take(MAX_WIRE as u64 + 1)
        .read_to_end(&mut raw)
        .map_err(|_| TableError::InvalidInput)
        .and_then(|_| {
            if raw.len() > MAX_WIRE {
                return Err(TableError::InvalidInput);
            }
            let input: Input =
                serde_json::from_slice(&raw).map_err(|_| TableError::InvalidInput)?;
            project(input)
        });
    let (code, value) = match result {
        Ok(layout) => (0, serde_json::json!({"ok":true,"layout":layout})),
        Err(reason) => (2, serde_json::json!({"ok":false,"reason":reason})),
    };
    let mut bytes = serde_json::to_vec(&value).expect("JSON values");
    if bytes.len() + 1 > MAX_WIRE {
        bytes = br#"{"ok":false,"reason":"response_limit"}"#.to_vec();
        bytes.push(b'\n');
        let _ = std::io::stdout().write_all(&bytes);
        std::process::exit(2);
    }
    bytes.push(b'\n');
    if std::io::stdout().write_all(&bytes).is_err() {
        std::process::exit(2)
    }
    std::process::exit(code);
}
