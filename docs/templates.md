# Plantillas de páginas de registro

Implementado y verificado en el segundo tramo de 3c: [evidencia](verification-phase-3c-templates.md), [ADR 0008](decisions/0008-template-instances-and-view-references.md). No son plantillas de esquemas ni de tareas/proyectos built-in.

## Datos y operaciones

Definición v1: `schema_version`, `title`, `document` semántico v1/v2/v3 y `values` por ID de propiedad. El nombre de plantilla es independiente del título inicial. Defaults sólo editables: texto/número/decimal/moneda/checkbox/fechas/select/multiselect/URL/email/files/relation. Dinero/decimal son strings; no title/timestamps/cálculos, ni siquiera null. Files/relations referencian los originales existentes.

Todas las operaciones reciben JSON por stdin o `--input`, con `--json --workspace RUTA`:

```sh
printf '%s\n' '{"collection_id":"ID","expected_collection_revision":1,"name":"Página","definition":{"schema_version":1,"title":"Registro","document":{"schema_version":2,"blocks":[]},"values":{}}}' | target/debug/noteka --json --workspace RUTA template save
printf '%s\n' '{"collection_id":"ID","limit":50}' | target/debug/noteka --json --workspace RUTA template list
printf '%s\n' '{"id":"ID_PLANTILLA"}' | target/debug/noteka --json --workspace RUTA template get
printf '%s\n' '{"id":"ID_PLANTILLA","expected_template_revision":1,"expected_collection_revision":1,"values":{}}' | target/debug/noteka --json --workspace RUTA template instantiate
```

Reemplazá los placeholders por IDs/revisiones confirmados; `collection get` ofrece la revisión del esquema. Actualizar `template save` exige además `id` y `expected_revision`, y no permite cambiar colección. Crear no los admite. Listado por ID, limit 1–200 (default 50), cursor devuelto por el núcleo. Guardar plantilla/instanciar no cambia revisión del esquema.

Instanciar toma title inicial salvo override explícito no vacío; title null es inválido. Omitir una propiedad conserva su default; null lo limpia. Se validan **todos** los defaults antes de fusionar, incluso los reemplazados. Revisión antigua de plantilla/esquema: revision_conflict (exit 4); validación: exit 2; formato de plantilla desconocido: unsupported_template_format (exit 6), con JSON recuperable por get/list. No se reemplaza una definición futura ni un valor futuro por uno reducido. Original ausente/inseguro no se convierte en copia vacía.

Cada instancia obtiene nueva entidad note, bloques nuevos y `inbox:false`. Adjuntos, relaciones de propiedades y vistas siguen apuntando a IDs originales; editar una instancia no cambia otras/plantilla. Sin placeholders dinámicos ni fechas relativas. Creación, documento, valores, historial e índice son atómicos.

Ejemplo de defaults: `values: {"ID_MONEDA":"145.69", "ID_TEXTO":"Preparar"}`. Override `values: {"ID_TEXTO":null}` conserva dinero y limpia texto; `{"ID_MONEDA":"48.57"}` cambia sólo el importe. Sustituí IDs por propiedades editables de esa colección. No admiten floats para dinero ni propiedades title/calculadas. Un default inválido no se oculta por un override válido.

## Escritorio

Collections → elegí colección → **Nueva plantilla**. Nombre del buffer identifica la plantilla; el JSON de definición contiene el título de futuras páginas, documento y defaults. **Editar plantilla**, **Instanciar** y **Recuperar plantilla** están junto a cada definición. **Más plantillas** pagina de a 50. Consultá el desplegable de esquema para los IDs; no se infiere un schema desde el JSON ni desde una nota ejemplo.

Instanciar usa un JSON de patch (`{}` conserva todos los defaults). El título del formulario es un override opcional; vacío **omite** title, no envía un título vacío. La revisión de plantilla/esquema visible se captura al comenzar; Actualizar colecciones no la rebasa. Ante conflicto/fallo se conserva el formulario y **Recuperar buffer JSON** incluye texto, ID y revisiones. Para reintentar con revisiones nuevas, recuperá primero el buffer y descartá/reabrí explícitamente; no hay merge automático.

Guardar, descartar o cancelar protege navegación, cambio de formulario/workspace y cierre. Un guardado confirmado limpia el buffer **antes** del refresco: si falla la lectura posterior, el mensaje distingue el commit y no ofrece repetir una creación ya confirmada. Una definición futura permanece recuperable; controles de editar/instanciar quedan deshabilitados cuando su estructura no es reconocida. La bandera `editable` es estructural, no garantiza que todos los defaults/destinos sigan vigentes: Rust los revalida al escribir.

Formulario inicial técnico de JSON/IDs; no plantillas built-in, clonación de schema, botones de automatización ni fechas relativas. Su refinamiento de UX queda en la cola de revisión, no como función implícita.

## Recuperación

SQLite v7 agrega sólo templates; migración conserva documentos v1/v2 y JSON de vistas/cálculos/valores, crea backup previo y revierte por completo ante fallo. Definiciones futuras conservan JSON original en backup; no se instancian ni actualizan automáticamente. Restore siempre crea otro workspace y conserva originales. Backup manifiesto v2 no cambió. No hay downgrade destructivo: para usar un cliente v6 restaurá el backup previo en otra ruta y conservá el workspace v7 con sus escrituras nuevas.
