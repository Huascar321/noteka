# People — referencia local

Ver el [detalle de Person](goal-person-detail.md) y su
[revisión acotada](verification-goal-person-detail.md): portada local,
campos y acciones existentes conservados, sin añadir documento de cuerpo.

La [página lateral People](people-page.md) reutiliza esta galería y las siete
vistas, añadiendo icono sobre título, Nav y New Person. Meeting Notes se define
en su [diseño propuesto](meeting-notes-design.md); esta referencia no acredita ASR.

Actualización: 24/10/2026. Home abre **Contacts** con tarjetas pequeñas, fotos
originales locales, nombre, relaciones y empresa. Muestra diez contactos y
**Load more** agrega la página siguiente conservando los cargados. New/New page
preparan un borrador; guardar confirma la persona mediante las guardias existentes.

## Vistas

| Vista | Regla compartida en Rust |
|---|---|
| Contacts | Personas locales sin procedencia o de People no archivadas en origen; orden acreditado de galería, luego título/ID |
| New | Contactos creados desde un mes civil antes, inclusivo |
| Updated | Contactos editados desde un mes civil antes, inclusivo |
| Check-Ins | Check-in asignado hasta un mes civil futuro, incluyendo atrasados; orden por fecha |
| Birthdays | Ocurrencias en el mes elegido, con navegación mensual; 29/2 se muestra el 28 en años no bisiestos |
| Pipeline | Prospect, Contacted, Negotiating, Rejected y Closed; columnas vacías visibles y Sin estado sólo si corresponde |
| Active Projects | Proyectos Doing/Ongoing activos localmente y en origen, agrupados por cada contacto relacionado admisible |

New/Updated usa el timestamp original acreditado si existe; de otro modo usa el
local, interpretado en la zona del workspace. Recuperar metadata hoy no convierte
un contacto antiguo en nuevo. Los límites mensuales mantienen el día cuando existe
y se ajustan al último día del mes si hace falta. No se asignan fechas al crear.

Las vistas diferentes de Contacts consultan hasta cien registros por página.
Los grupos cuentan apariciones de la página: un proyecto puede estar en varios
grupos de persona, pero conserva una sola identidad. El progreso usa sus tareas
locales relacionadas. Los errores se distinguen de las vistas vacías.

## Propiedades y fotos

El detalle permite editar empresa, relaciones, cumpleaños, próximo/último check-in
y pipeline. La procedencia y el orden recuperados se conservan al guardar.
Relaciones de persona admite hasta 32 etiquetas únicas de 128 bytes; empresa,
512 bytes. Las fechas son civiles canónicas y los estados tienen validación Rust.

La foto es un UUID de adjunto del workspace, con posición vertical entera 0..100.
Se leen PNG/JPEG/GIF/WebP de hasta 1 MiB mediante el lector/cache acotado existente.
Un adjunto ausente, excesivo o incompatible deja un espacio neutro. No se envían
URLs a Internet al abrir People ni se generan fotos de personas.

## Procedencia y recuperación

El importador original mezcló dos CSV llamados People: once contactos y seis
miembros del workspace. La recuperación acreditó los once con CSV, manifiesto,
título único y ID de su página original; acreditó los seis restantes con el otro
CSV. Home separa `source.collection = People` de `WorkspacePeople`; el listado
general mantiene las diecisiete entidades.

Se recuperaron once metadatos de contacto, seis procedencias de miembro, nueve
fotos observadas con su recorte y 38 relaciones explícitas Project→Person.
No se copiaron correos, teléfonos, cuerpos o atributos no usados por estas vistas.
Los archivos completos recibidos del conector, fotos, backup e informes permanecen
fuera de Git en el directorio privado de verificación.

El helper `scripts/recover-notion-person-metadata.py` hace dry-run por defecto.
Aplicar exige destino nuevo de backup, añade sólo campos ausentes, usa revisiones
esperadas y guarda cada resultado en un informe privado. Nunca incluye document
en un update. El informe confirma los 1.802 IDs y digest documental intactos.
Para recuperación ante un fallo se inspeccionan payload/revisiones anteriores;
no se restaura ciegamente sobre ediciones posteriores.

La metadata es un snapshot autorizado, sin sincronización continua. El orden
acreditado conserva la referencia; no replica la fórmula remota opaca de nombre.
No hay mensajes, recordatorios, CRM completo ni arrastre de Pipeline. Los clientes
anteriores con validadores cerrados deben actualizarse antes de editar los campos
nuevos. No cambia el esquema SQLite.

[Decisión](decisions/0042-person-metadata-and-local-gallery.md) ·
[Revisión acotada](verification-people-reference.md).
