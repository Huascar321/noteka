# Tasks

## 1. Persistencia y tipos
- [x] 1.1 Añadir contratos/migración v4 y operaciones colección/propiedad/registro sobre IDs compartidos; probar creación atómica, tipos, null/patch, archivo, fechas/dinero y rechazo de formatos/referencias incompatibles.
- [x] 1.2 Implementar relaciones de propiedad/inversa y concurrencia; probar múltiples colecciones, conflictos, migración fallida y backup/restauración con valores/originales intactos.

## 2. Vistas y clientes
- [x] 2.1 Guardar vistas y consultar filtros/orden/paginación deterministas; probar configuración inválida, decimales exactos, campos proyectados y paridad CLI/Tauri; regenerar/documentar contrato.
- [x] 2.2 Integrar controles de colecciones tabla/lista, esquema/captura/valores e incorporación por ID; proteger buffers/cierre/conflictos y abrir originales; probar recorrido nativo offline.

## 3. Cierre
- [x] 3.1 Ejecutar regresiones, fmt/Clippy, contratos/frontend/release/OpenSpec; revisar diff/diseño/escenarios y documentar ADR/evidencia/límites antes de sincronizar/archivar.

## Workflow follow-up
- Continuar 3b después del motor verificado; no mezclar fórmulas/harness ni presentar controles JSON como paridad total con Notion.
