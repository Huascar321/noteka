# Spec Delta

## ADDED Requirements

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
