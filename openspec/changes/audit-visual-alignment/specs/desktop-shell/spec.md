## ADDED Requirements

### Requirement: Alineación de propiedades y relaciones

La interfaz SHALL mantener espaciado interior legible en fechas compactas y alinear las etiquetas de relaciones con la primera línea de sus acciones y destinos, conservando el ajuste de contenido y los controles nativos.

#### Scenario: Fecha compacta

- **WHEN** una fecha se muestra o edita en un detalle
- **THEN** su contenido dispone de separación interior del borde y el selector nativo sigue funcionando

#### Scenario: Relaciones vacías o pobladas

- **WHEN** una fila de relaciones muestra una acción o destinos con nombres largos
- **THEN** la etiqueta se alinea con la primera línea y los destinos pueden envolver sin solaparse ni ocultar acciones
