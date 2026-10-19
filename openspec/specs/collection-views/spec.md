# collection-views Specification

## Purpose
Consultar y presentar entidades compartidas en vistas reutilizables tabla/lista/tablero/calendario/galería, con configuración persistente, cobertura paginada y reglas de dominio offline.

## Requirements

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

### Requirement: Filtros compuestos tipados

El sistema SHALL combinar condiciones anidadas AND/OR/NOT con igualdad, desigualdad, comparación ordenada compatible, contains e is_empty. MUST conservar filtros AND existentes, validar todo el árbol y comparar dinero exactamente sin SQL cliente. Un cálculo fallido referenciado por el filtro MUST producir error, incluso en una rama cortocircuitada.

#### Scenario: Consulta compuesta sobre originales
- **WHEN** una vista combina OR de condiciones, AND y NOT sobre propiedades originales y calculadas válidas
- **THEN** CLI y escritorio obtienen los mismos IDs en orden determinista, con semántica documentada de null y decimales exactos

#### Scenario: Árbol inválido o cálculo fallido
- **WHEN** el árbol excede límites, incluye grupos vacíos, propiedad ajena, comparación incompatible o cálculo referenciado fallido
- **THEN** la consulta o guardado inválido devuelve error identificable y no cambia configuración, revisión ni originales

### Requirement: Tablero agrupado sin copias

El sistema SHALL presentar registros en grupos de una propiedad select o checkbox de la misma colección, incluyendo grupos vacíos y sin valor. MUST calcular grupos en el núcleo, conservar el orden de opciones, distinguir false de null y abrir la entidad original sin cambiarla al presentar.

#### Scenario: Grupos y cambio de valor
- **WHEN** se consulta un tablero y luego se modifica la propiedad de agrupación con revisión vigente
- **THEN** la siguiente consulta coloca el mismo ID en el grupo actualizado, conserva los otros originales y muestra los grupos sin registros

### Requirement: Calendario civil mensual

El sistema SHALL consultar un mes explícito por propiedad date o date_range de la colección, con límites civiles y sin conversión a UTC. MUST incluir rangos solapados, referencias al mismo ID en sus días y grupo sin fecha; excluir fechas fuera del mes e informar configuración inválida sin mutación.

#### Scenario: Mes, rangos y vacíos
- **WHEN** se consulta un mes con fechas de sus extremos, un rango que cruza el límite del mes y un registro sin fecha
- **THEN** se muestran los días pertinentes y el grupo sin fecha con IDs originales; el rango no crea páginas duplicadas ni depende de la zona del sistema

#### Scenario: Configuración de calendario inválida
- **WHEN** el mes es inválido o la propiedad pertenece a otra colección o no es date/date_range
- **THEN** el núcleo rechaza el guardado o consulta sin alterar vistas ni registros

### Requirement: Galería y cobertura paginada

El escritorio SHALL presentar galería de tarjetas textuales con propiedades seleccionadas, errores calculados y apertura/edición del original. Todas las presentaciones MUST identificar que los grupos/días cubren la página actual, permitir recorrer páginas sin duplicar originales y no descargar previews externos o adjuntos automáticamente.

#### Scenario: Recorrer presentaciones
- **WHEN** una colección excede el tamaño de página y se consulta como tablero, calendario o galería
- **THEN** el recorrido cubre los IDs coincidentes sin pérdida, la UI explicita cobertura por página y abrir una tarjeta conserva identidad/documento

### Requirement: Configuración avanzada recuperable

CLI y escritorio SHALL guardar/reabrir los nuevos layouts y configuración con revisión esperada. MUST conservar vistas tabla/lista anteriores, AST/configuraciones futuros desconocidos, buffers ante fallo/conflicto y restaurar backups sin modificar originales. El sistema MUST NOT mostrar registros viejos bajo una configuración nueva si falla la consulta al cambiar vista o layout.

#### Scenario: Reapertura, conflicto y restauración offline
- **WHEN** se guarda una vista avanzada, otro cliente cambia su revisión y se restaura un backup sin red
- **THEN** la configuración confirmada se conserva, la edición vieja da conflicto y mantiene su buffer, y el backup restaura vista/IDs/valores originales

#### Scenario: Formato futuro y fallo de consulta
- **WHEN** una configuración persistida tiene campos/layout desconocidos o falla la consulta al cambiar de presentación
- **THEN** no se convierte ni sobrescribe el formato futuro y la UI no atribuye resultados anteriores a la nueva configuración

### Requirement: Presentación incrustada paginada de originales

El escritorio SHALL consultar la vista guardada de un bloque incrustado y presentar tabla, lista, tablero, calendario o galería de lectura con actualización/paginación por teclado. MUST reutilizar filtros, grupos, errores y cobertura del núcleo; no persistir el resultado en el documento ni descargar previews externos. Un fallo MUST limpiar ese resultado y mostrar error sin borrar la referencia.

#### Scenario: Cambios y páginas del mismo destino
- **WHEN** dos notas referencian la misma vista de más de una página y el CLI modifica un registro o la configuración guardada
- **THEN** al actualizar ambas consultan los mismos originales y la configuración vigente, recorren los IDs coincidentes y explicitan cobertura por página sin incrementar revisiones de las notas

#### Scenario: Referencia no disponible o cálculo fallido
- **WHEN** falla una consulta por destino no disponible, formato desconocido o cálculo fallido referenciado en un filtro
- **THEN** el bloque informa error sin presentar datos viejos como actuales, conserva la referencia y permite reintentar después de reparar el destino

### Requirement: Apertura incrustada protegida y acotada

Abrir un registro incrustado SHALL navegar a su entidad original mediante las guardias existentes de guardado/descarte/cancelación. MUST conservar edición pendiente ante cancelación, conflicto o fallo y evitar expandir automáticamente documentos de registros dentro de otras vistas incrustadas, incluso con referencias cíclicas.

#### Scenario: Apertura con cambios pendientes
- **WHEN** se abre un registro desde una nota con texto o referencia sin guardar y se cancela la navegación o falla su guardado
- **THEN** el buffer y la referencia permanecen; después de guardar/descartar se abre el mismo ID, no una nueva página

#### Scenario: Colección que incluye su contenedor
- **WHEN** una nota pertenece a la colección que referencia o dos páginas se referencian mediante vistas
- **THEN** se muestra una proyección paginada sin recursión de documentos ni creación de copias; abrir un registro es una navegación explícita
