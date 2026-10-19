# Design

## Context

Ver proposal y `docs/search.md`, `docs/evidence-search-ui.md`. Los dos motores
existen en API v1/52 y no comparten scores/cursor/snapshot. Adjuntos sólo busca
selecciones; el filtro archived/contexto pertenece a entidades. Baseline durable
454 archivos: `~/.local/share/noteka-verification/active-unified-search-path.txt`.
Prep está en aceptación; lector está implementado aisladamente. La integración se revalida junto con las UIs previas, sin cambiar Rust/CLI/corpus en ejecución.

## Goals / Non-Goals

**Goals:** una entrada literal para obtener dos grupos de evidencia local, con
paginación/errores independientes y apertura de notas usando App.

**Non-Goals:** ver proposal; no afirmar recuperación híbrida, ranking común,
consulta atómica, archivos/contexto de adjuntos ni inclusión de todo lo almacenado.

## Decisions

1. Tercera opción «Notas y adjuntos (grupos)» conserva ambos ámbitos individuales.
Un botón dispara search.query con archived false/limit 20 y extraction.search con
limit 20, sin filtros que no tengan semántica común. Los filtros detallados quedan
en sus ámbitos; no se construye una unión SQL ni índice duplicado.
2. Reutilizar AttachmentReader sin modificarlo y añadir lector de notas acotado
con lifetime/cupo propio uno. La consulta inicia ambas lecturas; cada una publica
su resultado/error sin esperar la otra y se rotula como snapshot independiente.
El cupo existente de adjuntos sigue compartido con su ámbito individual; el nuevo
cupo de notas se comparte entre montajes conjuntos. Un IPC iniciado no se cancela.
3. Un coordinador conserva el texto solicitado sólo en memoria; editar invalida
ambos lectores y lo retira. Siguiente/reintentar/inicio actúan sólo sobre su grupo.
No hay fan-out por hit ni fetching de originales. No se omiten hits para rellenar
errores ni se acumulan páginas. No comparar cantidades del índice con hits.
4. Notas presenta datos del SearchHit y delega abrir a App.openSearch por callback.
Adjuntos muestra texto completo/flag previo, procedencia/localizador y cobertura
completos, con JSON literal en detalles para no inventar metadata. La presentación
no abre un archivo ni ejecuta preparación. Todo contenido es escapado por Svelte.
5. Abrir otra nota no cambia la identidad de una búsqueda global; conserva sus grupos y usa la guardia. Cambiar sección/ámbito/workspace sí descarta el contexto. La integración toca sólo Search y sus ramas de loadList/refresh/montaje; no
modifica Home/editor/guardias. La aceptación de App se repite conjuntamente después de integrar cada panel.
El panel puede probarse aislado con CLI real/golden sin presentarlo como integración.
6. Cliente prueba secuencias/páginas/fallos parciales, cola/remount y consulta exacta.
Headless compara ambos grupos con CLI, fuente hostil, cobertura, índice concurrente
y guardias. Se reutiliza regresión/gates recientes sólo con hashes completos de
core/contratos/binarios/corpus iguales y alcance documentado; cambios o dudas
materiales fuerzan la verificación afectada. No es nueva evidencia histórica v11.

## Risks / Trade-offs

- [Nombre conjunto sugiere universo] → alcance visible «entidades activas» y
«generaciones seleccionadas», sin total global ni ranking común.
- [Una consulta falla] → estado/error por grupo; el otro queda como su snapshot,
sin ocultar la falla ni prometer un estado conjunto actual.
- [Cursor/envelope incompatible] → retirar el grupo entero y pedir acción
explícita; validación/cuotas Rust completas se conservan.
- [Callback abre durante buffer] → misma guardia App, sin escribir desde el panel.

## Migration Plan

Sin migración, contrato, dependencia ni índice nuevo. Revertir la opción/panel
conserva fuentes/selecciones y ámbitos individuales. Sync modifica un requisito
preservando sus tres escenarios y añade tres requisitos/siete escenarios nuevos.
No tocar otros requisitos ni sincronizar/archivar Home.
