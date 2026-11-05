# Verificación de Lock page — 26/10/2026

Cambio `add-note-page-lock`, apoyado en el menú ya implementado de
`refine-notion-note-editor`. Ver [ADR0054](decisions/0054-note-page-lock.md) y la
[guía del editor](note-editor.md). El nuevo campo se diseñó antes de modificar
persistencia; no hay migración SQL, operación o dependencia nueva.

## Implementación y contrato

- `NoteFields.page_locked` es `Option<bool>`; ausencia/null no bloquea. Lecturas
  anteriores no agregan el campo. El parche de bloqueo usa entity.update/CAS y
  omite documento/título; el desbloqueo temporal queda en la instancia de UI.
- Lock page conserva su switch activo durante la apertura temporal. Locked
  protege cuerpo/título/icono/propiedades/relaciones/asociaciones y Archive/Restore;
  Re-lock devuelve esa protección. Se conservan selección/copia, vínculos,
  navegación, exportación y lectura de adjuntos.
- Guardar/Descartar/Cancelar resuelve el borrador antes de bloquear/re-bloquear.
  Discard repone el documento guardado, y errores/CAS no afirman un cambio de modo.
- El contrato generado sólo agrega `NoteFields.properties.page_locked`. La
  estructura completa restante coincide con la versión previa de Git. El campo
  se conserva por get/export/backup y no es un permiso para clientes CLI/MCP.

## Validación pertinente

`npm run build` aprobó catálogo de **85 SVG locales**, contratos y Svelte/TypeScript
con **0 errores y 0 advertencias**, y bundle. Permanece la advertencia anterior
de Vite sobre el tamaño del chunk. Rustfmt de los dos archivos Rust modificados
y `git diff --check` aprobaron. El check global de formato encontró previamente
el orden de módulos goals/home en lib.rs; no se modificó ese archivo ajeno.

Las tres pruebas nuevas Rust, dentro de las siete de productividad aprobadas,
cubren tipo/ausencia/null, rechazo atómico, CAS, metadatos omitidos/futuros,
documento/IDs, relaciones, exportación JSON, backup/restore y versión SQLite
intacta. Se compiló el CLI actualizado offline/locked.
`cargo check --offline --locked -p noteka-desktop` también aprobó; esto verifica
compilación del host y no constituye aceptación de IPC/foco/cierre nativa.

La revisión Chromium headless aprobó **79 comprobaciones** con UI de producción
→ CLI Rust real → SQLite sintético: 35 del ciclo de bloqueo, 12 de guardias y
errores, 27 de teclado/temas/compatibilidad y 5 de disposición estrecha final.

| Área | Resultado observado |
| --- | --- |
| Estado durable | Un cambio de revisión al activar/desactivar, documento/IDs intactos; sin indicador antes de activar y estado correcto al reabrir |
| Edición protegida | Título, cuerpo, undo/atajos, checklist, clic inferior, icono, propiedades y relaciones no modifican la Note; lectura/copia/vínculos/exportación disponibles |
| Temporal | Re-lock y switch on; edición/guardado conserva true; misma instancia/editor al pasar a Center peek/New tab; reabrir restablece Locked |
| Borradores | Cancel conserva buffer/modo; Discard restaura guardado; Save and continue usa la revisión actualizada antes de persistir el bloqueo |
| Fallos | Storage error y revisión externa no alteran estado previo; un save fallido durante Re-lock conserva diálogo, borrador y modo temporal; refresh permite recuperar la revisión confirmada |
| Teclado y temas | Space/Enter, foco y omisión de Archive deshabilitado; Locked/Re-lock y menú en claro/oscuro a 1280/420 px; alineación izquierda revisada sobre el build final |
| Compatibilidad | Task/Goal/Milestone conservan editor y UI previa; Note nueva pide guardar antes de bloquear |

La revisión principal no registró errores de navegador ni solicitudes externas.
Las capturas son de la sesión propia, sin pantalla nativa ni workspace personal.
Durante la revisión se corrigió el título para que readonly permita selección;
el último ajuste alineó el indicador en 420 px. La preparación de compatibilidad
se corrigió para crear una Note normal desde Notes, en lugar de un Goal desde
Goals. Los intentos previos no cuentan como aceptación.

## Regresión amplia y fallos heredados

Se ejecutaron todos los tests unitarios y de integración de noteka-core:
**326 aprobados, 6 fallidos y 16 ignorados**. La suite general conserva estos
fallos, reproducidos compilando una copia aislada de `HEAD`
`fd4f4366a26131a0b35dcd181ac0300fe528c7cc`, sin page_locked:

- `attachment_search_tests::all_49_v9_input_output_contracts_remain_byte_identical`
- `pdf_contracts::tests::v8_schemas_and_root_references_stay_unchanged`
- `xlsx_tests::all_51_prior_operation_schemas_and_errors_keep_independent_baseline_hashes`
- `home::every_view_orders_and_paginates_unique_originals_and_declared_groups`
- `home::timestamp_and_id_orders_civil_age_and_archived_today`
- `home::view_membership_orders_and_partial_metadata_are_explicit`

Las tres comparaciones históricas fallan con los mismos hashes actual/esperado
en ambas versiones. Home reproduce los mismos fallos y valores; para comparar
sus listas se normalizaron únicamente los UUID aleatorios de cada fixture.
Esas seis pruebas, sus hashes esperados y sus aserciones quedaron intactos.
El check completo de la suite no se presenta como aprobado.

## Evidencia y límites

Evidencia ignorada en `output/playwright/page-lock-20261026/`: setup/review/guards/
visual/final-layout, JSON de resultados y resumen, ocho capturas por tema/ancho y
las dos de alineación final; logs Rust completos y por integración, comparación
del contrato y logs de las ejecuciones de baseline. La ruta de la copia fuente
aislada está en `baseline-root.txt`; no constituye un backup de datos personales.

No se acredita aceptación nativa IPC/foco/cierre, ejecución Windows ni un ACL.
Las suites históricas UI/MCP y los gates de búsqueda no se repitieron. No hubo
cambios de motores, modelos, proveedores, originales, dependencias o formato de
documentos. Las cuatro tareas están verificadas; deltas pendientes de
sincronización/archivo.
