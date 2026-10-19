# Spec Delta

## ADDED Requirements

### Requirement: Selección de adjunto administrado desde la nota

El escritorio SHALL ofrecer preparación separada de Search desde la nota actual, con referencias de archivo/imagen e ingreso explícito de ID administrado. MUST distinguir rótulos del bloque de metadatos de archivo y no importar, consultar, descargar originales ni procesar al montar o cambiar selección.

#### Scenario: Referencias compartidas y selección manual
- **WHEN** una nota referencia dos veces un adjunto o se ingresa un ID de un adjunto sin referencia
- **THEN** se puede elegir el ID sin inventar dueño/nombre y sin duplicar la referencia ni alterar el documento

#### Scenario: Apertura y cambio sin efectos
- **WHEN** se abre preparación, se cambia el adjunto o se alterna Search
- **THEN** no se encola, ejecuta, indexa, repara ni descarga el original y el buffer se conserva

### Requirement: Consulta explícita de procesamiento en escritorio

El panel SHALL consultar trabajos filtrados por adjunto en páginas reemplazables de 20, y consultar por ID el trabajo elegido, control global y selección del índice. MUST mostrar estados, revisión, intentos, soporte, hashes, configuración, generación anterior y errores del núcleo como snapshots separados, sin polling ni nombres inferidos.

#### Scenario: Página de trabajos y trabajo elegido
- **WHEN** se consultan más de veinte trabajos y se elige uno
- **THEN** siguiente reemplaza la página y la inspección muestra identidad/procedencia, soporte y resultado previo del trabajo, control e índice sin afirmar un snapshot conjunto

#### Scenario: Incompatible vacío o error
- **WHEN** no hay trabajos, hay configuración futura o falla una consulta
- **THEN** se distingue vacío de error/incompatibilidad, se conserva razón/código y no se representa información anterior como consulta vigente ni se modifica configuración

### Requirement: Acciones de cola independientes en escritorio

El panel SHALL encolar sólo con adjunto y extractor elegidos, ejecutar sólo el trabajo elegido y ofrecer pausa/reanudación global, cancelación y reencolado como acciones independientes. MUST reutilizar operaciones Rust y sus revisiones observadas, mostrar deduplicación/outcome/razón/recuperaciones y no ejecutar después de encolar, reanudar o reintentar.

#### Scenario: Extracción offline paso a paso
- **WHEN** se encola UTF-8, PDF o XLSX admitido, se reanuda y luego se ejecuta explícitamente su ID
- **THEN** cada paso tiene su propio resultado y sólo ejecutar aumenta intentos; el original y las entidades permanecen iguales y la generación no se selecciona para búsqueda automáticamente

#### Scenario: Pausa cancelación y reencolado
- **WHEN** se pausa/reanuda, cancela o reencola con revisión observada
- **THEN** se aplica sólo esa operación, resume no ejecuta y retry conserva generación previa; run informa que puede recuperar intentos vencidos según el núcleo

#### Scenario: Conflicto y resultado incierto
- **WHEN** otro cliente cambia una revisión o falla el transporte de una escritura
- **THEN** no hay rebase/repetición automática; se exige consultar antes de una nueva decisión y se aclara que el estado puede haber cambiado

### Requirement: Selección de generación observada en escritorio

El panel SHALL permitir seleccionar explícitamente una generación compatible observada, usando job_id, expected_generation_id y revisión observada del índice. MUST mostrar identidad/cobertura/selección anterior y conservar validación completa, cuotas y errores Rust, sin reset, parsing ni encadenamiento con búsqueda.

#### Scenario: Selección compatible sin ejecutar
- **WHEN** el usuario consulta un trabajo con generación compatible y la selecciona
- **THEN** put usa ambos CAS, muestra changed/revisión/entrada y no ejecuta helpers ni busca; cero fragmentos se distingue de ausencia de generación

#### Scenario: Generación o índice concurrentes
- **WHEN** otro cliente reemplaza la generación o cambia el índice antes de put
- **THEN** el conflicto no selecciona una versión distinta ni reintenta y pide nueva consulta explícita

### Requirement: Lifetime y cupo físico de preparación

El escritorio MUST descartar resultados tardíos al cambiar adjunto, trabajo, nota o workspace y acotar a una operación física de preparación simultánea entre montajes. SHALL mantener el cupo hasta completar la llamada iniciada, cancelar sólo esperas en cola y nunca convertir desmontar en cancelación del worker ni repetir una mutación.

#### Scenario: Lectura tardía y cambio de contexto
- **WHEN** una consulta finaliza después de cambiar identidad o desmontar el panel
- **THEN** no publica datos/errores viejos ni modifica el buffer y conserva el cupo físico hasta terminar

#### Scenario: Escritura tardía o encolada
- **WHEN** se abandona el panel durante una escritura o una espera todavía no iniciada
- **THEN** la escritura iniciada puede terminar sin repeticiones, la espera cancelada no ejecuta y al volver se consulta explícitamente el estado real

### Requirement: Preparación legible y acotada sin alterar edición

El panel SHALL presentar controles por teclado, foco visible, texto no confiable escapado y límites reales UTF-8/PDF/XLSX/cola/índice en ambos temas. MUST conservar buffers, guardias y contratos sin almacenar contenido en el navegador, activar remotos, modificar Home ni presentar pruebas headless como aceptación nativa.

#### Scenario: Temas teclado y texto hostil
- **WHEN** se usa el flujo por teclado en ambos temas con rótulos/configuración/errores que parecen HTML o instrucciones
- **THEN** se muestran como datos, sin navegación remota/ejecución y con controles accesibles en viewport estrecho

#### Scenario: Buffer y navegación protegida
- **WHEN** se opera preparación con nota pendiente y se intenta navegar con cancelación o fallo de guardado
- **THEN** se conservan documento, revisión capturada y guardias; las mutaciones derivadas no guardan ni reemplazan la nota
