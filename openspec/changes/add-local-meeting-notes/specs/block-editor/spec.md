# Spec Delta

## ADDED Requirements

### Requirement: Componente Meeting Notes y comando meet

El editor SHALL reconocer `/meet` como preparación de un bloque de reunión con
identidad estable y referencia local. MUST conservar notas/bloques existentes,
proteger borradores y referencias de otro workspace, y representar la nueva
estructura mediante versión de documento explícita con recuperación previa.
Preparar el bloque MUST NOT iniciar captura, guardar contenido ajeno ni activar modelos.

#### Scenario: Componente en una nota
- **WHEN** se invoca `/meet` y luego se guarda o abre el componente
- **THEN** se conserva su identidad/referencia y se puede usar sin transformar
  toda la nota contenedora en una transcripción ni duplicar una sesión existente

#### Scenario: Documento anterior o incompatible
- **WHEN** se abre un documento anterior o un cliente no puede representar el bloque
- **THEN** los anteriores mantienen su formato/IDs y el incompatible permanece
  recuperable con estado explícito, sin omitirlo o guardarlo como pérdida silenciosa

## MODIFIED Requirements

### Requirement: Documento enriquecido conservable

El sistema SHALL validar documentos v1 de texto, v2 enriquecidos, v3 con referencias a colección/vista y v4 con componentes de reunión, preservando IDs, orden, atributos y marcas admitidos. MUST rechazar tipos, marcas, propiedades y referencias no soportados sin reemplazar originales desconocidos. Una referencia incrustada MUST guardar identidad del destino, no una copia de registros.

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
