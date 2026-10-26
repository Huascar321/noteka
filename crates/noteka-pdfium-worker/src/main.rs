//! Worker PDFium: proceso descartable, sin workspace ni red.
use noteka_pdfium_protocol::*;
use pdfium_render::prelude::*;
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

fn library() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("NOTEKA_PDFIUM_LIB").map(PathBuf::from) {
        return path.is_file().then_some(path);
    }
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    [
        "libpdfium.dylib",
        "libpdfium.so",
        "pdfium.dll",
        "libpdfium.dll",
    ]
    .into_iter()
    .map(|name| dir.join(name))
    .find(|path| path.is_file())
}

fn extract(bytes: &[u8], lib: &Path) -> Result<Vec<PageText>, WorkerError> {
    let bindings = Pdfium::bind_to_library(lib.to_string_lossy().as_ref())
        .map_err(|_| WorkerError::PdfiumUnavailable)?;
    let pdfium = Pdfium::new(bindings);
    let document = pdfium
        .load_pdf_from_byte_slice(bytes, None)
        .map_err(|_| WorkerError::PdfDecodeError)?;
    let count = document.pages().len() as usize;
    if count == 0 {
        return Err(WorkerError::InvalidPdf);
    }
    if count > MAX_PAGES {
        return Err(WorkerError::PdfPageLimit);
    }
    let mut pages = Vec::with_capacity(count);
    let mut total = 0usize;
    for (index, page) in document.pages().iter().enumerate() {
        let text = page.text().map_err(|_| WorkerError::PdfDecodeError)?.all();
        if text.contains('\0') {
            return Err(WorkerError::PdfDecodeError);
        }
        total = total
            .checked_add(text.len())
            .ok_or(WorkerError::PdfTextLimit)?;
        if total > MAX_TEXT_BYTES {
            return Err(WorkerError::PdfTextLimit);
        }
        pages.push(PageText {
            page_number: index as u32 + 1,
            text,
        });
    }
    Ok(pages)
}

fn main() {
    noteka_parser_runtime::initialize();
    let mut header = [0; HEADER_BYTES];
    let outcome = if std::io::stdin().read_exact(&mut header).is_err() {
        Outcome::Error {
            reason: WorkerError::PdfWorkerProtocol,
        }
    } else if let Ok(header) = Header::decode(&header) {
        noteka_parser_runtime::shorten_budget(header.remaining_ms);
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
            } else if let Some(lib) = library() {
                match extract(&bytes, &lib) {
                    Ok(pages) => Outcome::Success {
                        source_sha256: format!("{:x}", Sha256::digest(&bytes)),
                        pages,
                    },
                    Err(reason) => Outcome::Error { reason },
                }
            } else {
                Outcome::Error {
                    reason: WorkerError::PdfiumUnavailable,
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
        noteka_parser_runtime::terminate(noteka_parser_runtime::EXIT_SETUP);
    }
}
