# Design

## Context

El sidebar tiene un indicador absoluto único detrás de siete botones. Su posición depende de `--active-index`, con transición elástica de 280 ms. El acabado previo es un tinte blanco y una sombra exterior. La ventana Tauri es opaca; ver la motivación y el checkpoint en `proposal.md`.

## Goals / Non-Goals

**Goals:** aproximar la luz rasante y el volumen de la referencia manteniendo la densidad actual y texto oscuro legible en claro.

**Non-Goals:** transparentar la ventana, copiar controles del sistema, cambiar iconos o extender el material a breadcrumbs, tabs o contenido.

## Decisions

- Mantener el indicador existente y sus eventos/transición; cambiar sólo su material CSS. La alternativa de siete superficies persistentes alteraría la jerarquía actual.
- Centralizar tinte, sheen, borde y sombra en tokens del sidebar por tema. El borde usa una máscara anular y un gradiente direccional; los reflejos interiores aportan espesor sin una sombra difusa amplia.
- Usar gradientes estáticos, sin blur, filtros SVG, bucles de animación ni dependencias. Un `backdrop-filter` sobre el fondo uniforme no reproduce la refracción de macOS y agrega composición sin contenido que refractar.
- Colocar el fallback sólido después de las reglas del indicador. Ocultar sus capas decorativas con transparencia reducida o contraste aumentado y desactivar su transición con movimiento reducido. Esto también corrige la cascada previa, cuyo fallback quedaba antes del estilo base.
- Las operaciones Rust y SQLite siguen siendo dueñas del contenido. No hay cambios de formato persistente ni migración.

## Risks / Trade-offs

- [Fondo uniforme] → la transparencia será menos visible que sobre el escritorio de la referencia; describirlo como aproximación CSS.
- [Borde en WebKit] → verificar máscara y fallback en Chromium/WebKit y observar la ventana nativa.
- [Contraste] → medir el texto contra el fondo compuesto en ambos temas y conservar alternativas sólidas.

## Migration Plan

CSS disponible por HMR local. El commit `affca8a` conserva el estado anterior; el acabado aceptado se incluye en el commit local conjunto pedido posteriormente por el usuario, conservando ese checkpoint para revisión/reversión. No requiere reiniciar ni migrar el workspace.
