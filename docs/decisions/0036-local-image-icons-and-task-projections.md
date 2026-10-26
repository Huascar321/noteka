# ADR 0036: imágenes de icono locales y proyecciones de Tasks

- Estado: adoptado
- Fecha:23/10/2026

## Contexto

El usuario solicita copiar el arte y composición de Tasks desde su Notion.
Lucide puede sustituir iconos nativos; logos/fotos personalizados deben conservar
su recurso real. El export anterior conservó entidades y documentos, pero omitió
iconos personalizados y relaciones. La UI no debe inventarlos por título.

## Decisión

Se interpreta `image-<UUID hexadecimal sin guiones>` en el campo icon existente.
El formato ya admite estas38 posiciones y los adjuntos conservan bytes/hash.
No se introduce otra tabla, migración, URL remota ni referencia de archivos
arbitrarios. Clientes anteriores muestran fallback; conservan el valor mientras
no sustituyan el icono. La validación sintáctica de claves anterior se mantiene.

AppIcon obtiene el workspace mediante un getter de contexto, verifica patrón,
MIME/tamaño y usa una cache acotada con cola física de2. Una instancia invalida
su publicación al cambiar de nombre/workspace o desmontar. Los resultados se
separan por ruta de workspace e ID. Cambiar color conserva la clave de imagen.
Se rechaza la representación de SVG/HTML; un PNG de interfaz rasterizado no
habilita documentos ejecutables.

Las seis vistas adicionales de Tasks se calculan en Rust antes de paginar,
con límites y cursores propios de home.query. La UI sólo presenta datos. Las
vistas de proyectos conservan una fila original por ID y apariciones por grupo,
ordenadas por creación local descendente. Las variantes anteriores no cambian.
El estado Doing de tareas y filtros/automatizaciones completos de Notion quedan
fuera de esta decisión; las vistas de proyecto declaran su alcance local.

La recuperación de arte y relaciones requiere un manifiesto explícito, backup,
correspondencia inequívoca y revisión esperada. No se asignan vínculos sólo por
posición de fila. El duplicado observado se resuelve únicamente con ID original,
propiedades exactas y trazabilidad completa al manifiesto del importador; si
esa evidencia no existe, queda pendiente. Los informes están
fuera de Git y permiten revertir sólo las modificaciones realizadas. Las
mutaciones registran revisiones/timestamps normales, sin cambiar documentos,
estados ni fechas de planificación/vencimiento.

## Consecuencias

El mismo icono se puede presentar en varias superficies offline. Una imagen
no disponible cae a Lucide y no bloquea el contenido. attachment.read conserva
su cap nativo; el límite1MiB se aplica a presentación/cache, después de leer.
Las pestañas conservan sus guardias y revisiones: un buffer anterior al
enriquecimiento puede requerir resolver el conflicto normal al guardar.

Las pruebas históricas conservan su alcance. Esta entrega usa compilación e
inspección visual de un snapshot con CLI real, sin ejecutar suites no pedidas
ni acreditar cierre/IPC nativos.
