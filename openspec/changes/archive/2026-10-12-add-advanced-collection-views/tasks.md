# Tasks

## 1. Contrato y recuperación

- [x] 1.1 Extender DTOs/layout/config/resultado y registrar migración 006/v6; regenerar contratos, probar v5→v6 con JSON antiguo intacto, fallo/backup/restauración y versión futura no tocada, y actualizar documentación de esquema antes de marcar la tarea.
- [x] 1.2 Probar configuración/layout persistidos desconocidos y revisiones viejas: lectura/edición rechazada sin reemplazo, JSON crudo recuperable en backup; documentar límites de recuperación de `collection-views`.

## 2. Consultas y proyección Rust

- [x] 2.1 Implementar/filter validar AST compuesto y ne/lt/le/gt/ge, manteniendo AND legacy; probar tipos/IDs/límites/grupos vacíos, null/false/cero, decimal exacto, ramas inválidas/cálculos fallidos y orden/paginación; documentar contrato/ejemplos de filtros.
- [x] 2.2 Implementar board/calendar/gallery ad-hoc y guardados, grupos por ID y matching_count; probar grupos select/checkbox/null/vacíos, recálculo tras patch y IDs/revisiones intactos, meses extremos/bisiestos/rangos solapados/sin fecha/tipos ajenos, y documentar proyección/cobertura por página.

## 3. Clientes compartidos

- [x] 3.1 Integrar layouts/formularios/rendering en escritorio con resultados/config/layout publicados coherentemente; probar navegación/cierre/conflicto/fallo de consulta sin relabelar registros viejos, tarjetas/errores/apertura original, accesibilidad de controles y paginación explícita; actualizar guía de UI.
- [x] 3.2 Añadir paridad CLI/Tauri y recorrido nativo offline/backup de vistas avanzadas: filtros complejos, tablero actualizado, calendario civil y galería comparten IDs/originales; verificar JSON/códigos de salida, reapertura y resultados sintéticos documentados.

## 4. Integración y cierre verificable

- [x] 4.1 Ejecutar regresiones Rust/CLI, fmt, Clippy, contratos, frontend/adaptador/nativo, release y OpenSpec estricto; revisar diff contra snapshot y los cinco requisitos/ocho escenarios; registrar ADR/evidencia y reconciliar plan sin declarar toda 3c completa.

## Workflow follow-up

- Sincronizar sólo requisitos implementados y verificados; archivar después del cierre.
- Proponer templates/bases incrustadas como siguiente tramo de 3c, con evolución de bloques y referencias compartidas; todavía no implementados aquí.
- Después continuar fases 4/5a y demás dependencias; no activar proveedores remotos ni cuentas para avanzar este cambio local.
