# Mover proyectos en Board

Board permite arrastrar una tarjeta antes de otra o al final de una columna.
Moverla a otra columna actualiza su estado. El botón **Mover proyecto**, visible
al pasar el puntero o enfocar la tarjeta, ofrece la misma elección con teclado:
estado y posición. Escape/Cancelar cierra el diálogo sin guardar.

El orden se guarda en `board_order`; los estados se validan en Rust. El adaptador
conserva el payload completo y actualiza únicamente orden/estado mediante
`changes.preview` y `changes.apply`, con revisiones esperadas, receipt y lote
atómico. No modifica documentos ni relaciones. Después se releen las secciones
Home. Un conflicto informa el error y actualiza la lectura sin reintentar escrituras.

El movimiento exige Board completo, hasta cien proyectos y metadatos disponibles.
Si la vista está paginada o contiene metadatos no interpretables, informa el límite
y deshabilita el movimiento. Active y Planned conservan sus consultas. El orden
del origen importado es un snapshot; los movimientos posteriores son locales.

[ADR0040](decisions/0040-atomic-project-board-movement.md) y
[alcance de revisión](verification-project-board-movement.md).
