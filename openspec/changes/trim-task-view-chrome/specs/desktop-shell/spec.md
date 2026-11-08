## MODIFIED Requirements

### Requirement: Página Tasks del menú lateral

Tasks SHALL abrir una página amplia con icono sobre título, navegación Inbox/Today/Week/Month, New Task y filas compactas con relaciones/fechas locales. Inbox MUST listar tareas pendientes sin relación Project local y MUST NOT agregar una nota explicativa adicional. MUST iniciar Today, acumular páginas únicas y mantener apertura/captura protegidas. Week y Month SHALL limitar el período a cien tareas cargadas, MUST NOT ofrecer Load more y MUST NOT presentar un bloque de tareas sin fecha. Scheduled, No Due y All SHALL cargar páginas de catorce tareas y acumular las siguientes con Load more. MUST NOT ofrecer la vista Unclassified ni un control manual de Inbox. Recurring y Active Content se identifican como pendientes deshabilitadas.

#### Scenario: Apertura y páginas independientes

- **WHEN** se navega a Tasks, se cambia de vista o se solicita Load more en una vista de lista
- **THEN** aparece Today inicialmente, cada vista conserva su identidad de consulta y las páginas siguientes acumulan tareas únicas; una respuesta vieja no reaparece tras cambiar vista o abandonar la página

#### Scenario: Calendario sin paginación

- **WHEN** se abre Week o Month con más tareas que las que caben en una página
- **THEN** el calendario dibuja sólo las tareas con fecha de su página cargada, no muestra el bloque `No date` ni un control Load more y no declara cobertura parcial

#### Scenario: Error y captura

- **WHEN** falla la lectura o se abre un original o New Task
- **THEN** el error permite reintentar sin fingir ausencia, y apertura/captura usan guardias y originales existentes; crear no asigna fechas ni guarda automáticamente

### Requirement: Pies limpios y acciones de fila completas

Home y las páginas que reutilizan sus secciones SHALL omitir contadores/fechas de consulta al pie, Siguiente Tasks y el aviso técnico de Home. MUST ofrecer Load more y New page textuales como filas clickeables de ancho completo, salvo Week y Month de Tasks, que no ofrecen Load more, conservar botones con borde, estados de carga y guardias de captura, y acumular páginas sin duplicar IDs. La cobertura sigue disponible en Opciones de vista.

#### Scenario: Cargar desde el extremo de la fila

- **WHEN** hay otra página y se activa Load more cerca del borde derecho
- **THEN** el botón ejecuta una única lectura y acumula elementos sin duplicarlos; el espacio no se reserva para un contador técnico y al terminar las páginas desaparece Load more

#### Scenario: Captura de ancho completo

- **WHEN** se activa New page sin borde desde cualquier punto de su fila
- **THEN** se prepara el borrador mediante las guardias existentes, sin guardarlo; las acciones de creación con borde mantienen tamaño y estilo

### Requirement: Resultados anteriores identificados durante refrescos

Home y las listas compatibles SHALL conservar sus resultados mientras se actualiza la misma consulta. Si falla MUST indicar que son anteriores y ofrecer reintento; al cambiar la identidad MUST limpiar los resultados previos. Una actualización exitosa SHALL volver a la primera página y MUST NOT mostrar un aviso textual sobre páginas adicionales reiniciadas. MUST deshabilitar el cursor anterior durante la actualización y mientras los resultados estén sin verificar.

#### Scenario: Actualización exitosa con páginas adicionales

- **WHEN** una lista con varias páginas cargadas se actualiza por foco o por solicitud manual
- **THEN** mantiene las filas visibles mientras consulta, publica la primera página nueva sin avisos y deja el cursor anterior disponible para volver a cargar las demás

#### Scenario: Actualización fallida

- **WHEN** falla la lectura de la misma consulta
- **THEN** mantiene las filas anteriores identificadas como no actualizadas, presenta el error y el reintento, y no permite continuar con el cursor anterior

#### Scenario: Consulta diferente

- **WHEN** se cambia vista o fecha seleccionada mientras hay resultados o una lectura pendiente
- **THEN** no se muestran los resultados de la identidad anterior como si correspondieran a la nueva
