# Design

## Context

Las secciones comparten `.task-view-actions`, New y botones de 28 px. El buscador tiene un wrapper de campo propio. Los menús ya usan filtros locales y varias recetas de radio/sombra. ASCII/Photo conserva dos botones `aria-pressed`. Ver la motivación en `proposal.md` y los tokens aprobados en `docs/design-system.md`.

## Goals / Non-Goals

**Goals:** compartir la óptica aprobada y expresar agrupaciones funcionales sin añadir nuevas operaciones.

**Non-Goals:** modificar cuerpos de contenido, datos, contratos, Rust, dependencias, ventana nativa o comportamiento de búsqueda/selección.

## Decisions

- Agregar un grupo semántico compacto alrededor de los iconos de cada barra; mantener el estado de lectura y New como hermanos. Material estático sin `backdrop-filter` en el grupo, para no crear un bloque contenedor que desplace menús de posición fija. Se descarta pintar cada icono con una cápsula persistente.
- Reutilizar `--glass-field-*` en el wrapper del buscador, conservando tamaño de texto, input, combobox, resultados y cierre. El anillo de foco se dibuja en el wrapper y el placeholder conserva tinta legible. La variante sobre el diálogo refuerza la tinta y usa un tinte gris oscuro al 40% en tema oscuro: el velo blanco del sidebar no deja suficiente contraste sobre contenido claro en WebKit sin blur compuesto.
- Marcar las superficies flotantes compartidas con una clase de material; reutilizar el borde anular del sidebar y tokens de radio/sombra propios para menús. Mantener filtros, coordenadas y eventos del componente dueño. Los paneles PageTools sólo adoptan ese marco cuando son flotantes en cabecera.
- Reutilizar `slidingTabs` para ASCII/Photo, sin otro reloj ni estado persistente. Conservar el posicionamiento absoluto del selector aunque la acción agregue la clase de host; botones sobre el indicador y visibilidad al enfocar.
- Los fallbacks se definen después de los materiales y retiran reflejos y filtros. El contenido pertenece a Rust/SQLite; no cambia su formato y no se requiere migración.

## Risks / Trade-offs

- [Nueva envoltura en toolbars] → verificar orden de teclado, estados y New fuera del grupo en Home/secciones/Milestones.
- [Posicionamiento y recorte] → probar menús reales y ancho estrecho; no agregar filtros a los ancestros.
- [Indicador en control absoluto] → verificar posición, cambio de ancho y movimiento reducido.
- [Composición sobre contenido] → mantener el tinte actual de overlays y medir legibilidad del campo; sin afirmar equivalencia nativa ni rendimiento GPU.

## Migration Plan

Cambios locales Svelte/CSS por HMR. Preferencias existentes y checkpoint `affca8a` se conservan. Sin publicación ni sincronización/archivo automático de deltas.
