# Design

## Context

Ver proposal.md y la inspección Playwright privada del 24/10. La Task de referencia en Notion está vacía. La Task local coincide por título inequívoco y no tiene documento. El rechazo actual está en entidades Rust y la inicialización frontend se limita a note. SQLite documents ya referencia cualquier entities.id y no tiene un CHECK que la limite a note.

## Goals / Non-Goals

**Goals:** añadir contenido a la Task original con el editor/CRUD existentes y acercar su composición a la referencia.

**Non-Goals:** convertir Tasks a Notes, rellenar cuerpos en lote, importar contenido inexistente, editar Notion, guardar datos personales de demostración, nuevas operaciones, esquemas SQL/documentales, comentarios/subtareas/recurrencias/time tracking o aplicar Meeting Notes. No suites nuevas/ejecutadas.

## Decisions

1. Documento opcional para task en la tabla documents existente; note conserva su documento por defecto. Task sin cuerpo retorna document=null y document_editable=true. Otros tipos siguen rechazando documento. En update se usa upsert de la misma fila con revisión/auditoría/validación dentro de la transacción existente.
2. Cuerpo desconocido presente sigue document_editable=false y no se reemplaza. La ausencia de input.document no borra nada. Los límites de 1 MiB/10000 bloques/profundidad 32 y versiones 1/2/3 son los mismos. Búsqueda, referencias, exportación y backup ya son genéricos por entidad/documento.
3. El frontend monta un párrafo vacío efímero con ID propio para una Task sin documento, pero no añade draft.document hasta una edición real. Modificar sólo metadatos omite document y no crea la fila. El cuerpo completo mantiene sus IDs; Task no usa el reconocimiento de prefijos importados de Notes.
4. BlockEditor recibe nombre/placeholder y estructuras opcionales, sin cambiar sus defaults para Notes. Enter desde el título Task enfoca el cuerpo. Página vacía sólo enfoca; Lista de pasos crea checklist y Notas de tarea crea encabezados/párrafos, sólo sobre el párrafo vacío único, sin reemplazar encabezados/listas ni otras estructuras existentes. Son estructuras de cuerpo, no plantillas de colección ni subtareas independientes. Guardar y ⌘/Ctrl S siguen explícitos.
5. Composición exclusiva Task: icono predeterminado discreto; título y propiedades principales compactas; Project usa entidades/relaciones reales. Extras siguen en View details y edición de relaciones reutiliza su control actual. No se dibujan acciones de Notion sin soporte local.
6. Validación acotada: compilación Rust/contratos/UI, revisión de las rutas compartidas y previsualización manual en copia restaurada sin guardar. No suites ni claims de persistencia/IPC nativos ejecutados en esta entrega. Fuentes, backup, capturas y logs fuera de Git.

## Risks / Trade-offs

- Cliente anterior interpreta Tasks sin cuerpo como no editables → la forma JSON no cambia y los cuerpos nuevos siguen legibles por entity.get/export/backup; binarios anteriores no pueden editarlos. Actualizaciones que omiten documento lo conservan, y un binario anterior que intenta reescribirlo lo rechaza.
- Primer cuerpo ausente necesita inserción → upsert con la misma entidad, observado por MutationContext y protegido por CAS/transaction; sin rellenado de Tasks existentes ni migración SQL.
- HMR puede actualizar presentación mientras compila el host → implementar primero el núcleo y compilarlo antes de habilitar el editor frontend; no se cierra manualmente la app del usuario ni se usa su pantalla.
- Copiar plantillas de Notion simularía módulos ausentes → estructuras locales identificadas por lo que crean; sin comentarios/subtareas/recurrencias/time tracking ficticios.
- Persistencia no ejercitada en esta revisión sin suites → registrar el alcance de compilación/revisión y no atribuir a Chromium prueba del guardado real.

## Migration Plan

Backup consistente antes del cambio; SQLite sigue en 20 y documentos 1/2/3. No se transforma ninguna Task al actualizar ni al leer. El primer cuerpo se publica en la escritura explícita normal. Para recuperar, el backup/restauración y exportación JSON actuales conservan cuerpo/adjuntos. Revertir código conserva las filas y bloquea la edición del cuerpo Task en binarios antiguos; no borrar documentos para degradar. ADR0048 registra esta ampliación de propietario documental.

## Sources

- Task de Notion abierta por Playwright en la sesión existente, sólo lectura.
- https://tiptap.dev/docs/editor/api/commands/content/set-content
- https://tiptap.dev/docs/editor/api/editor
