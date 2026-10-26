# Página People

El [detalle de Person](goal-person-detail.md) presenta foto local de portada,
propiedades editables en View details y contexto Notes/Projects/Tasks, sin cuerpo libre.

People lateral reutiliza las siete consultas y tarjetas de Home. Contacts es
inicial, con diez contactos por página, fotos originales locales, relaciones y
empresa; Load more acumula identidades únicas. No vuelve a mostrar el listado
genérico con kind/revisión en cada fila.

Icono grande, título, Nav y New Person siguen la composición de referencia.
Contacts y Pipeline del Nav seleccionan esas vistas; Cal abre **el calendario
de cumpleaños**, no un calendario externo. El menú Nav despliega las siete vistas.
New/Updated/Check-Ins, meses y proyectos conservan sus reglas Rust documentadas
en [People](people-reference.md).

El breadcrumb es Home → People → nombre. Apertura y captura reutilizan la
guardia de cambios pendientes; preparar y descartar no guarda una persona.
⌘R/Ctrl+R/F5 actualiza sólo la página activa, respeta diálogos y conserva el mes
de cumpleaños elegido. Fotos ausentes mantienen espacio neutro y nombre real.

Los miembros importados de WorkspacePeople conservan su entidad y acceso por
búsqueda local/API; Contacts sigue separando las once personas acreditadas como
contactos de los seis miembros importados del workspace.

[Meeting Notes](meeting-notes-design.md) es un diseño separado para captura,
transcripción y conclusiones. No se presenta como audio implementado en esta
composición de People. [Revisión acotada](verification-people-page.md).
