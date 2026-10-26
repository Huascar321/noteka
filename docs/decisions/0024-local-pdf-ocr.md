# ADR0024: composición PDF rasterizado y OCR existente

Status: Accepted — implementación y aceptación verificadas en `add-local-pdf-ocr`.

Se usa un helper CoreGraphics acotado y descartable para rasterizar una página,
y se reutiliza el helper Vision de imágenes sin duplicar reconocimiento ni
modificar sus caps/perfil. Rust valida ambos protocolos, PNG/framing/CRC,
provenance, geometría/decimales y documento completo con un presupuesto5s y cerca
compartidos. El PNG es efímero; se persiste su identidad dentro del canónico del
PDF y se conserva el original administrado.

La alternativa monolítica duplicaría Vision y sus validaciones; retener todos
los rasters incrementaría memoria. La composición secuencial paga arranques de
proceso y puede agotar cinco segundos: debe fallar completa, sin elevar límites.
No se agrega orientación inferida, reconstrucción de tablas ni fallback a otro
perfil. El lector PDFKit textual independiente queda intacto.

Se agrega016, dos tablas derivadas y lector público nuevo. La selección FTS se
mantiene explícita; páginas/regiones/raster/motor acompañan todo texto inferido.
La barrera16 impide que el cliente15 certifique un formato que no conoce; backup
previo/restauración en ruta nueva y preservación de unknowns anteceden al DDL.
Ver [contrato y límites](../pdf-ocr.md) y los artefactos activos para aceptación.
