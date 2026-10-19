# Spec Delta

## Purpose

Editar notas enriquecidas offline mediante bloques identificables y un formato semántico independiente de la interfaz de edición.

## ADDED Requirements

### Requirement: Documento enriquecido conservable

El sistema SHALL validar documentos v1 de texto y v2 enriquecidos, preservando IDs, orden, atributos y marcas admitidos. MUST rechazar tipos, marcas, propiedades y referencias no soportados sin reemplazar originales desconocidos.

#### Scenario: Edición y reapertura
- **WHEN** se guarda una nota v2 con encabezado, lista, checklist, cita, código, enlaces y texto marcado
- **THEN** se reabre con el mismo contenido semántico e IDs

#### Scenario: Documento desconocido
- **WHEN** el documento almacenado tiene una versión, bloque o campo desconocido
- **THEN** sigue recuperable como JSON y no se convierte al guardar

### Requirement: Editor local con buffer protegido

La interfaz SHALL ofrecer edición enriquecida, comandos de bloque por `/`, undo/redo y guardado explícito por teclado. MUST conservar el buffer ante error o conflicto y permitir guardar, descartar o cancelar al salir.

#### Scenario: Conflicto en nota enriquecida
- **WHEN** el CLI modifica la revisión de una nota con edición local pendiente
- **THEN** el guardado de la interfaz falla sin perder el buffer ni la revisión del CLI

#### Scenario: Uso offline
- **WHEN** se insertan bloques, se deshace una edición y se guarda sin red
- **THEN** la nota queda persistida sin cargar recursos externos
