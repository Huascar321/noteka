//! Adaptador universal PaddleOCR. Proceso externo configurado; sin red.
use crate::{
    batch_storage::Budget, contracts::PdfStructure, error::*, extraction_storage as store,
};
use base64::Engine;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

const MAX_OUTPUT: usize = 8 * 1024 * 1024;
const MAX_STDERR: usize = 8 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reply {
    parser_name: String,
    parser_version: String,
    source_sha256: String,
    pages: Vec<Page>,
    structure: PdfStructure,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Page {
    page_number: u32,
    text: String,
}

fn paths() -> Result<(PathBuf, PathBuf, PathBuf)> {
    let python = std::env::var_os("NOTEKA_PADDLE_PYTHON")
        .map(PathBuf::from)
        .ok_or_else(|| store::storage("paddle_python_unavailable"))?;
    let script = std::env::var_os("NOTEKA_PADDLE_WORKER")
        .map(PathBuf::from)
        .ok_or_else(|| store::storage("paddle_worker_unavailable"))?;
    let models = std::env::var_os("NOTEKA_PADDLE_MODEL_PATHS")
        .map(PathBuf::from)
        .ok_or_else(|| store::storage("paddle_models_unavailable"))?;
    for path in [&python, &script, &models] {
        if !path.is_file() {
            return Err(store::storage("paddle_worker_unavailable"));
        }
    }
    Ok((python, script, models))
}

pub(crate) struct Output {
    pub pages: Vec<noteka_pdf_protocol::PageText>,
    pub structure: PdfStructure,
}
pub(crate) fn run(
    bytes: &[u8],
    budget: &Budget<'_>,
    mut check: impl FnMut() -> Result<()>,
) -> Result<Output> {
    let (python, script, models) = paths()?;
    if bytes.is_empty() || bytes.len() > 1_048_576 {
        return Err(store::invalid("source_size_limit"));
    }
    let request = serde_json::json!({
        "source_b64": base64::engine::general_purpose::STANDARD.encode(bytes),
        "source_sha256": format!("{:x}", Sha256::digest(bytes)),
        "remaining_ms": budget.remaining_ms()?,
    });
    let mut command = Command::new(&python);
    command
        .arg(&script)
        .env_clear()
        .env("NOTEKA_PADDLE_MODEL_PATHS", &models)
        .env(
            "NOTEKA_PADDLE_DEVICE",
            std::env::var("NOTEKA_PADDLE_DEVICE").unwrap_or_else(|_| "cpu".into()),
        )
        .env("PYTHONNOUSERSITE", "1")
        .current_dir(
            script
                .parent()
                .ok_or_else(|| store::storage("paddle_worker_unavailable"))?,
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(home) = std::env::var_os("HOME") {
        command.env("HOME", home);
    }
    if let Some(path) = std::env::var_os("PATH") {
        command.env("PATH", path);
    }
    let mut child = command
        .spawn()
        .map_err(|_| store::storage("paddle_worker_unavailable"))?;
    child
        .stdin
        .take()
        .ok_or_else(|| store::storage("paddle_worker_failed"))?
        .write_all(serde_json::to_string(&request)?.as_bytes())
        .map_err(|_| store::storage("paddle_worker_failed"))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| store::storage("paddle_worker_failed"))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| store::storage("paddle_worker_failed"))?;
    let out = std::thread::spawn(move || {
        let mut data = Vec::new();
        let _ = stdout.read_to_end(&mut data);
        data
    });
    let err = std::thread::spawn(move || {
        let mut data = Vec::new();
        let _ = stderr.read_to_end(&mut data);
        data
    });
    let status = loop {
        check()?;
        budget.check()?;
        crate::scoped::check_shutdown()?;
        if let Some(status) = child
            .try_wait()
            .map_err(|_| store::storage("paddle_worker_failed"))?
        {
            break status;
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let output = out
        .join()
        .map_err(|_| store::storage("paddle_worker_failed"))?;
    let errors = err
        .join()
        .map_err(|_| store::storage("paddle_worker_failed"))?;
    if !status.success() || output.is_empty() || output.len() > MAX_OUTPUT {
        return Err(store::storage("paddle_worker_failed"));
    }
    if errors.len() > MAX_STDERR {
        return Err(store::storage("paddle_worker_protocol"));
    }
    let reply: Reply =
        serde_json::from_slice(&output).map_err(|_| store::storage("paddle_worker_protocol"))?;
    if reply.parser_name != "paddleocr_ppstructurev3"
        || reply.parser_version != "3.7.0"
        || reply.source_sha256 != format!("{:x}", Sha256::digest(bytes))
        || reply.pages.is_empty()
        || reply.pages.len() > 20
    {
        return Err(store::storage("paddle_worker_protocol"));
    }
    reply.structure.validate()?;
    let mut pages = Vec::with_capacity(reply.pages.len());
    let mut total = 0usize;
    for (index, page) in reply.pages.into_iter().enumerate() {
        if page.page_number as usize != index + 1 || page.text.contains('\0') {
            return Err(store::storage("paddle_worker_protocol"));
        }
        total = total
            .checked_add(page.text.len())
            .ok_or_else(|| store::storage("paddle_worker_protocol"))?;
        if total > 1_048_576 {
            return Err(store::invalid("pdf_text_limit"));
        }
        pages.push(noteka_pdf_protocol::PageText {
            page_number: page.page_number,
            text: page.text,
        });
    }
    Ok(Output {
        pages,
        structure: reply.structure,
    })
}
