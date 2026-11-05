# Página Notes

La entrada Notes del menú abre una página amplia con breadcrumb Home → Notes,
icono propio sobre título, Nav, accesos Inbox/Fav/All/Tags y el botón New. Las ocho
pestañas son Notes, Inbox, Fav., Clips, Voice, Journal, Meetings y All. Notes es
la vista inicial; sus reglas son las de la [biblioteca local](notes-reference.md).
All conserva todas las notas activas, incluidas páginas de otras bases importadas.

Las filas muestran arte local, relaciones navegables, tipo y días desde la
modificación acreditada. Las fechas provienen del snapshot disponible; esta página
no sincroniza Notion ni copia las fechas de las capturas de referencia.

## Tags integrado

Tags aparece debajo con Recent inicial y Fav./A-Z/Types. Presenta icono, última
nota navegable y clasificación disponibles. Sus reglas y resúmenes son los de
[Tags local](tags-reference.md); el encabezado y el botón de expansión abren
su listado general. El enlace rápido Tags desplaza y enfoca la sección integrada.
El contador adicional de notas se conserva en Home, fuera de esta variante.

## Lectura y captura

Cada sección carga diez filas y mantiene su propia vista, cursor, carga y error.
Load more acumula originales únicos y desaparece al terminar. Load more y
New page textuales responden en toda la fila. La acción de creación queda en el
botón New de la barra de vistas. La cobertura queda dentro de Opciones de vista.

Actualizar una sección sólo vuelve a leer esa sección. ⌘R en macOS, Ctrl+R en
Windows/Linux o F5 actualiza ambas conservando sus vistas y sin recargar el
WebView. No inicia consultas si hay diálogo/bloqueo, composición o repetición;
las páginas inactivas no ejecutan ese atajo. Al salir se descartan lecturas tardías.

New/New page preparan un borrador mediante las guardias existentes.
Crear desde Clips/Voice/Journal/Meetings preselecciona el tipo; Fav. preselecciona
su favorito e Inbox ya no es una marca manual.
No hay escritura hasta guardar. Voice clasifica notas; no graba ni transcribe.
Abrir una nota, relación o última nota conserva el editor y breadcrumb compartidos.
El [detalle de Note](note-detail.md) presenta propiedades locales compactas,
relaciones y cuerpo editable con inicio visible cuando está vacío.

## Alcance técnico

NotesPage reutiliza HomeReader, ReadQueue y home.query. Rust conserva filtros,
orden, procedencia y resúmenes; no hay cambios de contratos, SQLite, documentos,
relaciones ni importación de datos. Las variantes opcionales preservan Home y el
editor individual. Se reutiliza la composición de [ADR0043](decisions/0043-tasks-page-and-contextual-breadcrumb.md).

Ver [revisión y límites](verification-notes-page.md).
