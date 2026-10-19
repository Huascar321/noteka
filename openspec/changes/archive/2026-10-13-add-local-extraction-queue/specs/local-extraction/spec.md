# Spec Delta

## Purpose

Procesar originales administrados mediante trabajos locales separados de la edición, conservando derivados regenerables con procedencia y sin reemplazar el contenido de usuario.

## ADDED Requirements

### Requirement: Selección local explícita y ejecución separada

El sistema SHALL encolar extracción sólo por selección explícita de un adjunto administrado y un extractor soportado. MUST separar encolado de ejecución, comenzar pausado y funcionar offline sin modelo, cuenta ni ventana. Importar, abrir el workspace o consultar trabajos MUST NOT ejecutar extracción ni iniciar un proceso residente.

#### Scenario: Importación sin procesamiento implícito
- **WHEN** se importa un adjunto, se abre el workspace y se consulta la cola sin solicitar extracción
- **THEN** no aparece un trabajo nuevo ni se extrae contenido o inicia un worker

#### Scenario: Trabajo seleccionado sin red
- **WHEN** un cliente encola un adjunto elegible, habilita la cola y solicita ejecutar con red denegada
- **THEN** el trabajo se procesa localmente sin iniciar la UI ni usar modelos o proveedores

### Requirement: Identidad incremental y trazabilidad del trabajo

Cada trabajo SHALL registrar fuente/hash, extractor/configuración versionados, estado, revisión, intentos, timestamps y error identificable; no tendrá perfil de modelo. MUST deduplicar una fuente/configuración iguales sin volver a ejecutar ni perder resultados. Estados conocidos: queued, running, succeeded, failed, cancelled y stale.

#### Scenario: Encolado repetido
- **WHEN** se encola dos veces la misma fuente y configuración, incluso desde clientes concurrentes
- **THEN** se devuelve un único ID de trabajo sin duplicar intentos, resultados o revisiones

#### Scenario: Regeneración explícita
- **WHEN** se reintenta un trabajo terminado con revisión vigente
- **THEN** queda queued con el mismo ID, conserva el resultado previo hasta otra publicación válida y sólo incrementa intentos al comenzar otra ejecución

#### Scenario: Configuración persistida futura
- **WHEN** un trabajo contiene una versión/configuración que el cliente no soporta
- **THEN** la consulta identifica su incompatibilidad conservando los datos crudos; ejecutarlo o reintentarlo no los transforma ni descarta

### Requirement: Extracción UTF-8 literal y localizada

El primer extractor SHALL admitir originales `.txt` y `.md` de hasta 1 MiB, con UTF-8 válido y sin NUL, excluyendo medios binarios reconocidos. MUST conservar bytes/texto literal, producir fragmentos sin cortar caracteres y exponer offsets y líneas del original. No interpreta marcado, URLs, código o instrucciones del contenido.

#### Scenario: Unicode y saltos de línea
- **WHEN** se procesa texto con caracteres multibyte, BOM, CRLF y una línea mayor que un fragmento
- **THEN** los fragmentos reconstruyen exactamente el texto y sus rangos de bytes/líneas apuntan al original sin normalizarlo

#### Scenario: Contenido que parece una instrucción
- **WHEN** el texto contiene HTML, enlaces remotos, comandos o instrucciones para un agente
- **THEN** permanece texto literal; no se evalúa, ejecuta ni descarga contenido

#### Scenario: Formato o codificación no admitidos
- **WHEN** se solicita PDF, imagen, Excel, un nombre no elegible, UTF-8 inválido o texto con NUL
- **THEN** se informa el motivo explícito sin OCR, conversión con pérdida, fragmentos parciales ni modificación del original

#### Scenario: Original vacío
- **WHEN** se procesa un archivo elegible de cero bytes
- **THEN** termina succeeded con cero fragmentos y cobertura vacía explícita, no como un fallo ni texto inventado

### Requirement: Ejecución cooperativa con ownership acotado

La ejecución SHALL procesar como máximo un trabajo por llamada, liberar el lock de escritura durante lectura/extracción y utilizar ownership exclusivo, scope y vencimiento verificables. MUST acotar contención y ejecución; un worker vencido, cancelado o de otra ruta no puede publicar. Recuperar un trabajo interrumpido no lo reintenta automáticamente.

#### Scenario: Dos ejecutores independientes
- **WHEN** dos procesos intentan reclamar trabajo simultáneamente
- **THEN** sólo uno obtiene ownership activo; el otro no ejecuta ni publica el mismo trabajo, y otro cliente puede seguir editando mientras se extrae

#### Scenario: Worker interrumpido y publicación tardía
- **WHEN** un worker muere o vence y una ejecución explícita recupera su estado
- **THEN** queda failed con motivo recuperable; un token anterior no puede confirmar resultados y se requiere retry explícito

#### Scenario: Deadline o contención
- **WHEN** se agota la ejecución cooperativa o otro escritor supera 2.000 ms de bloqueo
- **THEN** termina con error recuperable, no se publica una generación parcial y no se ignoran límites para confirmar

### Requirement: Pausa cancelación y reintento con revisión

El sistema SHALL permitir pausar/reanudar la cola y cancelar o reintentar trabajos con revisión esperada. MUST rechazar revisiones obsoletas sin rebase. Pausar impide nuevos claims y detiene cooperativamente el activo sin perder su resultado previo; cancelar invalida ownership y retry no crea resultados hasta ejecutar nuevamente.

#### Scenario: Cola pausada
- **WHEN** se pausa con revisión vigente y luego se solicita ejecución
- **THEN** no se reclama un trabajo nuevo; un activo observa la pausa, se retira sin publicar y queda queued para una ejecución explícita posterior

#### Scenario: Cancelación durante extracción
- **WHEN** otro cliente cancela un queued o running con revisión vigente
- **THEN** queda cancelled, el worker anterior no publica y los originales y una generación previa válida permanecen disponibles

#### Scenario: Revisión antigua
- **WHEN** otro cliente ya cambió la revisión del trabajo o del control de cola
- **THEN** la modificación con revisión vieja devuelve revision_conflict sin sobrescritura ni reintento automático

#### Scenario: Reintento luego de un fallo
- **WHEN** el cliente reintenta failed, cancelled, stale o succeeded con revisión vigente
- **THEN** queda queued; un running no admite retry y una cancelación ya vigente no genera una nueva revisión

### Requirement: Publicación atómica de una generación

El sistema MUST confirmar juntos fragmentos, cobertura, generación y estado succeeded, después de revalidar fuente, configuración, revisión y ownership. Cualquier fallo SHALL impedir una publicación parcial y conservar una generación previa válida. Los derivados no modifican entidades, revisiones de notas, referencias, FTS textual ni historial de entidades.

#### Scenario: Generación completa observable
- **WHEN** termina un trabajo válido mientras otra conexión consulta sus fragmentos
- **THEN** el lector ve una generación completa anterior o la nueva, nunca fragmentos mezclados con un estado succeeded anticipado

#### Scenario: Falla antes de publicar
- **WHEN** falla un fragmento, validación o persistencia antes del commit
- **THEN** no se publican fragmentos parciales ni un éxito; el error/estado recuperable puede consultarse sin alterar el original

#### Scenario: Regeneración fallida
- **WHEN** falla una nueva ejecución de un trabajo que tenía resultados válidos
- **THEN** no se sustituye la generación previa por resultados incompletos; su antigüedad y condición de resultado previo quedan visibles

### Requirement: Lectura verificable y paginada de derivados

La lectura SHALL devolver fuente/hash, extractor/configuración, generación y localizador por fragmento, con paginación limitada y cursor ligado a workspace/ruta/trabajo/generación. MUST revalidar el original antes de presentar texto como vigente; una fuente alterada se identifica como stale sin reescribirla. No atribuye el fragmento a una nota que ya no lo referencia.

#### Scenario: Fuente alterada o ausente
- **WHEN** el original desaparece, cambia su hash/tamaño o se sustituye por una ruta no admitida
- **THEN** no se devuelve texto derivado como vigente y una ejecución detecta stale o fallo de almacenamiento sin reparar el original

#### Scenario: Página y cambio de generación
- **WHEN** se consulta más de una página y una regeneración publica antes de continuar
- **THEN** el cursor antiguo se rechaza; no mezcla generaciones ni afirma cobertura completa de una página parcial

#### Scenario: Ruta ajena y lecturas sin efectos
- **WHEN** se usa un cursor de otra ruta/trabajo o se consultan metadatos/fragmentos sin ejecución
- **THEN** el cursor ajeno falla y la consulta válida no cambia estado, intentos, control, entidades ni historial

### Requirement: Presupuestos explícitos y errores útiles

La cola SHALL publicar y verificar límites de fuentes, jobs, fragmentos, almacenamiento derivado, páginas, respuestas y ejecución. MUST rechazar exceso antes de publicar, sin truncar un éxito como completo ni exponer contenido íntegro en errores. No sube los límites vigentes de importación, transporte o batches para acomodar derivados.

#### Scenario: Fronteras y backpressure
- **WHEN** se alcanza un límite de 1 MiB de fuente, 1.000 queued/running, 10.000 trabajos o 32 MiB de texto derivado retenido
- **THEN** la frontera permitida se acepta y el exceso produce un motivo explícito, sin crear trabajos o resultados parciales

#### Scenario: Respuesta limitada
- **WHEN** una lectura pide un límite inválido o una respuesta completa supera 2 MiB
- **THEN** se rechaza o pagina conforme al contrato, sin ocultar cobertura, cursores, texto omitido o errores
