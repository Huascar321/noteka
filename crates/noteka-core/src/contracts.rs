pub use crate::attachment_search_contracts::*;
pub use crate::batch_contracts::*;
pub use crate::database_contracts::*;
use crate::error::CoreError;
pub use crate::extraction_contracts::*;
pub use crate::formula_contracts::*;
pub use crate::home_contracts::*;
pub use crate::ocr_contracts::*;
pub use crate::pdf_contracts::*;
pub use crate::template_contracts::*;
pub use crate::xlsx_contracts::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Note,
    Task,
    Project,
    Tag,
    Person,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelationKind {
    Project,
    Tag,
    Person,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Text {
    pub r#type: TextKind,
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub marks: Vec<Mark>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TextKind {
    Text,
    HardBreak,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Mark {
    pub r#type: MarkKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MarkKind {
    Bold,
    Italic,
    Strike,
    Code,
    Link,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BlockKind {
    Paragraph,
    Heading,
    BulletList,
    OrderedList,
    ListItem,
    TaskList,
    TaskItem,
    Blockquote,
    HorizontalRule,
    CodeBlock,
    Image,
    File,
    CollectionView,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Block {
    pub id: String,
    pub r#type: BlockKind,
    pub content: Vec<Text>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Block>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub schema_version: u32,
    pub blocks: Vec<Block>,
}
impl Default for Document {
    fn default() -> Self {
        Self {
            schema_version: 1,
            blocks: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateEntity {
    pub kind: EntityKind,
    pub title: String,
    #[serde(default)]
    pub payload: Value,
    pub document: Option<Document>,
    pub context: Option<ContextLink>,
    pub record: Option<RecordSeed>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContextLink {
    pub target_id: String,
    pub kind: RelationKind,
}
fn yes() -> bool {
    true
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NoteFields {
    #[serde(default = "yes")]
    pub inbox: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    #[default]
    Todo,
    Done,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskFields {
    #[serde(default)]
    pub state: TaskState,
    #[serde(default)]
    pub priority: Priority,
    pub due_date: Option<String>,
    pub planned_date: Option<String>,
    #[serde(default = "yes")]
    pub inbox: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectState {
    #[default]
    Planned,
    Doing,
    Done,
    Ongoing,
    OnHold,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectFields {
    #[serde(default)]
    pub state: ProjectState,
    #[serde(default)]
    pub purpose: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    Area,
    Resource,
    Topic,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TagFields {
    pub classification: Classification,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowView {
    Inbox,
    MyDay,
    Pending,
    Overdue,
    ActiveProjects,
    Context,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WorkflowQuery {
    pub view: WorkflowView,
    pub related_id: Option<String>,
    pub as_of: Option<String>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowPage {
    pub items: Vec<Entity>,
    pub next_cursor: Option<String>,
    pub date: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchQuery {
    pub query: String,
    pub kind: Option<EntityKind>,
    #[serde(default)]
    pub archived: bool,
    pub related_id: Option<String>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchHit {
    pub entity_id: String,
    pub title: String,
    pub kind: EntityKind,
    pub revision: u32,
    pub block_id: Option<String>,
    pub excerpt: String,
    pub score: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchPage {
    pub items: Vec<SearchHit>,
    pub next_cursor: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchIndexInfo {
    pub entities: u32,
    pub chunks: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateEntity {
    pub id: String,
    pub expected_revision: u32,
    pub title: Option<String>,
    pub payload: Option<Value>,
    pub document: Option<Document>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IdInput {
    pub id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RevisionInput {
    pub id: String,
    pub expected_revision: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct ListEntities {
    pub kind: Option<EntityKind>,
    pub state: Option<String>,
    pub classification: Option<String>,
    #[serde(default)]
    pub archived: bool,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AddRelation {
    pub source_id: String,
    pub target_id: String,
    pub kind: RelationKind,
    pub expected_revision: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RemoveRelation {
    pub id: String,
    pub expected_revision: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListRelations {
    pub entity_id: String,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct ListChanges {
    pub entity_id: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct CreateWorkspace {
    pub timezone: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BackupInput {
    pub destination: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RestoreInput {
    pub backup: String,
    pub destination: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct Empty {}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImportAttachment {
    pub source_path: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExportDocument {
    pub id: String,
    pub destination: String,
    pub format: ExportFormat,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Json,
    Markdown,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Attachment {
    pub id: String,
    pub name: String,
    pub mime: String,
    pub bytes: u64,
    pub sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AttachmentData {
    pub attachment: Attachment,
    pub base64: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportResult {
    pub path: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "operation", content = "input", deny_unknown_fields)]
pub enum Operation {
    #[serde(rename = "extraction.index")]
    ExtractionIndex(AttachmentIndexInput),
    #[serde(rename = "extraction.search")]
    ExtractionSearch(AttachmentSearchInput),
    #[serde(rename = "extraction.enqueue")]
    ExtractionEnqueue(EnqueueExtraction),
    #[serde(rename = "extraction.get")]
    ExtractionGet(IdInput),
    #[serde(rename = "extraction.list")]
    ExtractionList(ListExtraction),
    #[serde(rename = "extraction.run")]
    ExtractionRun(RunExtraction),
    #[serde(rename = "extraction.cancel")]
    ExtractionCancel(ExtractionRevision),
    #[serde(rename = "extraction.retry")]
    ExtractionRetry(ExtractionRevision),
    #[serde(rename = "extraction.control")]
    ExtractionControl(ExtractionControlInput),
    #[serde(rename = "extraction.fragments")]
    ExtractionFragments(ReadFragments),
    #[serde(rename = "extraction.pdf_fragments")]
    ExtractionPdfFragments(ReadPdfFragments),
    #[serde(rename = "extraction.xlsx_cells")]
    ExtractionXlsxCells(ReadXlsxCells),
    #[serde(rename = "extraction.ocr_regions")]
    ExtractionOcrRegions(ReadOcrRegions),
    #[serde(rename = "changes.preview")]
    ChangesPreview(BatchPlan),
    #[serde(rename = "changes.apply")]
    ChangesApply(ApplyBatch),
    #[serde(rename = "home.query")]
    HomeQuery(HomeQuery),
    #[serde(rename = "template.save")]
    TemplateSave(SaveTemplate),
    #[serde(rename = "template.get")]
    TemplateGet(IdInput),
    #[serde(rename = "template.list")]
    TemplateList(ListTemplates),
    #[serde(rename = "template.instantiate")]
    TemplateInstantiate(InstantiateTemplate),
    #[serde(rename = "formula.set")]
    FormulaSet(FormulaSet),
    #[serde(rename = "collection.create")]
    CollectionCreate(CreateCollection),
    #[serde(rename = "collection.get")]
    CollectionGet(IdInput),
    #[serde(rename = "collection.list")]
    CollectionList(ListCollections),
    #[serde(rename = "property.define")]
    PropertyDefine(DefineProperty),
    #[serde(rename = "record.create")]
    RecordCreate(CreateRecord),
    #[serde(rename = "record.get")]
    RecordGet(RecordInput),
    #[serde(rename = "record.attach")]
    RecordAttach(UpdateRecord),
    #[serde(rename = "record.update")]
    RecordUpdate(UpdateRecord),
    #[serde(rename = "record.inverse")]
    RecordInverse(InverseQuery),
    #[serde(rename = "view.save")]
    ViewSave(SaveView),
    #[serde(rename = "database.query")]
    DatabaseQuery(DatabaseQuery),
    #[serde(rename = "workspace.create")]
    WorkspaceCreate(CreateWorkspace),
    #[serde(rename = "workspace.open")]
    WorkspaceOpen(Empty),
    #[serde(rename = "workspace.backup")]
    WorkspaceBackup(BackupInput),
    #[serde(rename = "workspace.restore")]
    WorkspaceRestore(RestoreInput),
    #[serde(rename = "operations.describe")]
    Describe(Empty),
    #[serde(rename = "entity.create")]
    EntityCreate(CreateEntity),
    #[serde(rename = "entity.get")]
    EntityGet(IdInput),
    #[serde(rename = "entity.list")]
    EntityList(ListEntities),
    #[serde(rename = "entity.update")]
    EntityUpdate(UpdateEntity),
    #[serde(rename = "entity.archive")]
    EntityArchive(RevisionInput),
    #[serde(rename = "entity.restore")]
    EntityRestore(RevisionInput),
    #[serde(rename = "relation.add")]
    RelationAdd(AddRelation),
    #[serde(rename = "relation.remove")]
    RelationRemove(RemoveRelation),
    #[serde(rename = "relation.list")]
    RelationList(ListRelations),
    #[serde(rename = "changes.list")]
    ChangesList(ListChanges),
    #[serde(rename = "attachment.import")]
    AttachmentImport(ImportAttachment),
    #[serde(rename = "attachment.read")]
    AttachmentRead(IdInput),
    #[serde(rename = "document.export")]
    DocumentExport(ExportDocument),
    #[serde(rename = "workflow.query")]
    WorkflowQuery(WorkflowQuery),
    #[serde(rename = "search.query")]
    SearchQuery(SearchQuery),
    #[serde(rename = "search.rebuild")]
    SearchRebuild(Empty),
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub workspace: Option<String>,
    pub command: Operation,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Response {
    pub api_version: u32,
    pub request_id: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<CoreError>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceInfo {
    pub id: String,
    pub path: String,
    pub schema_version: u32,
    pub timezone: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Entity {
    pub id: String,
    pub kind: EntityKind,
    pub title: String,
    pub revision: u32,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub payload: Value,
    pub document: Option<Value>,
    pub document_editable: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Relation {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub kind: RelationKind,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Change {
    pub id: String,
    pub request_id: String,
    pub entity_id: String,
    pub operation: String,
    pub origin: String,
    pub previous_revision: u32,
    pub new_revision: u32,
    pub created_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RelationResult {
    pub relation: Relation,
    pub entity: Entity,
    pub changed: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BackupInfo {
    pub path: String,
    pub workspace_id: String,
    pub schema_version: u32,
    pub sha256: String,
}

// Raíces adicionales: un único esquema genera los DTOs consumidos por TypeScript.
#[derive(JsonSchema)]
pub struct Contract {
    pub batch_plan: BatchPlan,
    pub batch_preview: BatchPreview,
    pub batch_applied: BatchApplied,
    pub home: HomePage,
    pub template: Template,
    pub templates: Page<Template>,
    pub collection_bundle: CollectionBundle,
    pub collections: Page<Collection>,
    pub property: Property,
    pub record: RecordData,
    pub records: Page<RecordData>,
    pub view: CollectionView,
    pub database: DatabasePage,
    pub search: SearchPage,
    pub search_index: SearchIndexInfo,
    pub note_fields: NoteFields,
    pub task_fields: TaskFields,
    pub project_fields: ProjectFields,
    pub tag_fields: TagFields,
    pub workflow: WorkflowPage,
    pub request: Request,
    pub response: Response,
    pub entity: Entity,
    pub workspace: WorkspaceInfo,
    pub relation_result: RelationResult,
    pub entities: Page<Entity>,
    pub relations: Page<Relation>,
    pub changes: Page<Change>,
    pub backup: BackupInfo,
    pub attachment: Attachment,
    pub attachment_data: AttachmentData,
    pub export: ExportResult,
    // Añadir raíces al final conserva los nombres generados de Page<T> anteriores.
    pub extraction_job: ExtractionJob,
    pub extraction_control: ExtractionControl,
    pub extraction_run: ExtractionRun,
    pub enqueued_extraction: EnqueuedExtraction,
    pub read_xlsx_cells: ReadXlsxCells,
    pub xlsx_cells: XlsxCellPage,
    pub read_ocr_regions: ReadOcrRegions,
    pub ocr_regions: OcrRegionPage,
    pub extraction_jobs: Page<ExtractionJob>,
    pub extraction_fragments: FragmentPage,
    pub pdf_fragments: PdfFragmentPage,
    pub read_pdf_fragments: ReadPdfFragments,
    pub attachment_index: AttachmentIndexResult,
    pub attachment_search: AttachmentSearchPage,
}
