# Tasks

## 1. Codec y recuperación

- [x] 1.1 Registrar ADR 0020 y fixtures v12 creados con binario durable, verificar backup/contratos previos y guardar evidencia externa antes de editar DDL (local-workspace/core-operations).
- [x] 1.2 Implementar protocolo privado v2 decimal preservando v1; verificar bits, límites, malformed y helper real ambos perfiles, documentar codec (local-extraction).
- [x] 1.3 Agregar migración 013, representación canónica y validación íntegra OCR compartiendo retención; probar migración/rollback/old-client/futuro/backup/restore/snapshots sin helper y documentar formato (local-workspace/local-extraction).

## 2. Operaciones y búsqueda

- [x] 2.1 Integrar image_ocr con admisión, supervisor acotado, source/fences/cuotas/publicación atómica; probar fuente alterada, cancel/retry/concurrencia/errores reales y preservación de generación anterior (local-extraction).
- [x] 2.2 Añadir extraction.ocr_regions al núcleo/CLI/MCP y contratos generados; verificar lectura paginada/integridad/cursor/permisos, API53/MCP25/45/46, otros 50 contratos y batch9 idénticos; documentar comandos (core-operations/local-mcp).
- [x] 2.3 Implementar proyección/selección/búsqueda OCR explícita con hash/localizadores completos; probar oráculo independiente, corrupción, CAS/no-op/cuotas, tipos anteriores y búsqueda sin helper (local-extraction/local-text-search).

## 3. Interfaz y entrega local

- [x] 3.1 Integrar tipo/estados/límites OCR en preparación y lector de 20 regiones; verificar tests de lifetime/buffers/error y aceptación headless ambos temas sin llamadas implícitas; documentar alcance inferido (local-extraction).
- [x] 3.2 Presentar evidencia/localizadores OCR en Search individual y agrupada; verificar paginación/procedencia/errores y regresión de tres tipos previos con cero tráfico externo (local-text-search).
- [x] 3.3 Preparar helper para target/perfil macOS sin instalar componentes y verificar build host no-bundle sin ventanas, distribución de helpers previos y error explícito de disponibilidad (local-extraction).

## 4. Integración completa

- [x] 4.1 Ejecutar regresión Rust debug/release, oráculos/contención PDF/XLSX/OCR, MCP raw/SDK y UI previa sin reducir corpus/cuotas; conservar logs/fallos/hashes fuera de Git (todas las capabilities afectadas).
- [x] 4.2 Aprobar cuatro gates finales sin cambios de corpus/validación: query p95 <=300 ms y lock p95 <=150 ms; investigar fallos y documentar límites de medición (local-text-search).
- [x] 4.3 Revisar código/requisitos/diseño con matriz de escenarios, actualizar plan/docs sólo con resultados comprobados y validar OpenSpec estricto; mantener Home9/17 y cambios ajenos intactos (todas las capabilities afectadas).

## Workflow follow-up

- Sincronizar solamente los cinco deltas terminados, contrastar contenido y archivar add-local-image-ocr después de aceptación y revisión completas.
- Verificar raíz, specs principales y sólo Home activo; continuar siguientes recortes de 5a antes de 5b/5c/6, sin activar proveedores/OAuth/modelos o iniciar 7/8.
