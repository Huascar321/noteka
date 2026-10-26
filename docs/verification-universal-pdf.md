# Verificación del backend PDF universal

Estado: implementación y aceptación técnica local macOS aprobadas el 22/10/2026.
No se ejecutó Windows/Linux ni se declara soporte universal verificado en esas
plataformas. La calidad se limita al corpus del benchmark y a texto por página.

## Componentes y versiones

| Componente | Versión | Licencia | Uso |
| --- | --- | --- | --- |
| `pdfium-render` | 0.9.4 | MIT OR Apache-2.0 | Binding Rust PDFium |
| PDFium | 7881 (feature `pdfium_7881`) | BSD-3-Clause | Texto/render PDF |
| `pypdfium2` | 5.14.0 | BSD-3-Clause/Apache-2.0 | Worker Paddle (opcional) |
| PaddleOCR | 3.7.0 | Apache-2.0 | OCR/estructura |
| PaddlePaddle | 3.3.1 | Apache-2.0 | Runtime CPU |

Las versiones se consultaron en crates.io y PyPI el 22/10/2026; son las últimas
publicadas en ese momento. La librería PDFium y los pesos no se versionan en Git.

## Evidencia ejecutada

- Rust: **414 aprobadas/16 ignoradas por perfil debug/release** en el workspace
  excluyendo el host Tauri (`logs/universal-pdf-debug-final.log`,
  `logs/universal-pdf-release-final.log`).
- PDFium real (debug/release): extracción de un PDF sintético, parser
  `pdfium 0.9.4/pdfium-7881`, páginas/fragmentos, index put y búsqueda filtrada
  (`pdfium_tests::real_pdfium_extraction...`).
- PaddleOCR real (debug/release): extracción de `spanish-digital.pdf` con
  `paddleocr_ppstructurev3 3.7.0`, texto por página, estructura de tablas,
  index put y búsqueda filtrada (`paddle_tests::real_paddle_ocr...`). Tiempo de
  la prueba ~9–11 s; presupuesto del extractor 30 s.
- Tabla real `table-digital.pdf`: 1 tabla, 5 filas × 3 columnas, 15 celdas,
  primera celda `Miembro`; leída por CLI `extraction pdf-structure` y por
  `extraction.pdf_structure`. Backup/restore conserva hash/estructura.
- CLI real: `pdfium_text` y `paddle_ocr` encolados/ejecutados/leídos; index put
  y `extraction.search` devuelven extractor, página y texto.
- UI: `svelte-check` 0/0; 39 pruebas generation-reader; headless preparación
 154 checks; núcleo 592+14 teclado; lector 990 checks; sin requests externos.
- SDK MCP: **28 pruebas**; catálogo con los nuevos extractores.
- Contratos: `npm run contracts:check` y generación actualizados; no se agregaron
  operaciones, sólo el enum de extractores y metadatos de catálogo.
- Migración aditiva **019**: estructura nullable en `extraction_pdf_structures`;
  SQLite19/API63/MCP33/54/56/batch9. Gate attachment posterior: query p95 máximo
  124,967042 ms/lock19,768042 ms, dentro de300/150.

## Límites y pendientes

- `paddle_ocr` persiste texto y **estructura de tablas** (filas/columnas/celdas
  con spans/texto) en JSON ≤1 MiB. No persiste bbox/confianza por región ni
  orden visual; no convierte Markdown/HTML en autoridad.
- `pdfium_text` cubre texto nativo; no hace OCR. `paddle_ocr` cubre OCR de texto
  y tablas, no regiones/bbox/confianza.
- Windows/Linux no se ejecutaron. Requieren instalar PDFium, Python/modelos y
  verificar empaquetado/rendimiento; no se anuncia compatibilidad productiva.
- La librería PDFium y los modelos Paddle se instalan aparte; no hay descarga al
  abrir/buscar. Si faltan, el trabajo falla explícitamente y no publica parciales.
- La calidad Paddle se limita al corpus de 16 PDFs; no es una garantía general.

## Seguridad y procedencia

PDFium corre en un worker Rust con pipes no bloqueantes, deadline, stderr/stdout
acotados y hash del original. Paddle corre en un proceso Python con entorno
mínimo (`HOME`, `PATH`, variables Paddle), sin workspace/SQL/shell; el núcleo
valida hash, páginas, texto, tamaño y deadline. El worker no se invoca desde
open/search. Los manifests registran parser/versión reales.

## Fallos conservados

La primera prueba Paddle falló al leer el fragmento porque el lector PDF aún
exigía `pdf_text`/`pdfium_text`; se añadió `paddle_ocr` y se repitió. La prueba
PDFium release falló al no existir el binario release del worker; se construyó
y se repitió. No se cambió una aserción para ocultar esos fallos.
