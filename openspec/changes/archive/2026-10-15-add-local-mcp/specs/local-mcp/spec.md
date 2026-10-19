# Spec Delta

## Purpose

Permitir que clientes MCP locales descubran y ejecuten un subconjunto explícito de operaciones compartidas de Noteka por stdio, con workspace fijo, permisos verificables y sin necesitar escritorio, modelos ni servicios externos.

## ADDED Requirements

### Requirement: Servidor stdio local sin activación externa

El sistema SHALL ofrecer `noteka-mcp --workspace <ruta>` sobre un workspace existente y actual, sin iniciar escritorio, daemon, sockets, modelos ni proveedores. MUST funcionar con red denegada y reservar stdout para mensajes MCP; diagnósticos mínimos van a stderr, sin contenido de documentos, argumentos completos, credenciales ni trazas internas por defecto.

#### Scenario: Lectura offline con ventana cerrada
- **WHEN** un cliente local inicia el servidor sobre una fixture actual con red denegada y sin aplicación gráfica o LLM
- **THEN** puede descubrir herramientas y consultar una entidad persistida; no aparecen procesos gráficos, conexiones de red ni descargas

#### Scenario: Arranque y diagnóstico separados del protocolo
- **WHEN** se inicia con flags inválidos, ruta ausente o workspace incompatible, o se registra un error durante una sesión
- **THEN** el arranque inválido termina con código no cero y diagnóstico en stderr sin crear datos; en una sesión válida stdout contiene sólo mensajes JSON-RPC MCP y los logs no incluyen el contenido consultado

### Requirement: Workspace fijo y admisión sin migración

El servidor MUST fijar ruta canónica e identidad del workspace al arrancar y comprobarlas con su versión actual al ejecutar operaciones. SHALL rechazar rutas/identidades ajenas, creación, migración y reparación implícitas; ningún argumento, recurso, metadato o instrucción encontrada en contenido puede cambiar el scope o los permisos. El arranque y discovery no modifican el workspace.

#### Scenario: Formato anterior o futuro
- **WHEN** se intenta iniciar sobre una versión anterior a v10, futura, inconsistente o un directorio sin base
- **THEN** se rechaza con motivo recuperable, sin abrir el original para escritura, generar backup de migración, recuperar trabajos ni descartar datos

#### Scenario: Cambio de destino durante la sesión
- **WHEN** la ruta seleccionada pasa a contener otra identidad, o una llamada intenta aportar workspace, origen, destino o ruta alternativa fuera del DTO
- **THEN** la operación se rechaza sin modificar ninguno de los workspaces y no continúa sobre el nuevo destino por un check previo obsoleto

#### Scenario: Sondeo sin efectos
- **WHEN** un cliente inicia y cierra un proceso para discovery sin ejecutar herramientas
- **THEN** contenido, versiones, trabajos, originales y archivos del workspace permanecen iguales; no se publican temporales en él

### Requirement: Permisos locales efectivos por operación y acción

El servidor SHALL iniciar con 23 herramientas de lectura, admitir 43 con `--allow-write` y 44 sólo con ambos flags `--allow-write --allow-run`. MUST verificar permisos en cada llamada antes de efectos, incluidos control/index mixtos y lotes. Flags se fijan al arrancar; annotations, recibos de preview, identidad del cliente y texto recuperado no conceden permisos.

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
- **THEN** se distinguen las 51 operaciones del núcleo de las 23/43/44 herramientas habilitadas, sus acciones permitidas y las siete exclusiones; nombres son únicos y no se anuncian funciones futuras

#### Scenario: Filesystem y recursos excluidos
- **WHEN** se intenta llamar import/read/export, administración/backup/restauración, leer file:// o una URI distinta, o suscribirse al recurso
- **THEN** se rechaza sin copiar, leer originales binarios completos, publicar destinos, abrir URLs ni habilitar suscripciones; recursos sólo describen el contrato estático

### Requirement: Esquemas de herramientas derivados del contrato

Cada herramienta SHALL publicar inputSchema con raíz object y outputSchema del envelope v1 con su dato específico, conservando DTOs, referencias locales y errores del núcleo. MUST restringir control/index a get en modo lectura y anunciar todas sus acciones autorizadas en escritura. Los esquemas publicados no sustituyen validación real ni autorizan coerciones o campos adicionales.

#### Scenario: Esquemas completos y acciones mixtas
- **WHEN** un cliente valida esquemas de todas las herramientas y de control/index en ambos modos
- **THEN** todos los refs resuelven dentro del documento correspondiente, las raíces son object y sólo se aceptan las acciones y campos del catálogo permitidos por la política

#### Scenario: Entrada inválida sin efectos
- **WHEN** se envían campos desconocidos, argumentos no objeto, revisión omitida, cursor ajeno, fórmula/documento inválido o JSON por encima de los límites del núcleo
- **THEN** se conserva el rechazo de contrato o dominio correspondiente sin sustituir valores, ejecutar SQL/shell, reclamar trabajos ni confirmar mutaciones parciales

### Requirement: Resultados estructurados y errores distinguibles

Dentro de los límites de entrega, tools/call SHALL devolver el Response v1 íntegro en structuredContent y un único bloque text con su JSON equivalente, con isError acorde a ok. MUST distinguir fallos de protocolo de errores del núcleo, conservar request_id y motivos, y registrar origen confiable mcp en el historial. El ID JSON-RPC no sustituye al identificador generado por el núcleo ni habilita reintentos.

#### Scenario: Éxito y fallo de dominio
- **WHEN** una llamada obtiene una entidad, busca un ID ausente o usa una revisión obsoleta
- **THEN** ambos formatos representan el mismo envelope; isError es false para éxito y true para not_found/revision_conflict, sin perder detalles recuperables ni revelar una traza interna

#### Scenario: Error de protocolo y permiso
- **WHEN** se envía un mensaje inválido, método/nombre desconocido, argumentos MCP mal formados o una acción conocida sin permiso
- **THEN** se responde según JSON-RPC/MCP para los fallos de protocolo; la denegación de una acción conocida usa validation_error con reason permission_denied y no ejecuta el núcleo de dominio

#### Scenario: Origen e identificación confiables
- **WHEN** dos solicitudes usan el mismo ID JSON-RPC o incluyen metadata con un origen distinto y se confirma una mutación autorizada
- **THEN** cada ejecución admitida tiene su request_id propio, el historial usa origen mcp y la metadata no falsifica procedencia ni convierte repetir el mensaje en idempotencia

### Requirement: Ciclo de vida de ambas eras MCP

El servidor MUST interoperar por stdio con MCP 2026-07-28 mediante server/discover y metadata por solicitud, sin exigir initialize moderno, y con legacy 2025-11-25 mediante initialize/initialized. SHALL comprobar versión y estado antes de efectos; sólo anuncia tools y resources implementados, sin sampling, prompts, elicitation, tasks ni suscripciones.

#### Scenario: Cliente moderno sin handshake legacy
- **WHEN** un cliente compatible fija 2026-07-28, descubre y llama herramientas sin initialize
- **THEN** obtiene identidad/capabilities y resultados válidos de esa era, con metadata y resultType wire correctos; no se confunde su modelo SDK con el transcript del protocolo

#### Scenario: Cliente legacy y compatibilidad limitada
- **WHEN** un cliente negocia explícitamente 2025-11-25 y completa initialize/initialized antes de llamar
- **THEN** descubre las mismas herramientas permitidas y obtiene resultados válidos legacy, sin campos modernos obligatorios ni claims de compatibilidad con otras revisiones no probadas

#### Scenario: Versión o estado no admitidos
- **WHEN** una solicitud moderna omite metadata requerida o usa una versión no soportada, o legacy llama herramientas antes de inicializar
- **THEN** se devuelve el error de protocolo correspondiente sin ejecutar la operación ni cambiar permisos o workspace

### Requirement: Descubrimiento paginado estable

tools/list SHALL ordenar por nombre y paginar de ocho en ocho con cursor opaco de hasta 2 KiB ligado a catálogo/política/posición. MUST devolver todos los esquemas sin duplicados ni omisiones y rechazar cursores inválidos o de otra política. El recurso de catálogo y su única entrada resources/list no requieren acceso a datos ni notificaciones de cambios.

#### Scenario: Recorrido de páginas
- **WHEN** se recorren las páginas completas y se repite el listado sin cambiar binario o flags
- **THEN** se obtienen exactamente las herramientas habilitadas, en el mismo orden y con nextCursor sólo cuando existe otra página, sin abrir o mutar el workspace para descubrirlas

#### Scenario: Cursor manipulado o ajeno
- **WHEN** se aporta un cursor excesivo, inválido, fuera de rango o generado para otro catálogo o modo de permisos
- **THEN** se rechaza como parámetro inválido sin devolver una página parcial como completa ni interpretar el cursor como autorización

### Requirement: Framing y admisión con recursos acotados

El transporte SHALL limitar mensajes entrantes a 4 MiB, envelopes de herramienta a 2 MiB y mensajes salientes a 16 MiB, sin truncar JSON. MUST admitir como máximo cuatro solicitudes de protocolo y una ejecución core simultáneas, sin cola core pendiente; el exceso devuelve server_busy antes de ejecutar, o cierra si la salida está saturada. Mantiene caps y espera de escritor de 2 s del núcleo.

#### Scenario: Línea incompleta o excesiva
- **WHEN** llegan mensajes fragmentados, líneas sin terminador, JSON inválido o una línea que supera 4 MiB
- **THEN** el framing recompone sólo mensajes completos dentro del cap; al superar el límite cierra con diagnóstico acotado sin ejecutar el contenido ni acumular una línea ilimitada

#### Scenario: Saturación mientras ejecuta
- **WHEN** una operación core sigue activa y llegan otras llamadas de herramientas o más solicitudes de protocolo que el máximo admitido
- **THEN** el servidor no lanza ni encola más trabajo core, rechaza la sobrecarga de forma identificable y sigue atendiendo cancelación/cierre; nunca libera el cupo por abandonar sólo la espera

#### Scenario: Respuesta que excede el límite
- **WHEN** la ejecución termina pero su envelope supera 2 MiB o su framing superaría 16 MiB
- **THEN** no se envía contenido truncado; se devuelve un error de entrega identificable con request_id y aviso de posibles efectos ya confirmados para una mutación, sin presentarlo como rechazo anterior al commit ni reintentar

### Requirement: Cancelación y cierre sin garantías falsas de rollback

El servidor MUST impedir el despacho de solicitudes canceladas y de nuevas tras EOF, sin respuesta tardía para una ya cancelada. Una ejecución síncrona iniciada conserva su cupo hasta terminar; cancelar su espera o perder stdout no implica rollback ni extraction.cancel. SHALL documentar resultados potencialmente confirmados, reconciliación por lecturas y salida acotada sin reintentos automáticos.

#### Scenario: Cancelación antes o después del despacho
- **WHEN** se cancela una solicitud antes de entrar al núcleo o mientras una mutación síncrona ya está en curso
- **THEN** la primera no ejecuta; la segunda no libera anticipadamente el cupo ni promete reversión, suprime una respuesta aún no enviada y permite consultar después el estado confirmado sin repetir automáticamente la mutación

#### Scenario: EOF o salida perdida
- **WHEN** el cliente cierra stdin o deja de leer stdout con trabajo activo
- **THEN** no se admiten nuevas operaciones; el servidor espera como máximo 30 s antes de terminar y cerrar sus procesos auxiliares propios, sin workers residentes ni promesa de deshacer commits o claims ya confirmados

#### Scenario: Cancelación operativa separada
- **WHEN** una extracción explícita está activa y otro cliente CLI solicita extraction.cancel con revisión vigente
- **THEN** se conservan los fences y estados del núcleo, sin publicar generaciones obsoletas; una notificación MCP por sí sola no se presenta como esa cancelación operativa

### Requirement: Interoperabilidad y consentimiento verificables

La entrega SHALL probar un cliente SDK oficial independiente contra el proceso real en ambas eras, con fixtures sintéticas, ventana cerrada y red denegada. MUST documentar flags, exclusiones, límites y que un cliente externo puede reenviar resultados a su proveedor; no modifica configuraciones personales ni asume que stdio autoriza uso remoto o acceso desde ChatGPT.

#### Scenario: Recorridos con cliente real sin LLM
- **WHEN** se ejecutan discovery, paginación, lectura, denegación, preview/apply y extracción autorizada con el cliente SDK en modern y legacy
- **THEN** se verifican contratos, persistencia, errores y transcript de cada era, sin mocks como única evidencia, sockets externos ni servicios de inferencia

#### Scenario: Configuración y riesgo de retransmisión
- **WHEN** se publica la guía local de uso y su ejemplo de configuración
- **THEN** usa rutas ficticias y lectura por defecto, advierte sobre transmisión por clientes externos y no instala ni habilita cuentas, OAuth, clientes personales, modelos, HTTP o túneles
