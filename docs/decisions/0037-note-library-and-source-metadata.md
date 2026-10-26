# ADR 0037: biblioteca de notas y procedencia explícita

- Estado: adoptado
- Fecha: 23/10/2026

## Contexto

El usuario pide reproducir Notes desde su referencia. Las páginas importadas se
conservaron como notas; el importador anterior descartó la procedencia calculada.
El título y la hora de importación no distinguen la biblioteca de las demás páginas.

## Decisión

Añadir tipo, favorito y fuente opcionales al payload existente, validados en Rust.
La fuente almacena proveedor/colección/record_id y timestamps/archivo originales.
No se reemplazan documentos ni timestamps locales. Notas locales sin fuente siguen
perteneciendo a Notes; una fuente ajena a la colección Notes permanece en All.
Categorías/favoritos se consultan en el núcleo antes de paginar y tienen cursor
propio. La UI no filtra una página parcial para simular un conjunto completo.

Home proyecta datos del origen para ordenar y mostrar antigüedad, etiquetando su
procedencia. El dato exportado no simula sincronización continua. El listado de
entidades y All mantienen acceso a todas las notas. Campos desconocidos no se
convierten; datos inválidos se rechazan o cuentan como no interpretables.

La recuperación utiliza manifiesto y CSV originales y relaciones/iconos visibles
explícitos. Se permite resolver el ID original a una entidad renombrada conservando
su título local. Los duplicados del CSV se resuelven sólo si Created/Updated y las propiedades
del bloque de metadata original identifican inequívocamente su fila; no se lee
el cuerpo del Markdown ni se elige por posición. La metadata actual de Notion
puede actualizar fechas por ID original exacto, sin solicitar cuerpos. Backup antes de aplicar, CAS por entidad,
informe externo incremental y comparación de IDs/digest documental.

La UI comparte el ritmo visual de Tasks. Imágenes usan el formato local image-UUID
de ADR0036. Source es de lectura; tipo/favorito se editan con guardias normales.
Voice es una categoría de nota, sin grabación o transcripción.

## Consecuencias

No hay migración SQLite ni nuevo servicio remoto. El JSON es aditivo y las notas
anteriores siguen válidas. Clientes anteriores con campos cerrados deben actualizarse
antes de editar una nota enriquecida. La conversión del export sin zona usa la zona
declarada en el informe, sin modificar las fechas locales de la entidad.

La recuperación es por entidad, no una transacción global. Un error deja un informe
de las operaciones confirmadas para reanudar o revertir con revisión vigente. El
backup completo preserva el estado anterior; restaurarlo requiere considerar cambios
posteriores. No se acredita paridad de todos los filtros/automatizaciones de Notion.
