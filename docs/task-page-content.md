# Contenido y detalle de una Task

Actualizado el 28/10/2026. El detalle de Task conserva su identidad y relaciones,
y ofrece un cuerpo enriquecido propio bajo sus propiedades.

## Cómo editar

1. Hacé clic en el título para cambiarlo.
2. Para el cuerpo, hacé clic donde dice **Escribí acá**, o presioná **Enter**
   desde el título. Podés escribir directamente y usar `/` para agregar bloques.
3. En una página vacía también podés elegir **Página vacía**, **Lista de pasos**
   o **Notas de tarea**. Las dos últimas preparan una estructura en el borrador.
4. Confirmá con **Guardar** o **⌘/Ctrl+S**. Abrir, enfocar o elegir Página vacía
   no guarda contenido ni crea un cuerpo en una Task anterior por sí solos.

El cuerpo de Task comparte la interacción contextual de bloques de Notes: barra
de formato al seleccionar texto, menú `/` filtrable, controles laterales para
insertar, transformar, duplicar, mover y eliminar bloques, y clic debajo del
último bloque para continuar en un párrafo. Si el último párrafo está vacío,
se reutiliza. Los cambios permanecen en el borrador hasta guardarlos. Ver
[editor contextual](note-editor.md) y [verificación](verification-task-block-editor.md).

Status, Project y Due aparecen arriba. **View details** mantiene prioridad,
fecha planificada y metadatos; Inbox ya no es una marca manual. Due abre un
calendario local de vidrio (mes anterior/siguiente, Today y Clear) en lugar del
popup nativo; el campo es un botón con el material de campo compacto que muestra
la fecha corta o Empty, sin `input[type=date]` nativo, con cursor pointer y foco
de retorno al elegir con Enter o Escape. Project abre su original con las guardias de salida;
su control de edición y las acciones de persona/tag abren selectores locales por
nombre. En una tarea ya creada se puede modificar un vínculo sin guardar su
borrador; el vínculo se confirma por separado. Los vínculos existentes se pueden
abrir o desvincular sin eliminar las páginas relacionadas.

Una Lista de pasos contiene checkboxes del documento; no crea entidades Task
hijas ni cambia automáticamente el estado o progreso de la Task. Las estructuras
sólo se ofrecen sobre el párrafo vacío único: no reemplazan listas, encabezados,
imágenes ni otro contenido existente aunque sus textos estén vacíos.

## Persistencia y compatibilidad

- Notes conserva su comportamiento. Una Task puede tener `document` o `null`.
- Las Tasks anteriores no se rellenan, duplican ni convierten al abrirlas.
  Sólo una edición de cuerpo confirmada publica su documento.
- `document_editable=true` permite comenzar una Task sin cuerpo. Un documento
  almacenado con formato desconocido permanece no editable y recuperable como JSON.
- SQLite sigue en **20**; se reutiliza `documents.entity_id` y documentos **v1/v2/v3**.
  No se agrega migración, operación o tipo de entidad. CLI, MCP y escritorio comparten
  CRUD/CAS, validación, auditoría y búsqueda textual.
- Cuerpo, título y propiedades enviados se confirman en una transacción con la
  revisión esperada. Omitir `document` no borra un cuerpo previo ni reserializa
  el payload omitido. Límites y validación de referencias son los del editor existente.
- Exportación JSON/Markdown y backup usan las rutas genéricas existentes. JSON y
  originales conservan fidelidad; Markdown mantiene sus advertencias actuales.
- Un binario anterior puede leer/exportar el cuerpo y sus backups, pero rechaza
  editar un documento Task. No hay que borrar documentos para volver a una versión vieja.

## Referencia y alcance

Se abrió mediante Playwright la Task de Notion indicada por el usuario, sin
modificarla. Estaba vacía y ofrecía comenzar una página o elegir una plantilla.
Noteka toma esa composición e incorpora estructuras de contenido locales.

Content, subtareas/parent task, comentarios, recurrencias y registro de tiempo
requieren capacidades propias. Esta entrega no copia sus acciones como si ya
existieran. Tampoco recupera en lote cuerpos de otras tareas desde Notion ni
modifica el importador histórico.

El guardado sigue siendo explícito, con Guardar/Descartar/Cancelar y conservación
del buffer ante conflicto o fallo. Ver [ADR0048](decisions/0048-task-owned-documents.md)
y [revisión acotada](verification-task-page-content.md).
