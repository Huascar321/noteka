# ADR 0002: bloques semánticos y originales administrados

- Fecha: 12 de octubre de 2026.
- Estado: adoptada para `add-block-editor`.

## Decisión

Documento v2 es un árbol semántico propio: cada bloque tiene ID único, tipo, contenido inline, hijos y atributos conocidos. No persiste JSON de ProseMirror, HTML ni URLs remotas de imágenes. Un adaptador explícito convierte el contenido de presentación y falla ante tipos/IDs incompatibles. V1 sigue legible sin reescritura al migrar SQLite; editar contenido permite crear v2 con los IDs originales.

Tiptap 3.31.4 usa StarterKit, UniqueID y checklist. El portapapeles se limita a texto plano: evita originales falsos, imágenes remotas y descarte implícito de extensiones desconocidas. Los enlaces no se abren al editar. El guardado sigue explícito; el editor no se remonta por transacciones ni guardados, solamente por navegación/recarga o revisión externa sin buffer pendiente.

SQLite v2 suma metadatos `attachments`; los originales viven en `attachments/<uuid>` con SHA-256. Se copia y sincroniza antes de publicar metadatos; fallas eliminan sólo el archivo propio. Límite inicial: 64 MiB por archivo. No se ejecutan archivos ni se borra automáticamente un original al quitar su bloque. Se rechazan symlinks administrados; las rutas de importación se eligen explícitamente.

Backup v2 obtiene snapshot SQLite primero y copia exactamente sus originales inmutables. Restauración compara metadatos completos, tamaños y hashes antes de publicar. Se admite backup v1 sin adjuntos y migración posterior con backup previo. Exportación por nota conserva documentos, entidad, referencias de relaciones y originales; no lleva automáticamente las entidades relacionadas ni credenciales.

## Alternativas y consecuencias

- JSON de Tiptap directo: menor adaptación, mayor dependencia de un editor y peor control de extensiones; rechazado.
- Originales por nombre del usuario: colisiones y rutas arbitrarias; rechazado.
- Borrar huérfanos automáticamente: riesgo ante referencias futuras/compartidas y crash; diferido a una operación explícita de inspección/GC.
- Markdown como almacenamiento: no conserva identidad ni estructuras; queda intercambio con advertencias, no fuente de verdad.

No hay importador Notion, OCR, carga remota, firma ni garantía de paridad visual. Descargar/abrir archivos originales fuera del editor se resuelve mediante exportación; la UI no ejecuta un archivo al seleccionarlo.
