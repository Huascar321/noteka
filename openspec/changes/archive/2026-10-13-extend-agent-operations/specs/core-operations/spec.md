# Spec Delta

## MODIFIED Requirements

### Requirement: Contrato compartido y versionado

El sistema SHALL exponer un catálogo versionado con esquemas de entrada, salida y errores para todas las operaciones implementadas, incluidas colecciones, propiedades, fórmulas, vistas, plantillas, consultas y lotes. Interfaz y CLI MUST aplicar las mismas reglas; el catálogo MUST identificar operaciones compatibles con lotes, referencias admitidas y límites, sin anunciar funciones futuras como disponibles.

#### Scenario: Clientes equivalentes

- **WHEN** la interfaz y el CLI solicitan la misma actualización válida en workspaces equivalentes
- **THEN** obtienen estados y resultados de dominio equivalentes

#### Scenario: Descubrimiento sin interfaz

- **WHEN** se solicita el catálogo de operaciones desde el CLI
- **THEN** se obtienen versión y esquemas sin abrir la ventana de escritorio

#### Scenario: Catálogo y transportes completos

- **WHEN** se compara el catálogo con las operaciones públicas y los subcomandos CLI
- **THEN** cada operación tiene entrada, salida y errores documentados; changes.preview/apply y su alcance limitado son descubribles sin alterar solicitudes anteriores

#### Scenario: Paridad del lote

- **WHEN** núcleo y CLI ejecutan el mismo plan y recibo sobre copias equivalentes con el mismo scope
- **THEN** aplican iguales validaciones y producen estados, IDs y cálculos equivalentes, salvo origen, identificador de solicitud y timestamps de ejecución documentados

### Requirement: CLI operativo con ventana cerrada

El CLI SHALL permitir crear/abrir workspace, consultar catálogo, operar entidades/documentos/relaciones, colecciones, propiedades, registros, fórmulas, vistas, plantillas y consultas, archivar/restaurar entidades y realizar backup/restauración sin iniciar la interfaz gráfica. SHALL ofrecer changes preview y changes apply con entrada/salida JSON y las mismas validaciones del núcleo.

#### Scenario: Nota creada por CLI

- **WHEN** se crea una nota desde el CLI con la ventana cerrada y luego se abre ese workspace en escritorio
- **THEN** la interfaz puede consultar la misma nota y revisión

#### Scenario: Lote sin escritorio ni red

- **WHEN** se previsualiza y aplica un plan válido por CLI sin red, LLM ni ventana de escritorio
- **THEN** stdout contiene un único resultado JSON por llamada y los errores conservan los códigos de salida documentados, sin iniciar procesos gráficos

## ADDED Requirements

### Requirement: Planes de cambios acotados y referencias explícitas

El núcleo SHALL aceptar planes versionados de hasta 100 mutaciones permitidas y 10 comprobaciones finales, dentro de 2 MiB y 512 bindings. MUST resolver únicamente referencias explícitas a exports de pasos anteriores en slots de ID/revisión admitidos y claves de propiedades, sin sustituir texto literal ni relajar los DTOs finales. MUST rechazar campos, versiones y operaciones desconocidos o anidación de lotes sin efectos confirmados.

#### Scenario: Esquema y registros encadenados

- **WHEN** un plan referencia colección, propiedad, revisión, registro y vista producidos por pasos anteriores
- **THEN** se resuelven los IDs/revisiones y las claves de valores sin exigir IDs preexistentes ni omitir validaciones de dominio

#### Scenario: Referencia inválida o clave duplicada

- **WHEN** una referencia apunta al futuro, a un paso/export inexistente, a un slot no permitido o renombra dos claves al mismo ID
- **THEN** se devuelve validation_error con paso y binding identificables y ningún efecto confirmado

#### Scenario: Límites y operaciones excluidas

- **WHEN** el plan excede un límite, contiene un lote anidado, una operación no admitida o una acción de filesystem como attachment.import
- **THEN** se rechaza completo sin ejecutar parcialmente sus pasos, migrar ni copiar originales

#### Scenario: Texto que parece referencia

- **WHEN** títulos, texto de bloques o valores textuales contienen cadenas parecidas a aliases, IDs o referencias
- **THEN** permanecen literales; sólo los bindings declarados en slots admitidos pueden cambiar entradas del plan

### Requirement: Preview aislado del workspace

changes.preview SHALL evaluar el plan y sus comprobaciones sobre un snapshot consistente, sin confirmar contenido, revisiones, relaciones, valores, FTS ni historial en el original. MUST conservar adjuntos y no crear/migrar/restaurar workspaces, generar backups ni exportar archivos. SHALL rechazar de forma recuperable workspaces no actuales, esquemas SQL no reconocidos para lotes y snapshots fuera de los límites publicados.

#### Scenario: Preview exitoso y fallido sin mutaciones

- **WHEN** se previsualizan un plan válido y otro que falla después de varias mutaciones simuladas
- **THEN** entidades, documentos, colecciones, valores, revisiones, FTS, historial y hashes de originales permanecen iguales en el workspace

#### Scenario: Snapshot con WAL y escritor concurrente

- **WHEN** hay datos confirmados en WAL y un segundo cliente confirma una escritura durante la obtención del preview
- **THEN** el resultado corresponde íntegramente a un único estado confirmado, incluye su huella y no mezcla lecturas de estados distintos ni ignora el WAL

#### Scenario: Versión o estructura no admitida

- **WHEN** se solicita preview sobre un workspace ausente, anterior, futuro, con extensión SQL no admitida o por encima del límite de snapshot
- **THEN** se identifica el motivo sin actualizar formato, descartar datos, crear workspace o publicar un preview incompleto

### Requirement: Diff de dominio e identidades estables del plan

El preview SHALL devolver resultados por paso, comprobaciones, diff antes/después y un recibo versionado que vincule scope, plan, estado base y efecto esperado. MUST conservar IDs de dominio entre preview y apply del mismo plan, incluidos desempates de cálculos por ID; distinguir timestamps simulados de los de confirmación y fallar antes del commit si la respuesta supera el límite publicado, sin truncar un diff como completo.

#### Scenario: Crear y editar con diff verificable

- **WHEN** un plan crea una colección y modifica una nota existente
- **THEN** se distinguen altas de cambios, se identifican originales/revisiones afectados y el estado confirmado coincide con el efecto aceptado, excepto metadatos de ejecución explícitamente excluidos

#### Scenario: Reparto empatado y tamaño excesivo

- **WHEN** se repite preview del mismo plan sobre el mismo estado con un reparto empatado o un diff supera el límite
- **THEN** los IDs y el desempate son reproducibles; el exceso produce un error explícito y no una confirmación o aprobación de contenido omitido

### Requirement: Comprobaciones finales sobre el estado propuesto

Los planes SHALL permitir consultas database.query después de todas sus mutaciones usando los mismos filtros, fórmulas y snapshot. Las comprobaciones MUST rechazar errores de cálculo y cobertura incompleta por defecto; sólo flags explícitos pueden permitirlos, conservando errores/cursores visibles. Un fallo de comprobación MUST impedir la aplicación completa, sin convertir vacío o error en cero válido.

#### Scenario: Cálculo inválido al final

- **WHEN** una comprobación final detecta división por cero u otro error derivado, aunque los pasos de escritura sean válidos
- **THEN** preview/apply rechazan el plan con código de cálculo identificable y no dejan mutaciones confirmadas

#### Scenario: Página parcial declarada

- **WHEN** una comprobación tiene más resultados que su página
- **THEN** el modo completo predeterminado rechaza el plan; una comprobación explícitamente parcial conserva cursor y cobertura sin afirmar que verificó todos los registros

### Requirement: Aplicación atómica de planes validados

changes.apply MUST revalidar y ejecutar todas las mutaciones y comprobaciones en una única transacción sobre el workspace original. SHALL confirmar juntos contenido, esquema, membresías, valores, referencias, revisiones, FTS e historial aplicables, o revertir todo ante fallo de paso, comprobación, comparación, tamaño o persistencia. Las mutaciones de entidad SHALL compartir el request_id y origen de apply en el historial existente.

#### Scenario: Fallo intermedio y de persistencia

- **WHEN** un paso posterior es inválido o falla almacenamiento antes del commit después de crear colecciones, páginas y valores
- **THEN** no queda ninguno de esos efectos, incremento de revisión, fila FTS ni cambio de historial confirmado y se identifica el fallo sin presentar resultados parciales como aplicados

#### Scenario: Commit visible y agrupado

- **WHEN** apply confirma un plan válido con varias mutaciones de entidad y esquema
- **THEN** otra conexión observa el estado anterior o el completo confirmado; las mutaciones de entidad tienen el request_id/origen compartido y las de esquema están identificadas en el resultado, sin inventar historial persistente de esquema

#### Scenario: Misma validación que operaciones individuales

- **WHEN** se ejecutan valores, documentos, fórmulas, vistas o revisiones inválidos por operación individual y por lote
- **THEN** el lote conserva los errores/reglas existentes y agrega contexto de paso, sin aceptar tipos o referencias que la operación individual rechaza

### Requirement: Confirmación vinculada al preview y concurrencia segura

apply MUST exigir el plan y recibo esperado, comprobar identidad/ruta, estado base y efecto antes del commit y mantener revisiones esperadas de cada operación. SHALL rechazar preview obsoleto con revision_conflict y contención con workspace_busy dentro del timeout publicado. MUST rechazar recibos incompatibles, planes alterados y divergencias, sin migrar, rebasar revisiones, confirmar parcialmente ni reintentar escrituras automáticamente.

#### Scenario: Escritura entre preview y apply

- **WHEN** otro cliente cambia datos originales después del preview, incluso fuera de los objetos escritos por el lote
- **THEN** apply rechaza el recibo obsoleto, conserva el cambio ajeno y no confirma pasos del lote; un preview nuevo es necesario

#### Scenario: Revisión vieja o escritor ocupado

- **WHEN** un plan incluye una revisión esperada obsoleta o otro escritor mantiene el bloqueo más allá del timeout
- **THEN** termina con revision_conflict o workspace_busy, respectivamente, sin sobrescritura ni reejecución automática

#### Scenario: Recibo ajeno o efecto alterado

- **WHEN** se omite el recibo, pertenece a otra ruta/workspace, cambia el plan, usa versión inválida o no coincide el efecto esperado
- **THEN** la aplicación completa es rechazada con detalles recuperables y ningún efecto confirmado; la huella no se presenta como token de permiso o autenticación

#### Scenario: Respuesta perdida después del commit

- **WHEN** el cliente pierde la respuesta de un apply confirmado e intenta usar el mismo recibo para repetir una mutación efectiva
- **THEN** el estado base ya no coincide y el núcleo rechaza la repetición; el cliente puede consultar IDs originales para reconciliar, sin promesa de idempotencia persistente

### Requirement: Conservación y recuperación al agrupar operaciones

Los lotes MUST conservar campos, documentos, ASTs y valores futuros no modificados, y rechazar reemplazos de formatos que el núcleo no pueda editar. SHALL operar sólo metadatos/referencias de adjuntos ya existentes, sin modificar originales ni prometer atomicidad del filesystem. Backup/restauración SHALL conservar todo efecto confirmado sin introducir un nuevo formato persistente.

#### Scenario: Formatos futuros y referencia existente

- **WHEN** el workspace incluye documentos v1/v2/v3/futuros, payloads o ASTs desconocidos y un original referenciado
- **THEN** el preview y los planes rechazados conservan bytes/IDs; un lote válido no reescribe datos omitidos ni el original y un reemplazo no soportado aborta completo

#### Scenario: Reapertura y recuperación del lote

- **WHEN** se aplica un plan, se reabre y se restaura su backup en una nueva ruta
- **THEN** IDs, valores exactos, documentos, cálculos, vistas e historial confirmados se conservan; el recibo de la ruta original no autoriza apply en la restaurada

### Requirement: Construcción determinista del sistema de suscripción por CLI

El recorrido sintético SHALL construir mediante operaciones públicas y un lote previsualizado dos colecciones relacionadas, costo BOB 145,69, seis asientos, cinco miembros con pesos 2/1/1/1/1, pago, fórmula exacta y vista Pendientes incrustada en una página. MUST comprobar resultados antes de apply, conservar el total de 14.569 centavos y recalcular cambios sin LLM, red, SQL del cliente ni una pantalla financiera específica.

#### Scenario: Preview y construcción completa

- **WHEN** se ejecuta el recorrido en un workspace sintético actual con la ventana cerrada
- **THEN** preview muestra 48,57 y cuatro 24,28 sin persistir colecciones; apply confirma una sola estructura con los mismos IDs, seis asientos, total 145,69 y referencia de página a la vista original

#### Scenario: Fallo sin estructura parcial

- **WHEN** una variante del lote de suscripción contiene una relación o fórmula inválida en un paso posterior
- **THEN** no queda colección, propiedad, miembro, vista o página parcialmente creado

#### Scenario: Costo y pagos actualizados

- **WHEN** se cambia el costo por operación pública y se marca un miembro como pagado con revisión vigente
- **THEN** la consulta recalcula importes exactos, Pendientes excluye al pagado y la página sigue apuntando a la misma colección/vista sin copiar registros ni inventar reglas de vacantes o períodos
