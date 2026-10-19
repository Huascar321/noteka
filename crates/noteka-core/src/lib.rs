mod attachment_postings;
mod attachment_search;
mod attachment_search_contracts;
mod attachment_search_storage;
#[cfg(test)]
mod attachment_search_tests;
mod attachments;
mod batch_bindings;
mod batch_contracts;
mod batch_plan;
mod batch_storage;
#[cfg(test)]
mod batch_tests;
mod batches;
mod collection_views;
pub mod contracts;
mod database;
mod database_contracts;
mod database_filters;
mod documents;
mod entities;
pub mod error;
mod extraction;
mod extraction_contracts;
mod extraction_storage;
#[cfg(test)]
mod extraction_tests;
mod extraction_worker;
mod formula_contracts;
mod formulas;
mod home;
mod home_contracts;
mod mutations;
mod ocr_contracts;
mod ocr_index_projection;
mod ocr_storage;
mod ocr_supervisor;
#[cfg(test)]
mod ocr_tests;
mod pdf_contracts;
mod pdf_storage;
mod pdf_supervisor;
#[cfg(test)]
mod pdf_tests;
mod productivity;
pub mod scoped;
#[cfg(test)]
mod scoped_tests;
mod search;
mod template_contracts;
mod templates;
mod text_extractor;
pub mod workspace;
mod xlsx_contracts;
mod xlsx_index_projection;
#[cfg(test)]
mod xlsx_search_tests;
mod xlsx_storage;
mod xlsx_supervisor;
#[cfg(test)]
mod xlsx_tests;

use contracts::*;
use error::*;
use serde_json::{Value, json};
use std::path::PathBuf;
use workspace::Workspace;

pub fn contract_schema() -> Value {
    serde_json::to_value(schemars::schema_for!(Contract)).expect("Esquema serializable")
}
pub fn describe() -> Value {
    let mut inputs = json!({
        "changes.preview":schemars::schema_for!(BatchPlan),"changes.apply":schemars::schema_for!(ApplyBatch),
        "home.query":schemars::schema_for!(HomeQuery),
        "template.save":schemars::schema_for!(SaveTemplate),"template.get":schemars::schema_for!(IdInput),"template.list":schemars::schema_for!(ListTemplates),"template.instantiate":schemars::schema_for!(InstantiateTemplate),
        "workspace.create":schemars::schema_for!(CreateWorkspace),"workspace.open":schemars::schema_for!(Empty),
        "workspace.backup":schemars::schema_for!(BackupInput),"workspace.restore":schemars::schema_for!(RestoreInput),
        "operations.describe":schemars::schema_for!(Empty),"entity.create":schemars::schema_for!(CreateEntity),
        "entity.get":schemars::schema_for!(IdInput),"entity.list":schemars::schema_for!(ListEntities),
        "entity.update":schemars::schema_for!(UpdateEntity),"entity.archive":schemars::schema_for!(RevisionInput),
        "entity.restore":schemars::schema_for!(RevisionInput),"relation.add":schemars::schema_for!(AddRelation),
        "relation.remove":schemars::schema_for!(RemoveRelation),"relation.list":schemars::schema_for!(ListRelations),
        "changes.list":schemars::schema_for!(ListChanges),
        "formula.set":schemars::schema_for!(FormulaSet),"collection.create":schemars::schema_for!(CreateCollection),"collection.get":schemars::schema_for!(IdInput),"collection.list":schemars::schema_for!(ListCollections),"property.define":schemars::schema_for!(DefineProperty),
        "record.create":schemars::schema_for!(CreateRecord),"record.get":schemars::schema_for!(RecordInput),"record.attach":schemars::schema_for!(UpdateRecord),"record.update":schemars::schema_for!(UpdateRecord),"record.inverse":schemars::schema_for!(InverseQuery),"view.save":schemars::schema_for!(SaveView),"database.query":schemars::schema_for!(DatabaseQuery),
        "attachment.import":schemars::schema_for!(ImportAttachment),"attachment.read":schemars::schema_for!(IdInput),"document.export":schemars::schema_for!(ExportDocument),"workflow.query":schemars::schema_for!(WorkflowQuery),"search.query":schemars::schema_for!(SearchQuery),"search.rebuild":schemars::schema_for!(Empty)
    });
    inputs.as_object_mut().unwrap().extend(json!({
        "extraction.enqueue":schemars::schema_for!(EnqueueExtraction),"extraction.get":schemars::schema_for!(IdInput),"extraction.list":schemars::schema_for!(ListExtraction),"extraction.run":schemars::schema_for!(RunExtraction),"extraction.cancel":schemars::schema_for!(ExtractionRevision),"extraction.retry":schemars::schema_for!(ExtractionRevision),"extraction.control":schemars::schema_for!(ExtractionControlInput),"extraction.fragments":schemars::schema_for!(ReadFragments)
    }).as_object().unwrap().clone());
    inputs["extraction.pdf_fragments"] = json!(schemars::schema_for!(ReadPdfFragments));
    inputs["extraction.ocr_regions"] = json!(schemars::schema_for!(ReadOcrRegions));
    inputs["extraction.xlsx_cells"] = json!(schemars::schema_for!(ReadXlsxCells));
    inputs["extraction.index"] = json!(schemars::schema_for!(AttachmentIndexInput));
    inputs["extraction.search"] = json!(schemars::schema_for!(AttachmentSearchInput));
    let workspace = schemars::schema_for!(WorkspaceInfo);
    let entity = schemars::schema_for!(Entity);
    let relation = schemars::schema_for!(RelationResult);
    let mut outputs = json!({});
    for op in inputs.as_object().unwrap().keys() {
        outputs[op] = match op.as_str() {
            "extraction.enqueue" => json!(schemars::schema_for!(EnqueuedExtraction)),
            "extraction.get" | "extraction.cancel" | "extraction.retry" => {
                json!(schemars::schema_for!(ExtractionJob))
            }
            "extraction.list" => json!(schemars::schema_for!(Page<ExtractionJob>)),
            "extraction.run" => json!(schemars::schema_for!(ExtractionRun)),
            "extraction.control" => json!(schemars::schema_for!(ExtractionControl)),
            "extraction.fragments" => json!(schemars::schema_for!(FragmentPage)),
            "extraction.pdf_fragments" => json!(schemars::schema_for!(PdfFragmentPage)),
            "extraction.ocr_regions" => json!(schemars::schema_for!(OcrRegionPage)),
            "extraction.xlsx_cells" => json!(schemars::schema_for!(XlsxCellPage)),
            "extraction.index" => json!(schemars::schema_for!(AttachmentIndexResult)),
            "extraction.search" => json!(schemars::schema_for!(AttachmentSearchPage)),
            "changes.preview" => json!(schemars::schema_for!(BatchPreview)),
            "changes.apply" => json!(schemars::schema_for!(BatchApplied)),
            "home.query" => json!(schemars::schema_for!(HomePage)),
            "template.save" | "template.get" => json!(schemars::schema_for!(Template)),
            "template.list" => json!(schemars::schema_for!(Page<Template>)),
            "template.instantiate" => json!(schemars::schema_for!(RecordData)),
            "formula.set" | "collection.create" | "collection.get" => {
                json!(schemars::schema_for!(CollectionBundle))
            }
            "collection.list" => json!(schemars::schema_for!(Page<Collection>)),
            "property.define" => json!(schemars::schema_for!(Property)),
            "record.create" | "record.get" | "record.attach" | "record.update" => {
                json!(schemars::schema_for!(RecordData))
            }
            "record.inverse" => json!(schemars::schema_for!(Page<RecordData>)),
            "view.save" => json!(schemars::schema_for!(CollectionView)),
            "database.query" => json!(schemars::schema_for!(DatabasePage)),
            "workspace.create" | "workspace.open" | "workspace.restore" => json!(workspace),
            "workspace.backup" => json!(schemars::schema_for!(BackupInfo)),
            "entity.list" => json!(schemars::schema_for!(Page<Entity>)),
            "relation.list" => json!(schemars::schema_for!(Page<Relation>)),
            "changes.list" => json!(schemars::schema_for!(Page<Change>)),
            "attachment.import" => json!(schemars::schema_for!(Attachment)),
            "attachment.read" => json!(schemars::schema_for!(AttachmentData)),
            "document.export" => json!(schemars::schema_for!(ExportResult)),
            "workflow.query" => json!(schemars::schema_for!(WorkflowPage)),
            "search.query" => json!(schemars::schema_for!(SearchPage)),
            "search.rebuild" => json!(schemars::schema_for!(SearchIndexInfo)),
            "relation.add" | "relation.remove" => json!(relation),
            "operations.describe" => json!({"type":"object"}),
            _ => json!(entity),
        };
    }
    let mutations = batches::catalog(&inputs);
    let compatibility=inputs.as_object().unwrap().keys().map(|op|(op.clone(),json!({"batch_mutation":mutations.get(op).is_some(),"batch_check":op=="database.query"}))).collect::<serde_json::Map<_,_>>();
    let mut catalog = json!({"api_version":1,"inputs":inputs,"outputs":outputs,"error_schema":schemars::schema_for!(CoreError),"contract_schema":contract_schema(),"operation_capabilities":compatibility,"batch":{"version":1,"input_stage":"before_bindings","mutations":mutations,"check":{"operation":"database.query","resolved_input_schema":schemars::schema_for!(DatabaseQuery),"value_slots":batches::query_slots()},"limits":{"steps":batch_plan::MAX_STEPS,"checks":batch_plan::MAX_CHECKS,"bindings":batch_plan::MAX_BINDINGS,"key_chars":64,"pointer_bytes":512,"input_bytes":2_097_152,"output_bytes":batch_plan::MAX_RESPONSE_BYTES,"snapshot_bytes":batch_storage::MAX_SNAPSHOT_BYTES,"execution_ms":5000}},"limits":{"document_bytes":entities::MAX_DOCUMENT_BYTES,"blocks":entities::MAX_BLOCKS,"depth":entities::MAX_DEPTH,"busy_timeout_ms":workspace::BUSY_TIMEOUT_MS}});
    catalog["batch"]["base_hash_namespace"] = json!("noteka.batch.base.v2");
    catalog["extraction"] = json!({"version":1,"extractor":"utf8_text","extractor_version":1,"default_config":ExtractionConfig::default(),"paused_by_default":true,"limits":{"source_bytes":MAX_EXTRACTION_BYTES,"chunk_bytes":EXTRACTION_CHUNK_BYTES,"fragments":MAX_EXTRACTION_FRAGMENTS,"pending_jobs":MAX_EXTRACTION_PENDING,"total_jobs":MAX_EXTRACTION_JOBS,"retained_text_bytes":MAX_DERIVED_BYTES,"jobs_page_default":50,"jobs_page_max":100,"fragments_page_default":32,"fragments_page_max":64,"cursor_bytes":2048,"config_bytes":8192,"error_bytes":4096,"input_bytes":EXTRACTION_RESPONSE_BYTES,"output_bytes":EXTRACTION_RESPONSE_BYTES,"execution_ms":EXTRACTION_EXECUTION_MS,"lease_seconds":EXTRACTION_LEASE_SECONDS,"busy_timeout_ms":workspace::BUSY_TIMEOUT_MS}});
    catalog["extraction"]["extractors"] =
        json!(["utf8_text", "pdf_text", "xlsx_cells", "image_ocr"]);
    catalog["attachment_index"] = json!({"version":1,"profile":ATTACHMENT_INDEX_PROFILE,"selected_explicitly":true,"scope":"selected_generations","limits":{"generations":MAX_INDEX_ENTRIES,"fragments":MAX_INDEX_FRAGMENTS,"text_bytes":MAX_INDEX_TEXT_BYTES,"response_bytes":EXTRACTION_RESPONSE_BYTES,"cursor_bytes":2048,"query_bytes":256,"terms":16,"page_max":64,"get_page_default":32,"search_page_default":20,"execution_ms":5000,"busy_timeout_ms":workspace::BUSY_TIMEOUT_MS,"snapshot_bytes":batch_storage::MAX_SNAPSHOT_BYTES}});
    catalog["attachment_index"]["extractors"] =
        json!(["utf8_text", "pdf_text", "xlsx_cells", "image_ocr"]);
    catalog["attachment_index"]["xlsx_projection"] = json!({"version":"xlsx_cell_fields_v1","fields":["cached_text","cached_number","cached_boolean","cached_error","cached_date_iso","formula_text"],"chunk_bytes":4096,"fragments_per_generation":512,"text_bytes_per_generation":1048576,"includes_hidden_sheets":true,"cross_fragment_matching":false,"recalculates_formulas":false,"indexes_metadata":false});
    catalog["extraction"]["ocr"] = json!({"version":1,"profile":noteka_ocr_protocol::PROFILE,"platform":"macos_13_or_newer","runtime_availability":"checked_on_explicit_run","wire_version":2,"formats":["png","jpeg"],"coordinate_space":noteka_ocr_protocol::COORDINATE_SPACE,"limitations":noteka_ocr_protocol::LIMITATIONS,"scalar_encoding":"canonical_decimal_string","max_axis":4096,"max_pixels":4000000,"max_regions":512,"max_region_text_bytes":131072,"max_representation_bytes":1048576,"cpu_seconds":4,"rss_observed_bytes":134217728,"rss_poll_ms":5,"hard_rss_limit":false,"private_stdout_bytes":8388608,"private_stderr_bytes":8192});
    catalog["attachment_index"]["ocr_projection"] = json!({"version":"ocr_region_text_v1","indexes_metadata":false,"inferred":true,"cross_fragment_matching":false});
    catalog["extraction"]["pdf"] = json!({"version":1,"parser_name":"lopdf","parser_version":"0.45.0","versions":"1.0-1.7","fonts":"standard_latin_type1","encodings":["StandardEncoding","WinAnsiEncoding"],"scope":"page_text_objects","limitations":PDF_LIMITATIONS,"max_pages":100,"max_text_bytes":1_048_576,"page_stream_bytes":4_194_304,"load_stream_bytes":1_048_576,"heap_layout_bytes":134_217_728,"cpu_seconds":4,"private_stdout_bytes":8_388_608,"private_stderr_bytes":8192});
    catalog
}
pub fn error_response(error: CoreError) -> Response {
    envelope(workspace::id(), Err(error))
}
fn envelope(request_id: String, result: Result<Value>) -> Response {
    match result {
        Ok(data) => Response {
            api_version: 1,
            request_id,
            ok: true,
            data: Some(data),
            error: None,
        },
        Err(error) => Response {
            api_version: 1,
            request_id,
            ok: false,
            data: None,
            error: Some(error),
        },
    }
}
pub fn execute_json(value: Value, origin: &str) -> Response {
    let request_id = workspace::id();
    let result = (|| {
        // Limitar antes de deserializar, incluidos campos desconocidos profundos.
        entities::validate_request_limits(&value)?;
        let request: Request = serde_json::from_value(value)?;
        dispatch(request, origin, &request_id)
    })();
    envelope(request_id, result)
}
pub fn execute(request: Request, origin: &str) -> Response {
    execute_json(
        serde_json::to_value(request).expect("DTO serializable"),
        origin,
    )
}
fn dispatch(request: Request, origin: &str, request_id: &str) -> Result<Value> {
    if matches!(request.command, Operation::Describe(_)) {
        return Ok(describe());
    }
    if let Operation::WorkspaceRestore(input) = request.command {
        return Ok(serde_json::to_value(Workspace::restore(
            input.backup.as_ref(),
            input.destination.as_ref(),
        )?)?);
    }
    let path = request
        .workspace
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or_else(workspace::default_path)?;
    if let Operation::WorkspaceCreate(input) = request.command {
        return Ok(serde_json::to_value(
            Workspace::create(&path, input.timezone)?.info()?,
        )?);
    }
    match request.command {
        Operation::ExtractionIndex(input) => {
            return Ok(serde_json::to_value(attachment_search::index(
                &path, input,
            )?)?);
        }
        Operation::ExtractionSearch(input) => {
            return Ok(serde_json::to_value(attachment_search::search(
                &path, input,
            )?)?);
        }
        Operation::ChangesPreview(input) => {
            return Ok(serde_json::to_value(batches::preview(
                &path, input, origin, request_id,
            )?)?);
        }
        Operation::ChangesApply(input) => {
            return Ok(serde_json::to_value(batches::apply(
                &path, input, origin, request_id,
            )?)?);
        }
        _ => {}
    }
    let mut w = Workspace::open(&path)?;
    dispatch_on_workspace(request.command, &mut w, origin, request_id)
}
fn dispatch_on_workspace(
    command: Operation,
    w: &mut Workspace,
    origin: &str,
    request_id: &str,
) -> Result<Value> {
    let data = match command {
        Operation::ExtractionEnqueue(i) => serde_json::to_value(w.extraction_enqueue(i)?)?,
        Operation::ExtractionGet(i) => serde_json::to_value(w.extraction_get(i)?)?,
        Operation::ExtractionList(i) => serde_json::to_value(w.extraction_list(i)?)?,
        Operation::ExtractionRun(i) => serde_json::to_value(w.extraction_run(i)?)?,
        Operation::ExtractionCancel(i) => serde_json::to_value(w.extraction_cancel(i)?)?,
        Operation::ExtractionRetry(i) => serde_json::to_value(w.extraction_retry(i)?)?,
        Operation::ExtractionControl(i) => serde_json::to_value(w.extraction_control(i)?)?,
        Operation::ExtractionFragments(i) => serde_json::to_value(w.extraction_fragments(i)?)?,
        Operation::ExtractionPdfFragments(i) => {
            serde_json::to_value(w.extraction_pdf_fragments(i)?)?
        }
        Operation::ExtractionOcrRegions(i) => serde_json::to_value(w.extraction_ocr_regions(i)?)?,
        Operation::ExtractionXlsxCells(i) => serde_json::to_value(w.extraction_xlsx_cells(i)?)?,
        Operation::HomeQuery(i) => serde_json::to_value(w.home_query(i)?)?,
        Operation::TemplateSave(i) => serde_json::to_value(w.template_save(i)?)?,
        Operation::TemplateGet(i) => serde_json::to_value(w.template_get(&i.id)?)?,
        Operation::TemplateList(i) => serde_json::to_value(w.template_list(i)?)?,
        Operation::TemplateInstantiate(i) => {
            serde_json::to_value(w.template_instantiate(i, origin, request_id)?)?
        }
        Operation::FormulaSet(i) => serde_json::to_value(w.formula_set(i)?)?,
        Operation::CollectionCreate(i) => serde_json::to_value(w.collection_create(i)?)?,
        Operation::CollectionGet(i) => serde_json::to_value(w.collection_get(&i.id)?)?,
        Operation::CollectionList(i) => serde_json::to_value(w.collection_list(i)?)?,
        Operation::PropertyDefine(i) => serde_json::to_value(w.property_define(i)?)?,
        Operation::RecordCreate(i) => {
            serde_json::to_value(w.record_create(i, origin, request_id)?)?
        }
        Operation::RecordGet(i) => serde_json::to_value(w.record_get(i)?)?,
        Operation::RecordAttach(i) => {
            serde_json::to_value(w.record_update(i, true, origin, request_id)?)?
        }
        Operation::RecordUpdate(i) => {
            serde_json::to_value(w.record_update(i, false, origin, request_id)?)?
        }
        Operation::RecordInverse(i) => serde_json::to_value(w.record_inverse(i)?)?,
        Operation::ViewSave(i) => serde_json::to_value(w.view_save(i)?)?,
        Operation::DatabaseQuery(i) => serde_json::to_value(w.database_query(i)?)?,
        Operation::WorkspaceOpen(_) => serde_json::to_value(w.info()?)?,
        Operation::WorkspaceBackup(i) => serde_json::to_value(w.backup(i.destination.as_ref())?)?,
        Operation::EntityCreate(i) => {
            serde_json::to_value(w.entity_create(i, origin, request_id)?)?
        }
        Operation::EntityGet(i) => serde_json::to_value(w.entity_get(&i)?)?,
        Operation::EntityList(i) => serde_json::to_value(w.entity_list(i)?)?,
        Operation::EntityUpdate(i) => {
            serde_json::to_value(w.entity_update(i, origin, request_id)?)?
        }
        Operation::EntityArchive(i) => {
            serde_json::to_value(w.entity_archive(i, true, origin, request_id)?)?
        }
        Operation::EntityRestore(i) => {
            serde_json::to_value(w.entity_archive(i, false, origin, request_id)?)?
        }
        Operation::RelationAdd(i) => serde_json::to_value(w.relation_add(i, origin, request_id)?)?,
        Operation::RelationRemove(i) => {
            serde_json::to_value(w.relation_remove(i, origin, request_id)?)?
        }
        Operation::RelationList(i) => serde_json::to_value(w.relation_list(i)?)?,
        Operation::ChangesList(i) => serde_json::to_value(w.changes_list(i)?)?,
        Operation::AttachmentImport(i) => serde_json::to_value(w.attachment_import(i)?)?,
        Operation::AttachmentRead(i) => serde_json::to_value(w.attachment_read(i)?)?,
        Operation::DocumentExport(i) => serde_json::to_value(w.document_export(i)?)?,
        Operation::WorkflowQuery(i) => serde_json::to_value(w.workflow_query(i)?)?,
        Operation::SearchQuery(i) => serde_json::to_value(w.search_query(i)?)?,
        Operation::SearchRebuild(_) => serde_json::to_value(w.search_rebuild()?)?,
        _ => unreachable!(),
    };
    Ok(data)
}

fn dispatch_scoped(
    command: Operation,
    scope: &scoped::LocalScope,
    origin: &str,
    request_id: &str,
) -> Result<Value> {
    let path = scope.root();
    match command {
        Operation::Describe(_) => Ok(describe()),
        Operation::ExtractionIndex(input) => Ok(serde_json::to_value(
            attachment_search::index_scoped(path, input, Some(scope))?,
        )?),
        Operation::ExtractionSearch(input) => Ok(serde_json::to_value(
            attachment_search::search_scoped(path, input, Some(scope))?,
        )?),
        Operation::ChangesPreview(input) => Ok(serde_json::to_value(batches::preview_scoped(
            path,
            input,
            origin,
            request_id,
            Some(scope),
        )?)?),
        Operation::ChangesApply(input) => Ok(serde_json::to_value(batches::apply_scoped(
            path,
            input,
            origin,
            request_id,
            Some(scope),
        )?)?),
        other => {
            let mut w = scope.open(scoped::operation_access(&other) != scoped::Access::Read)?;
            dispatch_on_workspace(other, &mut w, origin, request_id)
        }
    }
}
