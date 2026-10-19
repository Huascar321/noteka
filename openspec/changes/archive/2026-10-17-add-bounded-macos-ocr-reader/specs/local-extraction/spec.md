# Spec Delta

## ADDED Requirements

### Requirement: Lector OCR local independiente acotado

Un lector descartable SHALL reconocer explícitamente una imagen PNG/JPEG hasta 1 MiB, dimensiones positivas de hasta 4.096 por eje y cuatro millones de píxeles. MUST usar sólo el perfil macOS revision 3/fast/es-ES/en-US sin corrección, detección automática, fallback, descargas o proveedor remoto; otro formato o perfil no disponible falla explícitamente.

#### Scenario: Imagen admitida sin red
- **WHEN** se envía una imagen sintética PNG o JPEG admitida con el perfil local disponible y red denegada
- **THEN** devuelve regiones reconocidas y procedencia del motor sin instalar pesos, abrir ventanas ni contactar un proveedor remoto

#### Scenario: Formato dimensiones o secuencia no admitidos
- **WHEN** se envía contenido inválido, otro formato, más de una imagen, dimensiones o bytes excesivos
- **THEN** rechaza la lectura completa con razón tipada, sin resultado parcial o conversión automática

#### Scenario: Perfil no disponible
- **WHEN** el sistema no dispone de la revisión o idiomas del perfil solicitado
- **THEN** informa indisponibilidad sin elegir otro modelo, perfil accurate, idioma o proveedor

### Requirement: Contrato OCR privado validado en Rust

El consumidor Rust MUST validar framing/versiones, hash fuente esperado, motor, perfil, metadatos, regiones y cuotas de la respuesta antes de admitirla. SHALL rechazar campos desconocidos, identidad incoherente, texto inválido y geometría/confianza no finitas o fuera de rango; los errores sólo contienen razones tipadas sin original o contenido reconocido.

#### Scenario: Respuesta y origen ligados
- **WHEN** se recibe una respuesta válida ligada al SHA-256 de los bytes enviados
- **THEN** Rust admite sus metadatos y regiones completas sin confiar sólo en un exit code exitoso

#### Scenario: Respuesta adulterada
- **WHEN** cambian hash, versión, perfil, campos, ordinales, conteos, dimensiones o geometría/confianza
- **THEN** la validación rechaza toda la respuesta, sin truncar, inferir campos o conservar texto parcial

#### Scenario: Framing o error inválidos
- **WHEN** entrada/salida tienen framing incompleto, bytes adicionales, JSON inválido o error ajeno al protocolo
- **THEN** no se admite contenido y se informa fallo de protocolo sin reflejar texto de la fuente en el error

### Requirement: Evidencia OCR inferida por región

El lector SHALL conservar texto reconocido, orden de observaciones, confianza y rectángulos normalizados con origen inferior izquierdo del marco orientado, junto con dimensiones/orientación fuente y motor/OS. MUST identificarlo como inferencia falible, sin prometer orden visual, texto literal, tablas o cobertura de todos los caracteres; cero regiones no demuestra imagen vacía.

#### Scenario: Texto orientación y geometría
- **WHEN** se reconoce texto con acentos, saltos o contenido parecido a instrucciones y orientación declarada
- **THEN** se devuelve como datos con localizadores del marco orientado y campos completos, sin ejecutar, corregir ni convertirlo en instrucciones

#### Scenario: Imagen sin detecciones
- **WHEN** el motor no devuelve regiones para una imagen válida
- **THEN** el resultado válido conserva dimensiones/procedencia y limitaciones, sin afirmar ausencia universal de texto o información visual

### Requirement: Cuotas OCR sin truncamiento

El resultado OCR SHALL tener hasta 512 regiones, 131.072 bytes UTF-8 por región y 1 MiB de representación JSON; stdout y stderr se acotan a 8 MiB y 8 KiB. MUST rechazar exceso completo sin purga, reducción de imagen, omisión de regiones o corte de texto. Los límites de fuente, derivados e índices existentes permanecen intactos.

#### Scenario: Frontera y exceso del resultado
- **WHEN** conteos/texto/representación alcanzan la frontera válida o la exceden
- **THEN** la frontera pasa validación y el exceso rechaza la respuesta completa sin un éxito parcial

#### Scenario: Salida excesiva o desconocida
- **WHEN** un helper produce stdout/stderr por encima del límite o una respuesta no reconocida
- **THEN** el consumidor de prueba la rechaza y termina/recolecta al proceso sin bloquear pipes o aceptar texto parcial

### Requirement: Contención OCR descartable y presupuesto

El helper MUST desactivar core dumps, limitar CPU a cuatro segundos y vida a cinco desde su inicio aun bloqueado en pipes; el presupuesto recibido sólo se acorta. SHALL terminar si el máximo RSS observado supera 128 MiB, documentando que el muestreo no equivale al allocator Rust ni a un límite duro de RSS/GPU; PDF/XLSX conservan su contención previa.

#### Scenario: Deadline bloqueo y presupuesto menor
- **WHEN** la entrada/salida se bloquea, el trabajo supera su presupuesto o recibe un plazo menor
- **THEN** el watchdog termina el proceso dentro de la tolerancia de prueba sin extender cinco segundos; el consumidor lo recolecta y no deja trabajo residente

#### Scenario: CPU memoria y fallo interno
- **WHEN** ocurre exceso de CPU/RSS observado o un fallo interno
- **THEN** el helper termina sin core dump ni texto en diagnósticos y el consumidor distingue el fallo del resultado válido

### Requirement: OCR aislado de fuentes y operaciones persistentes

El lector independiente MUST recibir bytes y no abrir workspaces ni recibir rutas originales, credenciales o configuración de proveedores. SHALL conservar SQLite v12/API v1/52, MCP, operaciones, fuentes, cuotas/validación/gates y Home, sin integrar procesamiento, indexación o reparación al montar o buscar; la integración persistente exige otro cambio verificado.

#### Scenario: Lectura sin workspace
- **WHEN** se ejecuta el lector sobre bytes sintéticos sin cuenta ni workspace
- **THEN** funciona de forma independiente sin crear tablas, jobs, índices o modificaciones de originales

#### Scenario: Regresión de contratos y Home
- **WHEN** se incorporan el protocolo/helper y se verifican las operaciones previas
- **THEN** catálogo, formatos persistentes, whitelist batch, lectores, búsqueda y guardias/Home conservan sus contratos y evidencia delimitada sin declarar OCR integrado ni toda 5a completa
