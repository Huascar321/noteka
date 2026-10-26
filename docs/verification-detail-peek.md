# Verificación de los modos de detalle

Fecha: 26/10/2026. Cambio: `add-detail-peek-modes`. Estado: implementado y
verificado localmente; deltas pendientes de sincronización y archivo.

## Scope and method

App compilada en Chromium headless propio con transporte IPC de prueba hacia el
CLI Rust real y SQLite sintético confinado. No se usó un workspace personal, la
pantalla del usuario ni un proveedor externo. Las señales pending_edits,
close_confirmed y el evento del host se observaron mediante shim: esto verifica
el recorrido de protección del shell, no el cierre físico/IPC de una ventana Tauri.

## Results

- `npm run build`: iconos/contratos vigentes, cero errores/advertencias Svelte y
  bundle generado. Continúa el aviso previo de tamaño del chunk principal.
- **100 comprobaciones focalizadas**: 38 de modos/edición/guardias, 11 de
  preferencias/teclado, 42 de captura/cierre/layout y 9 de foco/integración con
  outline/atajo de guardado/cierre resuelto. Evidencia privada e ignorada en
  `output/playwright/detail-peek-20261026/`, scripts, JSON y capturas.
- Side peek a la derecha desde una fila Home, origen visible/anclado e
  indicadores independientes. Center con backdrop, fondo inerte y foco contenido.
- Misma instancia de pane/editor al alternar Side/Center/Full y promover a New
  tab. Undo/redo, texto pendiente y IDs conservados; ninguna revisión implícita.
- Apertura y captura de Note/Task/Project/Tag/Person/Goal/Milestone. Descartar
  borradores no crea entidades; Doing conserva estado y captura Task conserva
  relación real al Project, sin fechas ocultas.
- Cancelar cierre/reemplazo mantiene buffer; fallo de SQLite inyectado bloquea
  el cierre y permite guardar/reintentar. Save and continue verifica persistencia
  antes de desmontar. Cierre de dueño, cambio de workspace y señal de host
  consideran también peeks inactivos y un origen cubierto por Full page.
- Predeterminado global persistido/reapertura, fallback inválido, fallo de
  escritura recuperable y selección por teclado. Cambiar default mantiene el
  modo actual. Los cuatro valores se comprobaron en aperturas posteriores.
- Frames sin overflow global en claro/oscuro, 1440×1000 y 420×800, para los tres
  modos de presentación. Capturas revisadas de Side y Center estrechos.
- Retorno de foco a la fila original y posición del origen conservados al cerrar.
  Mover foco al origen no dispara su refresco. ⌘S afecta sólo al pane enfocado
  cuando hay dos buffers; confirmación de cierre sólo después de resolver todos.

## Corrections and limits

El primer check numérico de scroll fue demasiado estricto: al reducir el ancho,
el navegador conserva el ancla de sección y redistribuye líneas. Se verificó
posición visual del mismo encabezado y aislamiento de ambos scrollers. Los
ensayos iniciales con selectores de captura/Goal incorrectos se corrigieron;
las rondas finales completas aprobaron. Una prueba de creación de heading en un
documento largo partió de una selección distinta de la prevista; se reemplazó
por un borrador sintético con posición conocida y comandos reales de la UI.

No se repitieron suites históricas de toda la aplicación ni aceptación nativa.
No cambió Rust, dependencias, contratos, SQLite o documentos persistidos del
usuario. Los peeks no se persisten como sesiones y el default es global, no por
vista. Las proyecciones del origen se actualizan explícitamente o al reactivar
su pestaña. Ver [guía](detail-peek.md) y [ADR0050](decisions/0050-session-detail-peeks.md).
