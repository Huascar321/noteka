//! Sólo desarrollo: permite comparar el parser real sin el framing del helper.
use noteka_xlsx_protocol::{MAX_SOURCE, Outcome, Reply};
use sha2::{Digest, Sha256};
use std::io::Read;

fn main() {
    noteka_parser_runtime::initialize();
    let mut bytes = Vec::new();
    std::io::stdin()
        .take(MAX_SOURCE as u64 + 1)
        .read_to_end(&mut bytes)
        .unwrap();
    let outcome = match noteka_xlsx_worker::parser::extract(&bytes) {
        Ok(workbook) => Outcome::Success {
            source_sha256: format!("{:x}", Sha256::digest(&bytes)),
            workbook,
        },
        Err(reason) => Outcome::Error { reason },
    };
    serde_json::to_writer(std::io::stdout().lock(), &Reply::new(outcome)).unwrap();
}
