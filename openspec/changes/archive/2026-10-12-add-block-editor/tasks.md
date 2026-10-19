# Tasks

## 1. Persistencia y contratos
- [x] 1.1 Añadir documento v2 y validador de forma, marcas, IDs, límites y referencias; probar round-trip v1/v2, desconocidos y rechazos atómicos.
- [x] 1.2 Migrar SQLite v1→v2 con backup y añadir import/read de adjuntos; probar copia, hash, referencia inválida y rechazo de futuros.
- [x] 1.3 Extender backup/restauración y exportación JSON/Markdown; probar originales faltantes/corruptos, destino ocupado y restauración fiel.

## 2. Editor y verificación
- [x] 2.1 Integrar Tiptap y adaptador semántico con IDs, bloques, `/`, undo/redo y adjuntos; probar UI offline y conflictos sin pérdida.
- [x] 2.2 Exponer operaciones por CLI/Tauri y regenerar contrato; ejecutar tests Rust, tests de adaptador y checks frontend.
- [x] 2.3 Ejecutar recorrido nativo y build release; registrar evidencia, decisiones y revisiones pendientes, validar OpenSpec y archivar sólo lo verificado.
