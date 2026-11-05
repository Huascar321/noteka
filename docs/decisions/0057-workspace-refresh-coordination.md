# ADR0057 — Coordinación de lecturas al recuperar el foco

Fecha: 28/10/2026. Estado: adoptada; aceptación nativa macOS verificada.

Noteka agrupa los eventos de foco, activación y actualización para evitar
trabajo repetido. Un pool de lecturas por ruta canónica de workspace limita a dos
operaciones reales en curso y conserva el cupo hasta que retorna Rust, incluso
cuando la interfaz abandona el resultado. Las lecturas en cola que ya no
corresponden pueden descartarse; las escrituras no pasan por ese pool.

Cada consulta tiene identidad propia. Las ráfagas se agrupan y una lectura en
curso puede dejar como máximo una actualización posterior. Los cambios de
workspace, vista, fecha, entidad o criterio invalidan la solicitud anterior.
Los resultados compatibles permanecen visibles como anteriores durante la
actualización y ante un error; en este último caso se ofrece reintento. Al
renovarse una lista paginada se reemplaza por su primera página y se informa
que las páginas adicionales pueden volver a cargarse. Los lectores
especializados mantienen sus validaciones y presentación de errores.

El Board no habilita movimientos mientras su lectura está en curso o no está
verificada. Los borradores se conservan y una revisión externa distinta usa la
guardia de conflicto existente. La fecha implícita de hoy se vuelve a calcular
en la zona horaria del workspace. Cada consulta conserva su propia instantánea
de SQLite: la coordinación no promete una instantánea global entre secciones.

Se eligió refrescar al foco o a pedido, sin consultas periódicas, detector
persistente ni `PRAGMA data_version`. Esto reduce la complejidad y garantiza
que se intente leer nuevamente, a cambio de que incluso un workspace sin
cambios vuelva a consultar y reinicie páginas adicionales. Podrá evaluarse un
detector en otro cambio si la medición justifica su costo. El límite afecta a
otras lecturas compatibles iniciadas desde la interfaz, no sólo a las automáticas.
Las medidas y los límites de la aceptación están en
[verificación](../verification-focus-refresh.md).
