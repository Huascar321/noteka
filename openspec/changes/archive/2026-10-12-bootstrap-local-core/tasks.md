# Tasks

Implementación y verificación de fase 0 completadas. Las tareas cuentan con pruebas SQLite/CLI, recorridos nativos y checks de build; ver `docs/verification-phase-0.md`. Alcance: únicamente `bootstrap-local-core`. Pendiente el workflow de archivo.

## 1. Scaffold y contratos compartidos

- [x] 1.1 Crear frontend Svelte 5 + TypeScript + Vite, host Tauri 2 y workspace Cargo con `noteka-core` y `noteka-cli`; verificar build de frontend y Cargo y registrar versiones/lockfiles reales.
- [x] 1.2 Definir DTOs v1, envoltura de resultado y catálogo de operaciones del diseño; verificar JSON Schema generado y validación de entradas válidas/inválidas (`core-operations`).
- [x] 1.3 Conectar adaptadores Tauri/CLI al mismo núcleo, sin reglas duplicadas; verificar que una operación de dominio equivalente desde ambos produzca el mismo resultado sobre fixtures equivalentes (`core-operations`).
- [x] 1.4 Crear `docs/development.md` con prerequisitos y comandos reales de instalación/build; verificar cada comando documentado en el scaffold y que `noteka-core` compile sin Tauri (`desktop-shell`, `core-operations`).

## 2. Workspace, almacenamiento y recuperación

- [x] 2.1 Implementar crear/abrir workspace con identidad, ruta explícita y default fuera del checkout; verificar rutas, selección y reapertura sobre SQLite real temporal sin red (`local-workspace`).
- [x] 2.2 Crear schema v1, FKs, revisión, transacciones y contención de 2.000 ms; verificar rollback, integridad referencial y `workspace_busy` con dos conexiones reales (`local-workspace`, `core-operations`).
- [x] 2.3 Implementar backup consistente con manifiesto e integridad usando API SQLite; verificar restaurabilidad de un backup tomado con una conexión abierta y datos confirmados (`local-workspace`).
- [x] 2.4 Implementar restauración validada en destino nuevo; verificar IDs/revisiones conservados y rechazo de backup corrupto y destino ocupado sin sobrescritura (`local-workspace`).
- [x] 2.5 Implementar migraciones numeradas con backup previo y rechazo de versiones futuras; verificar fixtures de versión anterior/futura, migración exitosa y falla inyectada con rollback (`local-workspace`).
- [x] 2.6 Documentar formato, rutas, backup/restauración y tratamiento de versiones en `docs/development.md`; verificar ejemplos contra workspaces sintéticos y que no aparezcan datos runtime en Git (`local-workspace`).

## 3. Entidades y documentos semánticos

- [x] 3.1 Implementar CRUD para note/task/project/tag/person, identidad, títulos, timestamps y estados básicos; verificar casos válidos y rechazo de tipo/título/estado inválidos y default de estados (`knowledge-entities`).
- [x] 3.2 Implementar tags area/resource/topic sin duplicación; verificar creación, modificación, filtros e identidad compartida de áreas/recursos (`knowledge-entities`).
- [x] 3.3 Implementar documento v1 de párrafos/texto y límites del diseño; verificar round-trip, orden e IDs y rechazo de duplicados/tamaño/profundidad (`knowledge-entities`).
- [x] 3.4 Conservar lectura JSON de formatos desconocidos y rechazar su reemplazo por documentos editables; verificar fixtures de bloque y versión desconocidos sin pérdida de campos (`knowledge-entities`).
- [x] 3.5 Implementar relaciones project/tag/person, consultas por ambos extremos y unicidad; verificar extremos inexistentes/incompatibles/otro workspace, ausencia de self-links y vínculo repetido sin duplicado (`knowledge-entities`).
- [x] 3.6 Implementar archivo/restauración sin cascada y listados paginados con filtros y orden estable; verificar desaparición/reaparición en activos, relaciones preservadas y páginas sin duplicados sobre datos estáticos (`knowledge-entities`).
- [x] 3.7 Documentar tipos, estados, relaciones, paginación y formato JSON en `docs/development.md`; verificar fixtures y ejemplos contra el contrato generado (`knowledge-entities`).

## 4. Operaciones, concurrencia y CLI

- [x] 4.1 Aplicar expected_revision y escritura atómica de contenido/revisión/historial; verificar conflicto entre dos escritores, revisión omitida, operación parcialmente inválida y falla de persistencia sin commit (`core-operations`).
- [x] 4.2 Implementar consulta de cambios con origen y revisiones; verificar que mutaciones confirmadas se registren y que operaciones rechazadas no figuren como aplicadas (`core-operations`).
- [x] 4.3 Implementar subcomandos workspace/operations/entity/relation/changes, ruta y entrada por archivo/stdin; verificar todos los grupos sin GUI y con payloads complejos mediante procesos CLI reales (`core-operations`).
- [x] 4.4 Implementar stdout JSON, stderr de logs y exit codes del diseño; verificar JSON parseable y códigos de éxito, validación, not_found, conflicto/contención, almacenamiento y formato no soportado (`core-operations`).
- [x] 4.5 Documentar catálogo, ayuda, errores y ejemplos CLI en `docs/development.md`; ejecutar los ejemplos y comprobar que leen/escriben el workspace seleccionado (`core-operations`).

## 5. Interfaz de escritorio mínima

- [x] 5.1 Implementar selección/creación de workspace, sidebar y listados con estado vacío/error; verificar recorrido de inicio y ruta inválida en el host Tauri sin mostrar datos de otro workspace (`desktop-shell`).
- [x] 5.2 Implementar creación/lectura/edición de notas con párrafos identificables y guardado explícito; verificar IDs preservados, lectura por CLI y reapertura de la nota (`desktop-shell`, `knowledge-entities`).
- [x] 5.3 Implementar estados dirty/saving/saved/error, reintento y cierre con cambios pendientes; verificar error de almacenamiento inyectado y opciones guardar/descartar/cancelar sin perder buffer (`desktop-shell`).
- [x] 5.4 Implementar refresh en activación y manejo de revision_conflict conservando buffer local; verificar modificación por CLI con la nota abierta y ausencia de sobrescritura (`desktop-shell`, `core-operations`).
- [x] 5.5 Implementar archivo/restauración visible y formato desconocido de solo lectura con recuperación JSON; verificar Archive, relaciones conservadas y fixture no editable sin conversión (`desktop-shell`).
- [x] 5.6 Implementar foco, navegación y acciones por teclado sin recursos remotos; verificar crear/guardar/archivar offline y documentar los atajos en `docs/development.md` (`desktop-shell`).

## 6. Verificación integrada y cierre documental

- [x] 6.1 Ejecutar el recorrido completo UI → CLI → cierre/reapertura → backup/restauración sin red ni modelos; registrar evidencia real por las cuatro capabilities y limitaciones que queden.
- [x] 6.2 Ejecutar tests de Rust, checks de TypeScript/frontend, build Tauri y recorridos UI pertinentes con los comandos establecidos; contrastar todos los escenarios con pruebas/evidencia y resolver diferencias entre código, diseño y specs.
- [x] 6.3 Actualizar estado real en README y plan general; ejecutar `openspec validate bootstrap-local-core --strict` y revisión de diff, y verificar que no se incorporaron runtime, credenciales, funciones de fases futuras o tareas marcadas sin evidencia.

## Workflow follow-up

- Verificar completitud, corrección y coherencia antes de sincronizar y archivar; la validación documental sola no cierra el cambio.
- Sincronizar únicamente los deltas implementados y verificados a specs principales y archivar el cambio según el workflow correspondiente.
- Proponer el próximo cambio a partir de las dependencias en `docs/plan-noteka.md`; no incorporar automáticamente toda la hoja de ruta en este bootstrap.
