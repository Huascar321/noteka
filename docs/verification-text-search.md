# Verificación: búsqueda textual local

12 de octubre de 2026, cambio `add-local-text-search`. Tramo FTS de 5a, no extracción ni embeddings.

## Checks ejecutados

- 32 tests Rust aprobados (`cargo test --offline -p noteka-core -p noteka-cli`): 2 unitarios de migración, 25 de core/integración y 5 CLI. Incluye tres tests FTS nuevos, rollback de rebuild/escritura y migración v2→v3 fallida.
- 5 tests de adaptador; contratos frescos; svelte-check sin errores/advertencias; bundle y release Apple Silicon `--no-bundle` aprobados.
- fmt y Clippy workspace/all-targets sin warnings.
- 70 checks nativos offline en tres sesiones WKWebView/IPC/CLI. Evidencia: `noteka-native-USNyK6/results.json` bajo el directorio temporal de pruebas fuera de Git.
- OpenSpec estricto del cambio aprobado. La validación global intermedia encontró sólo el nuevo cambio de colecciones aún sin sus deltas; no es un fallo de la aplicación ni de este cambio.
- [Benchmark reproducible](search.md): p95 directo 16,69 ms; operación completa/preflight 22,92 ms en 10.000 notas, caches calientes, M1 Max 64 GiB.

## Trazabilidad de los cinco escenarios

| Escenario | Evidencia |
| --- | --- |
| Bloque enriquecido y filtros | `nested_provenance_filters_pagination_and_literal_inputs`, CLI parity, localizador qp nativo |
| Consulta inválida | Límites, cursores inválidos, operadores/SQL tratados literalmente, entidades intactas |
| Actualizar y archivar | `updates_archiving_stale_revisions_rebuild_and_atomic_index_failure`: texto previo fuera, revisión stale excluida, archivo filtrado, fallo revierte todo |
| Rebuild y formato futuro | `rebuild_unknown_and_migration_backup_preserve_originals`, migración/backups y recuperación JSON sin cambio |
| Buscar/abrir offline | Resultado abre original con bloque resaltado por decoración ProseMirror; cancelación preserva buffer; rebuild UI no incrementa revisión; restauración conserva origen |

Revisados contratos, call flow de revisiones, migración/backup, consultas, adaptador/localizador y diff contra snapshot inicial. 4/4 tareas, 3/3 requisitos y 5/5 escenarios completos. Se corrigió localización basada en mutar DOM: ProseMirror administra su DOM; ahora el resaltado usa una decoración de plugin estable, no una clase que pueda desaparecer al enfocar.

## Límites

Índice textual sin archivo/OCR/modelo; cursor sin snapshot concurrente; medición no incluye UI ni caches frías. Release no firmado ni notarizado. No se enviaron notas ni fixtures a proveedores. Lo que requiere acceso externo sigue en review queue.
