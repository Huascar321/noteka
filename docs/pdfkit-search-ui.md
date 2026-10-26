# PDFKit en preparación, lectura y búsqueda

`add-local-pdfkit-search-ui` integra el backend nativo verificado con los paneles
existentes y la selección FTS explícita. Aceptación conjunta aprobada y
deltas sincronizados/contrastados. No completa toda fase5a. API54/MCP26/46/47/batch9 se conservan.

## Acciones y evidencia

En una nota, elegí `pdfkit_text` en preparación. Encolar, reanudar, ejecutar,
consultar y seleccionar son acciones separadas. Importar/ejecutar/leer no indexa.
El lector usa `extraction.pdfkit_pages` con20 páginas físicas por solicitud y
reemplaza la página anterior. Incluye `no_text`, texto literal, procedencia y
manifiesto completo. No requiere helper para leer ni para seleccionar/buscar.

Search individual añade filtro `pdfkit_text`; el ámbito conjunto conserva grupos
independientes de entidades y adjuntos, veinte hits por grupo. Sólo se proyecta
texto no vacío, sin cruzar páginas. Cada fragmento<=4096 bytes incluye:

- `kind: pdfkit_page_text` y `projection: pdfkit_page_text_v1`;
- perfil `apple_pdfkit_text_v1`, página física1-based y bytes totales de página;
- rango UTF-8 del texto derivado `[text_byte_start,text_byte_end)`;
- motor, versión OS/build y limitaciones completos.

Son bytes del derivado, no offsets del PDF, índices JavaScript o geometría.
PDFKit selecciona texto según el parser de sistema; no garantiza orden visual,
tablas u OCR. `no_text` no prueba que la página esté visualmente vacía. La UI
presenta contenido como texto escapado, incluso instrucciones/HTML hostiles.

Cambiar criterios descarta páginas y respuestas tardías. Errores retiran evidencia
vieja, sin reparar, reintentar o indexar automáticamente. Guardias y buffers del
editor siguen vigentes. El error de un grupo conserva el snapshot del otro.
Home, navegación, estilos/temas y composición permanecen intactos.

## Persistencia y compatibilidad

015 sólo incrementa la barrera a SQLite15 en la transacción de migración; conserva
las41 tablas, SQL y filas existentes. El cliente14 rechaza15 sin escribir. El
cliente15 rechaza16; backup14 restaurado con cliente14 conserva datos anteriores.
No hay downgrade in-place ni reindexación automática. Cursores vigentes previos
conservan significado cuando su revisión y workspace no cambian.

La nueva rama de hash `content.v1` incluye manifiesto nativo. Los otros formatos
conservan bytes/hash. Todo el índice/canónico/postings se valida aun al consultar
un filtro distinto o cero hits. Los originales se verifican para candidatos de
página/lookahead. Corrupción produce error; no se omiten validaciones para cumplir
latencia. Se conservan512 fragmentos/1MiB por selección,8MiB/1024 selecciones y
cap2MiB de página; metadata mayor puede causar un rechazo completo.

Por el límite de100 páginas y1MiB canónico, PDFKit no puede generar513 fragmentos
válidos. La prueba usa fragmentación máxima admitida con fronteras UTF-8 y conserva
el cap defensivo512; rechaza documentos excesivos sin fabricar un caso imposible.

Los52 contratos de operación no afectados se comparan con el cliente14 preservado;
sólo el input de índice y el input/output de búsqueda cambian por el tipo nuevo.
La whitelist batch de nueve y las siete exclusiones MCP no cambian.

## Verificación

La [evidencia](verification-pdfkit-search-ui.md) separa corpus real nativo, migración
con clientes independientes, SDK MCP, pruebas headless y gates300/150. El puente
headless usa CLI real; no acredita IPC nativo ni las ocho tareas Home pendientes.
No se activan modelos, proveedores, OAuth, pagos, publicaciones o fase6.
