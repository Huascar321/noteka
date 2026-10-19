# ADR 0001 — Núcleo compartido y escritura local

## Estado

Adoptado para `bootstrap-local-core`, 11 de octubre de 2026 (`America/La_Paz`).

## Contexto

App y CLI deben operar offline sobre los mismos datos, incluso con la ventana cerrada. La primera nota no debe condicionar el formato persistente al futuro editor.

## Decisión

- `noteka-core` concentra DTOs, validación, transacciones y persistencia; Tauri/CLI son adaptadores.
- SQLite incluido mediante rusqlite, WAL, FKs, durabilidad FULL y espera acotada a 2 s.
- Identidad común de entidades, revisión esperada y cambios confirmados dentro de la misma transacción.
- JSON semántico v1 independiente de Tiptap; formatos desconocidos legibles pero no reemplazables.
- Backup por API SQLite, manifiesto/hash, staging y restauración solamente a destino nuevo/vacío.
- Prevalidación privada para no modificar SHM de workspaces futuros. La copia de inspección no se utiliza como backup.
- Sin daemon ni superficie de red; datos por defecto fuera del checkout.
- Tipos TypeScript generados desde JSON Schema Rust; el build detecta desactualización.

## Alternativas y consecuencias

Duplicar dominio en TypeScript o usar Markdown como persistencia complicaría atomicidad/relaciones. Un daemon agregaría lifecycle/transporte sin necesidad actual. Se conserva un único escritor SQLite por vez; los clientes manejan conflictos sin merge automático. La inspección privada añade costo de I/O por apertura: se deberá medir antes de optimizar y mantener el rechazo de versiones futuras sin mutaciones.

No se decidió licencia del producto, distribución, motor vectorial ni protocolo remoto. La fixture v0 sólo prueba recuperación/migración, no declara una versión histórica publicada.
