# Cómo desarrollar Noteka con OpenSpec

Este repositorio usa el schema `spec-driven`. OpenSpec separa specs vigentes de cambios propuestos; no reemplaza pruebas, documentación de desarrollo ni decisiones arquitectónicas. [Documentación oficial](https://github.com/Fission-AI/OpenSpec/blob/main/docs/getting-started.md).

## Preparación inicial y estado actual

Las fases 0–4 y búsqueda textual están archivadas; consultar las verificaciones y [desarrollo](development.md). Templates/bases incrustadas cerró **17/17 tareas**, con SQLite v7/documento v3 y [166 checks nativos offline](verification-phase-3c-templates.md). [`add-collection-templates-and-embeds`](../openspec/changes/archive/2026-10-12-add-collection-templates-and-embeds/proposal.md) está sincronizado y archivado.

Cambio activo de Home: [`add-local-home-dashboard`](../openspec/changes/add-local-home-dashboard/proposal.md), código implementado y **9/17 tareas completamente verificadas**. Dos deltas contienen once requisitos/veintiséis escenarios; todavía no sincronizados ni archivados. [Verificación](verification-home.md): 81 tests Rust, 13 adaptador/cola/tema, 3 aislamiento/guardia y 606 checks headless aprobados; [revisión final cerrada](review-home.md), ocho tareas con aceptación nativa pendientes. No usar la pantalla sin consentimiento específico ni atribuir al puente CLI pruebas de IPC/cierre del host.

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
