# Spec Delta

## ADDED Requirements

### Requirement: Proveedor local de chat configurable e independiente del dominio

El harness SHALL aceptar un proveedor local con endpoint loopback, modelo, timeout, tokens y temperatura. MUST rechazar endpoints remotos, TLS, hostnames, credenciales y respuestas fuera de límite. Cambiar el modelo MUST NOT requerir migración ni modificar entidades; los errores MUST producir ejecución fallida con traza y sin respuesta inventada.

#### Scenario: Proveedor local responde
- **WHEN** el endpoint loopback devuelve una respuesta JSON válida para el modelo configurado
- **THEN** el harness la procesa con el mismo conjunto de herramientas y reglas, sin enviar contenido fuera de la máquina

#### Scenario: Endpoint remoto o caído
- **WHEN** el endpoint no es loopback, no responde o devuelve un error
- **THEN** la ejecución falla con código y traza consultable, no responde con contenido inventado y no modifica datos

### Requirement: Protocolo acotado y herramientas públicas compartidas

El harness SHALL aceptar sólo `tool_call` (una herramienta) o `final` (respuesta y citas). MUST rechazar JSON inválido, campos desconocidos, tipos incorrectos y herramientas fuera de `retrieval.search`, `entity.get`, `database.query`, `changes.preview` y `changes.apply` con `allow_apply` y recibo de preview. MUST NOT existir SQL, shell, archivos, workspace, exportación ni extracción.

#### Scenario: El modelo intenta una herramienta prohibida
- **WHEN** la respuesta nombra `shell`, `sql`, `workspace.backup` o una herramienta desconocida
- **THEN** el paso se rechaza, se registra el error y no se ejecuta ninguna operación

#### Scenario: Preview y apply en la misma ejecución
- **WHEN** el modelo llama `changes.preview` y luego `changes.apply` con el recibo exacto y `allow_apply=true`
- **THEN** el apply usa las reglas atómicas existentes; un recibo alterado o un apply sin preview previo falla sin modificar datos

### Requirement: Respuestas con citas verificables

La respuesta final SHALL incluir citas con job, generación, ordinal y texto. El harness MUST verificar que cada cita corresponda a un hit de `retrieval.search` de la misma ejecución y que el texto aparezca en el fragmento. Sin evidencia MUST exigir `insufficient_evidence=true`; una cita inventada o un texto ausente MUST fallar.

#### Scenario: Cita válida
- **WHEN** la respuesta cita un fragmento recuperado y el texto citado es substring del fragmento
- **THEN** la ejecución termina `succeeded` y devuelve las citas verificadas con su localizador y fuente

#### Scenario: Cita inventada o sin evidencia
- **WHEN** la respuesta cita un hit inexistente, cita texto ausente o afirma una respuesta sin citas sin declarar insuficiencia
- **THEN** la ejecución falla con `citation_unverified` o `citation_required`, conserva la traza y no presenta la respuesta como válida

### Requirement: Límites, cancelación y concurrencia

El harness MUST aplicar límites de pasos, contexto, resultado de herramienta, deadline, tokens y concurrencia. MUST verificar deadline y cancelación antes de cada llamada y herramienta. `agent.cancel` SHALL marcar una ejecución en curso; el bucle MUST detenerse en el siguiente punto de control sin aplicar cambios pendientes.

#### Scenario: Límite de pasos o deadline
- **WHEN** se agota el número de pasos, el presupuesto de contexto o el tiempo total
- **THEN** la ejecución termina `limited` con la razón, no inventa una respuesta y no deja mutaciones parciales

#### Scenario: Cancelación entre procesos
- **WHEN** otra operación marca `agent.cancel` mientras la ejecución espera o procesa un paso
- **THEN** la ejecución observa la marca en el siguiente punto de control y termina `cancelled`, conservando la traza

#### Scenario: Segunda ejecución concurrente
- **WHEN** ya existe una ejecución con reserva vigente en el workspace
- **THEN** una segunda solicitud se rechaza como ocupada sin iniciar otro proveedor ni escribir una traza incompleta

### Requirement: Trazas locales acotadas y recuperables

Cada ejecución SHALL persistir una traza local con ID, estado, proveedor/modelo, pregunta, pasos, respuesta/citas y error. MUST limitar la traza a 1 MiB y no incluir credenciales ni contexto completo. `agent.trace` SHALL consultar por ID o listar recientes; el backup/restore SHALL conservar trazas confirmadas sin reactivar reservas.

#### Scenario: Traza de éxito y de fallo
- **WHEN** se consulta una ejecución terminada o fallida por su ID
- **THEN** se devuelven estado, pasos, error y citas verificadas sin credenciales ni contextos completos

#### Scenario: Backup y restore
- **WHEN** se respalda un workspace con trazas y se restaura en una ruta nueva
- **THEN** las trazas confirmadas se conservan, las ejecuciones en curso no se presentan como activas y el dominio mantiene IDs/revisiones

#### Scenario: Límite de traza
- **WHEN** una salida de modelo o herramienta excede el límite de traza
- **THEN** se rechaza o se registra sólo el resumen permitido sin truncar silenciosamente la evidencia de una cita
