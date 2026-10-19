# Verificación de fase 3a: motor de colecciones

12 de octubre de 2026. Cambio `add-database-engine`. Implementado y verificado: SQLite v4, 11 operaciones nuevas, UI Collections, tabla/lista, propiedades y registros originales compartidos. No incluye 3b/3c, harness ni modelos.

## Checks ejecutados

- `cargo test --offline -p noteka-core -p noteka-cli`: **41 tests**, 35 core (3 unitarios, 32 integración) y 6 CLI. Siete de colecciones, rollback de dominio/valores/membresías/historial/FTS y migración v3→v4 fallida.
- fmt/Clippy workspace/all-targets sin warnings. Cinco tests de adaptador y contratos frescos.
- svelte-check sin errores/advertencias, Vite y release Apple Silicon `--no-bundle` aprobados.
- **85 checks nativos offline** en tres sesiones WKWebView/IPC/CLI: 75 principal, 2 reapertura, 8 restauración. Evidencia temporal fuera de Git: `noteka-native-rPSr4A/results.json`.
- OpenSpec `validate --all --strict`: 9/9 ítems aprobados antes de sincronizar.
- Revisados archivos nuevos y diff contra snapshot `noteka-before-database.tar.gz`, sin alterar contenido ajeno. Sin commit/push, firma/notarización ni publicación.

## Trazabilidad: 6 requisitos, 9 escenarios

| Escenario | Evidencia |
| --- | --- |
| Incorporar entidad/editar página | `shared_identity_creation_atomicity_patch_projections_and_unknown_values`, CLI parity, tarea incorporada/página original nativas |
| Creación inválida | Sin entidad/membresía parcial; trigger de historial fuerza rollback completo |
| Tipos y vacíos | `every_property_type_validates_values_without_coercion_or_loss`; strings exactas, null, patch, proyecciones |
| Referencias/tipos inválidos | Opciones, tipos, IDs ajenos, fecha civil, float monetario, archivos y esquema futuro rechazados |
| Relación compartida/inversa | `relationships_inverse_archived_targets_and_rollback`, inversa CLI y destino archivado conservado |
| Conflicto/restauración | Revisión vieja app/CLI conserva buffer; backup restaura valores/vista y archivo en propiedad; rollback de migración |
| Reabrir vista filtrada | Filtro decimal numéricamente exacto, columnas/layout guardados, lista restaurada nativa |
| Filtro inválido | Propiedad ajena, operador incompatible, cursores/límites y tipo de valor rechazados |
| Sistema sin modelos | Colección/esquema/registro/patch/vista UI; cancelación/cierre/recuperación, captura compartida; sandbox deny network |

La prueba nativa inicialmente encontró una selección de botón ambigua: «Descartar» coincidía con «Descartar formulario» detrás del modal y dejaba el dialog abierto. El runner ahora busca en el modal activo, prioriza texto exacto y verifica que execCommand haya aplicado la edición; se comprobó protección nativa antes del cierre. No se atribuyó ese fallo del recorrido a pérdida de datos de producción.

## Decisiones y límites

[ADR 0005](decisions/0005-shared-collection-records.md) y [contrato](collections.md). Formularios JSON/IDs funcionales pero provisionales; propiedades inmutables, consultas en Rust hasta 10.000 miembros/128 propiedades, paginación sin snapshot concurrente y sin benchmark de colecciones. No se alteran tipos desconocidos ni se presentan como importes válidos. Renderizar una tabla no implementa fórmulas, rollups, bases incrustadas o IA.
