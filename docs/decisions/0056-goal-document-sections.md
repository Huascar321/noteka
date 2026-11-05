# ADR0056 — Secciones de Goals dentro del documento existente

Fecha: 27/10/2026. Estado: adoptada para `enhance-goal-detail`.

Goal continúa siendo una Note tipada y su documento SQLite es la única fuente
para el contenido. Goal Overview, Additional Details y Journal se guardan como
bloques ordinarios de documento, delimitados por encabezados identificables con
una misma identidad aleatoria. Los editores separan visualmente las secciones y
recomponen el documento antes del guardado CRUD/CAS existente.

Sólo se reconoce una pareja íntegra de delimitadores de Overview y Journal; el
marcador intermedio de Additional Details es opcional para compatibilidad con una
versión sin esa sección. Bloques anteriores y prefijos de Notion se conservan;
marcadores incompletos o alterados no se interpretan y permanecen en el contenido
anterior. Abrir no agrega bloques ni escribe. La primera edición explícita crea
los delimitadores y secciones. Los documentos de formato desconocido mantienen
su ruta de lectura y recuperación sin conversión.

El Journal agrega un encabezado fechado y un párrafo editable dentro del mismo
documento. Proyectos usa las relaciones y datos ya cargados; no calcula en el
cliente métricas de progreso. No hay tabla, migración, operación ni dependencia
nueva. Una versión anterior puede mostrar los encabezados como contenido normal;
si modifica un delimitador, el documento se presenta como contenido anterior y
no se descarta. Se evita deducir automáticamente qué partes de una plantilla
importada corresponden a cada sección, porque la estructura puede ser ambigua.
