/* Generado desde noteka-core. No editar; npm run contracts:generate. */

export type AttachmentIndexResult =
  | {
      action: "get";
      coverage: AttachmentIndexCoverage;
      items: AttachmentIndexEntry[];
      next_cursor?: string | null;
      profile: string;
      revision: number;
      version: number;
    }
  | {
      action: "put";
      changed: boolean;
      entry: AttachmentIndexEntry;
      revision: number;
    }
  | {
      action: "reset";
      removed_fragments: number;
      removed_generations: number;
      revision: number;
    };
export type IndexValidation = "canonical_text_and_postings";
export type IndexCoverageScope = "selected_generations";
export type IndexSourceValidation = "not_checked" | "returned_candidates";
export type AttachmentTextLocator =
  | {
      bbox: OcrBoundingBox;
      confidence: string;
      coordinate_space: string;
      kind: "image_ocr_region_text";
      projection: OcrTextProjection;
      region_bytes: number;
      region_ordinal: number;
      text_byte_end: number;
      text_byte_start: number;
    }
  | {
      byte_end: number;
      byte_start: number;
      kind: "utf8_bytes";
      line_end: number;
      line_start: number;
    }
  | {
      kind: "pdf_page_text";
      limitations: PdfLimitation[];
      page_number: number;
      scope: PdfTextScope;
      text_byte_end: number;
      text_byte_start: number;
    }
  | {
      cell_ordinal: number;
      field: XlsxTextField;
      field_bytes: number;
      kind: "xlsx_cell_text";
      projection: XlsxTextProjection;
      reference: string;
      sheet_id: number;
      text_byte_end: number;
      text_byte_start: number;
    };
export type OcrTextProjection = "ocr_region_text_v1";
export type PdfLimitation = "no_ocr" | "no_visual_order" | "no_geometry" | "no_structured_tables";
export type PdfTextScope = "page_text_objects";
export type XlsxTextField =
  "cached_text" | "cached_number" | "cached_boolean" | "cached_error" | "cached_date_iso" | "formula_text";
export type XlsxTextProjection = "xlsx_cell_fields_v1";
export type FilterExpression =
  | {
      filter: PropertyFilter;
      op: "condition";
    }
  | {
      children: FilterExpression[];
      op: "all";
    }
  | {
      children: FilterExpression[];
      op: "any";
    }
  | {
      child: FilterExpression;
      op: "not";
    };
export type FilterOp = "eq" | "ne" | "lt" | "le" | "gt" | "ge" | "contains" | "is_empty";
export type ViewLayout = "table" | "list" | "board" | "calendar" | "gallery";
export type ScalarKind = "number" | "decimal" | "boolean" | "text";
export type PropertyKind =
  | "formula"
  | "rollup"
  | "title"
  | "text"
  | "number"
  | "decimal"
  | "money"
  | "checkbox"
  | "date"
  | "date_range"
  | "select"
  | "multi_select"
  | "url"
  | "email"
  | "files"
  | "relation"
  | "created_at"
  | "updated_at";
export type EntityKind = "note" | "task" | "project" | "tag" | "person";
export type ExportValue = string | number;
export type BatchOperation =
  | "entity.create"
  | "entity.update"
  | "collection.create"
  | "property.define"
  | "formula.set"
  | "record.create"
  | "record.attach"
  | "record.update"
  | "view.save";
export type BindingKind = "value" | "property_key";
export type BatchExport =
  | "entity_id"
  | "entity_revision"
  | "collection_id"
  | "collection_revision"
  | "title_property_id"
  | "property_id"
  | "view_id"
  | "view_revision";
export type Expression =
  | {
      op: "number";
      value: number;
    }
  | {
      currency?: string | null;
      op: "decimal";
      value: string;
    }
  | {
      op: "boolean";
      value: boolean;
    }
  | {
      op: "text";
      value: string;
    }
  | {
      op: "property";
      property_id: string;
    }
  | {
      left: Expression;
      op: "binary";
      operator: BinaryOp;
      right: Expression;
    }
  | {
      op: "not";
      value: Expression;
    }
  | {
      condition: Expression;
      op: "if";
      otherwise: Expression;
      then: Expression;
    }
  | {
      aggregate: Aggregate;
      inverse?: boolean;
      op: "rollup";
      property_id?: string | null;
      relation_id: string;
    }
  | {
      cost_id: string;
      op: "allocate";
      relation_id: string;
      scale: number;
      weight_id: string;
    };
export type BinaryOp = "add" | "sub" | "mul" | "div" | "eq" | "lt" | "and" | "or";
export type Aggregate = "count" | "sum" | "avg" | "min" | "max";
export type ErrorCode =
  | "validation_error"
  | "not_found"
  | "revision_conflict"
  | "workspace_busy"
  | "storage_error"
  | "unsupported_workspace_version"
  | "unsupported_document_format"
  | "unsupported_template_format"
  | "invalid_backup"
  | "destination_not_empty";
export type ExtractionState = "queued" | "running" | "succeeded" | "failed" | "cancelled" | "stale";
export type ExtractionOutcome = "idle" | "succeeded" | "interrupted";
export type Classification = "area" | "resource" | "topic";
export type ProjectState = "planned" | "doing" | "done" | "ongoing" | "on_hold";
export type HomeSummary =
  | {
      available: boolean;
      done?: number | null;
      kind: "project";
      overdue?: number | null;
      pending?: number | null;
      progress_percent?: number | null;
      total?: number | null;
    }
  | {
      available: boolean;
      kind: "tag";
      last_note?: EntityRef | null;
      note_count?: number | null;
    };
export type HomeView =
  | "tasks_today"
  | "tasks_inbox"
  | "tasks_all"
  | "tasks_done"
  | "notes_all"
  | "notes_inbox"
  | "projects_active"
  | "projects_planned"
  | "projects_board"
  | "tags_recent"
  | "tags_alphabetical"
  | "tags_types"
  | "people_contacts"
  | "people_new"
  | "people_updated";
export type OcrImageFormat = "png" | "jpeg";
export type PdfLocatorKind = "pdf_page_text";
export type PdfPageState = "text" | "no_text";
export type RelationKind = "project" | "tag" | "person";
export type Operation =
  | {
      input: AttachmentIndexInput;
      operation: "extraction.index";
    }
  | {
      input: AttachmentSearchInput;
      operation: "extraction.search";
    }
  | {
      input: EnqueueExtraction;
      operation: "extraction.enqueue";
    }
  | {
      input: IdInput;
      operation: "extraction.get";
    }
  | {
      input: ListExtraction;
      operation: "extraction.list";
    }
  | {
      input: RunExtraction;
      operation: "extraction.run";
    }
  | {
      input: ExtractionRevision;
      operation: "extraction.cancel";
    }
  | {
      input: ExtractionRevision;
      operation: "extraction.retry";
    }
  | {
      input: ExtractionControlInput;
      operation: "extraction.control";
    }
  | {
      input: ReadFragments;
      operation: "extraction.fragments";
    }
  | {
      input: ReadPdfFragments;
      operation: "extraction.pdf_fragments";
    }
  | {
      input: ReadXlsxCells;
      operation: "extraction.xlsx_cells";
    }
  | {
      input: ReadOcrRegions;
      operation: "extraction.ocr_regions";
    }
  | {
      input: BatchPlan;
      operation: "changes.preview";
    }
  | {
      input: ApplyBatch;
      operation: "changes.apply";
    }
  | {
      input: HomeQuery;
      operation: "home.query";
    }
  | {
      input: SaveTemplate;
      operation: "template.save";
    }
  | {
      input: IdInput;
      operation: "template.get";
    }
  | {
      input: ListTemplates;
      operation: "template.list";
    }
  | {
      input: InstantiateTemplate;
      operation: "template.instantiate";
    }
  | {
      input: FormulaSet;
      operation: "formula.set";
    }
  | {
      input: CreateCollection;
      operation: "collection.create";
    }
  | {
      input: IdInput;
      operation: "collection.get";
    }
  | {
      input: ListCollections;
      operation: "collection.list";
    }
  | {
      input: DefineProperty;
      operation: "property.define";
    }
  | {
      input: CreateRecord;
      operation: "record.create";
    }
  | {
      input: RecordInput;
      operation: "record.get";
    }
  | {
      input: UpdateRecord;
      operation: "record.attach";
    }
  | {
      input: UpdateRecord;
      operation: "record.update";
    }
  | {
      input: InverseQuery;
      operation: "record.inverse";
    }
  | {
      input: SaveView;
      operation: "view.save";
    }
  | {
      input: DatabaseQuery;
      operation: "database.query";
    }
  | {
      input: CreateWorkspace;
      operation: "workspace.create";
    }
  | {
      input: Empty;
      operation: "workspace.open";
    }
  | {
      input: BackupInput;
      operation: "workspace.backup";
    }
  | {
      input: RestoreInput;
      operation: "workspace.restore";
    }
  | {
      input: Empty;
      operation: "operations.describe";
    }
  | {
      input: CreateEntity;
      operation: "entity.create";
    }
  | {
      input: IdInput;
      operation: "entity.get";
    }
  | {
      input: ListEntities;
      operation: "entity.list";
    }
  | {
      input: UpdateEntity;
      operation: "entity.update";
    }
  | {
      input: RevisionInput;
      operation: "entity.archive";
    }
  | {
      input: RevisionInput;
      operation: "entity.restore";
    }
  | {
      input: AddRelation;
      operation: "relation.add";
    }
  | {
      input: RemoveRelation;
      operation: "relation.remove";
    }
  | {
      input: ListRelations;
      operation: "relation.list";
    }
  | {
      input: ListChanges;
      operation: "changes.list";
    }
  | {
      input: ImportAttachment;
      operation: "attachment.import";
    }
  | {
      input: IdInput;
      operation: "attachment.read";
    }
  | {
      input: ExportDocument;
      operation: "document.export";
    }
  | {
      input: WorkflowQuery;
      operation: "workflow.query";
    }
  | {
      input: SearchQuery;
      operation: "search.query";
    }
  | {
      input: Empty;
      operation: "search.rebuild";
    };
export type AttachmentIndexInput =
  | {
      action: "get";
      attachment_id?: string | null;
      cursor?: string | null;
      extractor?: IndexedExtractor | null;
      job_id?: string | null;
      limit?: number | null;
    }
  | {
      action: "put";
      expected_generation_id: string;
      expected_revision: number;
      job_id: string;
    }
  | {
      action: "reset";
      expected_revision: number;
    };
export type IndexedExtractor = "utf8_text" | "pdf_text" | "xlsx_cells" | "image_ocr";
export type ExtractionControlInput =
  | {
      action: "get";
    }
  | {
      action: "set";
      expected_revision: number;
      paused: boolean;
    };
export type MarkKind = "bold" | "italic" | "strike" | "code" | "link";
export type TextKind = "text" | "hard_break";
export type BlockKind =
  | "paragraph"
  | "heading"
  | "bullet_list"
  | "ordered_list"
  | "list_item"
  | "task_list"
  | "task_item"
  | "blockquote"
  | "horizontal_rule"
  | "code_block"
  | "image"
  | "file"
  | "collection_view";
export type ExportFormat = "json" | "markdown";
export type WorkflowView = "inbox" | "my_day" | "pending" | "overdue" | "active_projects" | "context";
export type CellValue =
  | {
      kind: "number";
      value: string;
    }
  | {
      kind: "text";
      value: string;
    }
  | {
      kind: "boolean";
      value: boolean;
    }
  | {
      kind: "error";
      value: string;
    }
  | {
      kind: "date_iso";
      value: string;
    };
export type FormulaKind = "normal" | "shared" | "array";
export type XlsxLocatorKind = "xlsx_cell";
export type Visibility = "visible" | "hidden" | "very_hidden";

export interface Contract {
  attachment: Attachment;
  attachment_data: AttachmentData;
  attachment_index: AttachmentIndexResult;
  attachment_search: AttachmentSearchPage;
  backup: BackupInfo;
  batch_applied: BatchApplied;
  batch_plan: BatchPlan;
  batch_preview: BatchPreview;
  changes: Page6;
  collection_bundle: CollectionBundle;
  collections: Page3;
  database: DatabasePage;
  enqueued_extraction: EnqueuedExtraction;
  entities: Page4;
  entity: Entity;
  export: ExportResult;
  extraction_control: ExtractionControl;
  extraction_fragments: FragmentPage;
  extraction_job: ExtractionJob;
  extraction_jobs: Page7;
  extraction_run: ExtractionRun;
  home: HomePage;
  note_fields: NoteFields;
  ocr_regions: OcrRegionPage;
  pdf_fragments: PdfFragmentPage;
  project_fields: ProjectFields;
  property: Property;
  read_ocr_regions: ReadOcrRegions;
  read_pdf_fragments: ReadPdfFragments;
  read_xlsx_cells: ReadXlsxCells;
  record: RecordData;
  records: Page;
  relation_result: RelationResult;
  relations: Page5;
  request: Request;
  response: Response;
  search: SearchPage;
  search_index: SearchIndexInfo;
  tag_fields: TagFields;
  task_fields: TaskFields;
  template: Template;
  templates: Page2;
  view: CollectionView;
  workflow: WorkflowPage;
  workspace: WorkspaceInfo;
  xlsx_cells: XlsxCellPage;
}
export interface Attachment {
  bytes: number;
  id: string;
  mime: string;
  name: string;
  sha256: string;
}
export interface AttachmentData {
  attachment: Attachment;
  base64: string;
}
export interface AttachmentIndexCoverage {
  fragments: number;
  index_validation: IndexValidation;
  indexed_generations: number;
  jobs_without_generation: number;
  scope: IndexCoverageScope;
  source_validation: IndexSourceValidation;
  text_bytes: number;
  unindexed_supported_generations: number;
  unsupported_generations: number;
}
export interface AttachmentIndexEntry {
  content_hash: string;
  fragment_count: number;
  indexed_at: string;
  result_from_previous_attempt: boolean;
  source: IndexedSource;
  text_bytes: number;
}
export interface IndexedSource {
  attachment_id: string;
  config_hash: string;
  extractor: string;
  extractor_version: number;
  generation_id: string;
  job_id: string;
  source_sha256: string;
}
export interface AttachmentSearchPage {
  coverage: AttachmentIndexCoverage;
  items: AttachmentSearchHit[];
  next_cursor?: string | null;
  profile: string;
  revision: number;
  version: number;
}
export interface AttachmentSearchHit {
  locator: AttachmentTextLocator;
  ordinal: number;
  result_from_previous_attempt: boolean;
  score: number;
  source: IndexedSource;
  text: string;
}
export interface OcrBoundingBox {
  height: string;
  width: string;
  x: string;
  y: string;
}
export interface BackupInfo {
  path: string;
  schema_version: number;
  sha256: string;
  workspace_id: string;
}
export interface BatchApplied {
  applied: boolean;
  batch_id: string;
  checks: BatchCheckResult[];
  diff: BatchDiff[];
  executed_at: string;
  history_summary: HistorySummary[];
  steps: BatchStepResult[];
}
export interface BatchCheckResult {
  data: DatabasePage;
  key: string;
}
export interface DatabasePage {
  config: ViewConfig;
  groups: ViewGroup[];
  layout: ViewLayout;
  matching_count: number;
  properties: Property[];
  records: Page;
}
export interface ViewConfig {
  archived?: boolean;
  calendar?: CalendarConfig | null;
  columns?: string[];
  filter_tree?: FilterExpression | null;
  filters?: PropertyFilter[];
  group_by?: string | null;
  sorts?: PropertySort[];
}
export interface CalendarConfig {
  month: string;
  property_id: string;
}
export interface PropertyFilter {
  op: FilterOp;
  property_id: string;
  value?: {
    [k: string]: unknown;
  };
}
export interface PropertySort {
  descending?: boolean;
  property_id: string;
}
export interface ViewGroup {
  entity_ids: string[];
  key: unknown;
  label: string;
}
export interface Property {
  collection_id: string;
  id: string;
  spec: PropertySpec;
}
export interface PropertySpec {
  calculation_type?: CalculationType | null;
  currency?: string | null;
  name: string;
  options?: string[];
  target_collection_id?: string | null;
  type: PropertyKind;
}
export interface CalculationType {
  currency?: string | null;
  kind: ScalarKind;
}
export interface Page {
  items: RecordData[];
  next_cursor?: string | null;
}
export interface RecordData {
  calculation_errors: {
    [k: string]: CalculationError;
  };
  collection_ids: string[];
  entity: Entity;
  values: {
    [k: string]: unknown;
  };
}
export interface CalculationError {
  code: string;
  message: string;
}
export interface Entity {
  archived_at?: string | null;
  created_at: string;
  document?: unknown;
  document_editable: boolean;
  id: string;
  kind: EntityKind;
  payload: unknown;
  revision: number;
  title: string;
  updated_at: string;
}
export interface BatchDiff {
  after?: unknown;
  before?: unknown;
  key: string[];
  kind: string;
}
export interface HistorySummary {
  entity_id: string;
  new_revision: number;
  operation: string;
  previous_revision: number;
}
export interface BatchStepResult {
  data: unknown;
  exports: {
    collection_id?: ExportValue;
    collection_revision?: ExportValue;
    entity_id?: ExportValue;
    entity_revision?: ExportValue;
    property_id?: ExportValue;
    title_property_id?: ExportValue;
    view_id?: ExportValue;
    view_revision?: ExportValue;
  };
  key: string;
  operation: BatchOperation;
}
export interface BatchPlan {
  batch_id: string;
  checks?: BatchCheck[];
  steps: BatchStep[];
  version: number;
}
export interface BatchCheck {
  bindings?: Binding[];
  input: unknown;
  key: string;
  require_complete?: boolean;
  require_no_calculation_errors?: boolean;
}
export interface Binding {
  kind: BindingKind;
  source: ExportSource;
  target: string;
}
export interface ExportSource {
  export: BatchExport;
  step: string;
}
export interface BatchStep {
  bindings?: Binding[];
  input: unknown;
  key: string;
  operation: BatchOperation;
}
export interface BatchPreview {
  applied: boolean;
  batch_id: string;
  checks: BatchCheckResult[];
  diff: BatchDiff[];
  evaluated_at: string;
  history_summary: HistorySummary[];
  receipt: BatchReceipt;
  steps: BatchStepResult[];
}
export interface BatchReceipt {
  base_hash: string;
  effect_hash: string;
  path_hash: string;
  plan_hash: string;
  version: number;
  workspace_id: string;
}
export interface Page6 {
  items: Change[];
  next_cursor?: string | null;
}
export interface Change {
  created_at: string;
  entity_id: string;
  id: string;
  new_revision: number;
  operation: string;
  origin: string;
  previous_revision: number;
  request_id: string;
}
export interface CollectionBundle {
  calculations: {
    [k: string]: Expression;
  };
  collection: Collection;
  properties: Property[];
  views: CollectionView[];
}
export interface Collection {
  id: string;
  name: string;
  revision: number;
}
export interface CollectionView {
  collection_id: string;
  config: ViewConfig;
  id: string;
  layout: ViewLayout;
  name: string;
  revision: number;
}
export interface Page3 {
  items: Collection[];
  next_cursor?: string | null;
}
export interface EnqueuedExtraction {
  deduplicated: boolean;
  job: ExtractionJob;
}
export interface ExtractionJob {
  attachment_id: string;
  attempts: number;
  /**
   * Crudo: una configuración futura no se reserializa en persistencia al leer.
   */
  config: {
    [k: string]: unknown;
  };
  config_hash: string;
  created_at: string;
  error?: CoreError | null;
  extractor: string;
  extractor_version: number;
  finished_at?: string | null;
  generation?: ExtractionGeneration | null;
  id: string;
  ownership_active: boolean;
  recoverable: boolean;
  result_from_previous_attempt: boolean;
  revision: number;
  source_bytes: number;
  source_sha256: string;
  started_at?: string | null;
  state: ExtractionState;
  supported: boolean;
  unsupported_reason?: string | null;
  updated_at: string;
}
export interface CoreError {
  code: ErrorCode;
  details: unknown;
  message: string;
}
export interface ExtractionGeneration {
  attachment_id: string;
  attempt: number;
  complete: boolean;
  config: unknown;
  config_hash: string;
  covered_bytes: number;
  extractor: string;
  extractor_version: number;
  fragment_count: number;
  id: string;
  published_at: string;
  source_bytes: number;
  source_sha256: string;
}
export interface Page4 {
  items: Entity[];
  next_cursor?: string | null;
}
export interface ExportResult {
  path: string;
  warnings: string[];
}
export interface ExtractionControl {
  paused: boolean;
  revision: number;
  updated_at: string;
  version: number;
}
export interface FragmentPage {
  covers_generation: boolean;
  generation: ExtractionGeneration;
  items: ExtractionFragment[];
  job_id: string;
  next_cursor?: string | null;
  result_from_previous_attempt: boolean;
}
export interface ExtractionFragment {
  byte_end: number;
  byte_start: number;
  line_end: number;
  line_start: number;
  ordinal: number;
  text: string;
}
export interface Page7 {
  items: ExtractionJob[];
  next_cursor?: string | null;
}
export interface ExtractionRun {
  incompatible_count: number;
  job?: ExtractionJob | null;
  outcome: ExtractionOutcome;
  reason?: string | null;
  recovered_count: number;
}
export interface HomePage {
  date: string;
  groups: HomeGroup[];
  items: HomeItem[];
  matching_count: number;
  next_cursor?: string | null;
  offset: number;
  timezone: string;
  unavailable_count: number;
  view: HomeView;
}
export interface HomeGroup {
  classification?: Classification | null;
  entity_ids: string[];
  key: string;
  label: string;
  project_archived: boolean;
  project_id?: string | null;
  project_state?: ProjectState | null;
}
export interface HomeItem {
  context: EntityRef[];
  context_total: number;
  entity: EntityRef;
  metadata: HomeMetadata;
  summary?: HomeSummary | null;
}
export interface EntityRef {
  archived_at?: string | null;
  created_at: string;
  id: string;
  kind: EntityKind;
  revision: number;
  title: string;
  updated_at: string;
}
export interface HomeMetadata {
  classification?: string | null;
  due_date?: string | null;
  inbox?: boolean | null;
  issues: string[];
  planned_date?: string | null;
  priority?: string | null;
  state?: string | null;
  updated_days_ago?: number | null;
}
export interface NoteFields {
  inbox?: boolean;
}
export interface OcrRegionPage {
  covers_generation: boolean;
  generation: ExtractionGeneration;
  items: OcrRegion[];
  job_id: string;
  manifest: OcrManifest;
  next_cursor?: string | null;
  result_from_previous_attempt: boolean;
}
export interface OcrRegion {
  bbox: OcrBoundingBox;
  confidence: string;
  ordinal: number;
  text: string;
}
export interface OcrManifest {
  coordinate_space: string;
  engine: OcrEngine;
  image: OcrImage;
  limitations: string[];
  profile: string;
  region_count: number;
  representation_bytes: number;
  representation_sha256: string;
  version: number;
}
export interface OcrEngine {
  automatic_language_detection: boolean;
  language_correction: boolean;
  languages: string[];
  os_build: string;
  os_version: string;
  recognition_level: string;
  revision: number;
}
export interface OcrImage {
  format: OcrImageFormat;
  height: number;
  orientation: number;
  oriented_height: number;
  oriented_width: number;
  width: number;
}
export interface PdfFragmentPage {
  covers_generation: boolean;
  generation: ExtractionGeneration;
  items: PdfFragment[];
  job_id: string;
  manifest: PdfManifest;
  next_cursor?: string | null;
  result_from_previous_attempt: boolean;
}
export interface PdfFragment {
  locator: PdfTextLocator;
  ordinal: number;
  text: string;
}
export interface PdfTextLocator {
  kind: PdfLocatorKind;
  page_number: number;
  text_byte_end: number;
  text_byte_start: number;
}
export interface PdfManifest {
  extracted_bytes: number;
  limitations: PdfLimitation[];
  page_count: number;
  /**
   * @minItems 1
   * @maxItems 100
   */
  pages: [PdfPageDescriptor, ...PdfPageDescriptor[]];
  parser_name: string;
  parser_version: string;
  scope: PdfTextScope;
  version: number;
}
export interface PdfPageDescriptor {
  fragment_count: number;
  page_number: number;
  state: PdfPageState;
  text_bytes: number;
  text_sha256: string;
}
export interface ProjectFields {
  end_date?: string | null;
  purpose?: string;
  start_date?: string | null;
  state?: "planned" | "doing" | "done" | "ongoing" | "on_hold";
}
export interface ReadOcrRegions {
  cursor?: string | null;
  id: string;
  limit?: number | null;
}
export interface ReadPdfFragments {
  cursor?: string | null;
  id: string;
  limit?: number | null;
}
export interface ReadXlsxCells {
  cursor?: string | null;
  id: string;
  limit?: number | null;
}
export interface RelationResult {
  changed: boolean;
  entity: Entity;
  relation: Relation;
}
export interface Relation {
  id: string;
  kind: RelationKind;
  source_id: string;
  target_id: string;
}
export interface Page5 {
  items: Relation[];
  next_cursor?: string | null;
}
export interface Request {
  command: Operation;
  workspace?: string | null;
}
export interface AttachmentSearchInput {
  attachment_id?: string | null;
  cursor?: string | null;
  extractor?: IndexedExtractor | null;
  job_id?: string | null;
  limit?: number | null;
  query: string;
}
export interface EnqueueExtraction {
  attachment_id: string;
  config?: ExtractionConfig | null;
  extractor: string;
}
export interface ExtractionConfig {
  chunk_bytes: number;
  version: number;
}
export interface IdInput {
  id: string;
}
export interface ListExtraction {
  attachment_id?: string | null;
  cursor?: string | null;
  limit?: number | null;
  state?: ExtractionState | null;
}
export interface RunExtraction {
  id?: string | null;
}
export interface ExtractionRevision {
  expected_revision: number;
  id: string;
}
export interface ReadFragments {
  cursor?: string | null;
  id: string;
  limit?: number | null;
}
export interface ApplyBatch {
  expected: BatchReceipt;
  plan: BatchPlan;
}
export interface HomeQuery {
  as_of?: string | null;
  cursor?: string | null;
  limit?: number | null;
  view: HomeView;
}
export interface SaveTemplate {
  collection_id: string;
  definition: TemplateDefinition;
  expected_collection_revision: number;
  expected_revision?: number | null;
  id?: string | null;
  name: string;
}
export interface TemplateDefinition {
  document: Document;
  schema_version: number;
  title: string;
  values?: {
    [k: string]: unknown;
  };
}
export interface Document {
  blocks: Block[];
  schema_version: number;
}
export interface Block {
  attrs?: unknown;
  children?: Block[];
  content: Text[];
  id: string;
  type: BlockKind;
}
export interface Text {
  marks?: Mark[];
  text: string;
  type: TextKind;
}
export interface Mark {
  href?: string | null;
  type: MarkKind;
}
export interface ListTemplates {
  collection_id: string;
  cursor?: string | null;
  limit?: number | null;
}
export interface InstantiateTemplate {
  expected_collection_revision: number;
  expected_template_revision: number;
  id: string;
  title?: string;
  values?: {
    [k: string]: unknown;
  };
}
export interface FormulaSet {
  collection_id: string;
  expected_revision: number;
  expression: Expression;
  name: string;
  property_id?: string | null;
}
export interface CreateCollection {
  name: string;
}
export interface ListCollections {
  cursor?: string | null;
  limit?: number | null;
}
export interface DefineProperty {
  collection_id: string;
  expected_revision: number;
  spec: PropertySpec;
}
export interface CreateRecord {
  collection_id: string;
  document?: Document | null;
  title: string;
  values?: {
    [k: string]: unknown;
  };
}
export interface RecordInput {
  collection_id: string;
  entity_id: string;
}
export interface UpdateRecord {
  collection_id: string;
  entity_id: string;
  expected_revision: number;
  values?: {
    [k: string]: unknown;
  };
}
export interface InverseQuery {
  cursor?: string | null;
  limit?: number | null;
  property_id: string;
  target_id: string;
}
export interface SaveView {
  collection_id: string;
  config: ViewConfig;
  expected_revision?: number | null;
  id?: string | null;
  layout: ViewLayout;
  name: string;
}
export interface DatabaseQuery {
  collection_id: string;
  config?: ViewConfig | null;
  cursor?: string | null;
  layout?: ViewLayout | null;
  limit?: number | null;
  view_id?: string | null;
}
export interface CreateWorkspace {
  timezone?: string | null;
}
export interface Empty {}
export interface BackupInput {
  destination: string;
}
export interface RestoreInput {
  backup: string;
  destination: string;
}
export interface CreateEntity {
  context?: ContextLink | null;
  document?: Document | null;
  kind: EntityKind;
  payload?: {
    [k: string]: unknown;
  };
  record?: RecordSeed | null;
  title: string;
}
export interface ContextLink {
  kind: RelationKind;
  target_id: string;
}
export interface RecordSeed {
  collection_id: string;
  values?: {
    [k: string]: unknown;
  };
}
export interface ListEntities {
  archived?: boolean;
  classification?: string | null;
  cursor?: string | null;
  kind?: EntityKind | null;
  limit?: number | null;
  state?: string | null;
}
export interface UpdateEntity {
  document?: Document | null;
  expected_revision: number;
  id: string;
  payload?: unknown;
  title?: string | null;
}
export interface RevisionInput {
  expected_revision: number;
  id: string;
}
export interface AddRelation {
  expected_revision: number;
  kind: RelationKind;
  source_id: string;
  target_id: string;
}
export interface RemoveRelation {
  expected_revision: number;
  id: string;
}
export interface ListRelations {
  cursor?: string | null;
  entity_id: string;
  limit?: number | null;
}
export interface ListChanges {
  cursor?: string | null;
  entity_id?: string | null;
  limit?: number | null;
}
export interface ImportAttachment {
  source_path: string;
}
export interface ExportDocument {
  destination: string;
  format: ExportFormat;
  id: string;
}
export interface WorkflowQuery {
  as_of?: string | null;
  cursor?: string | null;
  limit?: number | null;
  related_id?: string | null;
  view: WorkflowView;
}
export interface SearchQuery {
  archived?: boolean;
  cursor?: string | null;
  kind?: EntityKind | null;
  limit?: number | null;
  query: string;
  related_id?: string | null;
}
export interface Response {
  api_version: number;
  data?: unknown;
  error?: CoreError | null;
  ok: boolean;
  request_id: string;
}
export interface SearchPage {
  items: SearchHit[];
  next_cursor?: string | null;
}
export interface SearchHit {
  block_id?: string | null;
  entity_id: string;
  excerpt: string;
  kind: EntityKind;
  revision: number;
  score: number;
  title: string;
}
export interface SearchIndexInfo {
  chunks: number;
  entities: number;
}
export interface TagFields {
  classification: Classification;
}
export interface TaskFields {
  due_date?: string | null;
  inbox?: boolean;
  planned_date?: string | null;
  priority?: "low" | "normal" | "high";
  state?: "todo" | "done";
}
export interface Template {
  collection_id: string;
  definition: unknown;
  editable: boolean;
  id: string;
  name: string;
  revision: number;
}
export interface Page2 {
  items: Template[];
  next_cursor?: string | null;
}
export interface WorkflowPage {
  date: string;
  items: Entity[];
  next_cursor?: string | null;
}
export interface WorkspaceInfo {
  id: string;
  path: string;
  schema_version: number;
  timezone: string;
}
export interface XlsxCellPage {
  covers_generation: boolean;
  generation: ExtractionGeneration;
  items: XlsxCellItem[];
  job_id: string;
  manifest: XlsxManifest;
  next_cursor?: string | null;
  result_from_previous_attempt: boolean;
}
export interface XlsxCellItem {
  cell: Cell;
  locator: XlsxCellLocator;
  ordinal: number;
}
export interface Cell {
  cached?: CellValue | null;
  column: number;
  formula?: Formula | null;
  reference: string;
  row: number;
  style: number;
}
export interface Formula {
  kind: FormulaKind;
  reference?: string | null;
  shared_index?: number | null;
  text: string;
}
export interface XlsxCellLocator {
  kind: XlsxLocatorKind;
  reference: string;
  sheet_id: number;
}
export interface XlsxManifest {
  cell_count: number;
  date_1904: boolean;
  limitations: string[];
  parser: string;
  representation_bytes: number;
  representation_sha256: string;
  /**
   * @minItems 1
   * @maxItems 32
   */
  sheets: [XlsxSheetDescriptor, ...XlsxSheetDescriptor[]];
  styles: NumberStyle[];
  version: number;
}
export interface XlsxSheetDescriptor {
  cell_count: number;
  declared_range?: string | null;
  id: number;
  merges: string[];
  name: string;
  observed_range?: string | null;
  visibility: Visibility;
}
export interface NumberStyle {
  format_code?: string | null;
  num_fmt_id: number;
}
