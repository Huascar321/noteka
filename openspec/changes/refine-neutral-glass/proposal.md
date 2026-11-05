# Proposal

## Why

El vidrio anterior tiñe superficies amplias con el color primario y se aleja de las referencias de material neutro que el usuario eligió.

## What Changes

- Devolver un fondo neutro al contenido y Home, manteniendo continuidad con la portada ASCII.
- Usar grises translúcidos, bordes finos y reflejos estáticos en el marco, sidebar, búsqueda y menús flotantes.
- Reservar el color primario para foco, iconos y acciones; conservar la legibilidad y el fallback accesible.
- Extender el material a navegación contextual, diálogos pequeños y controles flotantes, y documentar su uso obligatorio para pantallas nuevas.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: material visual neutro para marco y overlays.

## Impact

CSS, pruebas visuales, `docs/design-system.md` y `AGENTS.md`; sin cambios de datos, dependencias, ventana nativa ni contratos.
