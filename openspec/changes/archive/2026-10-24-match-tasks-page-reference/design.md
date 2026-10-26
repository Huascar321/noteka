# Design

## Context

Ver proposal.md. WorkspacePane monta entity.list para Tasks, aunque HomeTaskSection y home.query ya ofrecen filtros, contexto, errores y grupos. ProjectFields ya incluye end_date validado en Rust. El usuario adjuntó una página Tasks con Projects debajo, no el detalle de una tarea individual.

## Goals / Non-Goals

**Goals:** Una composición de página que reutilice consultas/operaciones existentes, con lecturas independientes, identidad cancelable y reglas en Rust.

**Non-Goals:** Cambiar el cuerpo o las propiedades del detalle individual de tareas sin una referencia de ese detalle, importar datos adicionales, modificar fechas/estados, crear recurrencias, Content o Goals, ni copiar controles sin operación real.

## Decisions

1. TasksPage monta dos HomeReader con cola de dos lecturas: Tasks Today inicial de diez acumulables y Projects Active de cien acumulables. Home mantiene sus vistas iniciales actuales. Alternativa descartada: duplicar reglas mediante entity.list y filtros frontend.
2. HomeTaskSection tiene una variante de página con icono grande/título, navegación rápida, pestañas completas y pendientes deshabilitadas; la variante Home mantiene su composición. Guardias de WorkspacePane siguen siendo dueñas de apertura/captura.
3. HomeProjectSection recibe presentación en filas y vistas adicionales para esta página; Board reutiliza el componente y el guardado atómico/CAS vigentes. Los conteos de grupos describen la página acumulada.
4. home.query añade ProjectsDeadlines y ProjectsCompleted; proyecta opcionalmente project_end_date desde end_date existente con validación civil estricta. Deadlines excluye Done, usa end_date ascendente/ID final, no deriva fechas de tareas. Sin fecha se muestra vacío explícito. Completed usa estado Done, sin inventar fecha de finalización. No se agrega persistencia ni operación.
5. Atajos de actualización activos sólo con la página montada/activa: Meta/Ctrl+R o F5, sin Alt/Shift, sin repetir/composición y bloqueados por diálogos. No recargan WebView.
6. Sólo datos locales e imágenes acreditadas existentes. Controles de archivo/compartir/cover/backlinks de Notion sin equivalente no se simulan.

7. El pedido adicional del usuario incorpora un breadcrumb compartido a WorkspacePane: Home → sección canónica del kind → título vivo del registro, o Home → sección en las páginas del menú. Se omite Databases según la jerarquía pedida. Iconos de sección y registro reutilizan AppIcon/arte local; Home y sección usan navigate con las guardias existentes. En ancho estrecho el título se trunca visualmente y conserva su nombre completo accesible. El detalle individual conserva su cuerpo y herramientas.

8. El pedido posterior elimina los diagnósticos visibles del pie y el aviso técnico de Home. La cobertura queda dentro de Opciones de vista, sin reservar espacio en la fila. Load more usa un botón nativo de ancho completo, visible sólo si hay cursor; Home acumula páginas mediante HomeReader como ya hace Notes. Las filas New page/New task sin borde ocupan su contenedor; las tarjetas y capturas con borde conservan su estilo. No cambian consultas, caps ni el límite de movimiento del Board.

## Risks / Trade-offs

- Fechas/progreso pueden diferir de un snapshot Notion → usar valores locales y documentar definición/cobertura.
- Todas las pestañas no caben en ventana estrecha → desplazamiento de pestañas contenido y metadatos en segunda línea.
- Respuesta tardía o navegación durante consulta → HomeReader/ReadLifetime descartan la respuesta y dispose mantiene los límites físicos de cola.
- Board parcial → conserva deshabilitación de movimiento y aviso existentes.

## Migration Plan

No cambia SQLite ni payloads. Compilar CLI y generar contratos antes de compilar frontend. Reversión por copia del baseline privado de archivos afectados; no restaurar datos porque este cambio no los modifica. Inspección visual manual de la UI con una copia del workspace, sin ejecutar suites ni atribuir aceptación de IPC/cierre Tauri.
