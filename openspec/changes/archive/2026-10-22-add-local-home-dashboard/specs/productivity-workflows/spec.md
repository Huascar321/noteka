# Spec Delta

## ADDED Requirements

### Requirement: Consultas Home compartidas y acotadas

El núcleo SHALL consultar vistas Home de entidades originales activas con orden determinista, contexto y cobertura paginada, accesibles por escritorio y CLI. MUST acotar páginas a 1–100 elementos, usar 50 por defecto y rechazar vistas, fechas, campos o cursores inválidos sin mutar documentos, payloads, revisiones, relaciones ni historial.

#### Scenario: Paridad y páginas estables
- **WHEN** escritorio y CLI consultan la misma vista, día y páginas de un conjunto de 51 entidades sin escrituras concurrentes
- **THEN** reciben el mismo orden, IDs, contexto y cobertura; los elementos únicos se recorren sin pérdida ni duplicación y no aparecen mutaciones confirmadas

#### Scenario: Entrada inválida o cursor ajeno
- **WHEN** se solicita una vista no admitida, fecha imposible, límite fuera de rango, campo desconocido o cursor de otro workspace, vista o día
- **THEN** se devuelve validation_error sin publicar una página de otra consulta ni modificar originales

#### Scenario: Lectura de formato desconocido
- **WHEN** una entidad o contexto conserva campos futuros y un metadato requerido no se puede interpretar
- **THEN** su ID y JSON original permanecen recuperables; el dato afectado se identifica como no disponible y no se transforma en estado, conteo o progreso válido por defecto

### Requirement: Today por vencimiento civil sin alterar My Day

Home Today SHALL seleccionar tareas no completadas ni archivadas cuyo due_date sea menor o igual al día civil de referencia del workspace. MUST excluir vencimiento ausente y mostrar día y zona usados. My Day SHALL seguir usando planned_date igual al día y Overdue due_date menor al día, sin reinterpretarlos al abrir Home.

#### Scenario: Hoy incluye vencidas y no reemplaza planificación
- **WHEN** existen tareas vencidas, que vencen hoy, futuras, sin vencimiento, completadas y sólo planificadas para hoy
- **THEN** Today contiene sólo pendientes con vencimiento hasta hoy; My Day y Overdue conservan sus reglas previas y los mismos IDs

#### Scenario: Límite civil y zona
- **WHEN** se consulta con día explícito o el instante actual cae en días diferentes en UTC y la zona del workspace
- **THEN** se usa el día explícito o el de esa zona, respectivamente, se expone la referencia y no se convierte una fecha civil en instante ni se escribe planificación

### Requirement: Vistas locales Home explícitas

El sistema SHALL ofrecer Tasks Today/Inbox/All/Done, Notes Notes/Inbox, Projects Active/Planned/Board, Tags Recent/A–Z/Types y People Contacts/New/Updated con filtros y órdenes documentados. MUST excluir entidades archivadas, conservar referencias a contextos archivados y distinguir sus reglas propias de filtros de Notion no inspeccionados.

#### Scenario: Tareas y notas por estado e Inbox
- **WHEN** se consultan las vistas con tareas todo/done y notas dentro/fuera de Inbox
- **THEN** Tasks Inbox devuelve sólo todo con inbox, All incluye todo/done y Done sólo done; Notes incluye todas las notas activas e Inbox sólo las marcadas, sin inferir clasificación desde relaciones

#### Scenario: Proyectos y clasificación conservada
- **WHEN** se consultan proyectos en los cinco estados y tags area/resource/topic
- **THEN** Home Active incluye doing/ongoing, Planned sólo planned y Board los cinco estados, mientras el workflow Active Projects previo sigue incluyendo planned/doing/ongoing; Types conserva las tres clasificaciones sin convertir topic en otro tipo ni duplicar entidades

#### Scenario: Orden documentado de tags y personas
- **WHEN** se consultan Tags Recent/A–Z y People Contacts/New/Updated con timestamps y títulos empatados
- **THEN** se aplican los órdenes documentados con desempate por ID estable; no se interpreta New como un período relativo ni se inventan favoritos, organización o atributos de contacto

### Requirement: Resúmenes de proyectos desde tareas originales

El núcleo SHALL derivar por proyecto total, pendientes, completadas y vencidas desde tareas relacionadas no archivadas en el mismo snapshot de lectura. MUST usar due_date menor al día para vencidas y completadas/total para progreso, distinguir ausencia de tareas de cero progreso y no persistir porcentajes ni atribuirlos a la fórmula de Notion.

#### Scenario: Resumen completo aunque la lista esté paginada
- **WHEN** un proyecto tiene más de una página de tareas, algunas done y otras vencidas, y el CLI cambia una tarea o relación
- **THEN** el resumen usa todas las tareas únicas de ese proyecto, no sólo filas visibles, y la próxima consulta refleja el cambio sin incrementar la revisión del proyecto

#### Scenario: Sin tareas o metadatos futuros
- **WHEN** un proyecto no tiene tareas activas o alguna tarea relacionada tiene estado/vencimiento no interpretable
- **THEN** no se presenta progreso ficticio; se informa sin tareas o resumen no disponible, respectivamente, y los originales siguen intactos

### Requirement: Contexto y resúmenes de notas y tags

El núcleo SHALL proyectar relaciones de filas Home a proyectos, tags y personas con sus IDs y estado de archivo; para tags SHALL informar cantidad de notas activas relacionadas y última nota por modificación con desempate estable. MUST obtener resumen y referencias de un mismo snapshot y conservar los originales, sin expandir sus documentos.

#### Scenario: Última nota y conteo global
- **WHEN** un tag relaciona más de una página de notas, dos empatan en modificación y otra está archivada
- **THEN** cuenta sólo notas activas únicas y selecciona la última por modificación e ID documentados; archivar/restaurar se refleja al refrescar sin cambiar el tag

#### Scenario: Contexto archivado y documentos desconocidos
- **WHEN** una nota activa tiene un proyecto/tag/persona archivado o documento de versión futura
- **THEN** su fila conserva referencias originales con indicador de archivo y permite abrir el original recuperable, sin convertir ni cargar recursivamente el documento

### Requirement: Grupos Home como referencias de una página

Los grupos de tareas por proyecto, proyectos por estado y tags por tipo SHALL referenciar sólo IDs de la página devuelta y explicitar cobertura parcial. MUST distinguir elementos únicos de apariciones cuando una tarea pertenece a varios proyectos, incluir Sin proyecto y mostrar estados/tipos vacíos declarados sin inventar registros.

#### Scenario: Relaciones múltiples sin copias
- **WHEN** una tarea tiene dos proyectos y otra ninguno en una página de Tasks agrupada
- **THEN** la primera aparece como el mismo ID en dos grupos y la segunda en Sin proyecto; el total único y las cantidades por grupo se distinguen y no se crean entidades ni relaciones

#### Scenario: Grupos vacíos y segunda página
- **WHEN** se consultan Projects Board o Tags Types con grupos vacíos y más de una página de originales
- **THEN** se muestran los cinco estados o tres tipos declarados, las cantidades de cada grupo cubren sólo la página y recorrer páginas conserva la identidad y el orden documentado
