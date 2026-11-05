# Verificación de acciones de captura

01/11/2026. Se retiraron los botones de captura **New Task**, **New Note** y
**New Project** de las secciones de Tasks, Notes y Projects, tanto en las páginas
como en Home. La creación sigue disponible una sola vez en el botón **New** de
la barra de vistas; las filas contextuales `New page`/`New task` dentro de grupos
se conservan.

La acción `New` de la barra usa un vidrio teñido con el color primario elegido
(`--accent`, azul por defecto): `color-mix` al 85%, sheen, sombra teñida, radio
de cápsula y texto blanco. En Chromium y WebKit, el recorrido comprobó que
Tasks, Notes y Projects no renderizan `.task-capture`, que su `.task-new` sigue
visible y que el botón tiene `background-image` con gradiente,
`border-radius: 999px`, sombra, un fondo azul (canal B mayor que R) y color
blanco. Build
sin diagnósticos Svelte. El resto del recorrido de tareas (checkbox All/Inbox,
persistencia y refresco) siguió pasando. No se inspeccionó la ventana nativa ni
se midió CPU/GPU. Tags y People conservan sus botones New Tag/New Person y el
detalle de proyecto conserva sus New `primary` de acento, fuera de este recorte.

Las barras de vistas de sección se normalizaron al ritmo de Tags:
`padding: 0 0 13px`, `margin: 0`, `gap: 8px`. Las secciones sin bloque de
captura (Tasks, Notes, Projects y Goals) reservan `padding-top: 44px`, el alto
del bloque `.task-capture` de Tags/People, para que la fila de vistas quede a la
misma altura. Esto reemplaza los márgenes particulares de Goals (24px),
Milestones (18px), la subsección Projects (10px) y el gap de People (12px). En
Chromium y WebKit, el recorrido midió `tabsTop: 361` en las seis secciones y
verificó que la subsección Projects conserva `0 / 13px / 0 / 0 / 8px`; las
capturas de Notes y Tags muestran la fila alineada. Milestones comparte el
ritmo pero no la reserva, por ser una subsección. Build sin errores. La ventana
nativa se inspeccionó después de HMR y mostró la fila alineada en Notes y Tags;
no se midió CPU/GPU.

En Home, las seis secciones (Tasks, Notes, Projects, Tags, Goals y People)
comparten `padding: 10px 0 8px`, `margin: 0 0 5px` y `gap: 8px`. El recorrido
midió `tabsTop = headerBottom + 10` en todas, en Chromium y WebKit. Esto elimina
el `padding-top: 2px` de Notes/Tasks/Projects/Goals/People, el margen inferior
de 24px de Goals y el gap de 12px de People; Tags no cambió. La ventana nativa
se inspeccionó después de HMR y mostró el mismo espaciado en Notes y Tags.
