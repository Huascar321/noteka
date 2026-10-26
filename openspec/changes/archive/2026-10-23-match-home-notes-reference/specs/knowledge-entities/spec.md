# Spec Delta

## ADDED Requirements

### Requirement: Clasificación y procedencia opcionales de notas

El núcleo SHALL validar tipo y favorito opcionales de notas y procedencia explícita con proveedor, colección, ID y timestamps originales opcionales. MUST conservar documentos e IDs sin falsificar timestamps locales, admitir notas anteriores sin esos campos y rechazar valores inválidos sin escrituras parciales.

#### Scenario: Enriquecer con revisión vigente
- **WHEN** se añade metadata de fuente a una nota con revisión vigente
- **THEN** conserva su documento e identidad, registra la revisión normal y devuelve los campos con creación local intacta y actualización local normal

#### Scenario: Valor inválido o revisión vencida
- **WHEN** se envía tipo, procedencia o fecha inválida, o una revisión anterior
- **THEN** rechaza la operación y conserva la nota previa sin convertir su documento
