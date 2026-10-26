# Tasks

## 1. Verified Rust reader

- [x] 1.1 Preservar baseline16/API56, clientes/catálogos completos y fixture real mixta con seis formatos y desconocidos, backup/restauración, antes de integrar; comprobar pins y documentar recuperación sin DDL (core-operations/local-extraction).
- [x] 1.2 Implementar lectura Rust de PDF OCR completo/original, proyección con biblioteca cerrada, manifiesto/celdas/exteriores/ordinals y paginación scoped; tests debug/release verifican fuente/corrupción exterior/config/fence/previous generation, cruces/ambigüedad/cupos, vacíos y cursores tras cambio/restore; documentar contrato (local-extraction).
- [x] 1.3 Exponer API57/CLI/scoped/MCP29/49/50 y bindings, conservar56 contratos previos/roots/errores/batch9, verificar SDK modern/legacy/auto, paridad sin helpers, permiso previo y rechazo batch; ejecutar ejemplos de guía y contrastar cliente56/backup (core-operations/local-mcp).

## 2. Desktop reader

- [x] 2.1 Añadir criterios explícitos de tabla/página/grid/header y lectura20 celdas en panel/controlador existentes; tests/check/build verifican request exacto, cobertura diferenciada, exteriores referenciados, HTML escapado, invalidación/error/lifetime/cupo físico sin duplicar reglas Rust, y guía de uso (desktop-shell).
- [x] 2.2 Ejecutar headless dark/light con PDF propio real y golden previo: varias páginas, huecos/regiones múltiples, header/outside/capacity, fuente/canónico/cursores alterados, errores/tardíos, teclado, buffer/guardias y cero requests externos; guardar evidencia fuera de Git (desktop-shell/local-extraction).

## 3. Integrated verification

- [x] 3.1 Contrastar Search individual/agrupado PDF OCR y lecturas con celdas/regiones/procedencia, sin nuevos fragmentos/indexación implícita; comprobar56 contratos anteriores exactos, cuotas/retención/snapshots16 y unknowns/backup, documentando cobertura de tabla versus generación (todas las capabilities).
- [x] 3.2 Ejecutar regresión Rust/SDK por perfil, frontend/headless previos, fmt/Clippy/contracts/build host no-bundle sin ventana, dos pares originales de gates300/150 y extra de lectura tabla; investigar fallos sin cambiar límites, conservar pins/corpus/fuentes y evidencia (todas las capabilities).
- [x] 3.3 Revisar diff/requisitos/diseño, actualizar ADR/plan/config/guías con resultados reales y validar OpenSpec estricto; comprobar pendientes5a completos antes de habilitar5b, sin declarar integración por planificación (todas las capabilities).

## Workflow follow-up

- Sólo tras8/8 verificado, sincronizar/contrastar deltas seleccionados y archivar este cambio. Home9/17 no se edita, sincroniza o archiva.
- Reconciliar toda5a con plan/evidencia; después5b y5c, nunca fase6. Sin modelos/proveedores/OAuth/pagos/publicaciones.
- Sin límite horario vigente. Ante una interrupción, conservar cambio parcial y evidencia externa; no archivar ni sincronizar incompleto.


Autorización vigente21/10: el usuario levantó el límite de una hora; continuar hasta completar5a, luego5b y5c, y detenerse antes de fase6. Los cortes horarios citados arriba son históricos y están revocados. Se conservan todos los invariantes, Home9/17 y la prohibición de activar modelos/proveedores.
