# Spec Delta

## ADDED Requirements

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
