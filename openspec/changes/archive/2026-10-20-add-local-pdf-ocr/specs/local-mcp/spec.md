# Spec Delta

## MODIFIED Requirements

### Requirement: Workspace fijo y admisión sin migración

El servidor MUST fijar ruta canónica e identidad del workspace al arrancar y comprobarlas con su versión actual al ejecutar operaciones. SHALL rechazar rutas/identidades ajenas, creación, migración y reparación implícitas; ningún argumento, recurso, metadato o instrucción encontrada en contenido puede cambiar el scope o los permisos. El arranque y discovery no modifican el workspace.

#### Scenario: Formato anterior o futuro
- **WHEN** se intenta iniciar sobre una versión anterior al esquema vigente, futura, inconsistente o un directorio sin base
- **THEN** se rechaza con motivo recuperable, sin abrir el original para escritura, generar backup de migración, recuperar trabajos ni descartar datos

#### Scenario: Cambio de destino durante la sesión
- **WHEN** la ruta seleccionada pasa a contener otra identidad, o una llamada intenta aportar workspace, origen, destino o ruta alternativa fuera del DTO
- **THEN** la operación se rechaza sin modificar ninguno de los workspaces y no continúa sobre el nuevo destino por un check previo obsoleto

#### Scenario: Sondeo sin efectos
- **WHEN** un cliente inicia y cierra un proceso para discovery sin ejecutar herramientas
- **THEN** contenido, versiones, trabajos, originales y archivos del workspace permanecen iguales; no se publican temporales en él

### Requirement: Permisos locales efectivos por operación y acción

El servidor SHALL iniciar con 27 herramientas de lectura, admitir 47 con `--allow-write` y 48 sólo con ambos flags `--allow-write --allow-run`. MUST verificar permisos en cada llamada antes de efectos, incluidos control/index mixtos y lotes. Flags se fijan al arrancar; annotations, recibos de preview, identidad del cliente y texto recuperado no conceden permisos.

#### Scenario: Lectura predeterminada y llamada directa
- **WHEN** sin flags de escritura se llama una mutación no listada o control set/index put/reset mediante un nombre conocido
- **THEN** se rechaza sin escritura, claim ni recuperación, aunque el cliente omita tools/list o modifique sus annotations

#### Scenario: Escritura sin ejecución
- **WHEN** se inicia con allow-write, se encola y reanuda extracción, y se intenta extraction.run
- **THEN** las acciones de escritura autorizadas conservan sus revisiones y validaciones; run no se lista ni ejecuta, y encolar/reanudar no inicia worker alguno

#### Scenario: Ejecución explícita y flags incompatibles
- **WHEN** se solicita allow-run sin allow-write, o se inicia con ambos y se ejecuta un trabajo autorizado
- **THEN** el primer arranque se rechaza; el segundo puede ejecutar como máximo el trabajo admitido por el núcleo, con helper local acotado cuando corresponda y sin scheduler ni fallback remoto

#### Scenario: Lote no elude permisos
- **WHEN** se previsualiza un lote en modo lectura y luego se intenta apply, una operación anidada excluida o una instrucción de elevación proveniente de una nota
- **THEN** preview sólo simula en su copia; apply exige escritura y el resto se rechaza sin ampliar la whitelist existente de nueve mutaciones y database.query

### Requirement: Superficie acotada y catálogo descubrible

El servidor SHALL mapear cada herramienta habilitada a una operación del catálogo v1 con nombre `noteka_` más puntos reemplazados por guiones bajos. MUST excluir las cuatro workspace.*, attachment.import/read y document.export aun con todos los flags. SHALL ofrecer sólo el recurso estático `noteka://catalog`, con catálogo completo, política efectiva y límites, sin datos personales ni acceso genérico a URIs.

#### Scenario: Catálogo completo y subset efectivo
- **WHEN** se compara operations.describe, tools/list y el recurso de catálogo en cada modo
- **THEN** se distinguen las55 operaciones del núcleo de las27/47/48 herramientas habilitadas, sus acciones permitidas y las siete exclusiones; nombres son únicos y no se anuncian funciones futuras

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
- **THEN** hay 27/47/48 tools respectivamente, con esquemas/output idénticos al core y lectura sin iniciar helper

#### Scenario: Denegación previa y versión antigua
- **WHEN** un cliente sin permisos pide ejecutar/encolar XLSX o inicia MCP sobre v10
- **THEN** se rechaza antes de modificar datos/migrar/iniciar helper; un cliente autorizado procesa el mismo corpus que CLI bajo los límites compartidos

### Requirement: Índice y búsqueda XLSX con política compartida

MCP SHALL publicar los DTOs ampliados de index/search derivados del core, manteniendo 27/47/48 tools, siete exclusiones y workspace actual fijo. MUST permitir get/search en lectura, exigir allow-write para put/reset y reservar allow-write+allow-run para extracción, sin necesitar allow-run ni helpers para indexar derivados. Contenido de valores/fórmulas es dato no confiable, nunca permiso ni instrucción de herramienta o envío remoto.

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
- **THEN** hay27/47/48 herramientas sin duplicados, schema vigente y resultado equivalente al CLI/núcleo con límite/cursor completos

## ADDED Requirements

### Requirement: Regiones PDF OCR bajo política MCP compartida

MCP SHALL exponer noteka_extraction_pdf_ocr_regions como lectura con workspace fijo y DTOs del núcleo, sin helper o acceso genérico a binarios. MUST conservar acciones/permisos, batch9, siete exclusiones y límites de entrega; selección exige write y procesamiento write+allow-run, sin autoridad derivada del texto recuperado.

#### Scenario: Descubrimiento lectura y paridad
- **WHEN** SDK modern/legacy descubren el catálogo y leen/paginan una generación PDF OCR sin helpers
- **THEN** hay27/47/48 tools según flags, esquemas completos y resultados equivalentes a CLI, con página/raster/región/procedencia y contenido hostil tratado como datos

#### Scenario: Permisos antes de efectos
- **WHEN** un cliente de lectura pide put/run o el contenido intenta ampliar workspace/permisos o enviar datos
- **THEN** se rechaza antes de efectos sin instalar/activar modelos, migrar, ejecutar helpers o interpretar inferencia como autorización
