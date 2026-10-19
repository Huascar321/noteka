use noteka_parser_runtime as containment;
use noteka_xlsx_protocol::{HEADER_BYTES, Header, MAX_STDOUT, Outcome, Reply, WorkerError};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};

pub fn run() {
    containment::initialize();
    let reason = WorkerError::XlsxWorkerProtocol;
    let mut header = [0; HEADER_BYTES];
    let outcome = (|| {
        std::io::stdin()
            .read_exact(&mut header)
            .map_err(|_| reason)?;
        let header = Header::decode(&header).map_err(|_| reason)?;
        containment::shorten_budget(header.remaining_ms);
        let mut bytes = vec![0; header.source_bytes as usize];
        std::io::stdin()
            .read_exact(&mut bytes)
            .map_err(|_| reason)?;
        let mut extra = [0];
        if !matches!(std::io::stdin().read(&mut extra), Ok(0)) {
            return Err(reason);
        }
        let workbook = crate::parser::extract(&bytes)?;
        Ok(Outcome::Success {
            source_sha256: format!("{:x}", Sha256::digest(&bytes)),
            workbook,
        })
    })()
    .unwrap_or_else(|reason| Outcome::Error { reason });
    let reply = Reply::new(outcome);
    let bytes = match serde_json::to_vec(&reply) {
        Ok(bytes) if bytes.len() <= MAX_STDOUT => bytes,
        _ => containment::terminate(containment::EXIT_SETUP),
    };
    if std::io::stdout().lock().write_all(&bytes).is_err() || std::io::stdout().flush().is_err() {
        containment::terminate(containment::EXIT_SETUP);
    }
}
