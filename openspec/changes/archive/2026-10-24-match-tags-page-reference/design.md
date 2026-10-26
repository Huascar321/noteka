# Design

## Context

HomeTagSection y HomeTagRow ya usan proyecciones Rust con última nota,
clasificación, favoritos y actividad acotada. NotesPage incorpora su variante
compacta. WorkspacePane todavía dirige Tags lateral al listado genérico. Ver
proposal.md para motivación y el delta desktop-shell para comportamiento.

## Goals / Non-Goals

**Goals:** reutilizar lectores, resumen e identidades; presentar la página de la
referencia sin duplicar reglas de filtrado o alterar el dato canónico.

**Non-Goals:** cambiar el detalle individual, actividad/fórmulas de Notion,
sincronización remota, migraciones, dependencias, modelos o suites nuevas.

## Decisions

1. TagsPage mantiene un HomeReader `tags_recent`, diez filas y acumulación,
   ReadQueue/dispose/invalidación compartidos. Conecta openHome/createHome y
   navigate existentes; atajos activos con las guardias de las otras páginas.
2. HomeTagSection recibe `standalone`, separado de `pageMode` usado en Notes.
   La variante lateral añade encabezado, Nav, New Tag y Table; Home conserva
   cuatro vistas y Types inicial. Las listas laterales ocultan el contador de
   notas como la referencia. La última nota sigue siendo un enlace original.
3. Table es presentación local de `tags_alphabetical`, con columnas Tag,
   Last Note y Type. El modo visual se guarda en el componente y no se resetea
   al actualizar/continuar; seleccionar otra pestaña lo desactiva. La alternativa
   de añadir un HomeView público idéntico duplicaría semántica sin necesidad.
   La tabla reutiliza HomeItem/TagSummary y AppIcon con originales locales.
4. Estilos acotados: composición compartida de páginas, icono propio azul,
   tabla con overflow interno y filas listas reactivas a ancho estrecho. La
   paginación usa las acciones textuales de ancho completo ya implementadas.

## Risks / Trade-offs

- Actividad importada incompleta → Recent conserva su alcance documentado;
  no se igualan fechas ni relaciones a una captura remota.
- A-Z y Table comparten consulta → estado explícito de presentación diferencia
  pestañas y persiste durante actualización/paginación.
- Regresión de Home/Notes → valores por defecto y variante standalone separada;
  compilación y revisión manual acotada, sin atribuir aceptación nativa nueva.

## Migration Plan

No hay formato persistente nuevo. Snapshot privado de archivos antes de editar;
reversión limitada al cambio preservando el checkout previo. Inspección manual
con UI compilada, puente sólo lectura y copia restaurada del workspace. Sin
escrituras de entidades/documentos, publicación ni cambios de credenciales.
