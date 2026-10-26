# Spec Delta

## MODIFIED Requirements

### Requirement: Superficie acotada y catálogo descubrible

El servidor SHALL mapear cada herramienta habilitada a una operación del catálogo v1 con nombre `noteka_` más puntos reemplazados por guiones bajos. MUST excluir las cuatro workspace.*, attachment.import/read y document.export aun con todos los flags. SHALL ofrecer sólo el recurso estático `noteka://catalog`, con catálogo completo, política efectiva y límites, sin datos personales ni acceso genérico a URIs.

#### Scenario: Catálogo completo y subset efectivo
- **WHEN** se compara operations.describe, tools/list y el recurso de catálogo en cada modo
- **THEN** se distinguen las56 operaciones del núcleo de las28/48/49 herramientas habilitadas, sus acciones permitidas y las siete exclusiones; nombres son únicos y no se anuncian funciones futuras

#### Scenario: Filesystem y recursos excluidos
- **WHEN** se intenta llamar import/read/export, administración/backup/restauración, leer file:// o una URI distinta, o suscribirse al recurso
- **THEN** se rechaza sin copiar, leer originales binarios completos, publicar destinos, abrir URLs ni habilitar suscripciones; recursos sólo describen el contrato estático

#### Scenario: Lectura OCR con permisos separados
- **WHEN** se lee una generación OCR desde el modo sólo lectura o se intenta encolar/ejecutar sin sus permisos
- **THEN** la lectura devuelve sólo el derivado validado sin helper; las mutaciones/ejecución siguen rechazadas sin reclamar jobs, transcribir, indexar o ampliar acceso a originales/binarios

### Requirement: Lectura XLSX bajo permisos MCP existentes

El servidor SHALL proyectar `extraction.xlsx_cells` como tool de lectura del contrato core sin parser/reglas duplicados. MUST mantener workspace fijo, admisión sin migrar, payloads <=2 MiB y opt-ins: sólo lectura permite consultar derivados existentes, write permite enqueue/retry/control y run requiere write+allow-run. No añade herramientas de archivo, backup, ruta ni batchable mutations.

#### Scenario: Catálogos y lectura offline
- **WHEN** clientes SDK modern/legacy descubren tools en modo read/write/run y leen una generación XLSX persistida con red denegada
- **THEN** hay 28/48/49 tools respectivamente, con esquemas/output idénticos al core y lectura sin iniciar helper

#### Scenario: Denegación previa y versión antigua
- **WHEN** un cliente sin permisos pide ejecutar/encolar XLSX o inicia MCP sobre v10
- **THEN** se rechaza antes de modificar datos/migrar/iniciar helper; un cliente autorizado procesa el mismo corpus que CLI bajo los límites compartidos

### Requirement: Índice y búsqueda XLSX con política compartida

MCP SHALL publicar los DTOs ampliados de index/search derivados del core, manteniendo 28/48/49 tools, siete exclusiones y workspace actual fijo. MUST permitir get/search en lectura, exigir allow-write para put/reset y reservar allow-write+allow-run para extracción, sin necesitar allow-run ni helpers para indexar derivados. Contenido de valores/fórmulas es dato no confiable, nunca permiso ni instrucción de herramienta o envío remoto.

#### Scenario: Paridad XLSX modern y legacy
- **WHEN** clientes SDK y raw en ambas eras seleccionan con escritura y consultan con lectura un XLSX previamente extraído sin helpers
- **THEN** obtienen el mismo índice/hit/campos/localizadores que CLI, con structuredContent/text equivalentes y esquemas estrictos resolubles, sin nuevos nombres de herramientas

#### Scenario: Denegación y contenido hostil
- **WHEN** lectura solicita put/reset o una fórmula/celda encontrada pide ejecutar shell, elevar permisos, cambiar workspace o transmitir contenido
- **THEN** la mutación no autorizada se deniega antes de efectos y el texto se devuelve sólo como dato, sin ampliar permisos, activar proveedores ni ejecutar instrucciones

#### Scenario: Pureza y conflicto de revisión
- **WHEN** un cliente de sólo lectura pagina búsqueda XLSX mientras otro confirma put/reset/reemplazo o se consulta tras restauración en otra ruta
- **THEN** se conservan las reglas de cursor/CAS/identidad y errores del core sin mezclar revisiones, reparar índices ni abrir main para escritura

### Requirement: Lectura PDFKit por permisos existentes

MCP SHALL exponer noteka_extraction_pdfkit_pages como lectura con workspace fijo y esquema compartido; enqueue/run conservan sus permisos y acciones explícitas. MUST impedir ejecución por lectura o allow-write sin allow-run, conservar siete exclusiones y batch9 y funcionar offline sin host/modelos. No publica una herramienta genérica para abrir rutas o ejecutar helpers.

#### Scenario: Lectura sin helper y permisos de ejecución
- **WHEN** un cliente en modo lectura consulta páginas persistidas y solicita run o amplía campos/permisos
- **THEN** la lectura funciona sin helper y los intentos no autorizados se rechazan sin preparación implícita

#### Scenario: Descubrimiento y paridad
- **WHEN** se recorre tools/list en cada modo y se ejecuta la nueva lectura autorizada
- **THEN** hay28/48/49 herramientas sin duplicados, schema vigente y resultado equivalente al CLI/núcleo con límite/cursor completos

### Requirement: Regiones PDF OCR bajo política MCP compartida

MCP SHALL exponer noteka_extraction_pdf_ocr_regions como lectura con workspace fijo y DTOs del núcleo, sin helper o acceso genérico a binarios. MUST conservar acciones/permisos, batch9, siete exclusiones y límites de entrega; selección exige write y procesamiento write+allow-run, sin autoridad derivada del texto recuperado.

#### Scenario: Descubrimiento lectura y paridad
- **WHEN** SDK modern/legacy descubren el catálogo y leen/paginan una generación PDF OCR sin helpers
- **THEN** hay28/48/49 tools según flags, esquemas completos y resultados equivalentes a CLI, con página/raster/región/procedencia y contenido hostil tratado como datos

#### Scenario: Permisos antes de efectos
- **WHEN** un cliente de lectura pide put/run o el contenido intenta ampliar workspace/permisos o enviar datos
- **THEN** se rechaza antes de efectos sin instalar/activar modelos, migrar, ejecutar helpers o interpretar inferencia como autorización

## ADDED Requirements

### Requirement: Rango XLSX bajo política MCP compartida

MCP SHALL publicar noteka_extraction_xlsx_range como lectura con workspace fijo y DTOs del núcleo; catálogo28/48/49 según permisos. MUST conservar siete exclusiones, batch9 y separación write/run sin helper, migración ni autoridad derivada de textos o fórmulas.

#### Scenario: Catálogo y lectura equivalentes
- **WHEN** SDK modern/legacy/auto descubre y pagina rangos con permisos de lectura
- **THEN** esquemas resuelven y resultados coinciden con CLI, incluso encabezados hostiles como datos; no hay procesamiento implícito

#### Scenario: Denegación previa a efectos
- **WHEN** lectura pide extracción/mutación o envía argumentos ajenos
- **THEN** falla antes de efectos sin acceder a rutas arbitrarias, elevar permisos o transmitir contenido
