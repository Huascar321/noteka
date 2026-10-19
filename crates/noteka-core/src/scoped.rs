//! Entrada local restringida. No es una sandbox frente a otro proceso del mismo usuario.
use crate::{
    contracts::*,
    error::*,
    workspace::{self, Workspace},
};
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Access {
    Read,
    Write,
    Run,
    Excluded,
}

#[derive(Debug, Clone, Copy)]
pub struct Policy {
    write: bool,
    run: bool,
}
impl Policy {
    pub fn new(write: bool, run: bool) -> Result<Self> {
        if run && !write {
            return Err(CoreError::validation(
                "Ejecución requiere escritura explícita.",
            ));
        }
        Ok(Self { write, run })
    }
    pub fn allows(self, access: Access) -> bool {
        match access {
            Access::Read => true,
            Access::Write => self.write,
            Access::Run => self.write && self.run,
            Access::Excluded => false,
        }
    }
    pub fn allow_write(self) -> bool {
        self.write
    }
    pub fn allow_run(self) -> bool {
        self.run
    }
}

// Metadata in-process: no amplía ni modifica el catálogo público v1.
pub const READ: &[&str] = &[
    "changes.list",
    "changes.preview",
    "collection.get",
    "collection.list",
    "database.query",
    "entity.get",
    "entity.list",
    "extraction.control",
    "extraction.fragments",
    "extraction.get",
    "extraction.index",
    "extraction.list",
    "extraction.ocr_regions",
    "extraction.pdf_fragments",
    "extraction.search",
    "extraction.xlsx_cells",
    "home.query",
    "operations.describe",
    "record.get",
    "record.inverse",
    "relation.list",
    "search.query",
    "template.get",
    "template.list",
    "workflow.query",
];
pub const WRITE: &[&str] = &[
    "changes.apply",
    "collection.create",
    "entity.archive",
    "entity.create",
    "entity.restore",
    "entity.update",
    "extraction.cancel",
    "extraction.enqueue",
    "extraction.retry",
    "formula.set",
    "property.define",
    "record.attach",
    "record.create",
    "record.update",
    "relation.add",
    "relation.remove",
    "search.rebuild",
    "template.instantiate",
    "template.save",
    "view.save",
];
pub const EXCLUDED: &[&str] = &[
    "workspace.create",
    "workspace.open",
    "workspace.backup",
    "workspace.restore",
    "attachment.import",
    "attachment.read",
    "document.export",
];
pub fn named_access(name: &str, policy: Policy) -> Option<Access> {
    if matches!(name, "extraction.control" | "extraction.index") && policy.write {
        Some(Access::Write)
    } else if READ.contains(&name) {
        Some(Access::Read)
    } else if WRITE.contains(&name) {
        Some(Access::Write)
    } else if name == "extraction.run" {
        Some(Access::Run)
    } else if EXCLUDED.contains(&name) {
        Some(Access::Excluded)
    } else {
        None
    }
}
pub fn operation_access(op: &Operation) -> Access {
    use Operation::*;
    match op {
        ExtractionControl(ExtractionControlInput::Get {})
        | ExtractionIndex(AttachmentIndexInput::Get { .. }) => Access::Read,
        ExtractionControl(ExtractionControlInput::Set { .. })
        | ExtractionIndex(AttachmentIndexInput::Put { .. } | AttachmentIndexInput::Reset { .. }) => {
            Access::Write
        }
        Describe(_)
        | ChangesList(_)
        | ChangesPreview(_)
        | CollectionGet(_)
        | CollectionList(_)
        | DatabaseQuery(_)
        | EntityGet(_)
        | EntityList(_)
        | ExtractionFragments(_)
        | ExtractionGet(_)
        | ExtractionList(_)
        | ExtractionPdfFragments(_)
        | ExtractionSearch(_)
        | ExtractionXlsxCells(_)
        | ExtractionOcrRegions(_)
        | HomeQuery(_)
        | RecordGet(_)
        | RecordInverse(_)
        | RelationList(_)
        | SearchQuery(_)
        | TemplateGet(_)
        | TemplateList(_)
        | WorkflowQuery(_) => Access::Read,
        ChangesApply(_)
        | CollectionCreate(_)
        | EntityArchive(_)
        | EntityCreate(_)
        | EntityRestore(_)
        | EntityUpdate(_)
        | ExtractionCancel(_)
        | ExtractionEnqueue(_)
        | ExtractionRetry(_)
        | FormulaSet(_)
        | PropertyDefine(_)
        | RecordAttach(_)
        | RecordCreate(_)
        | RecordUpdate(_)
        | RelationAdd(_)
        | RelationRemove(_)
        | SearchRebuild(_)
        | TemplateInstantiate(_)
        | TemplateSave(_)
        | ViewSave(_) => Access::Write,
        ExtractionRun(_) => Access::Run,
        WorkspaceCreate(_) | WorkspaceOpen(_) | WorkspaceBackup(_) | WorkspaceRestore(_)
        | AttachmentImport(_) | AttachmentRead(_) | DocumentExport(_) => Access::Excluded,
    }
}
pub fn denied() -> CoreError {
    let mut e = CoreError::validation("Operación no autorizada para este scope local.");
    e.details = json!({"reason":"permission_denied"});
    e
}

#[derive(Debug, Clone)]
pub struct LocalScope {
    root: PathBuf,
    workspace_id: String,
    policy: Policy,
    stopping: Arc<AtomicBool>,
}
impl LocalScope {
    pub fn new(path: &Path, policy: Policy) -> Result<Self> {
        if path.to_str().is_none() {
            return Err(CoreError::validation("Ruta de workspace no UTF-8."));
        }
        let root = std::fs::canonicalize(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CoreError::new(ErrorCode::NotFound, "Workspace ausente.")
            } else {
                e.into()
            }
        })?;
        // The shared Request JSON includes the trusted path for its byte/depth
        // budget. Reject an unrepresentable host path before json! can panic.
        if root.to_str().is_none() {
            return Err(CoreError::validation("Ruta de workspace no UTF-8."));
        }
        let (version, workspace_id) = workspace::preflight_identity(&root)?;
        current(version)?;
        Ok(Self {
            root,
            workspace_id,
            policy,
            stopping: Arc::default(),
        })
    }
    pub fn policy(&self) -> Policy {
        self.policy
    }
    pub fn stop(&self) {
        self.stopping.store(true, Ordering::Release);
    }
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }
    pub(crate) fn check(&self, c: &Connection) -> Result<()> {
        current(workspace::version(c)?)?;
        let id: String = c.query_row("SELECT id FROM workspace_meta", [], |r| r.get(0))?;
        if id != self.workspace_id {
            let mut e = CoreError::new(
                ErrorCode::RevisionConflict,
                "Cambió la identidad del workspace.",
            );
            e.details = json!({"reason":"workspace_scope_mismatch"});
            return Err(e);
        }
        Ok(())
    }
    pub(crate) fn open(&self, write: bool) -> Result<Workspace> {
        self.open_inner(
            write,
            #[cfg(test)]
            None,
        )
    }
    #[cfg(test)]
    pub(crate) fn open_with_hook(&self, write: bool, hook: &mut dyn FnMut()) -> Result<Workspace> {
        self.open_inner(write, Some(hook))
    }
    fn open_inner(
        &self,
        write: bool,
        #[cfg(test)] before_open: Option<&mut dyn FnMut()>,
    ) -> Result<Workspace> {
        current(workspace::preflight(&self.root)?)?;
        #[cfg(test)]
        if let Some(hook) = before_open {
            hook();
        }
        let c = Connection::open_with_flags(
            self.root.join(workspace::DB_NAME),
            if write {
                OpenFlags::SQLITE_OPEN_READ_WRITE
            } else {
                OpenFlags::SQLITE_OPEN_READ_ONLY
            },
        )?;
        c.busy_timeout(Duration::from_millis(workspace::BUSY_TIMEOUT_MS))?;
        self.check(&c)?;
        if write {
            c.pragma_update(None, "foreign_keys", true)?;
            c.pragma_update(None, "synchronous", "FULL")?;
        }
        Ok(Workspace {
            conn: c,
            path: self.root.clone(),
        })
    }
    /// Sólo el host llama stop al cerrar. Cancelar una solicitud MCP no lo llama.
    pub fn execute_json(&self, command: Value, origin: &str) -> Response {
        let request_id = workspace::id();
        let result = (|| {
            crate::entities::validate_request_limits(
                &json!({"workspace":self.root,"command":command}),
            )?;
            let op: Operation = serde_json::from_value(command)?;
            if !self.policy.allows(operation_access(&op)) {
                return Err(denied());
            }
            if self.stopping.load(Ordering::Acquire) {
                return Err(CoreError::new(ErrorCode::WorkspaceBusy, "Scope cerrado."));
            }
            let _shutdown = ShutdownGuard::new(self.stopping.clone());
            crate::dispatch_scoped(op, self, origin, &request_id)
        })();
        crate::envelope(request_id, result)
    }
}
fn current(v: u32) -> Result<()> {
    if v == workspace::SCHEMA_VERSION {
        return Ok(());
    }
    let mut e = CoreError::new(
        ErrorCode::UnsupportedWorkspaceVersion,
        "Se requiere un workspace actual; migrá explícitamente por CLI.",
    );
    e.details = json!({"current_version":v,"supported_version":workspace::SCHEMA_VERSION,"migration_required":v<workspace::SCHEMA_VERSION});
    Err(e)
}

// Señal de apagado, no autorización ni caché: préstamo por invocación síncrona.
thread_local! { static SHUTDOWN: RefCell<Option<Arc<AtomicBool>>> = const { RefCell::new(None) }; }
struct ShutdownGuard(Option<Arc<AtomicBool>>);
impl ShutdownGuard {
    fn new(flag: Arc<AtomicBool>) -> Self {
        Self(SHUTDOWN.with(|s| s.replace(Some(flag))))
    }
}
impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        SHUTDOWN.with(|s| s.replace(self.0.take()));
    }
}
pub(crate) fn check_shutdown() -> Result<()> {
    if SHUTDOWN.with(|s| {
        s.borrow()
            .as_ref()
            .is_some_and(|v| v.load(Ordering::Acquire))
    }) {
        Err(CoreError::new(
            ErrorCode::WorkspaceBusy,
            "Host local cerrado.",
        ))
    } else {
        Ok(())
    }
}
