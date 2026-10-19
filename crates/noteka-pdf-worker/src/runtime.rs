use crate::containment;
use noteka_pdf_protocol::{HEADER_BYTES, Header, Outcome, Reply, WorkerError};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};

pub fn run() {
    containment::initialize();
    let mut header = [0; HEADER_BYTES];
    let outcome = if std::io::stdin().read_exact(&mut header).is_err() {
        Outcome::Error {
            reason: WorkerError::PdfWorkerProtocol,
        }
    } else if let Ok(header) = Header::decode(&header) {
        containment::shorten_budget(header.remaining_ms);
        let mut bytes = vec![0; header.source_bytes as usize];
        if std::io::stdin().read_exact(&mut bytes).is_err() {
            Outcome::Error {
                reason: WorkerError::PdfWorkerProtocol,
            }
        } else {
            let mut extra = [0];
            if !matches!(std::io::stdin().read(&mut extra), Ok(0)) {
                Outcome::Error {
                    reason: WorkerError::PdfWorkerProtocol,
                }
            } else {
                match crate::parser::extract(&bytes) {
                    Ok(pages) => Outcome::Success {
                        source_sha256: format!("{:x}", Sha256::digest(&bytes)),
                        pages,
                    },
                    Err(reason) => Outcome::Error { reason },
                }
            }
        }
    } else {
        Outcome::Error {
            reason: WorkerError::PdfWorkerProtocol,
        }
    };
    let reply = Reply::new(outcome);
    if serde_json::to_writer(std::io::stdout().lock(), &reply).is_err()
        || std::io::stdout().flush().is_err()
    {
        containment::terminate(containment::EXIT_SETUP);
    }
}
