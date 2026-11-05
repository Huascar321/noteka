# Design

## Context

`App.svelte` marca `blocked` mientras la búsqueda está abierta para impedir ediciones y refrescos del fondo. `HomePanel` usaba `!blocked` para activar `AsciiShark`, así que la portada se detenía. El `<dialog showModal>` atrapa el foco pero no cierra al hacer clic en el backdrop.

## Goals / Non-Goals

Mantener la animación ASCII y los controles del fondo protegidos; cerrar con clic externo; conservar Escape y Close. No cambiar resultados, datos ni el material visual.

## Decisions

- Pasar `searchOpen` desde `App` a `WorkspacePane` y `HomePanel`; la condición del canvas queda `visible && (!blocked || searchOpen) && (active || animateAlongsideSidePeek)`.
- Conservar `blocked` para el resto de los controles.
- En `<dialog>`, llamar a `onclose()` sólo cuando `event.target === dialog`; los clics en hijos no cierran.
- Escape y el botón Close no cambian.

## Risks / Trade-offs

Un clic sobre el borde de 1px del diálogo también cuenta como externo; es aceptable. La animación sigue consumiendo RAF debajo del modal, aunque el canvas ya estaba activo; con movimiento reducido permanece suspendida.
