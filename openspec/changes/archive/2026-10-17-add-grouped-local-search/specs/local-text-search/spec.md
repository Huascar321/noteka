# Spec Delta

## MODIFIED Requirements

### Requirement: Búsqueda de escritorio offline

La interfaz SHALL buscar notas/entidades y adjuntos en ámbitos explícitos separados y ofrecer consulta conjunta por grupos independientes, sin fusionar scores ni cursores. MUST abrir entidades con su localizador, conservar buffers pendientes al navegar y mostrar evidencia y límites de adjuntos sin servicios remotos ni extracción o indexación implícitas.

#### Scenario: Buscar y abrir sin red
- **WHEN** se busca una nota enriquecida sin red ni modelos
- **THEN** se abre la entidad original con referencia de bloque/revisión y se conserva la protección de edición pendiente

#### Scenario: Cambiar de ámbito sin cambiar el editor
- **WHEN** se cambia entre notas/entidades y adjuntos mientras hay un buffer pendiente
- **THEN** el buffer permanece intacto, los filtros de tipo/archivo/contexto sólo se aplican a entidades y cada ámbito usa su propio contrato sin mezclar resultados ni cursores

#### Scenario: Consultar adjuntos no es procesarlos
- **WHEN** se entra al ámbito de adjuntos, se busca o se inspecciona un hit
- **THEN** no se importa, encola, ejecuta, selecciona, resetea ni reconstruye contenido o índices; los adjuntos sin selección explícita no se incorporan a la búsqueda

#### Scenario: Consulta local conjunta explícita
- **WHEN** se solicita una consulta conjunta de texto
- **THEN** la misma entrada consulta entidades activas y adjuntos seleccionados, con dos grupos/coberturas independientes sin total o snapshot global inventados; los ámbitos individuales conservan sus filtros

## ADDED Requirements

### Requirement: Páginas independientes de consulta conjunta

La consulta conjunta SHALL mostrar hasta veinte hits por grupo con siguiente/inicio independientes, respetando contratos y orden del núcleo. MUST enviar la consulta literal sin truncarla, retirar ambos grupos al editar y no consultar al montar/cambiar texto. El alcance visible identifica entidades activas y generaciones seleccionadas, sin filtros cruzados ni suma de scores/conteos de cobertura como hits.

#### Scenario: Paginación por grupo
- **WHEN** hay más de veinte hits de notas y adjuntos y se pagina uno de los grupos
- **THEN** se reemplaza sólo su página, conserva el otro grupo y usa únicamente el cursor de su operación; inicio es explícito y no acumula todos los resultados

#### Scenario: Consulta excesiva y alcance
- **WHEN** se ingresa texto que excede un límite Rust, se edita la consulta o hay generaciones sin seleccionar
- **THEN** no se recorta la entrada para hacerla válida, los errores se distinguen por grupo y editar retira páginas/cursores sin consultar; el alcance no incorpora originales sin selección

### Requirement: Errores parciales y evidencia de consulta conjunta

Cada grupo SHALL mostrar carga, vacío, error y página propios; conservar fuente/revisión/bloque de notas y texto/procedencia/localizador/cobertura de adjuntos. MUST presentar contenido escapado y resultados anteriores como tales. Un fallo o cursor vencido retira sólo evidencia de ese grupo y no convierte la consulta parcial en ausencia universal ni repara/reinicia automáticamente.

#### Scenario: Fallo de una fuente
- **WHEN** falla la consulta de adjuntos o notas mientras el otro grupo responde
- **THEN** se conserva el grupo válido y se identifica el grupo fallido con código/razón y reintento explícito de esa lectura, sin repetir la otra consulta ni hacer writes

#### Scenario: Índice cambiado entre páginas
- **WHEN** un cambio del índice vence el cursor de adjuntos después de obtener ambos grupos
- **THEN** sólo adjuntos retira página/cursor y muestra conflicto, mientras notas conserva su snapshot sin etiquetarse como snapshot conjunto vigente

### Requirement: Lifetime y apertura protegida en consulta conjunta

La interfaz MUST descartar éxito/error tardíos de consultas invalidadas al cambiar texto/ámbito/sección/workspace o desmontar, manteniendo cupos físicos hasta terminar IPC y evitando esperas canceladas en cola. SHALL conservar buffers/guardias, apertura de notas por ID/localizador y controles de teclado/foco en ambos temas, sin alterar Home ni persistir contenido en el navegador.

#### Scenario: Criterios nuevos y llamadas tardías
- **WHEN** una consulta anterior termina después de editar o desmontar y se inicia otra
- **THEN** no publica datos/error en el nuevo contexto ni libera prematuramente el cupo; sólo las lecturas físicas admitidas por sus colas se ejecutan

#### Scenario: Abrir nota con buffer y usar teclado
- **WHEN** se consulta/pagina/inspecciona en ambos temas con nota pendiente y se abre un hit de entidad
- **THEN** el hit abre el original mediante guardar/descartar/cancelar y conserva el localizador/revisión; cancelar o fallo de guardado conserva buffer y los controles siguen legibles por teclado en viewport estrecho
