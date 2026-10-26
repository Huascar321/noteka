# ADR 0031: Backend PDF universal PDFium + PaddleOCR

- Fecha: 22 de octubre de 2026
- Estado: adoptado como prioridad universal; PDFKit/Vision quedan como alternativas macOS
- Ámbito: texto PDF, OCR de PDF y base de render

## Contexto

El lector PDF verificado usaba lopdf, PDFKit y CoreGraphics/Vision. PDFKit/Vision son específicos de macOS. El benchmark local del 21/10 comparó alternativas universales: PDFium como base portable y PaddleOCR PP-StructureV3 con los mejores resultados de tablas (86/86 celdas frente a 82/86 de Docling), a costa de mayor latencia. El usuario pidió priorizar componentes universales sobre los específicos de Mac.

## Decisión

1. **PDFium** es la base portable de texto/render. `noteka-pdfium-worker` (Rust, `pdfium-render 0.9.4`, PDFium 7881) extrae texto por página y carga una librería dinámica configurada por `NOTEKA_PDFIUM_LIB` o un sibling. Se registra `parser_name=pdfium` y versión.
2. **PaddleOCR PP-StructureV3** es el backend universal de OCR. `paddle_ocr` ejecuta un worker Python externo (`paddleocr 3.7.0`, `paddlepaddle 3.3.1`, `pypdfium2 5.14.0`, PP-OCRv6 small/estructura) configurado por `NOTEKA_PADDLE_PYTHON`, `NOTEKA_PADDLE_WORKER`, `NOTEKA_PADDLE_MODEL_PATHS` y `NOTEKA_PADDLE_DEVICE`.
3. Ambos escriben en el formato canónico `extraction_pdf_*` con parser/versión. No hay migración ni se reescriben generaciones lopdf/PDFKit/PDF OCR.
4. **Prioridad**: `pdfium_text` y `paddle_ocr` aparecen primero para PDF nuevo en las opciones universales; `pdf_text`, `pdfkit_text` y `pdf_ocr` siguen seleccionables. Si falta PDFium/Paddle, el trabajo falla explícitamente; no se cambia de motor en silencio.
5. El worker Paddle es un proceso externo sin workspace/SQL/shell/red; Rust valida hash, páginas, tamaño, deadline y salida. El presupuesto Paddle es 30 s por inicialización/estructura; no altera el target de búsqueda 300/150.
6. `paddle_ocr` persiste texto y estructura de tablas (filas/columnas/celdas con spans/texto) mediante la migración019 y `extraction.pdf_structure`. No persiste bbox/confianza ni orden visual; no se presenta HTML/Markdown como autoridad.

## Licencias y distribución

- `pdfium-render`: MIT OR Apache-2.0. PDFium es BSD-3-Clause; la librería dinámica no se versiona en Git y debe instalarse por plataforma.
- PaddleOCR/PaddlePaddle: Apache-2.0. Los modelos PP-OCRv6/estructura se instalan aparte; no se descargan al abrir/buscar. `pypdfium2` es BSD-3-Clause/Apache-2.0.
- No se versionan pesos, binarios PDFium, cachés ni datos personales.

## Alternativas

- PDFKit/Vision: se conservan y siguen siendo la ruta nativa macOS ya verificada, pero no son la prioridad universal.
- lopdf: parser acotado de texto; se conserva como `pdf_text`.
- Docling: más rápido en estructura en el ensayo, pero perdió celdas; no se adopta.
- RapidOCR: rápido para OCR simple, sin reconstrucción de tablas; alternativa liviana no adoptada.
- Enlazar PDFium estáticamente: descartado por tamaño, licencias y empaquetado.

## Consecuencias

- Windows/Linux requieren su propia instalación de PDFium/Python/modelos y verificación; no se declara soporte Windows ejecutado.
- Paddle añade latencia/memoria y depende de un entorno Python; el usuario debe instalarlo. El producto sigue funcionando sin él usando alternativas.
- La calidad de Paddle se limita al corpus del benchmark; no es una garantía general.

## Evidencia

`docs/verification-universal-pdf.md`. El benchmark original sigue en la evidencia externa `research/comparacion-y-recomendacion.md`.
