# Proposal

## Why

El texto PDFKit ya se prepara, persiste, lee y busca, pero un PDF escaneado no
aporta texto mediante ese perfil. Hace falta OCR explícito de PDF con página y
región verificables, conservando el original y los límites del núcleo.

## What Changes

- Añadir `pdf_ocr` acotado: rasterizador CoreGraphics descartable y reutilización
  del OCR Vision existente; no se instala un motor ni se recrea el lector PDFKit.
- Render de MediaBox/rotación a PNG gris, escala objetivo2, <=4MP/página,
  <=16MP/documento, <=20 páginas y <=1MiB/PNG. Fuente PDF <=1MiB; cinco segundos
  compartidos para todo el job. Exceso/error rechaza el resultado entero.
- Persistir JSON canónico con páginas, hash de raster, reconocimiento inferido,
  decimales y procedencia. Migración aditiva016 con recuperación desde15 real.
- Añadir lectura `extraction.pdf_ocr_regions`, selección/proyección FTS y casos
  explícitos en preparación/lector/Search individual y agrupada existentes.
- Mantener54 contratos anteriores salvo ampliaciones identificadas de index/search;
  API55/MCP27/47/48, batch9 y siete exclusiones. **BREAKING** para clientes15 que
  intenten abrir16: rechazo sin writes; backup15 restaurable con cliente anterior.

No incluye tablas PDF, ampliaciones XLSX, embeddings/harness, modelos/remotos,
OAuth/pagos/publicación, Home o fase6. No declara5a completa. La continuación
actual autoriza el ciclo completo, con parada máxima16:43:39UTC del20/10/2026;
si no se termina, se conserva el cambio abierto con evidencia y prompt de relevo.

## Capabilities

### New Capabilities

Ninguna; se amplían capacidades existentes.

### Modified Capabilities

- `local-extraction`: PDF OCR explícito, contención, persistencia, lector y paneles.
- `local-text-search`: proyección inferida por página/región y presentación fiel.
- `core-operations`: lector nuevo y DTOs estrictos compartidos/aditivos.
- `local-workspace`: migración016, recuperación, retención y snapshots completos.
- `local-mcp`: lectura autorizada y catálogo actualizado sin ampliar permisos.

## Impact

Rust conserva validaciones/cercas/cuotas. Un quinto sidecar macOS sólo rasteriza;
Vision reutiliza el auxiliar existente, con PNG real y hash, sin cambiar su perfil.
SQLite/originales siguen siendo autoridad. Cliente/checkpoint15 y fixture real de
cinco formatos/cinco selecciones/41 tablas están conservados antes de016. Los
límites/alternativas técnicas son decisiones propuestas, no requisitos nuevos del
usuario; aceptación real incluye calidad OCR declarada, procesos, recuperación,
UI headless, regresión y gates300/150 sin relajación.
