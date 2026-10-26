# Tasks

## 1. Baseline y recuperación

- [x] 1.1 Conservar cliente/checkpoint14 y fixture real de cinco formatos antes de015; documentar ADR0023 y verificar schema/filas/originales/backup14 exactos sin confundirlos con evidencia histórica v11.
- [x] 1.2 Implementar barrera015, registry y recuperación sin alterar tablas/fuentes; verificar migración real14, rollback antes/después de versión, clientes14/16, selecciones/cursores/hash previos y datos desconocidos.

## 2. Proyección y contratos Rust

- [x] 2.1 Implementar proyección PDFKit por página/rango UTF-8, locator/perfil/engine/limitaciones y selección explícita; verificar vacío, Unicode/fragmentación máxima y rechazo de documentos excesivos conservando cap512, sin metadata indexable, helper ni cambios de hashes anteriores.
- [x] 2.2 Integrar validación completa/hash/CAS/cobertura/invalidation en índice mixto; verificar corrupción fuera de filtro/página, original alterado, no-op/reemplazo, presupuestos y conservación de filas/formatos ajenos.
- [x] 2.3 Generar contratos y verificar52 inputs/outputs previos, API54/MCP26/46/47/batch9/exclusiones7; probar paridad CLI/MCP con SDK real offline y permisos, sin comandos/rutas nuevos.

## 3. Interfaz compartida

- [x] 3.1 Integrar tipo PDFKit en preparación y lector de20 páginas completas, con manifiesto/status/texto; verificar acciones separadas, respuestas tardías/cupo físico/errores y buffer/guardias/temas/teclado sin consultas al montar/editar.
- [x] 3.2 Presentar filtro/locator/procedencia/límites PDFKit en Search individual/agrupada; verificar rangos frente a lector real, páginas/cobertura independientes y texto hostil escapado sin procesamiento, escrituras o cambios de Home.

## 4. Aceptación y cierre

- [x] 4.1 Ejecutar corpus CLI nativo y fixture UI headless real, regresión Rust por perfil, frontend/MCP y checks de contrato/format/Clippy/host pertinentes; documentar resultados, riesgos y fallos fuera de Git.
- [x] 4.2 Ejecutar los cuatro gates originales con targets300/150 intactos y medir la consulta PDFKit acotada; investigar fallos sin ampliar cuotas/omitir validación y conservar todas las muestras/condiciones.
- [x] 4.3 Revisar diff/requisitos/escenarios/diseño/ADR/docs/plan, validar OpenSpec estricto y preservar Home9/17/originales/buffers; sincronizar/contrastar sólo lo implementado y archivar al completar10/10, sin declarar toda5a ni iniciar5b/6.
