# Proposal

## Why

La cola local ya está verificada, pero sólo extrae UTF-8 literal: no puede recuperar texto de un PDF ni señalar su página sin confundir bytes derivados con bytes originales. El siguiente recorte independiente de 5a incorpora PDF textual **con soporte acotado**, sin depender de la aceptación nativa pendiente de Home.

## What Changes

- Añadir el extractor explícito `pdf_text` v1 para originales administrados `.pdf` (PDF 1.0–1.7) de hasta 1 MiB y 1–100 páginas, no cifrados. Su primer subconjunto admite texto directo con fuentes Type1 latinas estándar y codificaciones Standard/WinAnsi; fuentes, estructuras o conversiones no admitidas producen un motivo identificable, no una transcripción incompleta presentada como completa.
- Conservar texto derivado por página, fragmentos UTF-8 de hasta 4.096 bytes y un manifiesto que diferencia páginas con texto de páginas sin texto extraíble. No prometer lectura visual, OCR ni reproducción de columnas/tablas.
- Añadir `extraction.pdf_fragments` y `noteka extraction pdf-fragments`: página física 1-based y offsets del **texto derivado de esa página**, nunca del binario PDF. Las ocho operaciones actuales, sus shapes y el extractor literal permanecen compatibles; catálogo propuesto de 49 operaciones/API v1. Ambos lectores de fragmentos rechazan el tipo de generación ajeno.
- Ejecutar el parser en un helper Rust local descartable, supervisado por el núcleo, sin UI, daemon, red ni modelos. Reutilizar claim, fencing, pausa/cancelación, retry, lease y publicación atómica; acotar también heap, streams, IPC y vida del helper ante caída del padre.
- Migrar mediante `009.sql` aditiva a SQLite v9, con backup previo y preservación exacta de tablas/filas existentes. Contabilizar el presupuesto compartido de 32 MiB sobre **texto retenido real**, no tamaño de PDFs; mantener snapshots completos de 64 MiB, namespace base v2 y whitelist de lotes sin ampliar.

## Non-goals

PDF genérico o lectura perfecta; fuentes embebidas/Type0/TrueType/Type3, ToUnicode y Differences en esta primera versión; texto en Form XObjects, annotations, formularios o archivos incrustados; OCR, render/preview, geometría, tablas estructuradas, contraseña/decryption, Excel, indexación FTS de adjuntos, embeddings, harness, MCP, modelos, proveedores remotos, UI de cola y scheduler. Sin cambios de Home/composición/clima/Goals ni pruebas nativas sin nueva autorización. Firma, notarización y distribución son otra entrega: sólo preparación e invocación local del helper.

## Capabilities

### New Capabilities

Ninguna: PDF amplía la extracción local y la procedencia existentes, no crea otra autoridad de contenido.

### Modified Capabilities

- `local-extraction`: selección PDF acotada, manifiesto/localizadores por página, helper supervisado, publicación y presupuesto mixto sin alterar UTF-8 literal.
- `core-operations`: lectura PDF aditiva, catálogo/CLI y errores compartidos; compatibilidad de contratos previos y de lotes con derivados PDF.
- `local-workspace`: migración recuperable a v9 y backup/restauración de resultados PDF, sin reescribir originales o estados existentes ni reconocer ownership de otra ruta.

## Impact

Núcleo: contratos/dispatcher, validación de fuentes, worker/storage, lectura PDF nueva, migración/backup y registry batch. Añadir `crates/noteka-pdf-worker/`, preparación local del binario para CLI/host Tauri y tests sintéticos; sin parser dentro del proceso de escritorio ni permisos de shell en frontend. Mantener Rust 1.99.0, documentos v1/v2/v3, IDs y los límites públicos actuales.

**Decisiones del usuario conservadas:** núcleo offline, SQLite/originales autoritativos, operaciones compartidas, ejecución explícita y pantalla no autorizada. **Elecciones técnicas propuestas:** `lopdf =0.45.0` MIT sin features predeterminadas, subconjunto conservador, helper y operación de lectura separada; [investigación versionada](research.md) y [diseño](design.md). El benchmark/publicidad de una biblioteca no prueba calidad ni límites en Noteka. No hay acceso externo bloqueante: compilación, árbol transitivo/licencias, corpus y mecanismos de contención deben verificarse durante apply; si fallan, se revisa el plan antes de ampliar soporte o sustituir parser.

Estado: **implementado, verificado, sincronizado y archivado, 21/21 tareas**, 14/10/2026.
SQLite v9/API v1/49 operaciones, helper/cola/lector PDF integrados;
[evidencia](../../../../docs/verification-pdf-extraction.md),
[guía](../../../../docs/pdf-extraction.md) y [ADR 0012](../../../../docs/decisions/0012-bounded-pdf-worker.md).
Corpus propio de 235 casos por parser y CLI reales debug/release, migración,
backup, concurrencia y mediciones locales; checklist/evidencia contrastados.
Home permanece 9/17 sin sincronizar ni
archivar. Este cambio no completa 5a ni toda dependencia 5b.
