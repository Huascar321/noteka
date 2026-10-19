# Noteka

Aplicación personal de conocimiento y productividad local-first y AI-first, inspirada en la interfaz de Notion y en las capacidades esenciales de Ultimate Brain. Stack: Tauri 2, Svelte 5, TypeScript, Rust y SQLite.

**Estado al 17 de octubre de 2026:** fases 0–4, búsqueda textual, cola/UTF-8, [PDF acotado](docs/verification-pdf-extraction.md), [búsqueda explícita de adjuntos](docs/attachment-search.md), MCP local, [extracción XLSX acotada](docs/xlsx-extraction.md) y [búsqueda XLSX](docs/xlsx-search.md) verificados y archivados. Baseline vigente: Rust/SQLite v13/API v1/53; sólo index/search amplían sus DTOs y metadatos. La comparación histórica exacta de los otros 50 contratos no se pudo repetir tras perder el baseline v11; [criterio alternativo autorizado](docs/xlsx-search-evidence-recovery.md). El cierre previo de adjuntos fue **19/19 tareas**, tres deltas sincronizados y [cambio cerrado](openspec/changes/archive/2026-10-14-add-local-attachment-search/proposal.md): [benchmarks históricos](docs/verification-attachment-search.md) query p95 214,20–231,07 ms, lock y scratch físico comprobados. Los cierres posteriores se detallan abajo. Tauri/Svelte/Tiptap, notas, productividad, colecciones, cálculos y vistas se conservan. PDF amplio/Excel genérico/OCR de PDF, IA y resto de5a pendientes. Home sigue 9/17: checks headless/CLI no prueban aceptación nativa.

**Cambio activo implementado, aún no archivado:** [`add-local-home-dashboard`](openspec/changes/add-local-home-dashboard/proposal.md), **9/17 tareas verificadas**. [Home](docs/home.md) ofrece quince vistas compartidas, Today por vencimiento civil y temas claro/oscuro; Goals/clima/fotos y auxiliares siguen pendientes. [Verificación](docs/verification-home.md): **606 checks headless aprobados**, sin ocupar pantalla; [revisión final cerrada](docs/review-home.md). Las ocho tareas restantes requieren aceptación del host nativo, no sustituida por el puente CLI.

**MCP local, al 15 de octubre:** [`add-local-mcp`](openspec/changes/archive/2026-10-15-add-local-mcp/proposal.md), **22/22 tareas verificadas**, dos deltas sincronizados y cambio archivado. Binario stdio independiente sobre workspace actual, 23 herramientas de lectura/43 con escritura/44 con ejecución explícita; siete operaciones administrativas/de archivos excluidas. [Guía](docs/mcp.md) y [evidencia](docs/verification-mcp.md): **235 tests Rust por perfil**, 17 SDK por perfil, tres rondas raw por perfil y 606 checks headless aprobados. Recorrido exacto FTS5: dos gates con query p95 **134,96–204,81 ms** (<300 ms), sin rebajar validación/caps; fallos anteriores conservados. Sin modelos, proveedores, pantalla ni configuración personal automática.

**Lector XLSX independiente archivado, 8/8 tareas:** [guía](docs/xlsx-reader.md), [cambio](openspec/changes/archive/2026-10-15-add-bounded-xlsx-reader/proposal.md) y [evidencia histórica](docs/verification-xlsx-reader.md): seis requisitos sincronizados, 216 casos por parser/helper y 245 tests Rust por perfil, regresión PDF/MCP y 606 checks headless. Ese cambio conservó v10/51 sin habilitar cola/CLI/MCP/FTS/UI. La integración se verificó después, con recuperación/backup diseñados explícitamente.

**Extracción XLSX archivada:** [`add-local-xlsx-extraction`](openspec/changes/archive/2026-10-15-add-local-xlsx-extraction/proposal.md), **11/11 tareas**, cuatro deltas sincronizados y contrastados con main. Cola explícita/lector tipado, migración aditiva v11, API v1/52 y MCP 24/44/45; conserva los 51 contratos previos. [Formato/recuperación](docs/xlsx-extraction.md), [ADR 0017](docs/decisions/0017-persisted-xlsx-cells.md) y [aceptación final3](docs/verification-xlsx-extraction.md): 261 tests Rust por perfil, 216 casos CLI/parser/helper XLSX por perfil, regresión PDF/MCP y 606 checks headless. Dos gates FTS query p95 **111,15–146,42 ms** (target 300 ms intacto). No implica indexación XLSX ni paridad Excel/5a completa; Home y pantalla permanecen fuera del cambio.

**Tres recortes nuevos de 5a verificados el 17/10:**
[preparación explícita](docs/attachment-preparation-ui.md),
[lectura completa por páginas](docs/attachment-reader-ui.md) y
[consulta agrupada](docs/grouped-search.md), con 9/9, 6/6 y 6/6 tareas verificadas,
respectivamente. [Aceptación conjunta](docs/verification-local-preparation-closure.md):
3.116 checks headless, 287 Rust por perfil y cuatro gates frescos aprobados;
catálogo/schema/MCP/cuotas intactos. Los tres deltas están sincronizados/contrastados y los cambios archivados por separado.
OCR y ampliaciones PDF/Excel siguen pendientes; **5a todavía no está completa**.
Home permanece activo 9/17.

**Lector OCR independiente verificado el 17/10:**
[guía](docs/ocr-reader.md) y [aceptación](docs/verification-ocr-reader.md).
PNG/JPEG mediante Vision local r3/fast, sin instalaciones, con protocolo Rust y
53 casos por perfil. En el corpus limpio declarado: CER 5/706 (0,7082%), WER
5/106 (4,7170%); no es garantía general. Ese cambio no integró cola/SQLite/API/
FTS/UI. Lector cerrado 6/6, delta sincronizado/contrastado y archivado el 17/10.
Home permanece activo 9/17; no se instalan pesos ni se activan proveedores remotos.

**OCR de imágenes integrado y archivado,12/12:**
[`add-local-image-ocr`](openspec/changes/archive/2026-10-17-add-local-image-ocr/proposal.md),
[guía](docs/image-ocr.md), [evidencia](docs/verification-image-ocr.md) y
[revisión](docs/review-image-ocr.md). SQLite v13/API v1/53, MCP25/45/46 y batch9;
preparación, lector de regiones y selección explícitos, evidencia inferida en
Search. Pasaron319 Rust por perfil,3442 checks headless, regresión completa
PDF/XLSX/OCR/MCP y cuatro gates finales: query p95 máximo254,54 ms, lock18,81 ms.
Cinco deltas sincronizados/contrastados y cambio archivado. PDF amplio/Excel y toda5a siguen
pendientes; Home9/17 intacto.

## Ejecutar

**Search con evidencia local, verificado:** [guía](docs/evidence-search-ui.md)
y [resultados](docs/verification-evidence-search-ui.md). Ámbitos «Notas y entidades» /
«Adjuntos», páginas de 20 hits UTF-8/PDF/XLSX con procedencia y cobertura completas;
sólo lectura de generaciones previamente seleccionadas explícitamente por UI/CLI/MCP. Sin extracción
ni indexación automáticas. Aprobaron 1.248 checks nuevos y los 606 heredados;
regresión de 287 Rust por perfil/MCP y cuatro gates aprobados, con query p95
121,37–235,47 ms y lock máximo 18,94 ms. Primer fallo conservado; límites y
validaciones intactos. **17/17 tareas**, único delta sincronizado/contrastado y
[cambio archivado](openspec/changes/archive/2026-10-16-add-local-evidence-search-ui/proposal.md).
Home permanece activo 9/17.

**Búsqueda XLSX archivada el 16/10, 19/19 tareas:** [cambio cerrado](openspec/changes/archive/2026-10-16-add-local-xlsx-search/proposal.md), [guía](docs/xlsx-search.md)
y [evidencia durable](docs/verification-xlsx-search.md): v12/API v1/52, 287 Rust por
perfil y 606 checks headless; gates mixtos p95 **167,02–270,74 ms** e intactos
**114,31–152,19 ms**, sin cambiar caps/validación/target de 300 ms.
[Criterio alternativo autorizado](docs/xlsx-search-evidence-recovery.md): baseline
v11 perdido; comparación histórica exacta no repetible, sin inventar uno nuevo.
Cinco deltas sincronizados/contrastados: 14 requisitos/49 escenarios, sin tocar Home.

PDF cerró **21/21 tareas**, con tres deltas sincronizados y [cambio archivado](openspec/changes/archive/2026-10-14-add-local-pdf-text-extraction/proposal.md).
PDF usa helper descartable con lopdf =0.45.0, Type1/Standard/WinAnsi y lector por
página. Preparalo con `npm run pdf:prepare` antes de ejecutar PDF por CLI;
los hooks Tauri preparan ambos helpers con `extraction:prepare` sin activar bundle. [ADR 0012](docs/decisions/0012-bounded-pdf-worker.md).

**Fase 4 archivada:** [lotes locales](docs/batches.md) con preview/apply, bindings y recibos de comparación; suscripción de 16 pasos/centavos exactos sin ventana/red/LLM. [Extracción](docs/extraction.md), [índice](docs/attachment-search.md) y XLSX suman doce operaciones no agrupables; cola pausada por defecto/ejecución explícita y schema v12. Base_hash v2 incluye control original, no actividad/índice derivados. No cierra Home nativo, 5a ni habilita inferencia/remotos.

Para MCP, prepará las dependencias y ejecutá `cargo build --offline --locked -p noteka-mcp`; luego `target/debug/noteka-mcp --workspace /ruta/ficticia/workspace`. No crea ni migra el workspace. Stdio no autoriza que un cliente retransmita su contenido a un proveedor remoto: eso requiere selección explícita aparte.

Con Node/npm, Xcode y Rust preparados según [desarrollo](docs/development.md):

```sh
npm ci
npm run tauri dev
```

La UI crea/edita los cinco tipos, captura en contexto, ofrece Inbox/My Day y dashboards, inserta originales locales y busca texto con apertura del bloque original. No requiere red ni modelos.

- [Plan autosuficiente del producto](docs/plan-noteka.md): objetivo, requisitos, arquitectura, fases, decisiones y criterios de aceptación.
- [Bootstrap archivado](openspec/changes/archive/2026-10-12-bootstrap-local-core/proposal.md) y [editor archivado](openspec/changes/archive/2026-10-12-add-block-editor/proposal.md).
- [Instrucciones para agentes](AGENTS.md).
- [Cómo trabajar con OpenSpec](docs/openspec-workflow.md).
- [Desarrollo y comandos reales](docs/development.md).
- [Evidencia de fase 0](docs/verification-phase-0.md).
- [Evidencia de fase 1](docs/verification-phase-1.md) y [cola de revisión del usuario](docs/review-queue.md).
- [Productividad](docs/verification-phase-2.md), [búsqueda verificada](docs/verification-text-search.md) y [benchmark](docs/search.md).
- [Motor de colecciones](docs/collections.md) y [evidencia 3a](docs/verification-phase-3a.md).
- [Fórmulas, rollups y fixture de suscripción](docs/formulas.md), [evidencia 3b](docs/verification-phase-3b.md).
- [Vistas avanzadas](docs/views.md) y [Home solicitado](docs/home-reference.md).
- [Home implementado y reglas](docs/home.md), [verificación integrada](docs/verification-home.md). Pruebas UI: `npm run test:headless`; el runner nativo requiere consentimiento específico de pantalla.
- [Plantillas](docs/templates.md), [bases incrustadas](docs/embedded-collections.md) y [evidencia 3c](docs/verification-phase-3c-templates.md): defaults estáticos, IDs independientes, referencias originales y apertura protegida; [cambio archivado, 17/17 tareas](openspec/changes/archive/2026-10-12-add-collection-templates-and-embeds/proposal.md).
- [Extracción local](docs/extraction.md), [verificación](docs/verification-extraction.md) y [ADR 0011](docs/decisions/0011-explicit-local-extraction.md): jobs persistentes, fencing/lease, fragmentos literales/procedencia y backup/restore independientes del editor.
- [MCP local](docs/mcp.md), [verificación](docs/verification-mcp.md) y [ADR 0014](docs/decisions/0014-scoped-local-mcp.md): permisos por operación/acción, workspace fijo, discovery modern/legacy y cancelación sin falso rollback.

LLMs y embeddings serán configurables. La integración de Google Sheets permitirá previsualizar y abrir en el navegador. El experimento Qwen3.5-4B + LoRA es adicional.

Para inspeccionar la planificación con la versión utilizada:

```sh
npx --yes @fission-ai/openspec@1.14.1 list
npx --yes @fission-ai/openspec@1.14.1 validate --all --strict
```

Estos comandos administran documentos; no sustituyen los tests ni recorridos nativos detallados en la documentación de desarrollo.


[Lector PDFKit independiente](docs/pdfkit-reader.md): perfil local acotado con
procedencia y limitaciones, sin integración con jobs/API/búsqueda/UI ni OCR dePDF.
[Aceptación](docs/verification-pdfkit-reader.md):326 Rust por perfil,59 casos nativos
por perfil y cuatro gates finales. Esto no cierra toda la fase5a.
