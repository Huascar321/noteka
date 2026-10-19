# Vistas locales incrustadas

Implementado y verificado en el segundo tramo de 3c: [evidencia](verification-phase-3c-templates.md), [ADR 0008](decisions/0008-template-instances-and-view-references.md). Home y funciones auxiliares siguen pendientes.

Documento v3 añade `collection_view`, hoja con ID estable, `content: []` y `attrs: {collection_id, view_id}`. Ambos destinos son IDs locales; no guarda config/layout/filas ni URLs. Es válido entre bloques enriquecidos y dentro de contenedores compatibles; los límites de documento/atributos desconocidos no cambian. V1/v2 se leen sin reescritura de migración; un nodo nuevo exige v3.

Al guardar se comprueba existencia de colección/vista y pertenencia; no se ejecuta el filtro para guardar texto. Una vista futura/no consultable conserva referencia y da error de lectura. Referencia inexistente/ajena rechaza la escritura conservando revisión/original. Un documento futuro no se edita ni convierte; export JSON mantiene todos sus campos.

## Autoridad y transporte

- Mostrar una vista no agrega texto ni filas al documento. La referencia es original; el resultado temporal viene de `database.query`.
- FTS indexa título/texto de la página fuente, no IDs/etiquetas de UI ni filas dinámicas. El texto de un registro se busca en ese registro y se abre por su origen, no como texto del contenedor. Templates no aparecen como notas.
- Export JSON conserva referencias y adjuntos propios; advierte que no transporta los esquemas/vistas/registros destino. Markdown incluye marcador con IDs y advertencia. No recorre referencias, incluso si la colección contiene su contenedor.
- Para transportar el conjunto usá backup completo del workspace: plantillas, documentos, destinos, ASTs, valores y todos los originales. No se cambian formatos de manifiesto.

## Autoría y lectura en escritorio

En una nota editable, **Base incrustada** (toolbar o menú `/`) abre un selector: elegí colección y vista guardada → **Insertar vista guardada**. **Más colecciones para incrustar** pagina las opciones de a 50. Elegir/cancelar sin insertar no escribe; la inserción queda pendiente hasta Guardar. La comprobación final de existencia/pertenencia vive en Rust. **Seleccionar vista incrustada** + **Quitar vista seleccionada** elimina sólo el bloque; Deshacer/Rehacer conserva ID y referencia. Las acciones estructurales explícitas separan eventos de historial con `closeHistory`, incluso si insertar/quitar ocurren en menos de 500 ms.

El lector carga al entrar en viewport o por botón explícito, sin timers ni polling. **Actualizar vista incrustada** vuelve al inicio con la configuración vigente; **Página siguiente incrustada** reemplaza la página; **Inicio de vista incrustada** reinicia el cursor. Después de fallo, **Cargar vista incrustada** reintenta. Controles son botones nativos con foco visible; actualizar no ensucia ni incrementa revisión de la nota. Abrir una fila usa la guardia guardar/descartar/cancelar y vuelve a leer el mismo ID; no navega desde un snapshot antiguo ni crea copias.

Los cinco layouts comparten `CollectionPresentation`, pero dentro del editor no hay acción Editar valores: abrí el original. Columnas, grupos, errores, layout y cobertura provienen de una sola respuesta `DatabasePage`. Límite 50; grupos/días cubren sólo esa página. Lecturas entre páginas no son un snapshot global frente a escrituras concurrentes. Dos referencias a la misma vista usan su config guardada, sin override local.

Máximo **dos consultas simultáneas por editor**. Una consulta IPC en curso no se puede abortar físicamente: desmontar invalida/descarta la respuesta y conserva su cupo hasta terminar; cancelar un trabajo todavía en cola evita ejecutarlo. Generación/disposed, observador y mount/unmount delimitan la vida del lector. El DOM temporal no se importa al documento. Un error limpia la proyección anterior sin borrar el bloque.

No contiene edición inline, expansión recursiva de documentos, previews remotos ni reglas de Home. Tarjetas sólo muestran propiedades; una colección puede contener el contenedor sin expandir sus documentos/adjuntos. No es actualización live ni exportación standalone de destinos.
