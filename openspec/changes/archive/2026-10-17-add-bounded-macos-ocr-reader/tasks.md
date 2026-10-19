# Tasks

## 1. Protocol and helper

- [x] 1.1 Verificar baseline durable/SDK/probes y desarrollar protocolo Rust estricto con límites, hash/perfil/OS/imagen/regiones y validación adversarial de campos, geometría, ordinales y fronteras/excesos; documentar contratos y ADR. Trazabilidad: contrato, evidencia y cuotas.
- [x] 1.2 Implementar helper Swift PNG/JPEG por bytes, preflight de dimensiones/orientación/frames, perfil local fijo y errores sanitizados; comprobar lectura offline y rechazo de formatos/perfil no disponible sin fallback. Preparador usa sólo SDK instalado y caché externa. Trazabilidad: lector, inferencia y aislamiento.
- [x] 1.3 Implementar watchdog/budget sólo acortable, CPU/core dumps/RSS observado, stdout/stderr acotados y pruebas de procesos/bloqueos/terminación con timeout/reap, sin publicar modos de instrumentación en el binario normal. Documentar método y límites reales de memoria. Trazabilidad: contención y cuotas.

## 2. Corpus and verification

- [x] 2.1 Generar corpus propio MIT PNG/JPEG con esperado independiente previo, español/inglés/acentos/orientación/estilos/blanco y entradas inválidas. Ejecutar helper real con red denegada y validar cada respuesta en Rust; comparar texto/geometría/procedencia y registrar CER/WER sin ajustar esperado al OCR. Exigir CER agregado <=5% en el subconjunto limpio declarado antes de ejecutar, manteniendo errores por fixture visibles. Trazabilidad: lector y evidencia inferida.
- [x] 2.2 Ejecutar tests Rust/protocolo debug/release, fmt/Clippy/workspace/contratos y frontend pertinentes; contrastar fuentes/binarios/core/corpus/gates y catálogos completos con aceptación UI reciente, documentar reutilización delimitada y repetir sólo checks afectados o ante dudas. Preservar Home, cuotas y objetivos sin presentar evidencia compartida como nueva. Trazabilidad: aislamiento y regresión.

## 3. Closure

- [x] 3.1 Revisar los seis requisitos/catorce escenarios y diseño contra código/evidencia, actualizar guía/plan/README/config con resultados reales, validar OpenSpec estricto y diff/pins contra baseline; conservar fallos/límites fuera de Git. Trazabilidad: delta completo.

## Workflow follow-up

- Sincronizar solamente los seis requisitos implementados/verificados y archivar este cambio tras completar las seis tareas. Conservar snapshot/pins y revalidar enlaces/status sin tocar Home activo 9/17.
- Proponer después la integración OCR con persistencia/recuperación/operaciones/lectura/búsqueda explícita; no declarar 5a terminada por un lector aislado. La meta por dependencias sigue hasta fase 6 inclusive, sin fases 7/8 ni activar accesos/modelos/proveedores prohibidos.
