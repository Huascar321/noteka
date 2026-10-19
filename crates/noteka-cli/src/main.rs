use clap::{Parser, Subcommand};
use noteka_core::{
    contracts::Response,
    error::{CoreError, ErrorCode},
};
use serde_json::{Value, json};
use std::{io::Read, path::PathBuf};

#[derive(Parser)]
#[command(
    name = "noteka",
    version,
    about = "Núcleo local de Noteka. Payload JSON por --input o stdin."
)]
struct Args {
    #[arg(long, global = true)]
    workspace: Option<PathBuf>,
    #[arg(long, global = true)]
    json: bool,
    #[arg(long, global = true)]
    input: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Extraction {
        #[command(subcommand)]
        action: ExtractionAction,
    },
    Home {
        #[command(subcommand)]
        action: HomeAction,
    },
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },
    Formula {
        #[command(subcommand)]
        action: FormulaAction,
    },
    Collection {
        #[command(subcommand)]
        action: CollectionAction,
    },
    Property {
        #[command(subcommand)]
        action: PropertyAction,
    },
    Record {
        #[command(subcommand)]
        action: RecordAction,
    },
    View {
        #[command(subcommand)]
        action: ViewAction,
    },
    Database {
        #[command(subcommand)]
        action: DatabaseAction,
    },
    Search {
        #[command(subcommand)]
        action: SearchAction,
    },
    Workflow {
        #[command(subcommand)]
        action: WorkflowAction,
    },
    Workspace {
        #[command(subcommand)]
        action: WorkspaceAction,
    },
    Operations {
        #[command(subcommand)]
        action: OperationsAction,
    },
    Entity {
        #[command(subcommand)]
        action: EntityAction,
    },
    Relation {
        #[command(subcommand)]
        action: RelationAction,
    },
    Changes {
        #[command(subcommand)]
        action: ChangesAction,
    },
    Attachment {
        #[command(subcommand)]
        action: AttachmentAction,
    },
    Document {
        #[command(subcommand)]
        action: DocumentAction,
    },
}
#[derive(Subcommand)]
enum TemplateAction {
    Save,
    Get,
    List,
    Instantiate,
}
#[derive(Subcommand)]
enum HomeAction {
    Query,
}
#[derive(Subcommand)]
enum ExtractionAction {
    Index,
    Search,
    Enqueue,
    Get,
    List,
    Run,
    Cancel,
    Retry,
    Control,
    Fragments,
    PdfFragments,
    XlsxCells,
    OcrRegions,
}
#[derive(Subcommand)]
enum FormulaAction {
    Set,
}
#[derive(Subcommand)]
enum CollectionAction {
    Create,
    Get,
    List,
}
#[derive(Subcommand)]
enum PropertyAction {
    Define,
}
#[derive(Subcommand)]
enum RecordAction {
    Create,
    Get,
    Attach,
    Update,
    Inverse,
}
#[derive(Subcommand)]
enum ViewAction {
    Save,
}
#[derive(Subcommand)]
enum DatabaseAction {
    Query,
}
#[derive(Subcommand)]
enum SearchAction {
    Query,
    Rebuild,
}
#[derive(Subcommand)]
enum WorkflowAction {
    Query,
}
#[derive(Subcommand)]
enum AttachmentAction {
    Import,
    Read,
}
#[derive(Subcommand)]
enum DocumentAction {
    Export,
}
#[derive(Subcommand)]
enum WorkspaceAction {
    Create,
    Open,
    Backup,
    Restore,
}
#[derive(Subcommand)]
enum OperationsAction {
    Describe,
}
#[derive(Subcommand)]
enum EntityAction {
    Create,
    Get,
    List,
    Update,
    Archive,
    Restore,
}
#[derive(Subcommand)]
enum RelationAction {
    Add,
    Remove,
    List,
}
#[derive(Subcommand)]
enum ChangesAction {
    List,
    Preview,
    Apply,
}
impl Command {
    fn operation(&self) -> &'static str {
        match self {
            Self::Extraction { action } => match action {
                ExtractionAction::Index => "extraction.index",
                ExtractionAction::Search => "extraction.search",
                ExtractionAction::Enqueue => "extraction.enqueue",
                ExtractionAction::Get => "extraction.get",
                ExtractionAction::List => "extraction.list",
                ExtractionAction::Run => "extraction.run",
                ExtractionAction::Cancel => "extraction.cancel",
                ExtractionAction::Retry => "extraction.retry",
                ExtractionAction::Control => "extraction.control",
                ExtractionAction::Fragments => "extraction.fragments",
                ExtractionAction::PdfFragments => "extraction.pdf_fragments",
                ExtractionAction::XlsxCells => "extraction.xlsx_cells",
                ExtractionAction::OcrRegions => "extraction.ocr_regions",
            },
            Self::Home { .. } => "home.query",
            Self::Template { action } => match action {
                TemplateAction::Save => "template.save",
                TemplateAction::Get => "template.get",
                TemplateAction::List => "template.list",
                TemplateAction::Instantiate => "template.instantiate",
            },
            Self::Formula { .. } => "formula.set",
            Self::Collection { action } => match action {
                CollectionAction::Create => "collection.create",
                CollectionAction::Get => "collection.get",
                CollectionAction::List => "collection.list",
            },
            Self::Property { .. } => "property.define",
            Self::Record { action } => match action {
                RecordAction::Create => "record.create",
                RecordAction::Get => "record.get",
                RecordAction::Attach => "record.attach",
                RecordAction::Update => "record.update",
                RecordAction::Inverse => "record.inverse",
            },
            Self::View { .. } => "view.save",
            Self::Database { .. } => "database.query",
            Self::Search { action } => match action {
                SearchAction::Query => "search.query",
                SearchAction::Rebuild => "search.rebuild",
            },
            Self::Workflow { .. } => "workflow.query",
            Self::Workspace { action } => match action {
                WorkspaceAction::Create => "workspace.create",
                WorkspaceAction::Open => "workspace.open",
                WorkspaceAction::Backup => "workspace.backup",
                WorkspaceAction::Restore => "workspace.restore",
            },
            Self::Operations { .. } => "operations.describe",
            Self::Entity { action } => match action {
                EntityAction::Create => "entity.create",
                EntityAction::Get => "entity.get",
                EntityAction::List => "entity.list",
                EntityAction::Update => "entity.update",
                EntityAction::Archive => "entity.archive",
                EntityAction::Restore => "entity.restore",
            },
            Self::Relation { action } => match action {
                RelationAction::Add => "relation.add",
                RelationAction::Remove => "relation.remove",
                RelationAction::List => "relation.list",
            },
            Self::Changes { action } => match action {
                ChangesAction::List => "changes.list",
                ChangesAction::Preview => "changes.preview",
                ChangesAction::Apply => "changes.apply",
            },
            Self::Attachment { action } => match action {
                AttachmentAction::Import => "attachment.import",
                AttachmentAction::Read => "attachment.read",
            },
            Self::Document { .. } => "document.export",
        }
    }
}
fn run(args: &Args) -> Response {
    let result = (|| -> Result<Value, CoreError> {
        let mut buffer = Vec::new();
        let mut reader: Box<dyn Read> = match &args.input {
            Some(p) if p.as_os_str() != "-" => Box::new(std::fs::File::open(p)?),
            _ => Box::new(std::io::stdin()),
        };
        use std::io::IsTerminal;
        if args.input.is_some() || !std::io::stdin().is_terminal() {
            reader.by_ref().take(2_097_153).read_to_end(&mut buffer)?;
        }
        if buffer.len() > 2_097_152 {
            return Err(CoreError::validation("Entrada excede 2 MiB."));
        }
        let input = if buffer.is_empty() {
            json!({})
        } else {
            serde_json::from_slice(&buffer)?
        };
        Ok(
            json!({"workspace":args.workspace.as_ref().map(|p|p.to_string_lossy()),"command":{"operation":args.command.operation(),"input":input}}),
        )
    })();
    match result {
        Ok(request) => noteka_core::execute_json(request, "cli"),
        Err(error) => noteka_core::error_response(error),
    }
}
fn print(response: Response, structured: bool) -> ! {
    let code = response.error.as_ref().map_or(0, CoreError::exit_code);
    if structured {
        println!("{}", serde_json::to_string(&response).unwrap());
    } else if response.ok {
        println!("{}", serde_json::to_string_pretty(&response.data).unwrap());
    } else {
        eprintln!("{}", response.error.as_ref().unwrap().message);
    }
    std::process::exit(code)
}
fn main() {
    let structured = std::env::args().any(|s| s == "--json");
    match Args::try_parse() {
        Ok(args) => print(run(&args), args.json),
        Err(e)
            if matches!(
                e.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            let _ = e.print();
            std::process::exit(0);
        }
        Err(e) => {
            if !structured {
                let _ = e.print();
            }
            print(
                noteka_core::error_response(CoreError::new(
                    ErrorCode::ValidationError,
                    "Argumentos inválidos; consultá noteka --help.",
                )),
                structured,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use std::collections::BTreeSet;
    #[test]
    fn cli_catalog_and_public_operation_sets_are_identical() {
        let catalog = noteka_core::describe();
        let declared = catalog["inputs"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut actual = BTreeSet::new();
        for group in Args::command().get_subcommands() {
            if group.get_name() == "help" {
                continue;
            }
            for action in group.get_subcommands() {
                if action.get_name() == "help" {
                    continue;
                }
                let args =
                    Args::try_parse_from(["noteka", group.get_name(), action.get_name()]).unwrap();
                actual.insert(args.command.operation().to_string());
            }
        }
        assert_eq!(actual, declared);
        assert_eq!(actual.len(), 53);
        let schema = noteka_core::contract_schema();
        let variants = schema["$defs"]["Operation"]["oneOf"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                v["properties"]["operation"]["const"]
                    .as_str()
                    .unwrap()
                    .to_string()
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, variants);
        assert_eq!(
            catalog["outputs"]
                .as_object()
                .unwrap()
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>(),
            declared
        );
        assert_eq!(catalog["batch"]["mutations"].as_object().unwrap().len(), 9);
        for (op, spec) in catalog["batch"]["mutations"].as_object().unwrap() {
            assert_eq!(spec["resolved_input_schema"], catalog["inputs"][op]);
            assert_eq!(
                catalog["operation_capabilities"][op]["batch_mutation"],
                true
            );
        }
    }
}
