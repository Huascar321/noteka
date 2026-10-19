# Spec Delta

## MODIFIED Requirements

### Requirement: Superficie acotada y catálogo descubrible

El servidor SHALL mapear cada herramienta habilitada a una operación del catálogo v1 con nombre `noteka_` más puntos reemplazados por guiones bajos. MUST excluir las cuatro workspace.*, attachment.import/read y document.export aun con todos los flags. SHALL ofrecer sólo el recurso estático `noteka://catalog`, con catálogo completo, política efectiva y límites, sin datos personales ni acceso genérico a URIs.

#### Scenario: Catálogo completo y subset efectivo
- **WHEN** se compara operations.describe, tools/list y el recurso de catálogo en cada modo
- **THEN** se distinguen las54 operaciones del núcleo de las26/46/47 herramientas habilitadas, sus acciones permitidas y las siete exclusiones; nombres son únicos y no se anuncian funciones futuras

#### Scenario: Filesystem y recursos excluidos
- **WHEN** se intenta llamar import/read/export, administración/backup/restauración, leer file:// o una URI distinta, o suscribirse al recurso
- **THEN** se rechaza sin copiar, leer originales binarios completos, publicar destinos, abrir URLs ni habilitar suscripciones; recursos sólo describen el contrato estático

#### Scenario: Lectura OCR con permisos separados
- **WHEN** se lee una generación OCR desde el modo sólo lectura o se intenta encolar/ejecutar sin sus permisos
- **THEN** la lectura devuelve sólo el derivado validado sin helper; las mutaciones/ejecución siguen rechazadas sin reclamar jobs, transcribir, indexar o ampliar acceso a originales/binarios

## ADDED Requirements

### Requirement: Lectura PDFKit por permisos existentes

MCP SHALL exponer noteka_extraction_pdfkit_pages como lectura con workspace fijo y esquema compartido; enqueue/run conservan sus permisos y acciones explícitas. MUST impedir ejecución por lectura o allow-write sin allow-run, conservar siete exclusiones y batch9 y funcionar offline sin host/modelos. No publica una herramienta genérica para abrir rutas o ejecutar helpers.

#### Scenario: Lectura sin helper y permisos de ejecución
- **WHEN** un cliente en modo lectura consulta páginas persistidas y solicita run o amplía campos/permisos
- **THEN** la lectura funciona sin helper y los intentos no autorizados se rechazan sin preparación implícita

#### Scenario: Descubrimiento y paridad
- **WHEN** se recorre tools/list en cada modo y se ejecuta la nueva lectura autorizada
- **THEN** hay26/46/47 herramientas sin duplicados, schema vigente y resultado equivalente al CLI/núcleo con límite/cursor completos
