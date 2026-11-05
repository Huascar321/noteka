# Proposal

## Why

Las páginas de Tasks, Notes y Projects mostraban un botón de captura
(`New Task`, `New Note`, `New Project`) arriba de la lista y además un botón
`New` en la barra de vistas. Las dos acciones hacían lo mismo y duplicaban la
jerarquía visual.

## What Changes

- Retirar los botones de captura duplicados de Tasks, Notes y Projects, también
  cuando esas secciones se usan en Home.
- Conservar una sola acción `New` en la barra de vistas y las acciones
  contextuales `New page`/`New task` dentro de grupos.
- Dar a esa acción `New` un vidrio teñido con el color primario elegido (`--accent`), con sheen, sombra y texto blanco.
- Igualar el espaciado de las barras de vistas de Home y de las páginas de
  sección al ritmo de Tags, y reservar el alto del bloque de captura en las
  páginas que ya no lo tienen, para que la fila de vistas quede alineada.
- Mantener Tags y People fuera de este recorte de captura.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: acciones de creación de las secciones de listas.

## Impact

Sólo markup y documentación de UI; sin cambios de datos, operaciones, contratos,
migraciones ni captura.
