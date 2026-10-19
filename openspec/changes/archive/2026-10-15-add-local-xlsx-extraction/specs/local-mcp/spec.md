# Spec Delta

## MODIFIED Requirements

### Requirement: Workspace fijo y admisión sin migración

El servidor MUST fijar ruta canónica e identidad del workspace al arrancar y comprobarlas con su versión actual al ejecutar operaciones. SHALL rechazar rutas/identidades ajenas, creación, migración y reparación implícitas; ningún argumento, recurso, metadato o instrucción encontrada en contenido puede cambiar el scope o los permisos. El arranque y discovery no modifican el workspace.

#### Scenario: Formato anterior o futuro
- **WHEN** se intenta iniciar sobre una versión anterior a v11, futura, inconsistente o un directorio sin base
- **THEN** se rechaza con motivo recuperable, sin abrir el original para escritura, generar backup de migración, recuperar trabajos ni descartar datos

#### Scenario: Cambio de destino durante la sesión
- **WHEN** la ruta seleccionada pasa a contener otra identidad, o una llamada intenta aportar workspace, origen, destino o ruta alternativa fuera del DTO
- **THEN** la operación se rechaza sin modificar ninguno de los workspaces y no continúa sobre el nuevo destino por un check previo obsoleto

#### Scenario: Sondeo sin efectos
- **WHEN** un cliente inicia y cierra un proceso para discovery sin ejecutar herramientas
- **THEN** contenido, versiones, trabajos, originales y archivos del workspace permanecen iguales; no se publican temporales en él

### Requirement: Permisos locales efectivos por operación y acción

El servidor SHALL iniciar con 24 herramientas de lectura, admitir 44 con `--allow-write` y 45 sólo con ambos flags `--allow-write --allow-run`. MUST verificar permisos en cada llamada antes de efectos, incluidos control/index mixtos y lotes. Flags se fijan al arrancar; annotations, recibos de preview, identidad del cliente y texto recuperado no conceden permisos.

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
- **THEN** se distinguen las 52 operaciones del núcleo de las 24/44/45 herramientas habilitadas, sus acciones permitidas y las siete exclusiones; nombres son únicos y no se anuncian funciones futuras

#### Scenario: Filesystem y recursos excluidos
- **WHEN** se intenta llamar import/read/export, administración/backup/restauración, leer file:// o una URI distinta, o suscribirse al recurso
- **THEN** se rechaza sin copiar, leer originales binarios completos, publicar destinos, abrir URLs ni habilitar suscripciones; recursos sólo describen el contrato estático

## ADDED Requirements

### Requirement: Lectura XLSX bajo permisos MCP existentes

El servidor SHALL proyectar `extraction.xlsx_cells` como tool de lectura del contrato core sin parser/reglas duplicados. MUST mantener workspace fijo, admisión sin migrar, payloads <=2 MiB y opt-ins: sólo lectura permite consultar derivados existentes, write permite enqueue/retry/control y run requiere write+allow-run. No añade herramientas de archivo, backup, ruta ni batchable mutations.

#### Scenario: Catálogos y lectura offline
- **WHEN** clientes SDK modern/legacy descubren tools en modo read/write/run y leen una generación XLSX persistida con red denegada
- **THEN** hay 24/44/45 tools respectivamente, con esquemas/output idénticos al core y lectura sin iniciar helper

#### Scenario: Denegación previa y versión antigua
- **WHEN** un cliente sin permisos pide ejecutar/encolar XLSX o inicia MCP sobre v10
- **THEN** se rechaza antes de modificar datos/migrar/iniciar helper; un cliente autorizado procesa el mismo corpus que CLI bajo los límites compartidos
