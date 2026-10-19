# Revisión final — Home local

Cambio `add-local-home-dashboard`, revisión del 13/10/2026. Alcance: implementación, deltas, diseño, ADR 0009, pruebas y diff contra el baseline anterior a Home. Los resultados ejecutados y el estado de aceptación viven en [verification-home.md](verification-home.md); esta revisión no convierte pruebas headless en aceptación del host Tauri.

| Dimensión | Resultado |
| --- | --- |
| Completitud | 9/17 tareas verificadas; ocho tareas de aceptación nativa pendientes, críticas antes de archivar |
| Corrección | Once requisitos/26 escenarios trazados; tests de núcleo/UI headless aprobados, cobertura del host aún incompleta |
| Coherencia | Diseño/ADR/diff conciliados; dos defectos corregidos y cobertura directa ampliada, sin migración ni ampliación de alcance |

Sugerencia no resuelta: bundle frontend 507,67 kB conserva el aviso orientativo >500 kB. No se cambió el umbral ni se afirma una optimización de tamaño.

## Diff y coherencia

- Baseline parcial de 406 archivos en `noteka-before-home.tar.gz`; extracción y manifiesto de revisión en `noteka-home-baseline-7wnSBY` y `noteka-home-review-20261013`, dentro del temporal aprobado indicado en la verificación. No es un commit. Archivos de bootstrap/configuración no capturados y diffs auxiliares del baseline no se atribuyen como nuevas adiciones de Home.
- Revisados lector/DTOs Rust, catálogo/dispatcher, CLI y contratos generados; ruta/captura/guardias, panel/filas, lectores, tema y CSS; instrumentación del host, bridge/runner y regresiones. Las modificaciones de presentación de colecciones/bases incrustadas sólo sustituyen colores por tokens.
- Comparación byte a byte: sin cambios en las siete migraciones, workspace, entidades, productividad previa, documentos, motor de colecciones, fórmulas, plantillas, adaptador IPC/editor/panel de colecciones y ambos lockfiles. Las once specs principales siguen idénticas al baseline: no se sincronizaron deltas propuestos.
- Diseño y ADR coinciden: Today por vencimiento civil, My Day/Overdue/Active Projects intactos, Active doing/ongoing, Topic sin conversión, porcentajes enteros, resumen completo, snapshot por respuesta y grupos de IDs de página. Sin espejo, expansión de documentos ni nueva persistencia. Goals/clima/fotos/Content/auxiliares siguen fuera del alcance.
- Inventario sin bases, backups, modelos o credenciales de ejecución en fuentes/fixtures; el PNG del icono local no es una foto personal. Búsqueda acotada de patrones concretos de claves privadas/tokens sin coincidencias en el diff revisado: no equivale a auditoría universal de secretos. El bridge y la inyección de transporte están fuera del frontend de producto; el host instrumentado está condicionado por `native-smoke`.

## Hallazgos resueltos

1. **Apertura/captura Home no serializada desde el primer await.** Al retener `entity.get` de apertura, ⌘N podía iniciar un buffer antes de terminar la navegación; la respuesta anterior podía limpiar ese buffer. El test headless nuevo falló con `Atajo inició un buffer durante apertura Home pendiente`, sin simular entidades. `openHome` ahora activa `working` antes de la primera lectura; `createHome` protege también la lectura de la lista y libera el bloqueo antes de crear/enfocar el draft. La guardia existente bloquea acciones competidoras, sin segundo guardado ni nuevas reglas de dominio. Dos intercalados con transporte retenido y teclado real comprueban original, tipo y foco en ambos temas.
2. **Temporal macOS enlazado rechazado como escape.** El bridge comparaba el `/var` de TMPDIR con el `/private/var` canónico. Se reprodujo un HTTP 500 `Symlink fuera del corpus de prueba` al crear el workspace. Se canoniza únicamente la raíz recién creada, conservando restricciones de rutas/symlinks. Test añadido para padre enlazado; se ejecutó también sin NOTEKA_TEST_TMP usando TMPDIR enlazado y con ruta canónica explícita.
3. **Cobertura demasiado indirecta de filtros/órdenes y frontera de fuentes.** Se agregó prueba directa de notas dentro/fuera de Inbox, tareas todo/done/inbox, cinco estados de proyecto, tres tipos de tag, Recent con empate por ID, A–Z distinto de Types, progreso 1/3 = 33%, metadatos irrelevantes/futuros/null y grupos no disponibles. La prueba de límite comprueba ahora 10.000 fuentes completas antes de rechazar 10.001; se conservan asserts previos de páginas, preservación y demás fronteras.

## Trazabilidad de los 26 escenarios

Los seis primeros requisitos están en [productivity-workflows](../openspec/changes/add-local-home-dashboard/specs/productivity-workflows/spec.md) y los cinco restantes en [desktop-shell](../openspec/changes/add-local-home-dashboard/specs/desktop-shell/spec.md). Cada fila corresponde a un escenario, no a un test nuevo. Las referencias de tests siguientes se ejecutan sobre SQLite real salvo las promesas/preferencias unitarias indicadas.

| Alias | Archivo / test |
| --- | --- |
| H1 | `crates/noteka-core/tests/home.rs` — `today_workflow_differences_civil_defaults_and_strict_cursors` |
| H2 | Mismo archivo — `every_view_orders_and_paginates_unique_originals_and_declared_groups` |
| H3 | Mismo archivo — `summaries_context_duplicates_archive_and_unknown_fields_are_conservative` |
| H4 | Mismo archivo — `preservation_backup_future_documents_and_scope_are_real` |
| H5 | Mismo archivo — `candidate_source_group_limits_and_context_truncation_do_not_fake_success` |
| H6 | Mismo archivo — `incompatible_links_full_link_cap_and_summary_archive_ties` |
| H7 | Mismo archivo — `timestamp_and_id_orders_civil_age_and_archived_today` |
| H8 | Mismo archivo — `view_membership_orders_and_partial_metadata_are_explicit` |
| S | `crates/noteka-core/src/home.rs` — tests de día civil determinista y snapshot WAL con escritor real |
| C | `crates/noteka-cli/tests/cli.rs` — `home_views_strict_errors_and_projection_parity`, `concurrent_cli_writer_and_home_reads_keep_reference_metadata_coherent` |
| R | `tests/home-reads.mjs` — generaciones, cancelación/cola/dispose, retry con cursor/día, respuesta incompatible y preferencias |
| U | `tests/native/home.js` — etapas initial/final/reopen, ejecutadas headless junto a la regresión íntegra `tests/native/smoke.js`; no certifican host nativo |
| K | `tests/headless-run.cjs` — teclado real e intercalados de apertura/captura, sólo transporte retenido |

| Requisito / escenario exacto | Implementación inspeccionada | Evidencia / límite |
| --- | --- | --- |
| Consultas compartidas — Paridad y páginas estables | `home.rs::home_read`, catálogo y dispatcher `lib.rs`, CLI `main.rs` | H2, C, U; IPC nativo pendiente |
| Consultas compartidas — Entrada inválida o cursor ajeno | `HomeQuery`, cursor estricto/scope/fecha/límite de `home_read` | H1, H4, H5, H6, C |
| Consultas compartidas — Lectura de formato desconocido | `read_row`, `matches_view`, disponibilidad de resúmenes; sin parser de escritura | H3, H4, H8 |
| Today — Hoy incluye vencidas y no reemplaza planificación | `matches_view(TasksToday)`; `productivity.rs` sin cambios | H1, H7, H8, U |
| Today — Límite civil y zona | `today_at`, `civil_date`, fecha/zona y día del cursor | H1, H7, S |
| Vistas explícitas — Tareas y notas por estado e Inbox | `matches_view`, defaults conservadores de `read_row` | H2, H8, U |
| Vistas explícitas — Proyectos y clasificación conservada | `matches_view`, `compare`, grupos declarados; workflows previos intactos | H2, H8, U |
| Vistas explícitas — Orden documentado de tags y personas | `compare` y desempate por ID | H2, H7, H8, U |
| Resúmenes de proyectos — Resumen completo aunque la lista esté paginada | `home_read`: vínculos/fuentes por conjuntos, enteros comprobados | H3, H8, S, U con cambios CLI |
| Resúmenes de proyectos — Sin tareas o metadatos futuros | `bad` y disponibilidad/counts/porcentaje null | H3, H8, U |
| Contexto/tags — Última nota y conteo global | `home_read`: fuentes activas únicas, orden modificación/ID | H3, H6, U |
| Contexto/tags — Contexto archivado y documentos desconocidos | Referencias ligeras/archivo de `home_read`, apertura `App.openHome` | H3, H4, U; apertura de documento futuro en regresión compartida |
| Grupos — Relaciones múltiples sin copias | `home_read`: IDs únicos antes de grupos y contexto completo previo a truncar chips | H2, H3, H5, U |
| Grupos — Grupos vacíos y segunda página | Cinco estados/tres tipos declarados, IDs sólo de página y cursor | H2, H8, U |
| Home visible — Workspace vacío y estructura offline | `HomePanel`: portada/secciones/clima/Goals/vacíos, shell de `App` | U initial, restricciones de red del runner; host pendiente |
| Home visible — Navegación compacta y funciones diferidas | `HomePanel`: callbacks reales y matriz textual sin botones ficticios | U, inspección DOM/código y referencia visual |
| Presentación — Cambio de vista y páginas | `HomeReader`, panel/grupos/paginación, `HomeItem` por ID | H2, R, U, K; teclado/foco nativo pendiente |
| Presentación — Progreso y metadatos no disponibles | `HomeItem`: Sin tareas/no disponible/0% real; People textual | H3, H8, U |
| Navegación/captura — Llegar a Home con buffer pendiente | `App.guarded`, `resolveGuard`, `save`, `navigate`; panel de colección existente | U: cancelar/fallo/conflicto entidad y cancelar/descarte colección; regresión compartida de fallo/conflicto colección; host pendiente |
| Navegación/captura — Crear en proyecto o estado | `HomePanel.createIn`, `App.createHome/startDraft/save`, transacción existente | U, K y CLI: relación/estado/sin fechas/descarte; intercalados sólo headless |
| Navegación/captura — Confirmación seguida de fallo de lectura | Commit limpia pending antes de montar Home; errores/retry por sección | U: creación única, sin draft confirmado ni filas viejas; cierre nativo pendiente |
| Lecturas — Respuesta tardía al cambiar vista o workspace | `HomeReader`, `ReadLifetime`, key por workspace y limpieza síncrona | R, U: retraso/vista/desmontaje/workspace |
| Lecturas — Fallo y reintento independiente | Estado/identidad de consulta por lector, cola dos, retry cursor/día | R, U; respuestas CLI reales salvo fallo de transporte inyectado |
| Lecturas — Actualizar después del CLI | `App.refresh`/evento focus único, `HomePanel.refreshPanel` | S, C, U; evento simulado no prueba activación NSWindow |
| Temas/foco — Tema persistido y buffer intacto | `theme.ts`, tokens de raíz y selector `App.changeTheme` | R, U; almacenamiento WebView nativo pendiente |
| Temas/foco — Foco y layouts en ambos temas | Controles nativos/aria, restauración acotada de foco y CSS responsive | U, K y capturas sintéticas; 1920×845/1280×800/900×800; WKWebView/lector de pantalla no certificados |

## Condición de cierre

No se detectó otra divergencia funcional confirmada dentro del diff revisado. Esto no prueba ausencia de cualquier defecto. La revisión de código/diseño/ADR y escenarios se separa de la aceptación pendiente:

| Tarea pendiente | Evidencia que falta para completarla |
| --- | --- |
| 2.1 | Navegación/captura/guardias y cierre en host Tauri integrado |
| 2.2 | Refresh/activación/workspace nativos con revisiones intactas |
| 2.3 | Tasks/Notes/páginas/captura/teclado nativos |
| 2.4 | Projects/Tags/resúmenes/grupos/cambios CLI en host |
| 2.5 | Composición/People/creación/apertura/páginas nativas |
| 2.6 | Preferencia WebView, buffers, foco/contraste/layouts nativos en ambos temas |
| 3.2 | Recorrido nativo conjunto completo offline, reapertura y restauración |
| 4.2 | Parte nativa de la regresión final completa; builds/tests no la sustituyen |

Estas ocho tareas bloquean el archivo bajo los criterios actuales. No se lanzan ventanas ni capturas de escritorio sin nueva autorización específica; no se rebajan criterios para marcar checks. Sin sincronización ni archivo prematuros. [Cola de intervención](review-queue.md).
