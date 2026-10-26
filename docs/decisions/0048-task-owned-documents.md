# ADR 0048: documento opcional propio de Task

- Estado: adoptado en código; persistencia no ejercitada en esta revisión
- Fecha: 24/10/2026

## Contexto

El usuario pidió editar una Task como página de Notion. La referencia leída con
Playwright no tiene cuerpo aún y presenta inicio de página/plantillas. El núcleo
rechazaba documentos fuera de Note y la UI nunca montaba BlockEditor para Task.
La tabla documents ya admite cualquier entity_id existente.

## Decisión

Permitir un documento opcional en Task mediante las operaciones compartidas.
Notes mantiene su documento predeterminado; los otros tipos conservan el rechazo.
Una Task sin cuerpo devuelve document=null y document_editable=true. Si hay cuerpo,
la capacidad depende de la misma validación de formato existente; un desconocido
permanece bloqueado para reemplazo y recuperable por JSON.

El primer cuerpo se inserta mediante upsert sobre documents con el mismo ID de
Task. La observación de MutationContext, revisión esperada, validación de referencias,
auditoría y reindexación permanecen en la transacción existente. La ausencia de
input.document no escribe en esa tabla ni borra contenido. Una actualización sin
payload conserva su representación original, incluidos campos ajenos.

No se migra SQLite ni el formato JSON: 20 y v1/v2/v3 respectivamente. No hay
rellenado de Tasks existentes. El editor prepara un párrafo efímero con ID propio
sin añadir draft.document hasta una edición real. Abrir/enfocar no genera una
escritura ni marca cambios. Task no interpreta prefijos de propiedades de Notes.

BlockEditor conserva defaults de Notes y recibe nombre accesible, placeholder y
estructuras opcionales. Las estructuras sólo reemplazan el párrafo vacío único;
listas o encabezados vacíos siguen siendo contenido. Página vacía enfoca; Lista
de pasos prepara bloques checklist y Notas de tarea, encabezados/párrafos. No
crean subtareas, recurrencias ni estados de productividad.

El detalle Task organiza Status/Project/Due arriba y campos secundarios en View
details, con relaciones originales. El título predeterminado no requiere un
icono grande; los iconos personalizados siguen disponibles. Notes, Goals y otros
detalles conservan su composición.

## Alternativas consideradas

Crear una Note por Task agregaría otra identidad y fragmentaría el contenido.
Guardar texto en payload duplicaría el formato documental, validación y búsqueda.
Una migración que cree documentos vacíos cambiaría todos los registros sin una
edición del usuario. Se reutiliza la tabla documental con publicación explícita.

## Compatibilidad y recuperación

Un binario anterior lee entity.get/export/backup de estas filas, pero su mutador
rechaza un documento Task. La actualización de metadatos que omite documento no
lo elimina. Para recuperar o degradar se conserva el backup/JSON/originales; no se
convierte ni descarta contenido. Los clientes que desean editar el cuerpo deben
usar el núcleo actualizado. No cambia la forma de contratos ni el catálogo de operaciones.

## Evidencia y límites

Compilación CLI/host y frontend, lectura de la Task original sin cambio de revisión
y revisión de código de las rutas genéricas. Inspección del borrador sobre copia
con puente de sólo lectura. No suites, escritura de prueba ni aceptación de
persistencia/IPC nativos en esta entrega. El alcance se detalla en la
[revisión](../verification-task-page-content.md).
