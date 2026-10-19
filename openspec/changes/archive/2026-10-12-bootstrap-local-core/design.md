# Design

## Context

El repositorio contiene documentación y planificación, sin código de producto. Ver [proposal.md](proposal.md) para motivación y alcance y [el plan general](../../../docs/plan-noteka.md) para la visión y dependencias posteriores. Las cuatro specs delta son contratos de esta primera entrega; el resto de la hoja de ruta no se implementa aquí.

El equipo de referencia es macOS Apple Silicon, M1 Max con 64 GiB. Este cambio necesita permitir escritura y lectura offline desde la app y desde un CLI, incluso con la ventana cerrada.

## Goals / Non-Goals

**Goals:**

- Separar dominio, almacenamiento y adaptadores de cliente desde el primer recorrido ejecutable.
- Conservar identidades y contenido al reiniciar, migrar o restaurar.
- Hacer detectables los conflictos entre app y CLI y los fallos de almacenamiento.
- Definir un formato mínimo que pueda evolucionar hacia bloques enriquecidos y colecciones sin reiniciar los datos.

**Non-Goals:**

- No implementar un daemon, event sourcing, sincronización ni un protocolo remoto.
- No crear esquemas vacíos de índices vectoriales, proveedores, adjuntos o bases de datos personalizadas.
- No implementar editor enriquecido, auth ni inferencia. La UI inicial usa párrafos de texto y no transforma formatos desconocidos.

## Decisions

### 1. Biblioteca Rust reutilizable y adaptadores delgados

Crear un workspace Cargo con `crates/noteka-core`, `crates/noteka-cli` y el host `src-tauri`. Svelte 5 + TypeScript + Vite vive en `src/`. Tauri y CLI traducen entrada/salida y llaman al mismo núcleo. El dominio no importa Tauri ni depende de una ventana.

Propuesta de dependencias: `serde` para contratos, `schemars` para JSON Schema, `clap` para CLI, `rusqlite` con SQLite incluido y soporte de backup, y generación de UUIDs. Elegir versiones estables compatibles en el scaffold y registrar lockfiles. No se necesita un ORM ni un servidor HTTP para este alcance.

Alternativas: lógica en TypeScript duplicada para CLI rompe la paridad; un daemon agrega transporte y lifecycle antes de necesitarlo. Un servicio local se podrá añadir después conservando el contrato.

### 2. Workspace con metadatos y SQLite consistente

El directorio contiene `noteka.sqlite` y configuración no secreta del workspace. La ubicación por defecto se resuelve con el directorio de datos de la aplicación; ambos clientes aceptan una ruta explícita. No se usa el cwd como ubicación implícita de datos. La selección y apertura verifican identidad y versión del workspace antes de exponer sus datos.

Configurar claves foráneas, WAL, durabilidad explícita y `busy_timeout` inicial de 2.000 ms. Usar transacciones cortas; no mantenerlas durante interacción UI ni futuros llamados a modelos. Los errores de contención se traducen a `workspace_busy`. No reintentar indiscriminadamente operaciones que quizá llegaron a commit.

En un workspace existente, comprobar primero su versión en modo de lectura. Si es futura, no abrir para migración o escritura ni cambiar journaling. En workspaces compatibles se aplican migraciones numeradas con backup previo. SQLite serializa escritores; las revisiones impiden sobrescrituras lógicas.

Alternativas: un JSON/Markdown por nota requiere resolver transacciones y relaciones aparte; varias bases por cliente producen duplicación. Una base compartida con límites claros cubre esta entrega.

### 3. Modelo mínimo normalizado

Tablas iniciales propuestas:

| Tabla | Campos y límites |
| --- | --- |
| `workspace_meta` | ID, versión de esquema, zona horaria, timestamps; una fila por workspace. |
| `entities` | ID UUID, tipo, título, revisión, `archived_at`, timestamps y payload validado según tipo. |
| `documents` | `entity_id` único, versión y JSON semántico; inicialmente para notas. |
| `relations` | ID, origen, destino, tipo; FKs y unicidad de la relación lógica. |
| `changes` | ID, request ID, entidad, operación, cliente, revisiones previa/nueva y timestamp. |

La revisión vive en la entidad; el documento comparte esa revisión. Timestamps son instantes UTC; la zona predeterminada se obtiene de la configuración local y se conserva como IANA en el workspace. Las fechas civiles de tareas se incorporarán en el cambio de productividad.

El payload está tipado por Rust, aunque se serialice a JSON. Tareas usan `todo`/`done`; proyectos `planned`/`doing`/`done`/`ongoing`/`on_hold`; tags requieren `area`/`resource`/`topic`. Notas y personas no necesitan propiedades adicionales ahora. Título vacío tras trim se rechaza. El tipo y ID de una entidad no se pueden mutar mediante `entity.update`.

Relaciones iniciales: `project` desde nota/tarea a proyecto, `tag` desde entidades no-tag a tag, `person` desde entidades no-person a person. Sin self-links. Un edge se consulta por ambos extremos; no se crea un edge inverso duplicado. Al añadir/quitar relación se exige revisión de origen y se incrementa esa revisión junto al registro de cambio; se valida el destino dentro de la misma transacción. Una relación repetida devuelve la existente sin otra mutación.

Archivar actualiza estado y revisión; no borra filas ni aplica archivo en cascada. `entity.get` puede leer archivadas; `entity.list` excluye archivadas por defecto y acepta filtro explícito. Propuesta de paginación: cursor sobre `(created_at, id)`, límite por defecto 50, máximo 200 y desempate por ID. No prometer snapshot consistente del listado si hay escrituras entre páginas.

Alternativas: tablas separadas sin identidad común dificultan relaciones universales; una única estructura JSON sin validación oculta tipos. El esquema mínimo elegido admite la futura migración de built-ins a colecciones compartidas sin construirlas ahora.

### 4. Documento semántico v1

Usar el formato de ejemplo de `docs/plan-noteka.md`: `schema_version: 1`, `blocks[]`, cada bloque con ID, `type: paragraph` y `content[]` de texto. El bootstrap acepta únicamente este formato, sin rich text ni nodos externos. Límites iniciales propuestos: 1 MiB de JSON UTF-8, 10.000 bloques y profundidad máxima 32; deben documentarse y probarse.

Una modificación del documento exige validación completa. La lectura `entity.get` conserva el JSON crudo y reporta si el formato es editable. Si ya hay bloques o versión desconocidos, se permite recuperar su contenido pero se rechaza modificar el documento, aunque la entrada nueva intente reemplazarlo por un párrafo. El archivo reversible de la entidad sigue siendo posible porque no toca el documento.

La UI usa campos de texto por párrafo y mantiene el ID de los párrafos conservados. Insertar uno crea ID nuevo; eliminarlo es una edición explícita. El adaptador Tiptap se implementará en otra fase, con conservación del esquema y migración cuando sea necesaria.

Alternativa: persistir directamente el estado completo del editor acelera el prototipo, pero acopla el contenido a plugins y estados de presentación. El formato semántico prioriza portabilidad.

### 5. Contrato de operaciones v1

Operaciones mínimas: `workspace.create`, `workspace.open`, `workspace.backup`, `workspace.restore`, `operations.describe`, `entity.create`, `entity.get`, `entity.list`, `entity.update`, `entity.archive`, `entity.restore`, `relation.add`, `relation.remove`, `relation.list`, `changes.list`.

La envoltura devuelve `api_version`, `request_id`, `ok` y `data` o `error { code, message, details }`. Los DTOs del núcleo generan JSON Schema; el build verifica que el contrato consumido por TypeScript esté actualizado. No admitir campos desconocidos de entrada como opciones activas sin validarlos.

Las actualizaciones de entidades usan `expected_revision`. Dentro de una transacción se compara la revisión, se valida contenido y referencias, y se escriben datos, revisión incrementada y registro de cambio. `revision_conflict` incluye la revisión actual; el cliente decide recargar o replantear su operación. No hay merge automático.

Este cambio registra mutaciones, pero no ofrece todavía deshacer de dominio ni lotes multioperación con preview. Una actualización individual sí es atómica; los lotes y preview se formalizan en `extend-agent-operations`.

### 6. CLI JSON y ayuda

El CLI `noteka` tendrá subcomandos `workspace`, `operations`, `entity`, `relation` y `changes`, equivalentes al catálogo. Usa `--workspace <ruta>` cuando corresponda, `--json` para salida estructurada e `--input <archivo>` o stdin para payloads complejos. No introducir comillas shell como requisito para enviar documentos grandes.

Códigos propuestos: 0 éxito; 2 validación/argumentos; 3 no encontrado; 4 conflicto o contención; 5 almacenamiento; 6 formato no soportado. `stderr` para logs/progreso; `stdout` un JSON por invocación en modo estructurado, incluidos errores. La ayuda humana puede usar texto sin `--json`.

Las pruebas comparan resultados de dominio app/CLI sin exigir que coincidan request IDs o timestamps generados independientemente. El CLI funciona sin la GUI y no requiere su sesión ni sus credenciales.

### 7. Escritorio inicial y buffer local

Sidebar mínima con las seis secciones previstas en la spec. Listados vacíos muestran su estado; notas permiten crear/editar título y párrafos; otras entidades se crean mediante CLI en esta fase y se pueden consultar, archivar y restaurar desde UI. No mostrar My Day, búsqueda, editor rico ni AI como controles operativos antes de implementarlos.

Guardado explícito con atajo y botón, estado `dirty`/`saving`/`saved`/`error`/`conflict`, buffer preservado si hay falla. Al activar una página se refresca si no hay ediciones; con buffer pendiente se conserva y se ofrece revisar cambios. Un conflicto al guardar muestra revisiones y acciones de recargar o copiar texto. No mantener polling agresivo para simular colaboración.

Se intercepta cierre con cambios pendientes y se permite guardar, descartar explícitamente o cancelar. El contenido no soportado se muestra como limitación de solo lectura con acción para recuperar JSON. Controles por teclado y foco visible; no recursos remotos de fuentes o iconos.

### 8. Backup y restauración inicial

Usar una API de backup consistente de SQLite; nunca copiar únicamente el archivo principal abierto. El bootstrap incluye base y manifiesto con versión/hash porque todavía no hay adjuntos. Validar integridad y esquema en el destino antes de publicar el backup como completo. Al añadir adjuntos, el formato de backup se extiende con originales y su manifiesto.

Una migración crea primero este backup. La restauración lee y valida en staging, luego publica en un destino nuevo o vacío reservado por la operación; no altera el workspace activo. Un error limpia únicamente temporales creados por esa operación. En esta fase no hay comando para reemplazar un destino ocupado.

## Risks / Trade-offs

- [Esquema semántico pequeño exige otro adaptador después] → Mantener IDs y versión; ensayar round-trip antes de Tiptap.
- [WAL permite lectura concurrente pero hay un solo escritor] → Transacciones cortas, timeout y revisión; prueba real con dos conexiones.
- [No hay daemon para refrescar clientes automáticamente] → Refresh en activación y conflicto en guardado; reevaluar solo al introducir workers sostenidos.
- [Una UI mínima se confunde con el producto completo] → Estado y funciones disponibles explícitos; mantener la hoja de ruta.
- [Payload JSON dificulta índices sobre estados] → Validación estricta y consultas acotadas; normalizar columnas cuando una fase lo justifique.
- [Backups ocupan espacio] → Informar ruta/tamaño; ninguna limpieza automática de originales en el bootstrap.

## Migration Plan

1. Crear schema v1 solo para un workspace nuevo; fixtures de versión previa prueban las rutas de migración.
2. En existente, comprobar versión sin escritura. Si es futura, rechazar sin cambiar archivos.
3. Crear backup consistente, ejecutar migración transaccional y confirmar versión al final.
4. Ante fallo, rollback; el backup permite restauración a una ruta nueva. No prometer downgrade de datos nuevos.
5. Validar integridad después de migrar/restaurar y registrar versiones y comandos de desarrollo reales.

## Validation Plan

| Capability | Evidencia necesaria antes de cerrar el cambio |
| --- | --- |
| `local-workspace` | SQLite real en directorio temporal, reapertura, offline, migración exitosa/fallida, versión futura sin mutación, backup con conexión abierta y restauración rechazada/válida. |
| `knowledge-entities` | CRUD, tipos y estados inválidos, IDs de bloques, límites, formato desconocido intacto, relaciones y archivo sin cascada, paginación estable. |
| `core-operations` | Paridad de DTOs/reglas, atomicidad ante falla, dos escritores, revisión vieja/omitida, errores, stdout JSON, exit codes y registros de mutaciones. |
| `desktop-shell` | Recorrido en Tauri: crear/guardar/reabrir/archivar/restaurar, cierre pendiente, fallo de guardado, modificación CLI concurrente, formato desconocido, foco y uso offline. |

Los comandos de Rust, frontend y Tauri se fijarán al crear el scaffold. No registrar builds, tests de UI o rendimiento como pasados antes de ejecutarlos. La validación OpenSpec es exclusivamente documental.

## Implementation Notes

La fase 0 adoptó estas decisiones; los comandos/versiones reales están en `docs/development.md` y la trazabilidad en `docs/verification-phase-0.md`. Los contratos JSON/TypeScript se generan desde Rust y el build detecta diferencias. Solicitudes/entrada CLI: máximo 2 MiB, independiente del documento de 1 MiB; títulos: máximo 4.096 bytes.

La apertura prevalida sobre una copia privada de SQLite/WAL para no modificar SHM de un workspace futuro; esta copia de inspección no se usa como backup. Se validan ID, zona IANA, versión y columnas necesarias, permitiendo extensiones adicionales. Backups usan la API SQLite y un manifiesto SHA-256; se publican por rename desde staging a destino reservado. La fixture v0 soportada es sintética, sin historial, no una versión previamente distribuida.

El recorrido nativo usa un feature de pruebas desactivado en producción para accionar el DOM del WKWebView y operaciones/CLI reales. No añade transporte remoto ni acceso general a filesystem en el frontend. Las sesiones de prueba se ejecutan con red denegada por sandbox de macOS.
