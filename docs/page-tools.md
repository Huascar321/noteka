# Herramientas por grupo en los detalles

El inventario encontró una sola implementación de «Más opciones», compartida en
WorkspacePane. Las menciones en documentación describían ese mismo control;
no hay otro menú con ese texto en la interfaz. Se aplica también a originales
abiertos desde Home, Search, Archive, contexto o registros de colecciones, según
su tipo de entidad.

| Contexto | Grupos aplicables | Condiciones conservadas |
| --- | --- | --- |
| Note, Goal, Milestone | Exportar, Recursos externos, Adjuntos | Exportar requiere documento guardado sin cambios; recursos requieren identidad local; adjuntos conserva preparación/lector explícitos |
| Task | Exportar, Recursos externos | Exportar queda deshabilitado hasta tener contenido guardado y sin cambios; no recibe herramientas de adjuntos de Notes |
| Project, Tag, Person | Recursos externos | Requiere entidad guardada; exportación sólo se ofrece si existe un documento almacenado |
| Documento de formato desconocido | Según su tipo | JSON conserva exportación; Markdown sigue deshabilitado si el formato no es editable |
| Borrador nuevo | Según su tipo, con disponibilidad explícita | Exportar/recursos esperan guardado; Adjuntos sólo existe para tipo note |

## Interacción

Desde el 27/10/2026 todos los detalles y borradores usan el botón `…` en la
barra superior, junto al estado de guardado. Las herramientas aplicables y
Archive/Restore aparecen en el menú; no se repiten al pie. Lock page se limita
a Notes normales. Ver [editor contextual](note-editor.md) y
[verificación](verification-page-actions-menu.md).

Seleccionar una herramienta abre su panel flotante; abrirlo no exporta, vincula,
inicia Picker, encola extracción ni ejecuta lectores. Las opciones no disponibles
indican su razón. Flechas, Home y End recorren el menú; Enter/Espacio activan y
Escape devuelve foco al botón. Los paneles ocultos no entran en el recorrido de
foco y los paneles y el menú se ajustan al viewport.

Los formularios y lectores permanecen montados al cerrar el grupo, conservando
campos, resultados y operaciones iniciadas. Cerrar no cancela un trabajo de
extracción; las acciones explícitas y sus avisos siguen dentro del panel.
Cambiar de entidad reinicia la selección del grupo y conserva las guardias
habituales de los borradores de página.

## Acciones preservadas

- **Exportar:** destino nuevo, JSON + originales, Markdown y mensajes de resultado.
  Los botones conservan la exigencia de ruta; Markdown además requiere formato editable.
- **Recursos externos:** vincular desde Google Drive, abrir/previsualizar cuando
  corresponda, cerrar preview y desvincular. Las escrituras siguen bloqueadas
  con cambios pendientes/carga y mantienen revisión esperada. Abrir un recurso
  sigue siendo una acción separada.
- **Adjuntos:** preparación de originales y lector de generaciones, con todos
  sus pasos, selección de extractor, paginación, errores y límites anteriores.
  Notes inserta enlaces desde la selección y archivos mediante `/`; los otros
  contextos conservan Formato y bloques.

Ver [decisión](decisions/0049-page-tool-groups-and-project-detail.md) y
[revisión](verification-project-detail-tools.md).
