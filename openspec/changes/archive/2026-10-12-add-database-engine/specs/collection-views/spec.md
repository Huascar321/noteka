# Spec Delta

## Purpose

Consultar y presentar entidades compartidas en vistas reutilizables tabla/lista con configuración persistente y reglas de dominio offline.

## ADDED Requirements

### Requirement: Vistas persistentes con consulta validada

El sistema SHALL guardar columnas, filtros básicos combinados por AND y ordenamiento de vistas tabla/lista, paginar y validar IDs/tipos. MUST aplicar consultas en Rust, no ejecutar SQL del cliente ni duplicar registros.

#### Scenario: Reabrir vista filtrada
- **WHEN** se guarda una vista con columnas, filtros y ordenamiento y se reabre
- **THEN** conserva su configuración y devuelve sólo originales que cumplen filtros en orden determinista

#### Scenario: Filtro inválido
- **WHEN** se consulta una propiedad ajena o un operador/valor no soportado
- **THEN** se devuelve validation_error sin modificar colección ni valores

### Requirement: Colecciones de escritorio offline

La interfaz SHALL crear colecciones/propiedades/registros, incorporar entidades por ID, editar valores y mostrar tabla/lista con apertura del original. MUST proteger buffers ante cierre/navegación/conflicto y mostrar errores de valores/esquema sin ocultar datos.

#### Scenario: Sistema simple sin modelos
- **WHEN** se crea una colección, se define una propiedad, se captura un registro y se edita desde la vista sin red
- **THEN** CLI y escritorio leen el mismo ID/valor y la vista guardada puede reabrirse
