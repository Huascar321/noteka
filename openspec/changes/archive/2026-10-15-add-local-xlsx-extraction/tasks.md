# Tasks

## 1. Persistence and recovery

- [x] 1.1 Registrar ADR 0017 y migración aditiva 011, integrarla a init/migrate y actualizar fixtures de versión sin debilitar verificaciones. Verificar migración/rollback v10, futuros y backups con tests Rust; documentar recuperación. Trazabilidad: local-workspace.
- [x] 1.2 Implementar DTOs/manifiesto público y almacenamiento canónico/chunks con preflight, canonicalidad, hash/owner/semántica y shape/integridad según versión; sumar retención XLSX sin ampliar caps. Tests de fronteras, texto largo/Unicode, corrupción fuera de página y presupuesto mixto; guía del formato. Trazabilidad: local-extraction lectura/presupuesto.

## 2. Explicit extraction

- [x] 2.1 Implementar supervisor descartable XLSX y tests de pipes/flood/hang/crash/hash/versión/esquema, deadline/fence y helper ausente, sin modificar supervisor/parser PDF. Verificar kill/reap y errores saneados; documentar límites reales. Trazabilidad: publicación cercada.
- [x] 2.2 Añadir admisión xlsx_cells y dispatch/publicación/fencing exhaustivos, corregir integridad no-PDF y negar indexación XLSX explícitamente. Tests de pausa/dedup/retry/cancel/stale/lease/config/saturación/reemplazo atómico con generaciones previas conservadas. Trazabilidad: selección/publicación/exclusión FTS.
- [x] 2.3 Integrar lector paginado con validación integral y cursor ligado a scope/op/job/generación/límite; pruebas de hojas vacías/dispersas, celdas largas/fórmulas y cursores inválidos/reemplazados. Verificar restore/read sin helper; documentar procedencia y no recálculo. Trazabilidad: lectura/local-workspace.

## 3. Shared operations and clients

- [x] 3.1 Publicar operación/CLI, policy MCP y contratos generados con 52 operaciones/24-44-45 tools; comparar automáticamente los 51 esquemas anteriores y whitelist de nueve mutaciones con baseline. Tests de subcomando/invoke y permisos/scoped read-only, guías CLI/MCP actualizadas. Trazabilidad: core-operations/local-mcp.
- [x] 3.2 Crear harness CLI real del corpus independiente completo (216 casos) y migración desde binario v10, rollback/restore/integridad/concurrencia. Ejecutar debug/release con red denegada y golden tipado sin usar parser como oracle; preservar evidencia fuera de Git. Trazabilidad: selección/lectura/recuperación.
- [x] 3.3 Añadir pruebas MCP reales SDK modern/legacy y raw de lectura/ejecución/permisos/catálogo XLSX y errores, sin reemplazar regresión previa; verificar sin pantalla/proveedores. Trazabilidad: local-mcp.

## 4. Integration acceptance

- [x] 4.1 Ejecutar fmt/check/Clippy y suites Rust debug/release, corpus/parser/helper XLSX/PDF y contención, CLI/MCP y regresión frontend/headless 606 checks; verificar Home y archivos ajenos por baseline. No marcar por sola validación documental.
- [x] 4.2 Medir integración release (latencia, locks, memoria, corpus/entorno/caps registrados) y pasar dos gates FTS intactos query p95 <=300 ms; conservar fallos y optimizar sin retirar validación ni subir caps/target.
- [x] 4.3 Revisar código/requisitos/escenarios/diseño y diff/links/hashes; reconciliar guía/ADR/roadmap/config con resultados reales. Validar OpenSpec estricto y registrar límites/incertidumbres sin anunciar 5a/5b completas.

## Workflow follow-up

- Sincronizar sólo los cuatro deltas implementados/verificados y contrastarlos con main; archivar después del cierre completo.
- Mantener Home activo 9/17 sin sincronizar/archivar ni usar pantalla sin nueva autorización.

## Verification evidence

[Aceptación final3, matriz de requisitos/escenarios, fallos retenidos y límites](../../../../docs/verification-xlsx-extraction.md).
261 tests Rust por perfil, 216 casos XLSX CLI/parser/helper por perfil, corpus PDF,
migración real v10, SDK/procesos MCP y 606 checks headless aprobados. Dos gates FTS
intactos query p95 111,15–146,42 ms; integración/locks/RSS medidos sin ampliar caps.
Cuatro deltas contrastados con main, 59 requisitos no afectados preservados;
153 source pins y 169 archivos protegidos comprobados. Validación documental
estricta 15/15 antes de archivo; no sustituye los checks de aplicación.
