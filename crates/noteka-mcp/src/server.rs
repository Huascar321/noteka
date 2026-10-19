use crate::{
    catalog::{Catalog, ENVELOPE_CAP, URI},
    transport::{self, Shared},
};
use noteka_core::{
    contracts::Operation,
    scoped::{self, Access},
};
use rmcp::{ErrorData, RoleServer, ServerHandler, model::*, service::RequestContext};
use serde_json::{Value, json};
use std::{
    borrow::Cow,
    sync::{Arc, atomic::Ordering},
};

pub struct Server {
    pub catalog: Catalog,
    pub shared: Arc<Shared>,
}
impl ServerHandler for Server {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().enable_resources().build())
            .with_protocol_version(ProtocolVersion::V_2025_11_25)
            .with_server_info(Implementation::new("noteka-mcp",env!("CARGO_PKG_VERSION")))
            .with_instructions("Workspace local fijo. Lectura por defecto; no retransmitas contenido a proveedores sin selección explícita. Cancelar no garantiza rollback; no reintentes mutaciones automáticamente.")
    }
    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Owned(vec![
            ProtocolVersion::V_2026_07_28,
            ProtocolVersion::V_2025_11_25,
        ])
    }
    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let (tools, next_cursor) = self.catalog.page(request)?;
        serde_json::from_value(
            json!({"tools":tools,"nextCursor":next_cursor,"resultType":"complete"}),
        )
        .map_err(|_| ErrorData::internal_error("Catálogo inválido.", None))
    }
    async fn list_resources(
        &self,
        request: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        if request.is_some_and(|p| p.cursor.is_some()) {
            return Err(ErrorData::invalid_params(
                "Recurso sin páginas adicionales.",
                None,
            ));
        }
        serde_json::from_value(json!({"resources":[{"uri":URI,"name":"Catálogo de operaciones y política local","mimeType":"application/json"}],"resultType":"complete"})).map_err(|_|ErrorData::internal_error("Recurso inválido.",None))
    }
    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        if request.uri != URI {
            return Err(ErrorData::invalid_params("URI no admitida.", None));
        }
        let result:ReadResourceResult=serde_json::from_value(json!({"contents":[{"uri":URI,"mimeType":"application/json","text":self.catalog.resource}],"resultType":"complete"})).map_err(|_|ErrorData::internal_error("Recurso inválido.",None))?;
        Ok(result.into())
    }
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let operation = self
            .catalog
            .names
            .get(request.name.as_ref())
            .ok_or_else(|| ErrorData::invalid_params("Herramienta desconocida.", None))?
            .clone();
        if request.input_responses.is_some() || request.request_state.is_some() {
            return Err(ErrorData::invalid_params("Extensión no admitida.", None));
        }
        let state = self
            .shared
            .state(&context.id)
            .ok_or_else(transport::cancelled)?;
        if state.cancelled.load(Ordering::Acquire) || self.shared.stopping() {
            return Err(transport::cancelled());
        }
        let command = json!({"operation":operation,"input":request.arguments.unwrap_or_default()});
        let effects = serde_json::from_value::<Operation>(command.clone())
            .ok()
            .is_some_and(|op| {
                scoped::operation_access(&op) != Access::Read
                    && self
                        .shared
                        .scope
                        .policy()
                        .allows(scoped::operation_access(&op))
            });
        let scope = self.shared.scope.clone();
        let response = transport::run_core(self.shared.clone(), state, effects, move || {
            scope.execute_json(command, "mcp")
        })
        .await?;
        let text = transport::encode(&response, ENVELOPE_CAP)
            .map_err(|_| transport::delivery(&response.request_id, effects))?;
        let text = String::from_utf8(text).expect("JSON UTF8");
        // Preserve the core's numeric values. Re-parsing the text with a
        // non-roundtrip JSON parser can move a BM25 f64 by one ULP, making
        // structuredContent disagree with the text seen by another client.
        let envelope: Value = serde_json::to_value(&response).expect("JSON envelope");
        let mut result = CallToolResult::structured(envelope);
        result.content = vec![ContentBlock::text(text)];
        result.is_error = Some(!response.ok);
        Ok(result.into())
    }
}
