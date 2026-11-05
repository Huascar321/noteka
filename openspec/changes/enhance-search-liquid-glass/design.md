# Design

## Context

La ventana Tauri es opaca y el material CSS no reproduce la refracción de Liquid Glass nativo. La búsqueda ya usa `--glass-search-surface`, blur de 10px y un borde de `--glass-stroke`, pero el resultado se percibe plano.

## Goals / Non-Goals

Acercar el panel a un vidrio líquido visible en ambos temas sin sacrificar lectura. No agregar imágenes de fondo, no transparentar la ventana del sistema ni replicar refracción nativa.

## Decisions

- Bajar el alfa de `--glass-search-surface` a 72% en claro y 60% en oscuro.
- Componer el fondo con un radial superior, `--glass-breadcrumb-sheen` y el tinte; usar `blur(22px) saturate(1.45)` local.
- Reforzar borde, reflejo superior y sombra interior; el campo, la fila activa y el pie usan capas de vidrio.
- Con `prefers-reduced-transparency: reduce` o `prefers-contrast: more`, volver a `--surface`, quitar el filtro y conservar el foco.
- Mantener el tamaño compacto del panel y el blur limitado a su caja.

## Risks / Trade-offs

El fondo ASCII se mueve detrás del panel; el contraste local puede variar. El fallback sólido cubre transparencia reducida y contraste alto. El blur más amplio tiene costo gráfico; no se midió GPU. No hay paridad con Liquid Glass nativo.
