//! Sólo harness de desarrollo: bytes sintéticos por stdin, sin rutas ni SQLite.
use noteka_pdf_protocol::{MAX_SOURCE_BYTES, Outcome, Reply, WorkerError};
use noteka_pdf_worker::{containment, parser};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};

fn main() {
    containment::initialize();
    let mut source = Vec::new();
    let outcome = match std::io::stdin()
        .take(MAX_SOURCE_BYTES as u64 + 1)
        .read_to_end(&mut source)
    {
        Ok(_) => match parser::extract(&source) {
            Ok(pages) => Outcome::Success {
                source_sha256: format!("{:x}", Sha256::digest(&source)),
                pages,
            },
            Err(reason) => Outcome::Error { reason },
        },
        Err(_) => Outcome::Error {
            reason: WorkerError::PdfWorkerProtocol,
        },
    };
    let reply = Reply::new(outcome);
    if reply.validate().is_err()
        || serde_json::to_writer(std::io::stdout().lock(), &reply).is_err()
        || std::io::stdout().flush().is_err()
    {
        containment::terminate(containment::EXIT_SETUP);
    }
}
