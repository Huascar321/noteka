# ADR0040 — Movimiento atómico de Projects Board

Fecha: 23/10/2026. Estado: aceptada.

## Contexto

El usuario eligió reordenar tarjetas dentro de columnas y entre estados.
ProjectFields ya admite estado y orden; el núcleo dispone de lotes con CAS,
receipt, hash global del estado base y transacción atómica.

## Decisión

Usar drag/drop nativo, indicadores y un diálogo equivalente con teclado. El origen
del arrastre debe pertenecer a la instancia actual del tablero. La UI calcula la
secuencia destino y lee entidades completas; conserva campos ajenos al movimiento.
Los únicos updates modifican `state`/`board_order` y mantienen revisiones esperadas.

Después del preview se contrasta el Board completo por orden de IDs, revisiones y
estados. El hash base del apply cubre cambios posteriores. No hay presentación
optimista de guardado ni reintento mutante automático. Un resultado no confirmado
obliga a releer; la atomicidad pertenece a Rust. Se cancela antes del apply si el
componente deja de estar activo; un apply ya enviado mantiene su transacción.

El ámbito es un Board completo de hasta cien proyectos interpretables. Se respeta
el límite existente Home/batch; no se ordenan registros ocultos. No hay nueva API,
migración ni edición de cuerpos/relaciones.

## Consecuencias

Una escritura concurrente ajena puede invalidar el receipt global. Se informa el
conflicto y se permite otra acción explícita después de actualizar la lectura.
Una escala mayor exige diseñar una operación de orden específica. La interacción
se revisa sobre copia aislada; no se guardan movimientos de revisión en el
workspace personal.
