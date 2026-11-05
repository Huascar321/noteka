# Proposal

## Why

La franja opaca del breadcrumb separa Home de su portada ASCII y contradice el material neutro compartido.

## What Changes

- Integrar el breadcrumb de Home sobre la portada ASCII sin una fila de fondo propia.
- Quitar fondos de barra en breadcrumbs de otras vistas, superponer la barra a su contenido y presentar las migas como cápsulas de vidrio neutro.
- Llevar la misma cápsula al sidebar, sólo en la sección activa, y animar su desplazamiento.
- Colapsar las migas de la derecha hacia la miga anterior al volver por el breadcrumb.
- Hacer nacer la miga de nivel 2 desde Home al entrar a una sección.
- Animar las tabs de vista de sección con un indicador deslizante compartido, gris y con brillo y sombra de vidrio.
- Conservar navegación, foco, contenido, scroll, temas y preferencias de accesibilidad.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: presentación y posición de breadcrumbs.

## Impact

Estilos de navegación y verificación; sin cambios de datos, IPC ni motor ASCII.
