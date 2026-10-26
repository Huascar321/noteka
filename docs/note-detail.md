# Detalle de una Note

La Note abre su página original con icono, título editable, propiedades compactas,
relaciones y cuerpo. La composición toma como referencia el detalle de Notes
observado en Notion; los valores principales pertenecen al workspace local.

- **Tag:** abre los tags locales relacionados. La flecha de edición abre las
  relaciones existentes; guardar primero los cambios pendientes permite agregar
  o quitar vínculos con las operaciones compartidas.
- **URL:** dato del export, sólo lectura cuando está disponible. No representa
  una sincronización viva ni un nuevo campo editable.
- **Favorite y Type:** controles locales editables. Las nueve categorías
  existentes se conservan; seleccionar Meeting o Voice no inicia audio ni ASR.
- **Relations:** proyectos y personas reales, con acceso a sus páginas originales
  y al editor de relaciones.

View details conserva Inbox, procedencia, revisión y propiedades completas de
origen. Los valores importados pueden ser anteriores a la edición local; no
reemplazan los controles principales.

## Escribir contenido

Actualización 26/10/2026: el cuerpo usa el [editor contextual](note-editor.md):
barra sobre selección, slash filtrable junto al cursor y controles laterales por
bloque. Export, External resources y Attachments pasan a la barra superior;
sus paneles conservan campos, condiciones y operaciones explícitas.

Hacé clic en el cuerpo o presioná Enter desde el título. `/` abre los comandos de
bloques. Una Note vacía ofrece Página vacía, Checklist y Nota con secciones;
preparan sólo el borrador. Encabezados, listas, código u otras estructuras
existentes no se reemplazan con estas opciones.

El contenido se guarda explícitamente con Guardar o ⌘S/Ctrl+S. Navegar con cambios
pendientes conserva Guardar/Descartar/Cancelar. Código, adjuntos, tablas y bases
incrustadas siguen usando el editor compartido y sus límites anteriores.

## Contenido importado

El prefijo del importador se proyecta como propiedades. En páginas acreditadas
como Notion/Notes, el primer párrafo plano de propiedades repetidas también puede
ir a View details: exige marcador reconocido, tres coincidencias exactas, nombres
admitidos únicos y ausencia de contradicciones. El texto ambiguo permanece en el
cuerpo. Editar recompone todos los bloques originales del prefijo, con sus IDs.
No se reescriben documentos al abrir ni se corrige contenido de la fuente.

Goals y Milestones mantienen sus propios campos y controles. El detalle de Task,
la biblioteca Notes y Home conservan sus flujos. No se agregan Content,
comentarios, autoguardado ni funciones de Meeting Notes.

Los [grupos de herramientas](page-tools.md) usan iconos para Exportar, Recursos
externos y Adjuntos. Cada icono abre su panel; las operaciones siguen siendo explícitas.

Ver [revisión acotada](verification-note-detail.md).

Actualización 26/10/2026: el detalle comparte [Side/Center peek, Full page y New
tab](detail-peek.md), con una preferencia global local para nuevas aperturas.
El [indicador persistente](scroll-outline.md) representa título y H1/H2/H3 del
cuerpo y mantiene scroll independiente del origen.
