# Design

## Context

Las secciones de Tasks, Notes y Projects reutilizan `HomeTaskSection`,
`HomeNoteSection` y `HomeProjectSection`. Cada una renderizaba un bloque
`.task-capture` con una acción de creación y, más abajo, un botón `New` en
`.task-view-actions`.

## Decisions

Eliminar el bloque `.task-capture` de las tres secciones y dejar la creación en
el botón `New` de la barra de vistas, que ya usa las mismas guardias y callbacks.
El botón `New` usa un vidrio teñido con el color primario elegido (`--accent`,
azul por defecto) al 85%, sheen, sombra teñida, radio de cápsula y texto blanco;
con transparencia reducida vuelve al fondo sólido `--accent` con texto blanco. No tocar las filas `New
page`/`New task` de grupos ni los botones New Tag/New Person. El CSS de
`.task-capture` se conserva porque Tags y People todavía lo usan. Las barras de
vistas de las páginas de sección comparten el ritmo de Tags: `padding: 0 0 13px`,
`margin: 0`, `gap: 8px`. Las secciones sin captura reservan `padding-top: 44px`
para alinear la fila de vistas con Tags/People; la subsección Projects y
Milestones usan el ritmo sin esa reserva y se retiran sus márgenes particulares.
En Home, todas las secciones usan `padding: 10px 0 8px`, `margin: 0 0 5px`,
`gap: 8px`; esto reemplaza el `padding-top: 2px` base, el margen inferior de
24px de Goals y el gap de 12px de People.

## Risks / Trade-offs

La acción de creación queda sólo en la barra de vistas; en ventanas angostas el
toolbar puede envolver, pero el botón `New` permanece visible. No cambia el
comportamiento de captura.
