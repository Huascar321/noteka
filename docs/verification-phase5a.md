# Auditoría del alcance de fase5a

Estado: fase5a completa y verificada el21/10/2026, en su alcance local acotado.
**Actualización22/10:** 5b y5c cerraron después; ver
[recuperación semántica](verification-semantic-retrieval.md) y
[harness](verification-agent-harness.md). Este documento conserva la foto de5a.
El lector de tablas cerró8/8 con cuatro deltas sincronizados/contrastados y archivo;
el refactor cerró6/6 sin deltas (skip_specs) y archivo independiente. Habilita5b;
5c sigue dependiendo de5b. No se inicia fase6.

## Alcance trazado a la hoja de ruta

| Necesidad de5a | Implementación y aceptación |
| --- | --- |
| Texto de notas sin modelos | add-local-text-search cerrado4/4; índice FTS5 derivado, referencias de entidad/bloque/revisión y filtros. |
| Preparación local explícita | Cola/UTF-8 cerrada20/20 y preparación UI9/9; pausa/retry/cancel/publicación cercada, sin procesamiento al abrir o buscar. |
| Texto PDF por página | Lector textual acotado21/21; lector PDFKit6/6, persistencia8/8 y UI/FTS10/10. Perfil nativo y límites explícitos, originales conservados. |
| Imágenes y PDF escaneado | OCR independiente6/6, imagen persistida12/12 y PDF OCR12/12; regiones/bbox/confianza/manifiesto, páginas vacías, fuente/raster/motor y selección explícita. |
| Excel local | Lector8/8, extracción11/11, búsqueda19/19 y rango/encabezado6/6; hojas/celdas/caché/fórmula/rangos, sin recálculo/macros o expansión inventada de merges. |
| Tablas PDF | Proyector independiente6/6 cerrado. Integración compartida add-local-pdf-table-reader8/8, sincronizada/archivada: grid explícito, celdas vacías/regiones múltiples/exteriores/cobertura y paginación. |
| Búsqueda de adjuntos y consulta conjunta | Índice UTF-8/PDF19/19, XLSX19/19, evidencia UI17/17, lector completo6/6 y grupos independientes6/6; OCR/PDFKit/PDF OCR incluidos por sus cierres propios. |
| Integridad y recursos | Revalidación completa, fuentes de candidatos/lookahead y límites300/150. optimize-local-index-verification6/6 cerrado elimina trabajo redundante y cierra brecha de estadísticas; aceptación final aprobada. |

La auditoría de disco encontró19 recortes previos con todas sus tareas marcadas,
en rutas archivadas exactas (`phase5a-archive-audit.json`, fuera de Git). Es
trazabilidad documental; no reemplaza la regresión de código/UI y los gates.
No se recreó, modificó o resincronizó el lector PDFKit ni el proyector cerrado.

## Condiciones de cierre comprobadas

- Lector de tablas8/8 y refactor6/6 verificados y archivados por separado.
  Refactor skip_specs no modificó main; sólo se sincronizó el lector ya completo.
- Regresión fresca Rust/CLI/MCP, lectores/preparación/búsqueda headless, host
  compilado sin ventana y dos pares originales de gates consecutivos aprobados.
- Compatibilidad contractual completa y backup/desconocidos/originales exactos;
  deltas del lector contrastados con main, títulos/Purpose/escenarios heredados
  y requisitos no afectados preservados.
- Hoja de ruta/configuración actualizadas con resultados reales;5b es el
  siguiente cambio por dependencia, después5c y parada antes de fase6.

## Límites y trabajo separado

La aceptación técnica existente es macOS Apple Silicon y perfiles acotados.
No equivale a PDF/Excel genéricos, OCR perfecto, orden visual universal o tablas
automáticas. Los rechazos/cupos y coberturas de cada perfil siguen explícitos.
Los límites no se ampliaron para aprobar pruebas.

El ensayo autorizado de alternativas locales usa16 PDFs propios/18 páginas,
modelos OCR aislados fuera de Git y red denegada en inferencia. Recomienda
PDFium+PaddleOCR para procesamiento estructural, con PDFKit opcional en Mac;
no adoptó esos motores en Noteka ni ejecutó la aplicación en Windows. La
portabilidad productiva requiere una integración/evaluación propia; la nueva
plataforma objetivo no se presenta como ya verificada.

Recuperación semántica y harness fueron5b/5c posteriores; al momento de esta
auditoría no estaban implementados. Cerraron el22/10 con evidencia separada. Descripción
visual con modelos, Drive/OAuth, proveedores cloud, publicación y LoRA no son
criterios de cierre de este recorte5a ni se activaron. La única instalación de
pesos autorizada fue el benchmark OCR aislado, no LLMs/embeddings en Noteka.

Home permanece9/17, con sus ocho aceptaciones nativas pendientes y sin alterar
composición/navegación/clima/secciones. Los checks headless no cierran Home ni
acreditan IPC/foco/cierre nativos. No se descarta ni resetea trabajo ajeno.

## Resultado integrado

384 pruebas Rust por perfil,28 SDK MCP por perfil,101 frontend y5146 checks
headless dark/light aprobaron; contracts/fmt/Clippy workspace y host no-bundle
sin ventana aprobados. La evidencia headless no sustituye aceptación nativa.
Los cuatro gates originales aprobaron consecutivamente por par, con máximos
query182,232ms y lock18,445ms; caps300/150 sin cambios. Se conservan fallos
previos y límites de corpus/caché en [verificación del refactor](verification-index-verification.md).

[Tabla PDF](verification-pdf-table-reader.md): cliente56 independiente y actual57
conservan56 contratos completos, seis formatos/desconocidos/backup y originales;
lectura release21,365/19,241/22,166ms p95. SQLite16/API57/MCP29/49/50/batch9,
sin017 o persistencia de layouts. Treinta fuentes protegidas y ocho pins extra
conservados; no se descartó ni reseteó trabajo local.

`2026-10-21-optimize-local-index-verification`6/6 y
`2026-10-21-add-local-pdf-table-reader`8/8 están archivados. El lector sincronizó
cinco requisitos nuevos/doce modificados en cuatro capabilities;33 nombres de
escenarios heredados,194 requisitos no afectados y títulos/Purpose exactos.
Las13 specs (211 requisitos) aprobaron validación estricta. El aviso no bloqueante
de archive por cantidad de requisitos no altera esta comprobación de cuatro deltas.
La evidencia durable de fuentes/binarios/clientes/catálogos/fixtures/backup se
conserva fuera de Git antes de comenzar cualquier formato persistente de5b.
