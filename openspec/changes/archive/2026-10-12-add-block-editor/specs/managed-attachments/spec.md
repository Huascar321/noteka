# Spec Delta

## Purpose

Conservar adjuntos originales locales y transportar notas con evidencia de integridad sin convertir los archivos de intercambio en fuente de verdad.

## ADDED Requirements

### Requirement: Originales administrados e íntegros

El sistema MUST completar y verificar una copia local antes de publicar metadatos de un adjunto. SHALL validar referencias dentro del workspace y conservar archivos al eliminar un bloque; ninguna importación ejecuta archivos ni carga contenido remoto.

#### Scenario: Importación y referencia
- **WHEN** se importa un archivo local y se inserta su ID en una nota
- **THEN** el original administrado tiene hash y tamaño verificables y puede recuperarse offline

#### Scenario: Referencia inexistente o copia fallida
- **WHEN** una referencia no existe o una copia falla
- **THEN** no se confirma una nota con una referencia inválida ni metadatos que prometan un archivo ausente

### Requirement: Exportación portable y límites visibles

El sistema SHALL exportar una nota como JSON original y adjuntos con hashes, o como Markdown con advertencias por pérdida de identidad/formato. MUST rechazar un destino ocupado; la exportación no modifica la nota.

#### Scenario: Exportación fiel
- **WHEN** se exporta una nota con imagen y archivo a un destino nuevo
- **THEN** el JSON preserva IDs/atributos y el paquete incluye ambos originales verificables

#### Scenario: Markdown con límites
- **WHEN** se exporta Markdown
- **THEN** se conserva texto representable y se informa que no es una exportación fiel de identidades y metadatos
