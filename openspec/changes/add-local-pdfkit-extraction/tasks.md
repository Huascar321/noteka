# Tasks

## 1. Baseline y recuperación antes de DDL

- [x] 1.1 Preservar542 fuentes/22 builds y preparar fixtures reales v13 vacía/mixta con backup antiguo; documentar ADR0022/recuperación y verificar SQL/filas/hashes/control/ownership/catálogos antes de escribir014.

## 2. Consumidor Rust

- [x] 2.1 Implementar runtime privado acotado y ejemplo, con documentación; verificar archivos nativos por ambos perfiles, respuestas hostiles, cancelación, límites, errores y recolección sin workspace o publicaciones.

## 3. Persistencia y cola

- [ ] 3.1 Implementar JSON canónico/chunks, tablas014, registry/preflight/backup/restore/snapshots/cuota y documentación; verificar migración real13, rollback antes/después de versión, cliente antiguo/futuro, configuración desconocida e integridad completa.
- [ ] 3.2 Integrar extractor explícito en elegibilidad/claim/run/publicación/cancelación y documentarlo; verificar fuentes/revisiones/cercas, retención mixta32MiB, fronteras/errores y ausencia de procesamiento al importar/encolar/leer/migrar/buscar.

## 4. Operaciones y transportes

- [ ] 4.1 Implementar extraction.pdfkit_pages/CLI JSON y guía de lectura; verificar páginas/cobertura/cursor ligados, ausencia de helper, tipos ajenos, rechazos FTS y equivalencia núcleo/CLI.
- [ ] 4.2 Exponer lectura MCP y catálogo/bindings compartidos, con documentación; verificar53 contratos previos,54 operaciones,26/46/47 tools, siete exclusiones/batch9 y SDK real offline/cancelación/permisos.

## 5. Packaging y aceptación conjunta

- [ ] 5.1 Preparar helper/staging por perfil y documentar procedencia; verificar empaquetado host no-bundle sin ventanas, perfiles reales/pins, guardias previas y ausencia de activación automática.
- [ ] 5.2 Completar regresión Rust/UI headless/MCP/corpus, cuatro gates originales, revisión/matriz/docs/plan y OpenSpec estricto; registrar sólo resultados comprobados y conservar fallos, originales, buffers y Home9/17.

## Workflow follow-up

- Sincronizar/contrastar sólo deltas implementados y verificados; archivar únicamente después de8/8 tareas completas.
- UI/FTS de PDFKit, OCR dePDF y ampliaciones Excel requieren otros recortes; no declarar toda5a por cerrar este backend.
- A las2026-10-18 02:16 UTC detener de forma segura si5a sigue incompleta, con informe de estado recuperable. No reiniciar contador ni archivar tareas pendientes.
