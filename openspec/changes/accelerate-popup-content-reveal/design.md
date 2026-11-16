# Design

## Context

La capa decorativa top layer de menús/pickers puede filtrar su propio texto hasta finalizar la entrada. Los diálogos/Center peek tienen material local detrás del texto y no requieren acelerar su geometría. Due también separa material y contenido.

## Decisions

- Centralizar entrada de menú en 160 ms y revelado de contenido en 100 ms. Menús retiran material temporal al completar 160 ms; no alcanza con adelantar la opacidad.
- Este pedido posterior sustituye la entrada uniforme de 400 ms de los cambios anteriores sólo para menús/pickers; las mediciones previas de rendimiento describen su estado verificado original.
- Diálogos conservan 400 ms de material y contenido inicial al 24%, pero opacidad completa en 100 ms. Menús/Due comienzan sin retención de opacidad cero y completan contenido en 100 ms. Cierre conserva 200 ms.
- Mantener reproducción WAAPI autónoma, recortes precalculados y limpieza actual. No introducir timers por frame, copias de contenido ni modificar óptica o carga de datos.
- Verificar selector de proyectos/menús y diálogos, tiempos y opacidad en ambos motores, interrupción/selección/foco y movimiento reducido. Capturas con corpus sintético, sin workspace personal.

## Risks / Trade-offs

La curva de menús se reproduce más rápido; es intencional para eliminar antes la fase borrosa. Los checks históricos de 400 ms se actualizan sólo para menús/pickers; los diálogos y cierres conservan sus aserciones.
