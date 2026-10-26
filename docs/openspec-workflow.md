# Cómo desarrollar Noteka con OpenSpec

Actualización 26/10/2026: `resize-workspace-panels` y
`add-random-ascii-cover-scenes` implementados/verificados localmente; deltas
pendientes de sincronización/archivo. Anchos con arrastre/teclado y memoria local,
caps 35%/65%, exclusión de Side mayor a 35% sin perder el editor, transición
reversible y movimiento reducido. Indicador más compacto, rail24/paso14. Portada
con tres variantes locales: tiburones conservados, koi y medusas; semilla efímera,
42s activos y fundido1,4s. Build, 16 pruebas puras, 71 checks de App y 12 casos
visuales aprobados; nueve muestras de fundido revisadas. Sin nueva aceptación
nativa ni cambios de dominio. Meeting Notes permanece 0/19.
[Paneles](panel-widths.md), [portada](ascii-cover-scenes.md),
[evidencia](verification-panel-widths.md) y [mediciones ASCII](verification-ascii-cover-scenes.md).

Actualización 26/10/2026: `add-persistent-scroll-outline` y `add-detail-peek-modes`
implementados y verificados localmente, con deltas pendientes de sync/archivo.
Side peek derecho inicial, Center/Full/New tab y default global validado; misma
instancia de editor y guardias agregadas de raíces/hijos. Indicador persistente
de seis secciones Home y título/H1–H3 en Note. Build y 127 checks focalizados
headless aprobados; no acredita IPC/cierre físico nativo. Catálogo 81 SVG.
[Peeks](detail-peek.md), [indicador](scroll-outline.md),
[evidencia](verification-detail-peek.md). Meeting Notes continúa 0/19.
Seguimiento del indicador: tamaño/espaciado reducidos y brillo retirado, con
compilación y revisión headless focalizada; ver [evidencia](verification-scroll-outline.md).

Actualización 26/10/2026: `refine-notion-note-editor` implementado y verificado
localmente, con deltas pendientes de sincronización/archivo. Notes ofrece edición
contextual y los tres grupos de herramientas en la cabecera. Build y 59 checks
headless aprobados sobre SQLite sintético; sin aceptación nativa nueva.
[Guía](note-editor.md), [evidencia](verification-note-editor.md).
`add-local-meeting-notes` sigue propuesto 0/19.

Actualización 25/10/2026: `fix-ascii-resize-flicker`, archivado 2/2, corrige la desaparición de
los animales durante resize: repintado en el mismo callback y reloj continuo.
Compilación, siete pruebas existentes y doce comprobaciones headless aprobadas;
32/33 bitmaps vacíos antes y cero después. Sin cambios de specs (71 requisitos)
ni nueva aceptación nativa. [Evidencia](verification-ascii-resize-flicker.md).

Actualización anterior 25/10/2026: `align-titlebar-and-animate-sidebar` implementado y
verificado, sincronizado y archivado 3/3: centros verticales compartidos y lateral con transición reversible
de 260 ms/movimiento reducido. 56 checks headless y compilación aprobados;
posición Mac al próximo inicio, sin nueva aceptación nativa.
[Evidencia](verification-titlebar-alignment.md). Desktop conserva 71 requisitos;
Meeting Notes sigue propuesto 0/19.

Actualización anterior 25/10/2026: `unify-navigation-appearance` implementado, verificado,
sincronizado y archivado 4/4; iconos primarios compartidos, color local en Preferences e interfaz en
inglés. Ver [revisión y límites](verification-appearance.md). Meeting Notes sigue
propuesto 0/19.

Este repositorio usa el schema `spec-driven`. OpenSpec separa specs vigentes de cambios propuestos; no reemplaza pruebas, documentación de desarrollo ni decisiones arquitectónicas. [Documentación oficial](https://github.com/Fission-AI/OpenSpec/blob/main/docs/getting-started.md).

## Preparación inicial y estado actual

**Estado vigente al 25/10/2026:** `add-local-meeting-notes` está propuesto con
diseño/modelos, **0/19 tareas de implementación**; el usuario pidió revisar primero
esa definición. No aplicar ni sincronizar sus deltas como comportamiento vigente.
[Diseño](meeting-notes-design.md), [modelos](meeting-notes-models.md) y
[ADR0046 propuesto](decisions/0046-local-meeting-notes-runtime.md).

Settings (25/10): [`add-sidebar-settings-popup`](../openspec/changes/archive/2026-10-25-add-sidebar-settings-popup/proposal.md)
implementado, verificado, sincronizado y archivado **3/3**. Desktop tiene 68 requisitos.
Botón/engranaje, lateral simplificado y popup amplio con navegación/búsqueda,
Preferencias y General del workspace. Compilación y 33 comprobaciones headless
aprobadas; guardias/buffers conservados y sin nueva aceptación nativa.
[Guía](settings.md) y [evidencia](verification-settings.md).

Dispersión ASCII (25/10): [`spread-ascii-ocean-paths`](../openspec/changes/archive/2026-10-25-spread-ascii-ocean-paths/proposal.md)
implementado, verificado, sincronizado y archivado **3/3**. Recorridos por zonas/alturas distintas, sin cambios de estilo o
cadencia; catorce checks, compilación y revisión headless aprobados.
[Evidencia](verification-ocean-spread.md). Desktop conserva 67 requisitos y
Meeting Notes permanece propuesto 0/19.

Nado ASCII día/noche (24–25/10): [`refine-shark-swimming-reference`](../openspec/changes/archive/2026-10-25-refine-shark-swimming-reference/proposal.md)
implementado, verificado, sincronizado y archivado **4/4**. Esqueleto articulado de longitud estable, normales/aletas
solidarias, giro suave, profundidad y atlas por tema. Catorce checks existentes,
compilación y revisión headless aprobados; ≈60 FPS observados y p95 Canvas hasta
2,1 ms, sin aceptación física de 120 Hz. [Guía](ascii-ocean-cover.md) y
[evidencia](verification-shark-swimming.md). Desktop conserva 67 requisitos;
Meeting Notes permanece propuesto 0/19.

Referencia previa de peces ASCII: [`match-ascii-shark-day-reference`](../openspec/changes/archive/2026-10-24-match-ascii-shark-day-reference/proposal.md), 4/4;
un requisito modificado, cuatro escenarios anteriores conservados y uno añadido,
sincronizado/archivado. Desktop mantiene 67 requisitos. [Guía](ascii-ocean-cover.md)
y [revisión](verification-shark-day-reference.md). Catorce checks y UI compilada,
con diversidad de mallas, celdas sombreadas, pausas y cadencia anteriores intactas.
Meeting Notes sigue propuesto 0/19.

Base anterior del océano ASCII: [`refine-ascii-ocean-cover`](../openspec/changes/archive/2026-10-24-refine-ascii-ocean-cover/proposal.md), 4/4;
un requisito modificado con escenarios anteriores conservados, sincronizado y
archivado. Desktop mantiene 67 requisitos. [Guía](ascii-ocean-cover.md) y
[revisión](verification-ascii-ocean-cover.md). Hasta 120 FPS programados, reloj
controlado y dibujo medidos por separado de pantalla física; doce checks y UI
compilada aprobados. Sin datos, contratos, Home o dependencias modificados.

Detalles Goal/Person: [`match-goal-and-person-detail-reference`](../openspec/changes/archive/2026-10-24-match-goal-and-person-detail-reference/proposal.md), 4/4;
dos requisitos añadidos sincronizados y cambio archivado. Desktop tiene 67
requisitos. [Guía](goal-person-detail.md) y
[revisión](verification-goal-person-detail.md). Sin ampliar cuerpos/modelos,
operaciones ni datos; compilación y recorrido UI sobre copia, sin suites ni
persistencia/IPC nativo. Meeting Notes permanece propuesto 0/19.

Detalle Tags: [`match-tag-detail-reference`](../openspec/changes/archive/2026-10-24-match-tag-detail-reference/proposal.md), 3/3;
dos requisitos añadidos sincronizados y cambio archivado. Desktop cerró ese cambio con 65
requisitos. [Guía](tag-detail.md) y [revisión](verification-tag-detail.md).
Compilación y recorrido UI de borrador sobre copia; sin suites, cambios de datos,
proveedores o aceptación nativa.

Herramientas/Project: [`refine-page-tools-and-project-detail`](../openspec/changes/archive/2026-10-24-refine-page-tools-and-project-detail/proposal.md), 4/4;
tres requisitos añadidos sincronizados y cambio archivado. Desktop cerró ese cambio con 63
requisitos. [Inventario](page-tools.md), [detalle](project-detail.md),
[ADR0049](decisions/0049-page-tool-groups-and-project-detail.md) y
[revisión UI](verification-project-detail-tools.md). Compilación, teclado/paneles,
temas y 420 px sobre copia; sin operaciones de proveedor ni persistencia/IPC nativo.

Detalle Notes: [`match-note-detail-reference`](../openspec/changes/archive/2026-10-24-match-note-detail-reference/proposal.md), 4/4; dos requisitos añadidos sincronizados
y cambio archivado. Desktop cerró ese cambio con 60 requisitos y block-editor 5. Propiedades
locales compactas, prefijo Notes conservado y cuerpo con inicio visible;
compilación/revisión del borrador en copia, sin suites ni guardado/IPC nativo.
[Guía](note-detail.md) y [revisión](verification-note-detail.md).

Contenido Task: [`add-task-document-editing`](../openspec/changes/archive/2026-10-24-add-task-document-editing/proposal.md), 4/4; tres deltas sincronizados y
cambio archivado. Desktop cerró ese cambio con 59 requisitos. Documento opcional, inicio visible del editor
y detalle Task, sin migración ni nuevas operaciones. [Guía](task-page-content.md)
y [revisión acotada](verification-task-page-content.md).

Barra superior: [`integrate-native-window-titlebar`](../openspec/changes/archive/2026-10-24-integrate-native-window-titlebar/proposal.md), 3/3;
dos requisitos sincronizados y cambio archivado. Desktop cerró ese cambio con 58 requisitos. Una fila compartida, lateral
plegable, overlay Mac y controles Windows con validación nativa diferida.
[Guía](window-titlebar.md) y [revisión](verification-window-titlebar.md).

People lateral: [`match-people-page-reference`](../openspec/changes/archive/2026-10-24-match-people-page-reference/proposal.md), **3/3 tareas**, sincronizado y archivado.
Un requisito añadido, 55 previos y 16 otras specs preservados. Compilación y
recorrido manual en copia local; sin cambios Rust/datos ni aceptación nativa nueva.
[Guía](people-page.md) y [revisión](verification-people-page.md).

Las fases 0–4 y búsqueda textual están archivadas; consultar las verificaciones y [desarrollo](development.md). Templates/bases incrustadas cerró **17/17 tareas**, con SQLite v7/documento v3 y [166 checks nativos offline](verification-phase-3c-templates.md). [`add-collection-templates-and-embeds`](../openspec/changes/archive/2026-10-12-add-collection-templates-and-embeds/proposal.md) está sincronizado y archivado.

Home quedó **archivado 17/17** como `2026-10-22-add-local-home-dashboard`, con sus once requisitos sincronizados, 592 checks headless y **592 checks nativos** en seis sesiones Tauri. [Verificación](verification-home.md) y [revisión](review-home.md). En ese cierre no quedaron cambios activos.

El ajuste [`refine-desktop-navigation-and-tabs`](../openspec/changes/archive/2026-10-23-refine-desktop-navigation-and-tabs/proposal.md)
cerró **8/8 tareas** el 23/10: menú de seis entradas, buscador compartido,
pestañas independientes, listados centrales, detalles compactos y portada local.
Dos deltas sincronizados; 96 tests puros, 102 checks nuevos, 1.248 de búsqueda
y 592 compartidos + 14 de teclado aprobados headless con CLI/SQLite reales.
El cierre efectivo de Tauri no se verificó en esta entrega. [Guía](desktop-navigation.md)
y [evidencia](verification-desktop-navigation.md). En ese cierre no quedaron cambios activos.

Goals: [`add-goals-sidebar-and-milestones`](../openspec/changes/archive/2026-10-24-add-goals-sidebar-and-milestones/proposal.md), **4/4 tareas**, sincronizado y archivado.
Tres deltas: subtipos y recuperación, consultas/progreso/actividad y navegación/UI.
Seis requisitos añadidos y tres vigentes reconciliados, sin migración ni operación
pública nueva. [Guía](goals.md), [ADR0045](decisions/0045-goals-as-typed-pages.md)
y [revisión acotada](verification-goals.md); sin suites ni aceptación nativa nueva.

Página Tags: [`match-tags-page-reference`](../openspec/changes/archive/2026-10-24-match-tags-page-reference/proposal.md), **3/3 tareas**, sincronizado y archivado. Dos requisitos desktop-shell, sin cambios Rust,
contratos, migración ni datos. [Guía](tags-page.md) y
[revisión acotada](verification-tags-page.md); sin suites ni aceptación nativa nueva.

Página Projects: [`match-projects-page-reference`](../openspec/changes/archive/2026-10-24-match-projects-page-reference/proposal.md), **4/4 tareas**, sincronizado y archivado. Dos requisitos desktop-shell, tres vistas de lectura nuevas
y contratos aditivos, sin migración ni cambios de datos. [Guía](projects-page.md),
[ADR0044](decisions/0044-projects-page-and-relation-projections.md) y
[revisión acotada](verification-projects-page.md); sin suites ni nueva aceptación nativa.

Página Notes: [`match-notes-page-reference`](../openspec/changes/archive/2026-10-24-match-notes-page-reference/proposal.md), **3/3 tareas**, sincronizado y archivado. Dos requisitos desktop-shell,
sin migración ni cambios de datos. [Guía](notes-page.md) y
[revisión acotada](verification-notes-page.md); sin suites ni aceptación nativa nueva.

Tasks, breadcrumb y acciones de fila: [`match-tasks-page-reference`](../openspec/changes/archive/2026-10-24-match-tasks-page-reference/proposal.md), **5/5 tareas**, sincronizado y archivado. Cinco requisitos desktop-shell,
sin migración ni modificaciones de datos personales. [Guía](tasks-page.md) y
[revisión acotada](verification-tasks-page.md): compilación e inspección manual,
sin ejecutar suites ni nueva aceptación nativa.

Encabezado Home: [`simplify-home-header-and-refresh`](../openspec/changes/archive/2026-10-24-simplify-home-header-and-refresh/proposal.md), **2/2 tareas**, sincronizado y archivado. Se retiran
Navegación/Actualizar Home y se consulta con ⌘R/Ctrl+R o F5.
[Revisión acotada](verification-home-refresh.md).

Ajuste de People: [`match-home-people-reference`](../openspec/changes/archive/2026-10-24-match-home-people-reference/proposal.md), **4/4 tareas**, sincronizado y archivado. Galería/fotos
locales y siete vistas compartidas. Recuperación con backup e identidad/documentos
conservados. [Guía](people-reference.md) y [revisión](verification-people-reference.md).
Compilación e inspección headless; no se ejecutan suites ni aceptación nativa nueva.

Corrección de carga de animación: [`fix-sidebar-animation-dev-loading`](../openspec/changes/archive/2026-10-23-fix-sidebar-animation-dev-loading/proposal.md), **2/2 tareas**, archivado con `skip_specs: true`.
Corrige el Vite de desarrollo sin cambiar el requisito vigente;
[causa y revisión geométrica](verification-icon-motion-dev-fix.md).

Catálogo y movimiento de iconos: [`adopt-static-animateicons`](../openspec/changes/archive/2026-10-23-adopt-static-animateicons/proposal.md), **3/3 tareas**, sincronizado y archivado.
SVG seleccionados estáticos y reproductor diferido sólo al clic lateral.
[Guía](icons.md) y [revisión](verification-static-animateicons.md).

Movimiento de Projects: [`enable-project-board-dragging`](../openspec/changes/archive/2026-10-23-enable-project-board-dragging/proposal.md), **3/3 tareas**, sincronizado y archivado.
Arrastre/diálogo y persistencia atómica con límites explícitos.
[Guía](project-board-movement.md) y [revisión](verification-project-board-movement.md).

Ajuste de Tags: [`match-home-tags-reference`](../openspec/changes/archive/2026-10-23-match-home-tags-reference/proposal.md), **4/4 tareas**, sincronizado y archivado.
Cuatro vistas, Entity adicional y resúmenes/recuperación acreditados.
[Guía](tags-reference.md) y [revisión](verification-tags-reference.md).

Ajuste de Projects: [`match-home-project-reference`](../openspec/changes/archive/2026-10-23-match-home-project-reference/proposal.md), **4/4 tareas**, sincronizado y archivado.
Tres requisitos desktop-shell y uno knowledge-entities sincronizados. Board de
cinco columnas, proyecciones Rust, relaciones e iconos acreditados con backup.
[Guía](projects-reference.md) y [revisión acotada](verification-projects-reference.md).
Compilación e inspección visual; no se repitieron suites ni aceptación nativa.

Ajuste de Notes: [`match-home-notes-reference`](../openspec/changes/archive/2026-10-23-match-home-notes-reference/proposal.md),
**5/5 tareas**, cuatro requisitos sincronizados y cambio archivado.
Clasificación/procedencia opcionales en Rust, ocho vistas y recuperación explícita
con backups, sin modificar documentos. [Guía](notes-reference.md) y
[revisión](verification-notes-reference.md). Las suites previas conservan su alcance.

Ajuste visual posterior de Tasks: [`match-home-task-reference`](../openspec/changes/archive/2026-10-23-match-home-task-reference/proposal.md),
**7/7 tareas**, cuatro requisitos desktop-shell sincronizados. Iconos de imagen
locales, seis vistas de tareas en Rust y recuperación explícita de14 vínculos,
con backup/trazabilidad. [Alcance de revisión](verification-task-reference.md):
compilación e inspección visual, sin suites ni aceptación nativa repetidas.

Cambio independiente cerrado: [`extend-agent-operations`](../openspec/changes/archive/2026-10-13-extend-agent-operations/proposal.md), **22/22 tareas**, verificado, sincronizado y archivado. Catálogo/CLI de 40 operaciones con preview, apply atómico, bindings y construcción de suscripción por lote. [Evidencia](verification-phase-4.md) y [contrato](batches.md); delta core-operations de diez requisitos/29 escenarios comparado íntegramente con main, sin cambiar otras specs. Su cierre no modifica los criterios nativos pendientes de Home.

Cambio independiente cerrado: [`add-local-extraction-queue`](../openspec/changes/archive/2026-10-13-add-local-extraction-queue/proposal.md), **20/20 tareas**, SQLite v8/API v1/48 operaciones; tres deltas sincronizados y archivado. [Cola local/UTF-8 literal](extraction.md), [evidencia](verification-extraction.md) y [ADR 0011](decisions/0011-explicit-local-extraction.md). Recorte de 5a, no PDF/Excel/OCR/indexación ni cierre completo; no basta para toda dependencia de 5b. Home permanece intacto sin sincronizar/archivar.

Cambio PDF cerrado: [`add-local-pdf-text-extraction`](../openspec/changes/archive/2026-10-14-add-local-pdf-text-extraction/proposal.md), **21/21 tareas**, tres deltas sincronizados y archivado; SQLite v9/API v1/49 operaciones. [Guía](pdf-extraction.md), [evidencia](verification-pdf-extraction.md), [ADR 0012](decisions/0012-bounded-pdf-worker.md). Corpus propio de 235 casos por parser/CLI reales, migración/backup/procesos y mediciones locales. No recrear propuesta ni volver al baseline experimental; sin UI/OCR/indexación/modelos ni dependencia de Home nativo. Home permanece activo 9/17.

Cambio cerrado: [`add-local-attachment-search`](../openspec/changes/archive/2026-10-14-add-local-attachment-search/proposal.md), **19/19 tareas**, tres deltas sincronizados y contrastados íntegramente con main: 15 requisitos añadidos/40 escenarios nuevos y un requisito de migración reconciliado/cinco escenarios heredados. Archivado el 14/10/2026; SQLite v10/API v1/51 con índice/búsqueda explícitos de generaciones UTF-8/PDF; [guía](attachment-search.md) y [ADR 0013](decisions/0013-explicit-attachment-index.md). [Evidencia](verification-attachment-search.md): 212 tests Rust por perfil, 20 rondas CLI por perfil, 606 checks headless y dos benchmarks con query p95 **214,20–231,07 ms** sobre el corpus fijado frente a <=300 ms; lock aprobado y scratch físico medido. Se preservan baselines fallidos y validación completa sin cambiar caps/target. No declara 5a/5b completas ni resultados universales para cualquier corpus/caché. Home sigue 9/17 sin modificar.

Cambio independiente cerrado: [`add-local-mcp`](../openspec/changes/archive/2026-10-15-add-local-mcp/proposal.md), **22/22 tareas**, dos deltas sincronizados y contrastados con main; archivado el 15/10/2026. Scope, binario stdio, **235 tests Rust por perfil**, 17 SDK por perfil, tres rondas raw por perfil y 606 checks headless aprobados; [evidencia/límites](verification-mcp.md), [tareas](../openspec/changes/archive/2026-10-15-add-local-mcp/tasks.md). Dos gates de consulta p95 **134,96–204,81 ms**, con validación/caps/target intactos y fallos anteriores conservados. Home permanece independiente 9/17, sin modelos/proveedores ni pantalla.

- `docs/plan-noteka.md` contiene la visión completa y la hoja de ruta autosuficiente.
- `openspec/config.yaml` contiene contexto y reglas para generar artefactos.
- `openspec/changes/archive/` conserva los cambios verificados, incluidos motor de colecciones, fórmulas, vistas avanzadas, templates/bases incrustadas, lotes, extracción literal, PDF textual acotado, índice/búsqueda de adjuntos y MCP local.
- `openspec/specs/` contiene trece capabilities del comportamiento entregado. Home conserva deltas activos, no sincronizados; MCP tiene ambos deltas sincronizados y cambio archivado. El [lector XLSX independiente](../openspec/changes/archive/2026-10-15-add-bounded-xlsx-reader/proposal.md) cerró **8/8 tareas**, seis requisitos/dieciséis escenarios sincronizados y cambio archivado como recorte previo a integrar la cola, sin modificar v10/API v1/51 ni anunciar soporte Excel en el producto.
- [`add-local-xlsx-extraction`](../openspec/changes/archive/2026-10-15-add-local-xlsx-extraction/proposal.md)
  cerró **11/11 tareas**: cuatro deltas/14 requisitos/44 escenarios sincronizados
  y contrastados con main; archivado el 15/10/2026. Migración/recuperación v11 y lector API v1/52/CLI/MCP
  24/44/45 según ADR 0017. [Aceptación final3](verification-xlsx-extraction.md):
  261 tests Rust por perfil, corpus XLSX/PDF, procesos/SDK, 606 checks headless y
  dos gates FTS intactos query p95 111,15–146,42 ms. Home permanece 9/17 independiente.
- Cambio cerrado: [`add-local-xlsx-search`](../openspec/changes/archive/2026-10-16-add-local-xlsx-search/proposal.md),
  **19/19 tareas**, cinco deltas/14 requisitos/49 escenarios sincronizados y
  contrastados con main; 32 escenarios heredados/75 requisitos no afectados
  conservados. Archivado el 16/10, baseline vigente v12/API v1/52, proyección
  explícita de campos sin helpers. [Evidencia](verification-xlsx-search.md) y
  [guía](xlsx-search.md): 287 Rust por perfil/606 checks headless, dos gates
  mixtos 167,02–270,74 ms y dos intactos 114,31–152,19 ms p95. Se respetaron
  caps/validación/target de 300 ms. La [revalidación alternativa autorizada](xlsx-search-evidence-recovery.md)
  conserva la limitación de comparación exacta v11 no repetible, sin fabricar un
  baseline independiente. Home continúa activo 9/17, sin sync/archivo ni pantalla.
- Cambio cerrado: [`add-local-evidence-search-ui`](../openspec/changes/archive/2026-10-16-add-local-evidence-search-ui/proposal.md),
  **17/17 tareas implementadas/verificadas**. Un delta `local-text-search`, un
  requisito modificado y seis añadidos/diecinueve escenarios para exponer evidencia
  de adjuntos ya indexados desde Search, separada de notas. Fixture/controlador
  verificados; 1.248 checks UI nuevos y 606 previos y regresión Rust/MCP aprobados;
  cuatro gates y revisión documental aprobados, primer fallo conservado. Único
  delta sincronizado/contrastado y cambio archivado el 16/10; trece requisitos ajenos
  y el escenario heredado preservados. No recrear ni resincronizarlo.
  [Evidencia](verification-evidence-search-ui.md). La validación de artefactos por sí sola es documental. Conserva
  v12/API v1/52, los gates/caps y Home activo 9/17, sin pantalla/modelos/remotos.
- `.agents/skills/` contiene seis workflows generados para Codex: explore, propose, apply, update, sync y archive. Otros agentes pueden leer los mismos artefactos y usar el CLI.

Inicialización verificada con OpenSpec **1.14.1**. Para usar la misma versión sin modificar una instalación global:

```sh
npx --yes @fission-ai/openspec@1.14.1 context --json
npx --yes @fission-ai/openspec@1.14.1 list
npx --yes @fission-ai/openspec@1.14.1 list --specs
npx --yes @fission-ai/openspec@1.14.1 list --json
npx --yes @fission-ai/openspec@1.14.1 validate --all --strict
```

`npx` puede necesitar red la primera vez para obtener la herramienta. Este detalle afecta al tooling de desarrollo; no altera el requisito de funcionamiento offline del producto.

## Flujo por cambio

1. Leer contexto, plan, specs vigentes y cambios activos. Si falta una decisión material, explorarla antes de especificar.
2. Proponer una unidad coherente de trabajo con nombre descriptivo. Consultar `openspec instructions <artefacto> --change <nombre> --json` antes de crear cada artefacto y respetar dependencias.
3. Escribir `proposal.md`, deltas en `specs/<capability>/spec.md`, `design.md` cuando corresponda y `tasks.md`. Requirements observables con SHALL/MUST y escenarios `#### Scenario:` con WHEN/THEN; conservar headings estructurales en inglés y prosa española.
4. Revisar alcance, decisiones y escenarios; implementar por tareas usando apply cuando el usuario solicite desarrollo.
5. Verificar requisitos contra código, ejecutar pruebas pertinentes y completar tareas con evidencia. Actualizar planificación si cambian decisiones; no hacer pasar una ampliación del alcance por una tarea trivial.
6. Sincronizar deltas implementados a specs principales y archivar después de verificar. No usar archive para dar por completo un plan sin código.

Las skills de este proyecto se encuentran en `.agents/skills/`. En Codex Desktop se pueden seleccionar desde Skills; en interfaces compatibles se usa `$openspec-propose` o `$openspec-apply-change`. Los slash commands de otros agentes dependen de su integración; los archivos y el CLI no dependen del chat original.

El perfil generado no incluye una skill local `verify`; antes de cerrar un cambio se debe revisar igualmente completitud, corrección y coherencia con pruebas. Si se quiere ese workflow adicional, configurarlo explícitamente más adelante, sin suponer que ya está instalado aquí.

## Dónde va cada información

| Ubicación | Contenido |
| --- | --- |
| `docs/plan-noteka.md` | Objetivo, decisiones, arquitectura propuesta, requisitos de visión y fases. |
| `docs/development.md` | Comandos y prerequisitos realmente existentes para ejecutar, probar y construir. |
| `docs/decisions/` | ADRs de decisiones adoptadas con contexto, alternativas y consecuencias. |
| `openspec/specs/<capability>/spec.md` | Comportamiento implementado y verificado, una vez sincronizado. |
| `openspec/changes/<change>/` | Propuesta activa y su plan ejecutable; deltas añadidos/modificados/eliminados. |
| `openspec/changes/archive/` | Historial de cambios terminados. |

La organización por capabilities debe permanecer estable. Reutilizar sus nombres cuando cambien requisitos; no crear una capability por cada pantalla, archivo o tarea. Un roadmap amplio se divide por entregas; solo la próxima entrega necesita deltas ejecutables completos.

## Condición de cierre

Un cambio se puede considerar listo para archivar cuando sus tareas están realizadas, los escenarios tienen evidencia, el diseño coincide con el código, los checks relevantes pasaron y las limitaciones están documentadas. `status` detecta archivos; `validate` revisa forma y contenido documental. Ninguno demuestra por sí solo que funcione la app.

Cuando cambie una decisión de producto, revisar tanto los artefactos activos como el plan general. Conservar un único lugar principal para cada detalle; usar enlaces para evitar planes contradictorios.

**Tres recortes nuevos de 5a verificados el 17/10:**
[preparación explícita](attachment-preparation-ui.md),
[lectura completa por páginas](attachment-reader-ui.md) y
[consulta agrupada](grouped-search.md), con 9/9, 6/6 y 6/6 tareas verificadas,
respectivamente. [Aceptación conjunta](verification-local-preparation-closure.md):
3.116 checks headless, 287 Rust por perfil y cuatro gates frescos aprobados;
catálogo/schema/MCP/cuotas intactos. Los tres deltas están sincronizados/contrastados y los cambios archivados por separado.
OCR y ampliaciones PDF/Excel siguen pendientes; **5a todavía no está completa**.
Home permanece activo 9/17.

`add-local-image-ocr` tiene aceptación técnica aprobada. [Verificación](verification-image-ocr.md):
319 Rust por perfil,3442 checks headless,regresión completa en ambos perfiles y
cuatro gates finales aprobados sin modificar corpus/cuotas/validación/targets.
Cerrado12/12, sus cinco deltas están sincronizados/contrastados y
[archivados](../openspec/changes/archive/2026-10-17-add-local-image-ocr/tasks.md);
152 bloques no afectados y veinte nombres de escenarios heredados preservados. Home9/17 queda intacto; OCR de PDF/ampliaciones PDF/Excel y toda5a siguen
pendientes.


`add-bounded-pdfkit-reader` tiene aceptación técnica aprobada,6/6: lector privado
independiente,326 Rust por perfil,59 casos nativos por perfil y cuatro gates
consecutivos dentro de300/150ms. Dos fallos mixtos anteriores conservados, causa
no demostrada. Único delta sincronizado/contrastado y cambio archivado como2026-10-17-add-bounded-pdfkit-reader.
Ver [evidencia](verification-pdfkit-reader.md). No integra el perfil en SQLite13,
API53, MCP25/45/46 o UI y no completa5a; Home9/17 permanece separado.

**Integración PDFKit con aceptación8/8 (20/10):**
[guía](pdfkit-extraction.md)/[verificación](verification-pdfkit-extraction.md).
SQLite14/API54, MCP26/46/47, batch9; recuperación real13 en dos estados/perfiles,
339 Rust por perfil,25 SDK por perfil,3442 checks headless y cuatro gates originales
aprobados con máximos176,788/16,197ms. Cuatro deltas sincronizados/contrastados y cambio archivado como2026-10-20-add-local-pdfkit-extraction.
No cierra5a: UI/FTS PDFKit, OCR dePDF, tablas/ampliaciones PDF/Excel siguen pendientes.
Home9/17 intacto; la instrucción del20/10 sólo autoriza completar fase5 y detenerse.


**PDFKit en búsqueda/UI verificado (20/10):** [guía](pdfkit-search-ui.md) y
[evidencia](verification-pdfkit-search-ui.md). SQLite15/API54/MCP26/46/47,
batch9;345 Rust por perfil,25 SDK por perfil y4264 checks headless. Cuatro gates
originales aprobaron query p95 máximo174,526ms/lock17,094ms; prueba nativa grande
143,365/10,405ms. Cuatro deltas sincronizados/contrastados y cambio archivado10/10.
OCR dePDF, tablas y ampliaciones XLSX siguen pendientes de5a; Home9/17 intacto.


Estado20/10: `add-local-pdf-ocr` verificado12/12, cinco deltas sincronizados/contrastados
y archivado. Fuente vigente SQLite16/API55/MCP27/47/48/batch9; ver
[aceptación PDF OCR](verification-pdf-ocr.md). Fase5a aún requiere tablas/ampliaciones PDF y XLSX;
5b/5c no iniciados, Home9/17 intacto y fase6 excluida. Sin publicación ni modelos.


`add-local-xlsx-range-reader`21/10 tiene aceptación6/6 (SQLite16/API56/MCP28/48/49)
y sync/archivo sólo tras contraste de sus cuatro deltas. Ver [evidencia](verification-xlsx-range-reader.md).
El proyector PDF independiente es otro cambio activo; Home9/17 no se toca.


`add-bounded-pdf-table-projector` tiene aceptación independiente6/6, sin operación
pública o cambios de formato. [Evidencia](verification-pdf-table-projector.md).
Sólo su delta local-extraction se sincroniza tras contraste; tablas PDF requieren
integración propia y5a permanece incompleta.

## Cierre de fase 5 (22/10/2026)

`add-semantic-retrieval` (23/23) y `add-agent-harness` (20/20) están
implementados, verificados, sincronizados y archivados como
`2026-10-22-add-semantic-retrieval` y `2026-10-22-add-agent-harness`. Aportaron
nueve requisitos a `semantic-retrieval`, cinco a `agent-harness`, uno a
`core-operations` por cada cambio y uno a `local-workspace` por cada cambio.
Las specs principales tienen ahora **17 capabilities/257 requisitos** y pasan
validación estricta. Ver [5b](verification-semantic-retrieval.md),
[5c](verification-agent-harness.md), el cierre 6b y ADR0029/0030.

SQLite18/API62/MCP32/53/55/batch9. No se instalaron pesos de Jina ni se activó
un modelo/proveedor; las fixtures no acreditan calidad real. Home permanece9/17
y no se inició fase6. No recrear ni resincronizar estos cambios.

## Cierre 6b (22/10/2026)

`add-google-drive-references` quedó **10/10**, sincronizado y archivado como
`2026-10-22-add-google-drive-references`. Creó la capability `google-drive` con
seis requisitos: OAuth de escritorio, Picker con `drive.file`, asociaciones
externas genéricas, listado paginado, preview read-only Docs/Sheets y listado
Drive acotado. SQLite20/API66/MCP34/57/59/batch9; 433 tests Rust/16 ignorados y
17 capabilities/257 requisitos. La UI permite vincular, previsualizar, abrir y
desvincular. No se usaron scopes restringidos ni se persiste contenido de
preview. Ver [verificación Drive](verification-google-drive.md),
[recursos externos](external-resources.md) y ADR0032/0033.


## Cierre 6a (22/10/2026)

`add-cloud-model-providers` quedó **9/10** y archivado como
`2026-10-22-add-cloud-model-providers`. Aportó la capability `cloud-providers`
con tres requisitos (ChatGPT, API keys/proveedores explícitos y proxy loopback).
La tarea 2.3 de keys/cuotas reales de OpenAI/Gemini queda **diferida por decisión
del usuario** y no bloquea el archivo; las pruebas locales cubren selección
explícita, ausencia de fallback, timeout, errores estructurados y no exposición de
tokens. Las specs principales quedan en **17 capabilities/257 requisitos**.
