# ADR 0051: dimensiones locales y Side peek suprimido

Fecha: 26/10/2026. Estado: adoptada localmente.

## Context

El usuario pidió arrastre y memoria de ancho, límites 35%/65% y exclusión del
menú cuando Side peek supera 35%. El editor ya vive en panes de sesión estables
(ADR0050); cerrar el pane para abrir el menú perdería continuidad del borrador.

## Decision

Guardar píxeles preferidos globales en `noteka.panel-widths.v1` y derivar anchos
efectivos acotados al viewport de la ventana. No sobrescribirlos al achicarla.
Los límites usan porcentajes enteros para conservar el caso exacto de 35% sin
error de coma flotante. Confirmar el arrastre persiste; Escape/captura perdida
recuperan el snapshot y la visibilidad anterior.

Separar visibilidad de existencia: un Side ancho se oculta/inert al abrir el
menú pero permanece montado con buffer, revisión, selección, scroll e historial.
Revelar el mismo ID no recarga el editor; otro ID usa su guardia. La activación
de pestaña y resize reaplican la exclusión. Guardias agregadas revelan un pane
pendiente oculto antes de preguntar.

Animar posición derecha/opacidad durante 260ms y espacio del origen, sin
transform permanente que altere los menús fixed. Movimiento reducido/arrastre
eliminan interpolación. Sólo un cierre explícito resuelto desmonta tras la salida.
Generaciones invalidan revelados tardíos y los cambios de pestaña no roban foco.

## Alternatives

- Desmontar al ocultar: pierde historial/selección y complica recuperación.
- Persistir sólo porcentajes: no recupera la distancia en píxeles elegida.
- Medir pantalla física: no coincide con el espacio real de una ventana más chica.

## Consequences

El shell distingue foco, activación, visibilidad y cierre. Todos los panes,
incluidos los suprimidos, participan en protección nativa y cierre agregado.
No hay cambios de dominio/Rust/SQLite; los errores de preferencia son recuperables.
