# Design

## Context

El refresco de Home conserva resultados, pero agrega dos avisos verticales por sección. El anclaje del navegador ajusta el scroll cuando aparecen. La restauración de foco puede además desplazarse hacia un botón fuera de pantalla.

## Goals / Non-Goals

Evitar movimiento artificial con resultados conservados y mantener avisos accesibles, errores, actualizaciones y protección de borradores. No congelar el scroll frente a cambios reales del contenido ni interferir con el desplazamiento del usuario.

## Decisions

Durante la actualización con página previa, mostrar el estado en la barra de acciones sin alterar la altura; mantener mensajes de carga inicial, error y reinicio de páginas. Restaurar el foco con `preventScroll` sin asignar posiciones de scroll.

## Risks / Trade-offs

Cambios reales o eliminación de páginas pueden alterar naturalmente la posición. La prueba usa transporte de prueba y SQLite sintético; el comportamiento de foco físico debe distinguirse de la simulación.
