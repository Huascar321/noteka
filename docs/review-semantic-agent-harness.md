# Revisión de 5b/5c

Alcance revisado: `add-semantic-retrieval` y `add-agent-harness` sobre el árbol
local, sin commit/push. No se modificó, sincronizó ni archivó Home 9/17.

## Hallazgos y correcciones

1. **Huella y espacio vectorial**: el perfil cubre proveedor, runtime, modelo,
   revisión, dimensión, normalización, tarea, prompts, tokenizer, chunking y
   cuantización. La comparación sólo usa la generación activa; no se mezclan
   huellas aunque compartan dimensión.
2. **Activación atómica**: la generación se escribe `building` en transacciones
   cortas y se activa en una transacción final que verifica conteos, digest,
   dimensión y norma. Una falla marca `failed`, borra chunks parciales y conserva
   la activa. La búsqueda nunca lee `building`.
3. **Obsolescencia**: `source_state` compara job→generación, hash del original y
   hash de la entrada FTS cuando existe. Los obsoletos se excluyen por defecto y
   se informan; no se presentan como vigentes.
4. **Fallback**: los errores de proveedor se distinguen de corrupción del índice;
   sólo los primeros degradan a FTS. Un índice corrupto no se oculta como
   “proveedor caído”.
5. **Presupuesto**: se corta antes de exceder bytes; no se trunca una cita. El
   texto exacto duplicado del mismo adjunto se deduplica y se informa.
6. **Harness**: el protocolo JSON rechaza campos desconocidos; apply exige
   preview de la misma ejecución y `allow_apply`; no hay SQL/shell/archivos. Las
   citas se verifican contra hits y quote textual. La cancelación se consulta
   antes de herramienta y de final.
7. **Persistencia**: migraciones 017/018 aditivas, con backup previo y
   restauración. Los tests de downgrade simulan versiones anteriores; el cliente
   v16 rechaza v18 sin modificar el archivo.
8. **Arneses de UI**: se corrigió el uso de `target/` y schema 16 en fixtures;
   no fue un cambio de producto.

## Riesgos residuales

- Calidad real de Jina y del modelo de chat no medida.
- Escaneo exacto lineal; límite 8.192 chunks.
- Snapshot de lotes de 64 MiB puede rechazar índices/trazas grandes.
- El proveedor local es confiable por definición del usuario; sólo loopback.
- La traza no sustituye auditoría de dominio ni historial de entidades.

## Veredicto

Código, contratos, migraciones, documentación y escenarios están alineados con
las tareas y la verificación. No hay aceptación nativa pendiente para 5b/5c;
Home conserva sus ocho tareas nativas fuera de este alcance. Se puede sincronizar
y archivar cada cambio por separado.
