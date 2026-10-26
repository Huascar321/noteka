# Tasks

## 1. Exact verification refactor

- [x] 1.1 Conservar fuentes/binarios16/API57 previos como baseline diagnóstico y documentar fallos, A/B y prueba de equivalencia del algoritmo; verificar pins, catálogos y alcance sin cambios de formatos o reglas.
- [x] 1.2 Comparar todas las copias/tipos/identidades contra referencias canónicas y diferir FTS esperado al fallback; verificar debug/release con oráculo anterior independiente, corpus repetido/único/vacío y corrupción de filas/tipos/canónico/postings/estadísticas, presupuesto y pureza; documentar el refactor.
- [x] 1.3 Contrastar clientes57 antes/después sobre fixture mixta y catálogos completos, ranking/texto/cursores/cobertura y fallos fuera de filtro/sin hits; verificar backup/desconocidos y conservar resultados fuera de Git.

## 2. Integrated acceptance

- [x] 2.1 Ejecutar regresión Rust/SDK por perfil, frontend y5146 checks headless, fmt/Clippy/contracts y build host no-bundle sin ventana; comprobar fuentes protegidas/Home y documentar fallos sin exceptuarlos.
- [x] 2.2 Medir A/B y recursos con corpus fijado; aprobar dos pares consecutivos de gates originales query300/lock150 sin compilación/inferencia concurrente, conservando todos los intentos y sin relajar validación/caps/targets.
- [x] 2.3 Revisar diff/diseño/oráculo y actualizar ADR/plan/guía/evidencia con resultados reales; validar OpenSpec estricto y verificar que lector de tablas sigue sin sincronizar/archivar hasta su cierre propio.

## Workflow follow-up

- Archivar sólo este refactor tras6/6 verificado. Specs omitidas por skip_specs: no hay comportamiento nuevo ni deltas que sincronizar.
- Retomar add-local-pdf-table-reader y completar toda5a antes de crear5b; después5c, nunca fase6. Home9/17 permanece sin editar/sincronizar/archivar.
- El usuario autorizó el ciclo autónomo completo de fase5; no hay límite horario vigente ni permiso para activar proveedores/modelos de Noteka.
