# Spec Delta

## ADDED Requirements

### Requirement: Lectura XLSX por hoja rango y encabezados explícitos

`extraction.xlsx_range` SHALL leer una hoja y rango A1 canónico de hasta64 columnas, con encabezado opcional dentro del rango, desde la generación confirmada. MUST devolver celdas dispersas completas con ordinal global, manifiesto y encabezados por columna sin inferencia, recálculo o expansión de merges/grilla.

#### Scenario: Región dispersa y encabezados elegidos
- **WHEN** se consulta una hoja oculta con huecos, nombres duplicados/vacíos, números largos y fórmulas sin caché, indicando o no header_row
- **THEN** conserva hoja/visibilidad, tipos, fórmulas y cachés; encabezados sólo aparecen por elección y distinguen celda ausente de valor ausente; todas las celdas del rango siguen en items

#### Scenario: Rangos y límites estrictos
- **WHEN** el rango no es A1 canónico ordenado dentro de A1:XFD1048576, excede64 columnas, falta hoja o el encabezado queda fuera
- **THEN** devuelve error explícito sin devolver resultado parcial, ejecutar helpers o modificar datos; un rango vacío válido devuelve cero items

### Requirement: Paginación XLSX de rango ligada a criterios completos

El lector SHALL paginar1–64 celdas/default32 con cursor ligado a workspace/ruta/operación/job/generación/límite/hoja/rango/encabezado. MUST validar original y representación completos incluso fuera del rango, conservar cobertura del rango separada de generación y mantener respuesta<=2MiB sin truncamiento.

#### Scenario: Cursor y cambios de criterios
- **WHEN** se cambia hoja/rango/encabezado/límite/job/ruta/generación al continuar o se aporta cursor inválido
- **THEN** se rechaza sin mezclar páginas; recorrido estable conserva ordinals y cubre exactamente las celdas seleccionadas, sin acumular la grilla

#### Scenario: Integridad pureza y recuperación
- **WHEN** se lee sin helper, tras restore o con fuente/derivado alterados fuera del rango
- **THEN** una lectura íntegra no escribe y la alterada falla completa sin reparación; selección FTS, generaciones, originales y desconocidos permanecen iguales
