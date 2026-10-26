# Proposal

## Why

Tasks del menú lateral todavía abre un listado genérico sin las vistas ni la composición de la página Tasks mostrada por el usuario. Home ya tiene consultas y filas reutilizables, pero esa navegación no las monta.

## What Changes

- Página Tasks amplia con icono sobre título, navegación corta, New Task, pestañas y filas compactas; Today inicial y Load more acumulativo.
- Bloque Projects debajo con Active inicial, Planned, Board, Deadlines y Completed. Listas agrupadas con contexto, pendientes, vencidas y progreso; Board reutiliza movimiento y guardias existentes.
- Dos vistas Rust adicionales: proyectos pendientes con end_date ordenados por esa fecha y proyectos Done. Fechas y estados provienen del modelo local existente.
- Recurring y Active Content visibles como pendientes deshabilitadas. No se agregan recurrencias, Content, Goals, colaboración, sincronización ni propiedades ficticias.
- Breadcrumb compartido Home → sección → registro en detalles y Home → sección al navegar el menú; iconos propios, títulos y navegación protegida. Pedido adicional explícito del 24/10.
- Retirar contadores y fechas de consulta al pie de las cinco secciones, el aviso técnico de Home y Siguiente Tasks. Load more y New page sin borde ocupan toda la fila; los botones con borde conservan su presentación.
- Apertura/captura protegidas, lecturas independientes, temas y ancho estrecho; acceso por teclado y actualización sin recargar el WebView.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: página Tasks del menú lateral, presentación de Projects, breadcrumb jerárquico global y dos consultas compartidas adicionales.

## Impact

WorkspacePane, componentes Tasks/Projects reutilizables, página dedicada, breadcrumb reutilizable, CSS y home.query/DTO generado. Sin dependencias nuevas, migración, cambios de payload persistido ni mutaciones de datos del usuario. Compilación e inspección visual manual; no se agregan ni ejecutan suites. Dieciséis selectores existentes de regreso a Home se adaptan al breadcrumb sin nuevos escenarios. Los datos de la referencia siguen fuera de Git.
