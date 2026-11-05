# Verificación del vidrio líquido en búsqueda

02/11/2026. El diálogo de búsqueda (`SearchDialog.svelte`) usa un material más
translúcido: `--glass-search-surface` baja a 72% en claro y 60% en oscuro; el
fondo compone un radial superior, `--glass-breadcrumb-sheen` y el tinte; el
filtro local es `blur(22px) saturate(1.45)`. El borde usa `--glass-stroke`, con
reflejo superior y sombra interior. El campo, la fila activa y el pie repiten el
vidrio.

Verificación en Chromium y WebKit (headless, CSS de producción, 1440x900):

- Oscuro: fondo `rgba(40,40,40,0.60)`, `backdrop-filter: blur(22px)
  saturate(1.45)`, borde `rgba(255,255,255,0.314)`, sombras interiores; campo con
  borde y sheen; fila activa con 7% de tinte y sheen; pie con 5% de tinte.
- Claro: fondo `rgba(247,247,247,0.72)`, campo `rgba(255,255,255,0.59)` con borde
  blanco al 70%, fila activa con 7% de tinta y sheen. La captura a 1440x900 deja
  ver la portada ASCII a través del panel con texto legible.
- Nativo macOS: captura de Cmd+K a 1728x1023; el ASCII se ve a través del panel y
  el foco del campo es visible. Escape cierra el diálogo.
- Transparencia reducida (CDP en Chromium): fondo sólido `rgb(32,32,32)`, sin
  `backdrop-filter`; campo, pie y fila activa sólidos; el foco conserva el anillo.
- Interacción: escribir `home` devuelve la sección Home; Escape cierra y elimina
  el diálogo; sin errores de página.
- Build sin errores.

Límites: no hay refracción nativa ni escritorio detrás de la ventana opaca; el
contraste sobre ASCII en movimiento puede variar y el fallback sólido es la
garantía. No se midió GPU ni se repitió la suite nativa completa.
