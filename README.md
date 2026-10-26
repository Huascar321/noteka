# Noteka

Aplicación personal de conocimiento y productividad local-first y AI-first, inspirada en la interfaz de Notion y en las capacidades esenciales de Ultimate Brain. Stack: Tauri 2, Svelte 5, TypeScript, Rust y SQLite.

**Estado al 22 de octubre de 2026:** **fases 0–6 completas en su alcance acotado**, Home archivado; 7/8 son investigación/futuro. Fases 0–4, búsqueda textual, extracción PDF/Excel/OCR, tablas PDF, recuperación semántica y harness de agentes están verificados y archivados por recortes. Baseline: Rust/SQLite20, API v1/66 operaciones, MCP34/57/59, batch9. [5b](docs/verification-semantic-retrieval.md): perfil/huella, adaptador loopback, migración17, escaneo exacto, generaciones e híbrida con fallback FTS. [5c](docs/verification-agent-harness.md): proveedor local, citas verificables, herramientas públicas, preview/apply, límites, cancelación y trazas; migración18. 433 Rust/16 ignoradas y28 SDK por perfil,94 frontend y5.120 checks headless; gates originales dentro de300/150 (máximos189,807/22,925ms). ChatGPT se probó con `gpt-6.1-sol` y Qwen3.5-4B tuvo un smoke local; ninguno acredita calidad real. El OCR Paddle local se ejecuta sólo en extracción explícita. Recorte PDF universal: **PDFium + PaddleOCR** prioritarios sobre PDFKit/Vision; ver [guía](docs/universal-pdf.md), [ADR0031](docs/decisions/0031-universal-pdf-backend.md) y [verificación](docs/verification-universal-pdf.md). Paddle persiste texto y estructura de tablas (filas/columnas/celdas); no bbox/confianza ni orden visual. Home está archivado 17/17. 6a/6b tienen autenticación real verificada en `noteka-auth`: ChatGPT y Google Drive; Google Picker permite seleccionar archivos con `drive.file`, `google selected` devuelve metadatos y `external.link|unlink|list` persisten asociaciones genéricas en SQLite20. 6a y 6b están archivados; la prueba de API keys/cuotas reales de 6a queda diferida por decisión del usuario. Fase 6 cerrada en su alcance acotado. [Recursos externos](docs/external-resources.md).

**Home archivado:** [`add-local-home-dashboard`](openspec/changes/archive/2026-10-22-add-local-home-dashboard/proposal.md), **17/17 tareas verificadas**. [Home](docs/home.md) ofrece quince vistas compartidas, Today por vencimiento civil y temas claro/oscuro; Goals/clima/fotos y auxiliares siguen pendientes. [Verificación](docs/verification-home.md): **592 checks headless + 592 nativos** en seis sesiones Tauri; [revisión final cerrada](docs/review-home.md). La aceptación nativa se completó con seis sesiones Tauri y 592 checks; ver la evidencia.

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
- [Home implementado y reglas](docs/home.md), [verificación integrada](docs/verification-home.md), [iconos Lucide y selector de color](docs/icons.md).
- [Importación local del export de Notion](docs/notion-import.md): 1.103 páginas, 147 adjuntos y límites de jerarquía/iconos originales. Pruebas UI: `npm run test:headless`; el runner nativo requiere consentimiento específico de pantalla.
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

**Integración PDFKit verificada (20/10):** [páginas persistidas](docs/pdfkit-extraction.md),
8/8 tareas y [aceptación](docs/verification-pdfkit-extraction.md): SQLite14/API54,
MCP26/46/47 y batch9. Conserva53 contratos, originales y Home9/17;339 Rust por
perfil y3442 checks headless. Cuatro gates query p95 máximo176,788ms y lock16,197ms.
PDFKit UI/FTS, OCR dePDF y ampliaciones PDF/Excel siguen pendientes de5a; no se
iniciaron5b/5c ni fase6.


**PDFKit en búsqueda/UI verificado (20/10):** [guía](docs/pdfkit-search-ui.md) y
[evidencia](docs/verification-pdfkit-search-ui.md). SQLite15/API54/MCP26/46/47,
batch9;345 Rust por perfil,25 SDK por perfil y4264 checks headless. Cuatro gates
originales aprobaron query p95 máximo174,526ms/lock17,094ms; prueba nativa grande
143,365/10,405ms. Cuatro deltas sincronizados/contrastados y cambio archivado10/10.
OCR dePDF, tablas y ampliaciones XLSX siguen pendientes de5a; Home9/17 intacto.


Estado20/10: `add-local-pdf-ocr` verificado12/12, cinco deltas sincronizados/contrastados
y archivado. Fuente del cierre20/10 SQLite16/API55/MCP27/47/48/batch9; ver
[aceptación PDF OCR](docs/verification-pdf-ocr.md). Fase5a aún requiere tablas/ampliaciones PDF y XLSX;
5b/5c no iniciados, Home9/17 intacto y fase6 excluida. Sin publicación ni modelos.


Lectura XLSX por hoja/rango/encabezado explícitos verificada: [guía](docs/xlsx-range-reader.md).
Fuente21/10 SQLite16/API56/MCP28/48/49; fase5a aún requiere tablas PDF, antes de5b/5c.

**Cierre final22/10:** las tablas PDF y el cierre5a se completaron; después se
cerraron5b y5c. Fuente vigente SQLite20/API66/MCP34/57/59/batch9, con asociaciones externas y 6b archivado. Ver el estado
al inicio de este README y las verificaciones de [5b](docs/verification-semantic-retrieval.md)
y [5c](docs/verification-agent-harness.md). Home, 6a y 6b archivados; 6c conserva su recorte inicial.
