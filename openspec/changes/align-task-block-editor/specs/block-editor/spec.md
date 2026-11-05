# Spec Delta

## ADDED Requirements

### Requirement: Edición contextual de bloques en Tasks

El cuerpo editable de Task SHALL ofrecer la misma interacción de bloques que una Note normal: formato de selección, menú `/` filtrable y controles laterales para insertar, convertir, duplicar, mover y eliminar bloques. SHALL permitir continuar desde el espacio vacío inferior con un párrafo nuevo o reutilizando el último vacío. MUST conservar el guardado explícito, el documento opcional, IDs, historial y guardias; abrir o enfocar una Task sin cuerpo MUST NOT persistir un documento.

#### Scenario: Comandos y controles de bloques
- **WHEN** se escribe un comando `/` o se usan las herramientas contextuales en una Task
- **THEN** funcionan como en Notes, con los mismos bloques admitidos, selección, historial y acciones de bloque

#### Scenario: Continuar bajo el último bloque
- **WHEN** se pulsa el espacio inferior de una Task editable
- **THEN** se enfoca el párrafo final vacío o se añade uno nuevo, sin acumular vacíos al repetir el gesto ni guardar automáticamente

#### Scenario: Documento opcional y guardias
- **WHEN** se abre o enfoca una Task sin documento o se intenta salir con cambios pendientes
- **THEN** la apertura no persiste contenido y la salida mantiene las guardias de guardar, descartar o cancelar
