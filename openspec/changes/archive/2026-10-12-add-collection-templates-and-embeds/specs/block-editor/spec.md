# Spec Delta

## MODIFIED Requirements

### Requirement: Documento enriquecido conservable

El sistema SHALL validar documentos v1 de texto, v2 enriquecidos y v3 con referencias a colección/vista, preservando IDs, orden, atributos y marcas admitidos. MUST rechazar tipos, marcas, propiedades y referencias no soportados sin reemplazar originales desconocidos. Una referencia incrustada MUST guardar identidad del destino, no una copia de registros.

#### Scenario: Edición y reapertura
- **WHEN** se guarda una nota v2 con encabezado, lista, checklist, cita, código, enlaces y texto marcado
- **THEN** se reabre con el mismo contenido semántico e IDs

#### Scenario: Documento desconocido
- **WHEN** el documento almacenado tiene una versión, bloque o campo desconocido
- **THEN** sigue recuperable como JSON y no se convierte al guardar

#### Scenario: Referencia v3 entre bloques
- **WHEN** se inserta una referencia a vista guardada entre bloques enriquecidos, se deshace/rehace, guarda y reabre sin red
- **THEN** se conservan IDs y referencias; documentos v1/v2 existentes no se reescriben por migrar y el resultado consultado no se serializa en el bloque

#### Scenario: Destino o atributos inválidos
- **WHEN** una escritura incluye colección inexistente, vista ajena o atributos/contenido no admitidos en la referencia
- **THEN** se rechaza sin cambiar la revisión, el documento original ni el buffer pendiente

## ADDED Requirements

### Requirement: Referencias dinámicas distinguibles de texto fuente

Exportación JSON SHALL conservar referencias incrustadas y Markdown SHALL advertir que no incluye los registros destino. La búsqueda textual MUST indexar sólo texto de la entidad/bloque fuente, no copiar filas dinámicas en el índice de la página contenedora. El backup completo MUST conservar documentos, plantillas, destinos y adjuntos.

#### Scenario: Exportar y buscar sin copias
- **WHEN** se exporta una nota con vista incrustada y se busca un término presente sólo en un registro de esa vista
- **THEN** JSON conserva la referencia y Markdown informa su límite; la búsqueda localiza el registro original y no atribuye ese texto a la nota contenedora

#### Scenario: Restauración del conjunto
- **WHEN** se restaura un backup con plantilla, nota v3, colección/vista y adjuntos locales
- **THEN** sus IDs/configuraciones/originales se conservan y la nota consulta los registros del workspace restaurado sin usar red
