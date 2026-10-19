# Spec Delta

## Purpose

Encontrar títulos y texto local sin modelos, manteniendo referencias verificables al contenido original y un índice derivado reconstruible.

## ADDED Requirements

### Requirement: Búsqueda textual acotada con fuentes

El sistema SHALL buscar términos literales en títulos y bloques admitidos, devolver entidad/revisión/bloque de origen y extracto de texto, filtrar tipo/archivo/contexto y paginar. MUST rechazar consultas vacías o excesivas sin interpretar SQL, shell ni instrucciones contenidas en documentos.

#### Scenario: Bloque enriquecido y filtros
- **WHEN** se busca texto de un bloque anidado dentro de un contexto
- **THEN** se obtiene el ID/revisión del original y sólo resultados que cumplen filtros

#### Scenario: Consulta inválida
- **WHEN** la consulta está vacía, supera límites o contiene texto parecido a SQL/operadores
- **THEN** se rechaza el input inválido o se busca literalmente sin ejecutar instrucciones

### Requirement: Índice consistente y reconstruible

El sistema MUST actualizar el índice junto a las escrituras de dominio, no devolver revisiones desactualizadas como actuales y preservar originales desconocidos sin convertirlos para indexar. SHALL reconstruir FTS desde SQLite sin modificar entidades, revisiones ni adjuntos y migrar con backup previo.

#### Scenario: Actualizar y archivar
- **WHEN** cambia el texto o se archiva una entidad
- **THEN** la búsqueda vigente deja de presentar texto/revisión anterior y respeta el estado de archivo

#### Scenario: Reconstrucción y formato futuro
- **WHEN** se reconstruye el índice con un documento desconocido almacenado
- **THEN** sus originales y revisión quedan intactos; sólo el título reconocido es indexable

### Requirement: Búsqueda de escritorio offline

La interfaz SHALL buscar y abrir el original con su localizador, conservar buffers pendientes al navegar y mostrar límites de fuente sin cargar servicios remotos.

#### Scenario: Buscar y abrir sin red
- **WHEN** se busca una nota enriquecida sin red ni modelos
- **THEN** se abre la entidad original con referencia de bloque/revisión y se conserva la protección de edición pendiente
