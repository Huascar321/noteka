# Página Tags

Al abrir una fila, el [detalle del Tag](tag-detail.md) presenta Type/Favorite,
contador canónico y secciones de contexto Notes/Clips/Tasks.

Tags lateral presenta icono propio sobre el título, breadcrumb Home → Tags,
Nav con Tasks/Notes/Projects y New Tag. La página comienza en Recent, con diez
filas por lectura; Load more acumula identidades únicas hasta agotar el cursor.
Las filas usan iconos locales, última nota navegable y clasificación disponibles.

Las cinco pestañas son Recent, Fav., A-Z, Types y Table. Las primeras cuatro
conservan las consultas, filtros, actividad y límites Rust descritos en
[Tags](tags-reference.md). Table presenta Tag, Last Note y Type en una tabla
semántica con el orden de `tags_alphabetical`, sin una consulta pública nueva.
Su selección se mantiene al actualizar y continuar; cambiar a A-Z vuelve a listas.

Nav abre secciones mediante la navegación protegida existente. El tag y la
última nota abren sus registros originales con breadcrumb canónico, incluso
cuando la nota pertenece a Notes. New Tag/New/New page preparan Resource; en
Fav. se prepara favorito y la creación dentro de Types conserva su tipo. La
captura no se guarda automáticamente y mantiene Guardar/Descartar/Cancelar.

New page y Load more ocupan toda su fila. New Tag conserva borde y tamaño de
botón. Opciones de vista contiene explicación y cobertura; no se agregan
contadores técnicos al pie. ⌘R/Ctrl+R/F5 actualiza sólo la página activa sin
recargar el WebView, con bloqueo por diálogos y descarte de respuestas tardías.

Home conserva Types inicial y cuatro vistas; Tags incrustado en Notes conserva
Recent y cuatro vistas independientes. Los registros archivados en origen se
excluyen de estas consultas, conservando sus originales locales. Recent usa un
snapshot acotado y su orden puede diferir de la actividad actual de Notion.

La tabla permite scroll horizontal interno en ventanas estrechas; las listas
distribuyen metadata debajo del título. Claro/oscuro y guardias compartidos.
Sin cambios de formato persistente, Rust, API, dependencias ni datos personales.
La [revisión acotada](verification-tags-page.md) distingue compilación e inspección
manual de la aceptación nativa histórica.
