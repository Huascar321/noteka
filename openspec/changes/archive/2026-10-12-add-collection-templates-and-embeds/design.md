# Design

## Context

Ver la motivación en `proposal.md`. Estado **previo** inspeccionado al 12/10/2026: SQLite v6, documentos v1/v2 y cinco layouts implementados, sin plantillas ni nodo de vista incrustada. Las decisiones siguientes se implementaron y verificaron en este cambio: [evidencia](../../../../docs/verification-phase-3c-templates.md). Las fronteras enumeradas describen el punto de partida, no código aún pendiente.

Fronteras reutilizables comprobadas:

- `database.rs::record_create` usa `entities.rs::entity_create_confirmed`: una transacción confirma entidad note, documento, pertenencia, valores, historial e índice. La entidad creada queda fuera de Inbox.
- `database.rs::validate_value/write_values` concentra tipos, moneda exacta, pertenencias de relaciones y verificación de archivos. Title/timestamps/formula/rollup no son valores editables.
- `documents.rs::validate` valida estructura y `validate_references` comprueba originales dentro de la transacción de escritura. `document_editable` verifica estructura sin consultar destinos; formatos desconocidos conservan JSON recuperable.
- `database.query` entrega `DatabasePage` con propiedades/config/layout/grupos/cobertura del mismo snapshot. `DatabasePanel.svelte` mezcla hoy esta presentación con formularios y controles de consulta.
- `BlockEditor.svelte` ya usa nodos atómicos para adjuntos y UniqueID; `editor-adapter.ts::fromEditor` produce v2. `App.svelte::guarded/resolveGuard`, `setPending` y el callback de apertura de colecciones protegen buffers/cierre.
- `workspace.rs` usa backup SQLite consistente, todos los originales administrados, migración transaccional y preflight que no toca un workspace futuro. Manifiesto de backup v2; exportación de documento independiente v1.
- `search.rs::reindex` indexa título y contenido inline de bloques reconocidos, no los resultados de vistas ni propiedades dinámicas.

## Goals / Non-Goals

**Goals:**
- Reutilizar validadores y una sola frontera de commit para crear instancias, sin un segundo motor de registros.
- Separar referencia persistente, buffer del editor y resultado de consulta temporal; preservar los tres ante sus respectivos errores.
- Reutilizar el renderer de los cinco layouts sin habilitar escritores dentro del editor y acotar consultas/ciclo de vida.
- Mantener lectura/exportación de originales desconocidos y recuperación de migraciones, sin reescritura masiva.

**Non-Goals:**
- No ampliar tipos de entidad, payloads built-in, lenguaje de fórmulas o reglas de My Day; consultar exclusiones de `proposal.md`.
- No snapshots persistidos de una vista incrustada, inclusión transitiva de destinos en exportaciones individuales, expansión recursiva de páginas ni edición inline de valores.
- No actualización en vivo/polling de todos los bloques ni coherencia de snapshot entre llamadas paginadas separadas.

## Decisions

### 1. Plantilla independiente de una entidad y de su esquema

Añadir `templates(id, collection_id, name, revision, definition)` con FK a `collections` e índice de listado `(collection_id,id)`. `definition` es JSON original versionado: `{schema_version: 1, title, document, values}`. El documento admite las versiones conocidas; `values` usa IDs de propiedades y defaults estáticos. El nombre identifica la plantilla; `title` es el título inicial de cada instancia, no una propiedad title duplicada. No guardar filas de ejemplo, resultados calculados ni payloads de otras entidades.

Lectura devuelve metadatos, `definition` como JSON crudo y bandera estructural `editable`. La deserialización estricta de la definición sólo se usa para validar/escribir/instanciar. Una versión/campo desconocido no rompe el listado ni autoriza reemplazarlo por un DTO reducido; actualización/instanciación se rechazan con error recuperable. JSON sintácticamente corrupto se informa como error de almacenamiento y no se repara automáticamente. Los backups conservan la cadena almacenada íntegra.

Alternativas: usar notas ocultas como plantillas introduciría Inbox/archivo/búsqueda y relaciones ajenas a su propósito; guardar sólo el ID de una página haría que sus futuras ediciones cambien defaults sin una revisión propia; clonar colección/esquema excede este cambio. La definición explícita ofrece concurrencia independiente y no duplica identidad de registros.

### 2. Operaciones compartidas y revisiones explícitas

Añadir al catálogo Rust/DTOs, dispatcher y CLI el grupo `template`:

| Operación propuesta | Entrada principal | Salida |
| --- | --- | --- |
| `template.save` | collection_id, name, definition estricta, expected_collection_revision; id/expected_revision sólo para actualizar | Template |
| `template.get` | id | Template con definición cruda |
| `template.list` | collection_id, limit/cursor | Page<Template> |
| `template.instantiate` | id, expected_template_revision, expected_collection_revision, title opcional y values como patch | RecordData |

Crear requiere id/revisión de plantilla ausentes; actualizar requiere ambos, misma colección y revisión positiva vigente. Actualizar incrementa sólo la revisión de plantilla; no cambia la revisión de colección porque no cambia su esquema. No mover una plantilla entre colecciones ni añadir borrado físico en este tramo. Nombre reutiliza el límite de 256 bytes; título el de 4.096 bytes; documento 1 MiB/10.000 bloques/profundidad JSON 32; defaults/patch hasta 128 propiedades. Listado por ID estable, 50 por defecto y límites existentes 1–200. La envoltura de operación conserva su límite de 2 MiB y `api_version: 1`, con adiciones al catálogo.

La revisión de colección se captura al iniciar un buffer de escritura, no se reemplaza silenciosamente al refrescar. Guardar/instanciar valida contra el esquema vigente dentro del lock. No exigir que la colección conserve para siempre la revisión con que se creó la plantilla: un cliente que leyó el nuevo esquema puede usarla tras revalidar todos los defaults. La revisión de plantilla usada por el cliente sí debe seguir vigente.

Defaults y overrides se validan **por separado y completos**, incluso un default que el override reemplazará o limpiará. Omitir una clave conserva el default; null limpia el valor y nunca permite escribir propiedades calculadas/proyectadas. Omitir title usa el inicial; un title vacío/null no es una instrucción de limpieza válida. No hay overrides del documento ni fechas relativas: el contenido se toma de la revisión comprobada de plantilla. Relations/Files conservan los IDs explícitos y las reglas actuales para destinos archivados; no se crean relaciones built-in implícitas.

Alternativa: añadir template_id a `record.create` mezcla dos modos de validación y concurrencia en un contrato ya usado. La operación separada explicita las dos revisiones sin cambiar creación de registros sin plantilla. No validar sólo la fusión: eso ocultaría defaults corruptos bajo un override válido.

### 3. Una transacción de instanciación, no creación seguida de attach

Extraer el cuerpo de creación a un helper interno que recibe conexión/transacción, raíz de originales, CreateEntity, origen y request_id. `entity_create_confirmed` mantiene su API y abre/commitea su transacción; `record.create` mantiene su semántica. `template.instantiate` abre una transacción Immediate, lee plantilla y colección, compara revisiones, valida definición/defaults/patch y construye una note con `inbox:false`. Recorre todos los bloques anidados asignando UUIDs nuevos; conserva contenido, atributos y referencias, nunca cambia la plantilla.

La misma transacción llama al helper y a `database::attach`, registra la creación en el historial con origen/request_id e indexa la nueva entidad antes del commit. El historial existente identifica la creación de entidad; no se inserta una plantilla como si fuera una entidad en la tabla `changes`. Cualquier fallo, incluido historial/índice, revierte todo. Extraer un validador no mutante de mapas de valores para usarlo en template.save/instantiate y write_values, sin simular un registro para validar defaults.

Alternativas descartadas: invocar `record.create` después de leer la plantilla fuera del lock permite una carrera; llamar un método que abre otra transacción obliga a anidarla; crear primero y aplicar defaults después deja estados parciales. El helper compartido evita esos tres casos y conserva las regresiones existentes de entity/record.

### 4. Referencia semántica mínima en documento v3

Añadir `BlockKind::CollectionView`, admitido sólo en v3, como hoja atómica con `content: []`, sin hijos y `attrs: {collection_id, view_id}` (UUIDs locales). No admite filtros/layout/filas, atributos adicionales ni URLs. Los demás bloques v3 conservan las reglas v2 y restricciones de anidación de listas.

La validación estructural es independiente del destino. En escrituras, `documents::validate_references` comprueba que colección/vista existen y la vista pertenece a esa colección mediante sus columnas SQLite, dentro de la misma transacción. No ejecuta la consulta ni interpreta el JSON de la vista para autorizar guardar texto: un destino futuro/no consultable debe dar error local de presentación, no volver desconocida toda la nota. Un destino inexistente sí impide confirmar una referencia inválida; se puede eliminar/corregir el bloque conservando el buffer.

Abrir documentos v1/v2 no los reescribe. El adaptador añade el mapeo explícito collection_view ↔ collectionView y selecciona v3 al introducir el nodo; conserva v3 en sesiones/documentos que ya lo usan aunque se quite la última referencia. V1 sigue actualizándose a v2 sólo con edición explícita. La recuperación pendiente incluye la referencia semántica, no el DOM de la NodeView. Documentos/bloques/campos futuros conservan el rechazo de edición y export JSON del formato original.

Alternativas: un iframe introduce red y autoridad externa; un bloque HTML/Tiptap persistido acopla datos al editor; usar v2 sin cambiar el workspace dejaría escritores antiguos interpretar parcialmente el nuevo formato. La combinación documento v3/workspace v7 declara la compatibilidad real.

### 5. Renderer compartido, lector incrustado con ciclo de vida propio

Extraer `CollectionPresentation.svelte` como presentación pura de DatabasePage: columnas de `page.properties`, filas/grupos por ID, errores de cálculo, cobertura y callbacks `onopen`/`onedit` opcional. DatabasePanel conserva consultas, formularios, revisión de buffers y paginación; sigue ofreciendo edición mediante el callback. No duplicar lógica de filtros/grupos en TypeScript.

Añadir `EmbeddedCollectionView.svelte`, que recibe workspace/collection_id/view_id y callback de apertura, y usa únicamente `database.query` con view_id, límite 50 y cursor. No hay overrides locales de config/layout: dos referencias a una vista comparten su configuración confirmada. Publicar atómicamente resultado/config/layout/grupos/cobertura de la misma respuesta, sin etiquetarlos con un bundle previo. Mostrar los IDs de destino; no depender de `collection.get` de todas las vistas para leer una referencia ya guardada. Un error limpia la página y ofrece reintentar, sin escribir el documento. Actualizar y volver al inicio reinician el cursor; página siguiente reemplaza, no acumula grupos de páginas distintas. Los cambios entre llamadas conservan los límites de la paginación actual: no se promete un snapshot global.

Carga inicial al entrar el bloque en viewport y botón accesible de cargar/actualizar; consultas incrustadas con concurrencia acotada a dos por editor, sin timers/polling. Cada lector usa generación y estado disposed para descartar respuestas tras desmontaje, cambio de destino o consulta nueva; no tiene acceso de escritura ni activa pending por leer. La actualización de resultados no dispara onchange del documento. Las tarjetas no renderizan documentos, adjuntos ni otras bases: sólo la proyección de columnas ya usada por las vistas.

La extensión Tiptap crea una NodeView atómica no editable y monta el lector mediante Svelte 5 mount/unmount. `update` conserva el montaje sólo cuando tipo/ID/referencia no cambian; cambios de referencia recrean el lector. `destroy` invalida consultas, desconecta observador y desmonta el componente. Interceptar eventos de controles interiores para que sean utilizables sin que ProseMirror los interprete como escritura; no interceptar indiscriminadamente selección/navegación/borrado de nodos. Las mutaciones del DOM de presentación no se importan al documento. Incorporar el tipo a UniqueID y conservar pegado seguro actual (texto plano, sin fabricar referencias desde HTML externo).

Alternativas: incrustar DatabasePanel entero expondría escritores/formularios y buffers anidados; copiar el renderer divergiría en errores/cobertura; consultar todos los bloques al montar una nota de 10.000 bloques sobrecargaría IPC. Un renderer puro más lector visible acotado separa responsabilidades sin una dependencia nueva.

### 6. Autoría y navegación usando las guardias actuales

DatabasePanel incorpora listado paginado y formularios JSON explícitos de crear/editar plantilla y de instanciación con título opcional/patch de valores. Los buffers capturan ID y ambas revisiones según la operación; refresh no los rebasa. Conflicto, validación o almacenamiento conservan formulario y recuperación JSON. Tras commit confirmado un fallo de refresco se identifica como fallo de lectura, no ofrece repetir creación como si no hubiera ocurrido. No presentar la plantilla como una nota del listado principal.

BlockEditor incorpora selector de colección/vista guardada y comando `/`/botón para insertar referencia, así como eliminación por selección de nodo y undo/redo. Seleccionar sin insertar no modifica la nota; confirmar inserción sólo cambia el buffer hasta Guardar. El selector usa collection.list paginado y collection.get para las opciones; errores de esquema/formato conservan el editor y no fuerzan una conversión. La comprobación definitiva sigue siendo Rust al guardar.

Propagar a BlockEditor el mismo callback de apertura de original usado por DatabasePanel, que llega a la guardia de App y vuelve a leer la entidad por ID al navegar. No saltarse la decisión guardar/descartar/cancelar usando sólo el Entity de la página consultada. La app mantiene pending por texto/referencia/plantilla sin guardar, no por un resultado de lectura; cierre nativo y recuperación usan las guardias existentes. El buffer de plantilla JSON no monta lectores incrustados, por lo que editar una definición no consulta sus destinos accidentalmente.

La ausencia de expansión de documentos, no un conjunto global de páginas visitadas, es la prevención de recursión: una colección puede incluir su contenedor y dos notas pueden apuntarse sin render recursivo. Navegar explícitamente al mismo ID sigue siendo una acción protegida.

### 7. Exportación, búsqueda y backup con autoridad explícita

Export JSON conserva la referencia exactamente y copia sólo los adjuntos propios del documento, como hoy. Tanto el manifiesto/resultado JSON como Markdown advierten que los registros, esquemas y vistas referenciados no se incluyen transitivamente; sólo un backup completo transporta esos destinos. Markdown agrega un marcador textual con ambos IDs y advertencia, sin consultar ni inventar filas. No hay exportación standalone de plantillas en este tramo: template.get ofrece recuperación de la definición.

El índice FTS sigue recorriendo sólo título/texto de bloques. El nuevo bloque atómico carece de texto fuente; no se indexan etiquetas de UI, IDs ni filas mostradas. Tampoco se indexan definiciones de plantillas como notas. Backup completo copia la nueva tabla con SQLite y todos los adjuntos administrados; conserva definiciones futuras sin deserializarlas para reescribirlas. Formatos de manifiesto existentes no cambian porque su estructura no cambia.

Alternativa: materializar filas en Markdown/FTS haría que un cambio externo parezca texto original de la nota y abriría ciclos de exportación. La advertencia y backup del conjunto separan intercambio limitado de recuperación fiel.

## Risks / Trade-offs

- [Refactor de creación altera atomicidad existente] → Mantener contratos y correr regresiones entity.create/record.create; inyectar fallos de valores, historial e índice durante instanciación y comprobar ausencia de cambios parciales.
- [Default deja de ser válido tras cambio de esquema/original] → Comparar revisiones de cliente y revalidar defaults completos en cada escritura/instanciación; no limpiar campos ni sustituir referencias automáticamente.
- [Una vista desconocida afecta la edición de la nota] → Separar estructura/referencias existentes de ejecución; error sólo en el lector, con recuperación del JSON de destino por mecanismos existentes.
- [NodeView pierde foco, duplica consultas o filtra resultados tardíos] → Ciclo de vida/generación explícitos, concurrencia acotada y pruebas nativas con controles por teclado, eliminación/undo y navegación entre notas/workspaces.
- [Lectura paginada no es snapshot entre llamadas] → Cobertura de página visible, actualizar reinicia recorrido y tests con dataset estable; no prometer actualización continua ni consistencia global nueva.
- [Export de nota confunde referencia con transporte del conjunto] → Advertencia también en export JSON y marcador Markdown; verificar restauración completa con plantillas/destinos/originales.
- [Formulario de plantilla inicialmente técnico] → JSON semántico documentado y recuperable; registrar refinamientos de UX como revisión no bloqueante, sin ampliar a creación de esquemas/automatizaciones.

## Migration Plan

1. Antes de modificar persistencia, implementar pruebas de fixtures v6 con documentos v1/v2, cálculos, vistas y originales, y verificar el backup previo. Registrar la decisión adoptada en un ADR al aplicar el cambio.
2. Añadir migración `007.sql`: tabla/índice de templates y actualización coordinada de `workspace_meta.schema_version`/`PRAGMA user_version` a 7. No actualizar JSON de documentos/vistas/cálculos/valores ni reconstruir FTS sólo por migrar.
3. Integrar en create/open/migrate y schema_check condicional para v7; conservar soporte de backups v0–v6 y manifiestos 1/2 según versión. Verificar integridad/FK y rollback transaccional con fallo inyectado desde v6.
4. Regresiones que simulan versiones antiguas eliminan la tabla nueva de sus fixtures, sin cambiar qué versión/formato prueban. Verificar backup v7 con definición futura cruda y restauración a workspace nuevo, y rechazo de workspace futuro sin tocar main/WAL.
5. Un cliente anterior soporta hasta v6 y debe rechazar v7. No se ofrece downgrade SQL que descarte templates o nodos v3. Recuperación: restaurar backup previo en **otro** directorio para usar el cliente anterior; conservar workspace v7 y sus originales para no perder escrituras posteriores.
6. Ejecutar core/CLI/adaptador/nativo con red denegada, contratos/frontend/fmt/Clippy/release y OpenSpec estricto; revisar los seis requisitos del delta y sus escenarios, incluyendo regresiones conservadas de documentos. Sólo entonces sincronizar specs y archivar; Home continúa pendiente.
