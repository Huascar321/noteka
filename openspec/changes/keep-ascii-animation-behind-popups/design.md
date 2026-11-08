# Design

## Context

App calcula `visible` por pestaña mostrada y no cubierta por una presentación
Full. `active` expresa foco y `blocked` expresa interacción bloqueada.
`HomePanel` usaba ambas señales más excepciones (`searchOpen`,
`animateAlongsideSidePeek`) para decidir si `AsciiShark` seguía animando. Eso
detenía la portada al abrir Settings (`blocked`) y al pasar a Center peek (foco
en el detalle), aunque el canvas siguiera en pantalla.

## Goals / Non-Goals

**Goals:** una única regla de animación basada en visibilidad real; conservar
las pausas existentes y el ahorro al ocultar o desmontar; mantener controles y
atajos bloqueados sin tocar el canvas.

**Non-Goals:** cambiar el renderer, el reloj, las escenas, el acento, las
preferencias o los datos; habilitar controles del fondo; animar cuando la
portada está cubierta, fuera de vista, con movimiento reducido o con el
documento oculto; prometer una mejora de CPU/GPU.

## Decisions

- `AsciiShark` recibe `active={visible}` desde `HomePanel`. `visible` ya combina
  pestaña mostrada y ausencia de cobertura Full; el `IntersectionObserver` del
  canvas resuelve «fuera de vista», y `document.hidden` más
  `prefers-reduced-motion` siguen dentro de `AsciiShark`. Alternativa
  descartada: sumar excepciones por popup (`Settings`, `Center peek`), que
  crecen con cada overlay y usan el foco como sustituto de visibilidad.
- Se eliminan `searchOpen` y `animateAlongsideSidePeek` de App →
  WorkspacePane → HomePanel. Quedan cubiertos por `visible` y evitan props
  muertos o contradictorios.
- `blocked` y `active` siguen controlando atajos, controles y editores; no se
  reutilizan para suspender el render.
- Auditoría: Settings y Center peek eran los únicos overlays que detenían la
  portada. Search ya tenía excepción. La guardia de cambios pendientes, el
  diálogo de mover Project, menús, pickers y paneles plegables no alteran
  `visible` ni el reloj. Full page debe seguir suspendiendo porque cubre Home;
  pestaña inactiva y portada fuera de vista también.

## Risks / Trade-offs

- [Costo con Settings o Search abiertos] → se reutiliza el único RAF existente,
  sin timers, listeners ni preferencias nuevos; el renderer conserva su tope de
  120 FPS y se suspende apenas Home se cubre o la portada sale de vista. No se
  afirma una mejora de rendimiento sin medirla.
- [Regresión del Side peek] → `visible` ya era true junto al detalle; la prueba
  cubre Side, Center, Full, Settings y Search.
- [Confusión futura] → un comentario en HomePanel y la guía de diseño indican
  que foco y bloqueo no pausan el canvas.

## Migration Plan

Sin migración. Un rollback no deja datos ni preferencias que deshacer.
