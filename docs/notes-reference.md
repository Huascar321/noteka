# Notes y metadata de origen

Notes usa encabezado grande, New Note, ocho vistas con iconos, filas compactas
y Load more. Contextos locales aparecen antes del tipo y la antigüedad. Los
iconos de clave usan el [catálogo local vigente](icons.md); imágenes personalizadas conservan adjuntos locales.

## Vistas y edición

- Notes: notas locales sin procedencia importada y notas generales de la base
  Notes, incluidas Voice y excluidas las archivadas en origen y los tipos Web
  Clip/Journal/Daily/Meeting. Usa `notes_general`; `notes_library` conserva su regla
  anterior que excluye Voice. El filtro remoto adicional por URL todavía no se
  replica: la URL importada reside en el documento, no en la proyección Home.
- Inbox: notas activas sin relación local Project; Tag y People no las excluyen.
  Se deriva de relaciones guardadas y ya no existe una marca manual.
- Unclassified: notas de biblioteca sin relación local Tag, Project ni People,
  con independencia de Inbox (que sólo mira Project y no restringe a biblioteca).
  No comprueba Content, que aún no está modelado.
- Fav.: favorito explícito dentro de la biblioteca.
- Clips, Voice y Meetings: tipo explícito dentro de la biblioteca. Journal reúne
  Journal y Daily mediante `notes_diary`; `notes_journal` conserva sólo Journal.
- All: todas las notas activas de Noteka, incluidas páginas importadas de otras
  bases. La pestaña All de la página Notes y la búsqueda conservan ese acceso completo.

Rust filtra antes de paginar. Las vistas nuevas usan modificación de fuente
cuando está disponible, local en otro caso, con desempate por ID. Las anteriores
conservan sus reglas. Load more agrega páginas de diez, elimina duplicados por
ID y conserva el cursor del núcleo. Actualizar o cambiar de vista inicia otra
lectura. Errores y cobertura no disponible se muestran explícitamente.

En el [detalle de Note](note-detail.md), Type y Favorite se editan en la fila
principal; la metadata de origen permanece en View details y ya no hay marca
Inbox. Crear desde una categoría preselecciona ese tipo; desde Fav. preselecciona
su favorito. No hay escritura hasta guardar. Voice clasifica notas: grabación/transcripción siguen pendientes.

## Datos

NoteFields admite `note_type`, `favorite` y `source` opcionales. Los tipos son
reference, plan, web_clip, voice, journal, daily, meeting, topic_research,
outlier_content, creator_tips, tools, book, recipe, idea y lecture. Clientes con
enums cerrados anteriores deben actualizarse antes de editar los tipos nuevos.
Source conserva provider, collection, record_id y fechas originales opcionales,
además del archivo en origen. Sus cadenas/fechas se validan en Rust.

Las fechas locales no se falsifican: un update registra revisión/modificación
normales. El badge importado muestra días civiles de la fuente, cuyo timestamp
se explica al pasar el puntero; no afirma una sincronización viva con Notion.
El origen se presenta en lectura. Un cliente anterior que valida payloads con
campos cerrados necesita actualizarse para editarlos; el backup permite recuperar
el estado anterior. No se migra SQLite ni el documento.

## Recuperación

El script `scripts/recover-notion-note-metadata.py` utiliza el manifiesto de la
importación, CSV y metadata visible autorizada. Dry-run inicial; `--apply` exige
backup canónico nuevo. Resuelve IDs originales y coincidencias únicas de extremos,
aplica con revisión esperada, conserva iconos/tipos/favoritos locales existentes
y guarda informe incremental con payload/revisión anteriores.
El tipo explícito `Voice Note` se traduce a `voice` cuando falta el tipo local;
no se modifican automáticamente notas ya importadas. Ejecutar la recuperación
requiere las fuentes originales y una aplicación explícita con backup.

Una consulta de metadata actual de Notion puede refrescar timestamps originales
por record_id exacto, sin solicitar cuerpos. La entrega contrastó las diez filas
visibles y corrigió dos fechas que el export tenía desactualizadas. Las otras
notas conservan la metadata exportada; no hay sincronización continua.

Los títulos renombrados se conservan: el ID del manifiesto acredita la identidad.
Las filas sin correspondencia inequívoca se registran y no reciben una categoría
inventada. No se abren cuerpos de páginas remotas. Los manifiestos, imágenes,
backups e informes personales quedan fuera de Git. El digest de documentos y los
IDs se comparan antes y después.

La zona del export se pasa explícitamente; la entrega usa America/La_Paz porque
el export no declara un offset. Revertir mediante informe requiere la revisión
vigente; restaurar un backup completo reemplaza cambios posteriores y exige
elegir ese alcance explícitamente.

Ver [revisión acotada](verification-notes-reference.md) y [ADR0037](decisions/0037-note-library-and-source-metadata.md).

La entrada del menú usa ahora la [página Notes](notes-page.md), con encabezado
propio y Tags integrado. Las reglas de biblioteca, categorías y procedencia de
esta guía se comparten con Home; no se duplican filtros en la interfaz.

## Corrección de procedencia durante Tags — 23/10/2026

Se repararon dos notas cuyos títulos completos excedían 240 caracteres y habían
quedado como Pages por truncar el índice CSV de recuperación. Identidad original,
backup y CAS acreditados; no cambian cuerpos. Clips muestra ahora 52, Notes 132 y
All 1.172. Los conteos anteriores quedan como evidencia del snapshot previo.
Ver [revisión de Tags](verification-tags-reference.md).
