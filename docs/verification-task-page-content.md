# Revisión acotada del contenido de Task

Fecha: 24/10/2026. Cambio `add-task-document-editing`.

## Referencia y diagnóstico

Se buscó y abrió con Playwright la Task de Notion indicada por el usuario en la
sesión ya autenticada. Se observaron título, Status/Project/Content/Due, relaciones,
comentarios y comienzo de página/plantillas. El cuerpo estaba vacío. No se editó
esa página ni se eligieron sus plantillas. Las fuentes/capturas quedan privadas.

El núcleo rechazaba documentos fuera de Note y la UI sólo inicializaba su editor
para note. La lectura de la Task local encontró una coincidencia inequívoca y
sin documento. Tras compilar el núcleo, otra lectura conservó ID y revisión 2,
document=null y document_editable pasó de false a true. No hubo relleno de cuerpo.

## Código, almacenamiento y compilación

- Backup consistente del workspace anterior al cambio, fuera de Git.
- Revisión de CRUD/CAS: Task permite documento opcional; la primera inserción
  usa upsert sobre su mismo ID y validación/referencias/observación/auditoría/FTS
  permanecen en la transacción compartida. Un cuerpo desconocido sigue bloqueado.
- Búsqueda, exportación y backup ya recorren entidades/documentos de forma
  genérica. SQLite 20, documentos v1/v2/v3, catálogo y forma de contratos intactos;
  no se agregó migración, tipo de entidad, operación o dependencia.
- `cargo build --locked --offline -p noteka-cli -p noteka-desktop`: aprobado.
- `npm run build`: aprobado, 0 errores y 0 advertencias Svelte; el aviso previo
  de Vite por bundle mayor a 500 kB permanece.

## Inspección del borrador sobre copia

UI compilada → Chromium → puente CLI → copia restaurada del backup. El puente
admite workspace.open y lecturas get/list/query/read; rechaza mutaciones de
entidades. Todos los borradores de demostración se descartaron sin guardar.

- La Task original abre un textbox **Contenido de la tarea**, sigue Guardado y
  muestra entrada visible y tres estructuras. Status/Project/Due aparecen arriba;
  Inbox, prioridad normal y planificación vacía siguen en View details.
- Enter desde el título enfoca el cuerpo sin marcar cambios ni intentar escritura.
  La primera observación inmediata ocurrió antes del requestAnimationFrame de
  foco de Tiptap; al esperar el foco real se confirmó activo/Guardado. El primer
  registro se conserva como límite del controlador, sin atribuirle un fallo al producto.
- Escritura directa, undo/redo y alternar el lateral conservaron el texto y el
  mismo nodo de editor. Abrir Project con cambios mostró Guardar/Descartar/Cancelar;
  Cancelar conservó el cuerpo y el estado pendiente.
- Lista de pasos preparó dos checkboxes, uno marcado, cinco IDs distintos y sin
  volver a ofrecer estructuras que reemplazaran el contenido. El estado de Task
  permaneció todo: los pasos son bloques del documento, no entidades hijas.
- Notas de tarea preparó Contexto/Notas. Notes nueva conservó su textbox anterior,
  icono de 62 px y cero propiedades/estructuras específicas de Task.
- Ambos temas revisados. A 420 px, documento/viewport 420/420 y cuerpo 380/380;
  título, propiedades y contenido siguen accesibles con scroll vertical normal.
- Se detectó en el NodeView actual que los ítems usan data-checked sin data-type:
  los estilos previos no limitaban la anchura de sus inputs. Se añadieron reglas
  exclusivas del cuerpo Task para alinearlos junto al texto, sin cambiar Notes.
  Medición final: input de 15 px en x=561, label de 18 px y contenido en x=589;
  se retiró la anchura de 728 px que centraba el checkbox.
- Cero errores/advertencias en la consola de Noteka y cero solicitudes externas
  observadas en la carga inicial. Cero intentos de mutación en el puente.

Un ref de resultado Notion quedó obsoleto durante su actualización asíncrona;
se corrigió usando el resultado de título exacto observado. El error del
controlador y los registros previos se conservaron.

## Límites

No se añadieron ni ejecutaron suites. No se ejercitó create/update de una Task
con cuerpo, búsqueda posterior, exportación o restauración de ese cuerpo nuevo:
sus rutas fueron revisadas por código y compiladas. La inspección del borrador no
acredita persistencia ni IPC nativos. No se capturó la pantalla nativa del usuario
ni se cerró su app manualmente; su modo dev puede recargar/recompilar cambios.
No se modificaron entidades personales ni Notion. Subtareas, comentarios, Content,
recurrencia y seguimiento de tiempo conservan su alcance pendiente.

## Cierre

Tres requisitos añadidos contrastados y sincronizados: desktop-shell 58→59,
block-editor 3→4 y knowledge-entities 14→15. Los 75 requisitos previos se
conservaron, igual que las otras 14 specs y los ocho artefactos de Meeting Notes;
sólo el Purpose del editor se amplió para incluir tareas. Contratos/adaptadores
y prefijo CSS conservados. Archivado 4/4 como
`2026-10-24-add-task-document-editing`; validación estricta final 18/18 aprobada.
Meeting Notes continúa 0/19.
