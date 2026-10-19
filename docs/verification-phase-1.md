# Verificación de fase 1: editor y originales

Fecha: 12 de octubre de 2026. `add-block-editor`; no demuestra fases posteriores.

## Evidencia reproducible

- `cargo test --offline -p noteka-core -p noteka-cli`: **22 tests** aprobados: 1 unitario de migración, 11 de núcleo inicial, 1 de validación del workspace, 6 de bloques/adjuntos y 3 de CLI real.
- `npm run test:editor`: cinco tests del adaptador: round-trip, upgrade explícito v1, originales por referencia, IDs/nodos inválidos y recuperación JSON.
- `npm run build`: contratos frescos, svelte-check sin diagnósticos y bundle de producción.
- `cargo fmt --all` y `cargo clippy --offline --workspace --all-targets -- -D warnings`: sin advertencias.
- `npm run test:native`: **43 checks** aprobados en 3 sesiones WKWebView/Tauri, app y CLI con red denegada. Evidencia fuera de Git: `/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/opencode/noteka-native-rK55mC/results.json`.
- `npm run tauri build -- --no-bundle`: release Apple Silicon construido; no firmado/notarizado/publicado.
- `openspec validate add-block-editor --strict`: aprobado; verifica documentos, no sustituye las pruebas anteriores.

## Trazabilidad por escenario

| Capability / escenario | Prueba |
| --- | --- |
| block-editor / edición y reapertura | `rich_roundtrip_ids_conflicts_and_validation_are_atomic`, fixture compartida y test de adaptador |
| block-editor / documento desconocido | `unknown_v2_properties_are_recoverable_not_replaced`, regresiones v1/futuros, exportación JSON intacta |
| block-editor / conflicto | concurrencia SQLite y runner nativo: buffer enriquecido + imagen intactos, CLI no sobrescrito, recuperación JSON con hijos/referencias |
| block-editor / offline/undo/bloques | runner WKWebView real: headings, listas/checklist, citas/código/enlace, undo/redo, `/`, importación/render de imagen y archivo, exportación y reapertura |
| managed-attachments / importación/referencia | `attachments_copy_references_failed_import_and_retention` y procesos CLI |
| managed-attachments / inexistencia/copia fallida | ID ausente, trigger de commit fallido, exceso de tamaño, contenido alterado y symlinks |
| managed-attachments / exportación fiel | `attachments_backups_restore_exports_and_missing_files`: imagen+archivo, JSON/relaciones, destino ocupado |
| managed-attachments / Markdown | mismo test: warning, heading, checklist y fence de código |
| local-workspace / backup abierto | writer con título no confirmado mientras API de backup toma snapshot, más regresión bootstrap |
| local-workspace / originales | snapshot+metadatos exactos, restauración de referencias, rechazo de faltantes/alterados |

## Límites

Adjuntos <=64 MiB; texto pegado plano; sin importación Markdown/Notion ni GC físico. Archivo gestionado no se ejecuta al abrir la nota. No hay crash recovery del buffer no guardado, colaboración, OCR, modelos ni benchmarks de rendimiento. Las revisiones de usuario están en [review-queue.md](review-queue.md). Las dependencias de Google/proveedores no fueron autenticadas ni probadas mediante sesiones del usuario.

## Revisión de completitud, corrección y coherencia

Se verificaron 6/6 tareas, 5/5 requisitos y 10/10 escenarios. Se revisó el diff contra el snapshot anterior fuera de Git y los archivos nuevos; no se descartó trabajo previo. Sin hallazgos críticos pendientes. La arquitectura comparte core/CLI/Tauri, el editor adapta semántica y SQLite/adjuntos son autoritativos. Se corrigieron fallos detectados por el recorrido nativo: clonar proxies de Svelte, habilitación de controles en tests y reemplazo involuntario de un adjunto seleccionado al insertar otro.
