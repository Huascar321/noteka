# Verificación de 3c: plantillas y bases incrustadas

12 de octubre de 2026, macOS Apple Silicon/M1 Max 64 GiB. Cambio `add-collection-templates-and-embeds`, segundo tramo de 3c. Datos exclusivamente sintéticos fuera del checkout; sin importar Notion ni activar cuentas/modelos. Implementación SQLite **v7**, documentos **v1/v2/v3**, manifiestos de backup sin cambios.

## Summary

| Dimensión | Resultado |
| --- | --- |
| Completitud | **17/17 tareas verificadas**; seis requisitos/catorce escenarios del delta cubiertos abajo. |
| Corrección | 69 tests Rust, 8 tests adaptador/cola y **166 checks nativos offline** aprobados. |
| Coherencia | Una transacción de creación, referencias semánticas, renderer compartido y lectores acotados; ADR 0008. |

No se detectaron divergencias pendientes de specs/diseño. Formulario JSON técnico y límites de lectura son decisiones explícitas, no excepciones a requisitos. Cierre documental/sincronización/archivo se registra al final.

## Comandos ejecutados

Con la toolchain aislada de [desarrollo](development.md):

```sh
cargo fmt --all --check
cargo test --offline -p noteka-core -p noteka-cli
cargo clippy --offline --workspace --all-targets -- -D warnings
cargo clippy --offline -p noteka-desktop --all-targets --features native-smoke,custom-protocol -- -D warnings
npm run contracts:check
npm run test:editor
npm run check
npm run build
npm run tauri build -- --no-bundle
file target/release/noteka-desktop
NOTEKA_TEST_TMP=/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/opencode npm run test:native
npx --yes @fission-ai/openspec@1.14.1 validate --all --strict
```

- Rust: CLI 9, workspace 6, advanced_views 6, block_editor 6, database 7, formulas 8, local_core 11, productivity 4, search 3, templates_and_embeds 7, workspace_validation 2: **69/69**.
- Adaptador: seis round-trips/recuperación, incluida anidación v3; cola: dos tests de concurrencia/descarte: **8/8**.
- Check Svelte/TypeScript: cero errores y warnings. Contratos generados reproducibles; catálogo 37 operaciones con sus inputs/outputs.
- Build release `--no-bundle`: `target/release/noteka-desktop`, **Mach-O 64-bit arm64**. No firmado para distribución, notarizado ni publicado.
- Native: **143 main + 7 reapertura + 16 restauración = 166**, tres sesiones WKWebView/Tauri con IPC y procesos CLI reales, app/subprocesos bajo `sandbox-exec (deny network*)`. No se simula el almacenamiento.
- OpenSpec estricto antes del archivo: **12/12** (cambio y once specs); sólo prueba documentos.

Evidencia nativa final: `/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/opencode/noteka-native-2g2Hnm/results.json`. Incluye workspace, backup y restaurado sintéticos.

Ejemplos de `docs/templates.md` ejecutados por procesos CLI con IDs/revisiones reales: las cuatro operaciones, defaults BOB `145.69`, override `48.57`, omisión y null; stdout JSON exclusivo/stderr vacío. Evidencia: `…/opencode/noteka-templates-example-0rwt21mu/results.json`.

## Trazabilidad de requisitos y escenarios

| Requisito | Escenario | Implementación y evidencia |
| --- | --- | --- |
| database-engine: Plantillas de registro versionadas | Reapertura y edición concurrente | `templates.rs` save/get/list + DatabasePanel: UI confirma, CLI modifica revisión, refresh no rebasa buffer, conflicto conserva JSON; reabre en otras sesiones. Core valida límites/tipos y dueño. |
| mismo | Definición futura y restauración | Definición cruda en `Template`, parse estricto al escribir/instanciar; core conserva bytes en backup y rechaza conversión, native restaura campo future/estado no editable y originales. |
| database-engine: Instanciación atómica con defaults | Dos instancias independientes | `create_in_transaction` recibido bajo Immediate: nuevos IDs anidados, dinero exacto, null/omisión, referencias compartidas e Inbox false; core/CLI/native comparan instancias/plantilla y original. |
| mismo | Entrada inválida o revisión antigua | Revisiones de esquema/plantilla comprobadas bajo lock; defaults y overrides completos separados. Tests rechazan floats/calculadas/IDs ajenos incluso null; triggers en documentos/membresía/valores/historial y FTS ausente dejan counts/originales intactos. UI mantiene buffer. |
| block-editor: Documento enriquecido conservable | Edición y reapertura | Regresiones block_editor y adaptador v2; native reabre heading/checklist/cita/código/enlace/imagen sin perder IDs. |
| mismo | Documento desconocido | local_core/block_editor preservan JSON/versiones/campos desconocidos; native recuperación sin conversión. |
| mismo | Referencia v3 entre bloques | `documents.rs`, adaptador y NodeView/UniqueID: inserción slash/botón, quitar/undo/redo con ID estable, save/reopen offline. JSON no contiene filas/DOM; fixture v6 migra v1/v2 sin reescritura. |
| mismo | Destino o atributos inválidos | Validación estructural + columnas de pertenencia dentro de escritura; tests rechazan vista ausente/ajena, attrs/contenido/hijos/versiones incorrectos, conservando documento/revisión/counts. Guardado no ejecuta vistas futuras. |
| block-editor: Referencias dinámicas distinguibles de texto fuente | Exportar y buscar sin copias | Export JSON exacto, warning no transitivo también en manifiesto/resultado; marcador Markdown con IDs, sin recursión propia. FTS/rebuild encuentra texto sólo en registro original, no contenedor ni template. |
| mismo | Restauración del conjunto | Core compara template/colección/AST/registros/documentos v3/bytes del original tras restore; native nota/restaurado consulta IDs originales, errores calculados y referencias de archivo offline. |
| collection-views: Presentación incrustada paginada de originales | Cambios y páginas del mismo destino | CollectionPresentation usa sólo `DatabasePage`; native recorre **51 IDs en los cinco layouts**, grupos/cobertura por página, modificación de vista y fila por CLI, dos contenedores; refresh no incrementa revisión ni ensucia. |
| mismo | Referencia no disponible o cálculo fallido | Falla formato futuro y cálculo en filtro: elimina proyección vieja, preserva referencia y reintenta tras reparación; error en celda/tarjeta no se muestra como cero. Core valida destino ausente. |
| collection-views: Apertura incrustada protegida y acotada | Apertura con cambios pendientes | Callback a App vuelve a leer por ID bajo guardia; native cancelación, fallo SQLite y conflicto CLI mantienen texto/referencia; descartar abre ID original sin copia. |
| mismo | Colección que incluye su contenedor | Presentación no expande documentos/adjuntos. Core/native incluyen la propia instancia en su colección y dos notas con la misma vista sin recursión; referencias permanecen originales. |

## Recuperación e integridad

Migración 007 agrega tabla templates, FK/índice y versión coordinada. El test v6 comprueba documentos v1/v2, vistas, propiedades, cálculos, valores y FTS sin reescritura; fallo inyectado revierte tabla/versiones y conserva backup v6 restaurable en otro directorio. Backup v7 sin templates se rechaza incluso con hash recalculado. Rechazo futuro conserva main/WAL sin tocar; fixtures antiguas sólo eliminan la nueva tabla y actualizan expectativas de versión actual.

Backup completo transporta definiciones crudas y todos los originales. Export de nota **no** transporta destinos transitivos; no confundirlo con recuperación del workspace. No hay downgrade destructivo, GC ni importación automática.

## Correcciones encontradas al verificar

- Insertar/quitar rápido podían agruparse en el mismo evento de historial de ProseMirror: `closeHistory` separa las acciones estructurales. No se resolvió agregando un sleep al test ni cambiando el requisito.
- El botón de actualización perdía foco por `disabled` durante IPC: después de `tick` restaura sólo ese foco si cayó al body, no roba el de otro control/navegación. Assert de foco/outline aprobado sin cambiar CSS.
- Helpers nativos ahora esperan controles habilitados/listado listo y buscan títulos **en el listado**, no en tarjetas de una nota que se está desmontando. El host recupera foco real si lo perdió; no elimina asserts. Waits de conflicto comprueban estado/modal, no casing de una cadena.
- Las lecturas CLI del recorrido restaurado apuntan al directorio **restaurado**, no al workspace original. Instrumentación exclusiva de native-smoke.
- Ensayos iniciales fallidos por estas causas/expectativas de fixture no constituyen evidencia de aprobación; el resultado final citado cubre el recorrido completo.

## Revisión y límites

Snapshot previo: `…/opencode/noteka-before-templates.tar.gz`; diff acotado: `…/opencode/noteka-templates-review.diff`. Se revisaron runtime, contratos generados, tests, recuperación, docs y artefactos, no un diff vacío de Git (el repositorio sigue sin commit inicial). Sin credenciales/modelos/backups/datos personales en Git; no commit/push/publicación.

No benchmark nuevo de latencia/memoria ni prueba de un documento máximo de 10.000 bloques. La cola verifica máximo dos IPC y descarta generaciones/disposed; no cancela físicamente una llamada ya iniciada. Los resultados paginados no son snapshot entre llamadas. Galería textual, sin previews; no edición inline, templates built-in/de schema, fechas relativas ni automatizaciones. UI JSON técnica registrada para revisión.

El runner maneja DOM/eventos programáticos en WKWebView y prueba foco real; no equivale a validación de periféricos físicos o lectores de pantalla. Release local no prueba distribución. Home/Goals/fotos/clima, extracción, embeddings, harness, MCP e integraciones siguen pendientes. Ninguna dependencia de LLM ni red se agregó al núcleo.

## Cierre OpenSpec

**17/17 tareas completas**. Se sincronizaron los seis requisitos/catorce escenarios del delta en block-editor, database-engine y collection-views (cinco requisitos nuevos y uno modificado, con escenarios previos conservados); comparación de bloques completa y validación estricta de las once specs aprobadas. El CLI confirmó `all_done` antes de archivar con `--skip-specs --yes`, porque la sincronización ya estaba aplicada/verificada. Archivo: [`2026-10-12-add-collection-templates-and-embeds`](../openspec/changes/archive/2026-10-12-add-collection-templates-and-embeds/proposal.md). Después del archivo, `list --json` confirmó cero cambios activos, `validate --all --strict` aprobó **11/11 specs**, se volvió a comparar cada bloque del delta con main y se comprobaron tareas/metadatos/enlaces locales. Home sigue pendiente.
