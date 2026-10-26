# PDF universal: PDFium y PaddleOCR

Prioridad de componentes universales para PDF. PDFKit/Vision quedan como opciones macOS ya verificadas. [ADR0031](decisions/0031-universal-pdf-backend.md).

## Extracción de texto con PDFium

`pdfium_text` usa `noteka-pdfium-worker` (`pdfium-render 0.9.4`, PDFium 7881) y escribe en el mismo formato de páginas PDF. La librería se carga dinámicamente:

```sh
export NOTEKA_PDFIUM_LIB=/ruta/a/libpdfium.dylib   # o libpdfium.so / pdfium.dll
```

Si no se indica, el worker busca `libpdfium.dylib`, `libpdfium.so`, `pdfium.dll` o `libpdfium.dll` junto al ejecutable. No se descarga nada. El manifiesto registra `parser_name=pdfium` y `0.9.4/pdfium-7881`.

```sh
noteka --json --workspace "$W" extraction enqueue \
  --input '{"attachment_id":"<id>","extractor":"pdfium_text"}'
```

## OCR universal con PaddleOCR

`paddle_ocr` ejecuta un worker externo Python con PDFium/PaddleOCR PP-StructureV3 y persiste **texto por página + estructura de tablas** (filas/columnas/celdas con spans/texto).

Requisitos instalados por el usuario:

- Python con `paddleocr 3.7.0`, `paddlepaddle 3.3.1`, `pypdfium2 5.14.0`.
- Modelos PP-OCRv6 small + estructura en una ruta local.
- Variables:

```sh
export NOTEKA_PADDLE_PYTHON=/ruta/venv/bin/python
export NOTEKA_PADDLE_WORKER=/ruta/noteka/native/universal/paddle_ocr_worker.py
export NOTEKA_PADDLE_MODEL_PATHS=/ruta/paddle-model-paths.json
export NOTEKA_PADDLE_DEVICE=cpu
```

El JSON de modelos es el del ensayo: nombres `PP-OCRv6_small_det`, `PP-OCRv6_small_rec`, layout/estructura, etc. No se descargan pesos. Si falta el entorno, el trabajo falla con `paddle_*` y no publica parciales.

```sh
noteka --json --workspace "$W" extraction enqueue \
  --input '{"attachment_id":"<id>","extractor":"paddle_ocr"}'
```

## Selección y compatibilidad

- `pdfium_text`: primera opción universal para PDF con texto.
- `paddle_ocr`: primera opción universal para PDF escaneado.
- `pdf_text` (lopdf), `pdfkit_text` y `pdf_ocr` (CoreGraphics/Vision) siguen disponibles; no se reescriben generaciones existentes.
- El lector `extraction.pdf_fragments` y la búsqueda muestran el parser real. Los filtros distinguen cada extractor.
- `paddle_ocr` persiste texto y tablas (≤1 MiB, 512 tablas). No persiste bbox/confianza por región ni orden visual. `extraction.pdf_structure` lee la estructura; la UI la muestra como JSON.

## Límites

- PDFium: 100 páginas, 1 MiB de fuente/texto, 1 MiB de salida, deadline 5 s.
- Paddle: 20 páginas, 1 MiB de texto, salida 8 MiB, deadline 30 s por inicialización/estructura.
- El proceso Paddle necesita memoria/CPU considerable; no se ejecuta al abrir o buscar.
- Windows/Linux requieren instalar PDFium/Python/modelos y verificar su perfil; no hay aceptación Windows ejecutada.

## Verificación

Ver [verificación universal](verification-universal-pdf.md). Los tests reales usan la librería PDFium y el entorno Paddle del benchmark local; no hay red externa. La calidad se limita al corpus de 16 PDFs y no se generaliza.
