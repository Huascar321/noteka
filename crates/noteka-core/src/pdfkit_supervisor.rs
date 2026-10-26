//! Adapta el consumidor privado a presupuesto y cercas del núcleo.
use crate::{batch_storage::Budget, error::*, extraction_storage as store};
use noteka_pdfkit_protocol::Document;
use noteka_pdfkit_runtime::Error;
use std::path::PathBuf;

fn runtime() -> Result<PathBuf> {
    let exe = std::env::current_exe().map_err(|_| store::storage("pdfkit_worker_unavailable"))?;
    let mut dir = exe
        .parent()
        .ok_or_else(|| store::storage("pdfkit_worker_unavailable"))?;
    if dir
        .file_name()
        .is_some_and(|n| n == "deps" || n == "examples")
    {
        dir = dir
            .parent()
            .ok_or_else(|| store::storage("pdfkit_worker_unavailable"))?;
    }
    Ok(dir.join("noteka-pdfkit-worker"))
}
pub(crate) fn run(
    bytes: &[u8],
    budget: &Budget<'_>,
    mut check: impl FnMut() -> Result<()>,
) -> Result<Document> {
    let ms = budget.remaining_ms()?.min(5000);
    if ms == 0 {
        return Err(store::error(ErrorCode::WorkspaceBusy, "execution_deadline"));
    }
    noteka_pdfkit_runtime::read(&runtime()?, bytes, ms, || {
        crate::scoped::check_shutdown()?;
        budget.check()?;
        check()
    })
    .map_err(|error| match error {
        Error::Guard(e) => e,
        Error::Deadline | Error::InvalidBudget => {
            store::error(ErrorCode::WorkspaceBusy, "execution_deadline")
        }
        Error::SourceLimit => store::invalid("source_size_limit"),
        Error::Unavailable => store::storage("pdfkit_worker_unavailable"),
        Error::ObservedMemoryLimit => store::invalid("pdfkit_memory_limit"),
        Error::Setup => store::storage("pdfkit_worker_setup"),
        Error::Protocol => store::storage("pdfkit_worker_protocol"),
        Error::Failed => store::storage("pdfkit_worker_failed"),
        Error::Rejected(reason) => match serde_json::to_value(reason)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
        {
            Some(reason) => store::invalid(&reason),
            None => store::storage("pdfkit_worker_protocol"),
        },
    })
}
