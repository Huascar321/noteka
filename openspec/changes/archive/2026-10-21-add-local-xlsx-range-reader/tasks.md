# Tasks

## 1. Rust range reader

- [x] 1.1 Implementar DTOs/rangos/encabezados y cursor scoped, compartiendo loader completo sin DDL; verificar casos dispersos, fronteras, corrupción exterior, pureza y cursor en ambos perfiles, y documentar semántica en docs/xlsx-range-reader.md (local-extraction).
- [x] 1.2 Exponer operación API/CLI/scoped/MCP y bindings generados; contrastar55 contratos con baseline independiente16, verificar permisos/SDK y rechazo batch, cliente anterior/backup/restore y actualizar guía ejecutable (core-operations/local-mcp).

## 2. Desktop reader

- [x] 2.1 Integrar criterios de rango y presentación de encabezados/celdas/cobertura en lector existente; tests de controlador y check/build verifican solicitudes exactas, cambios/errores/descarte tardío sin duplicar reglas Rust (desktop-shell).
- [x] 2.2 Verificar UI headless dark/light por teclado, paginación, encabezados hostiles, buffers/guardias, error/cambio de criterios y cero requests externos; registrar evidencia fuera de Git y guía (desktop-shell/local-extraction).

## 3. Integrated verification

- [x] 3.1 Ejecutar regresión Rust debug/release, SDK por perfil, contratos, fmt/Clippy/frontend y headless previos; dos pares de gates intactos<=300ms query/150ms lock; registrar fallos/evidencia y proteger Home/nativos (todas las capabilities).
- [x] 3.2 Revisar diff/requisitos/diseño, actualizar estado/plan/config/documentación con resultados reales y validar OpenSpec estricto, comprobando preservación de contratos/archivos/procedencia y tareas pendientes (todas las capabilities).

## Workflow follow-up

- Sólo tras aceptación completa, sincronizar deltas verificados en mirror externo, contrastar main y archivar este cambio; nunca Home.
- Continuar pendientes reales5a (PDF tables), luego5b/5c por dependencia; detenerse al vencer la hora aunque este cambio esté incompleto.
