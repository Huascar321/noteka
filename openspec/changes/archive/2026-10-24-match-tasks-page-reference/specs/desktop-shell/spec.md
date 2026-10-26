# Spec Delta

## ADDED Requirements

### Requirement: Página Tasks del menú lateral

Tasks SHALL abrir una página amplia con icono sobre título, navegación Inbox/Today/Week/Month, New Task y filas compactas con relaciones/fechas locales. MUST iniciar Today, acumular páginas únicas y mantener apertura/captura protegidas. Recurring y Active Content se identifican como pendientes deshabilitadas.

#### Scenario: Apertura y páginas independientes
- **WHEN** se navega a Tasks, se cambia de vista o se solicita Load more
- **THEN** aparece Today inicialmente, cada vista conserva su identidad de consulta y las páginas siguientes acumulan tareas únicas; una respuesta vieja no reaparece tras cambiar vista o abandonar la página

#### Scenario: Error y captura
- **WHEN** falla la lectura o se abre un original o New Task
- **THEN** el error permite reintentar sin fingir ausencia, y apertura/captura usan guardias y originales existentes; crear no asigna fechas ni guarda automáticamente

### Requirement: Projects integrado en la página Tasks

La página Tasks SHALL presentar Projects debajo con Active inicial y vistas Planned/Board/Deadlines/Completed. MUST usar listas compactas agrupadas para Active/Planned/Completed, fechas locales en Deadlines y tablero existente en Board, con referencias navegables, pendientes, vencidas y progreso real; conservar temas, teclado y ancho estrecho.

#### Scenario: Listas y tablero
- **WHEN** se abre Tasks y se alternan vistas de Projects
- **THEN** Active agrupa Doing/Ongoing, Planned muestra planificados, Completed muestra Done y Board ofrece sus cinco columnas con movimiento protegido; ambas secciones se leen y actualizan por separado

#### Scenario: Datos ausentes y actualización
- **WHEN** no hay fechas, tareas relacionadas, recursos locales o hay un diálogo abierto
- **THEN** se presenta ausencia explícita sin inventar porcentajes, fechas ni imágenes; actualizar con el atajo de plataforma o F5 conserva vistas y no recarga el WebView ni ejecuta cambios mientras existe un diálogo

### Requirement: Consultas de proyectos por fecha y completados

Rust SHALL ofrecer proyectos Done y proyectos no completados con end_date mediante home.query. MUST excluir archivo local/de origen, ordenar fechas civiles ascendentes con desempate estable, filtrar antes de paginar y conservar caps/snapshot/cursor. Fechas inválidas se señalan como metadatos no disponibles; leer no altera payloads ni documentos.

#### Scenario: Selección y fechas
- **WHEN** se consultan Completed o Deadlines con proyectos locales
- **THEN** Completed selecciona Done y Deadlines selecciona proyectos no Done con fecha de fin válida, ordenados por fin; ninguna vista toma fechas de tareas como plazo de proyecto

#### Scenario: Metadatos o cursor inválidos
- **WHEN** existe una fecha de fin inválida o se reutiliza un cursor de otra vista
- **THEN** la consulta identifica los metadatos no interpretables o rechaza el cursor ajeno, sin reparar datos ni aceptar una página de otro criterio

### Requirement: Breadcrumb jerárquico de secciones y registros

El escritorio SHALL mostrar Home → sección en las páginas del menú y Home → sección canónica → registro en sus detalles, con iconos propios y título actual. MUST omitir niveles externos ajenos, permitir navegar Home/sección mediante guardias existentes y conservar teclado, título accesible y ancho estrecho; el registro actual no es un enlace de guardado.

#### Scenario: Secciones y registro abierto
- **WHEN** se abre Tasks o el proyecto Proyecto de ejemplo
- **THEN** el breadcrumb muestra respectivamente Home → Tasks o Home → Projects → Proyecto de ejemplo, con los iconos locales de cada tramo y sin Volver a Home ni Databases

#### Scenario: Cambios pendientes y nombre largo
- **WHEN** se activa un antecesor con cambios pendientes o se reduce el ancho disponible
- **THEN** navegar respeta Guardar/Descartar/Cancelar sin perder el buffer; el título puede truncarse visualmente conservando su valor completo accesible y los antecesores siguen siendo alcanzables con teclado

### Requirement: Pies limpios y acciones de fila completas

Home y las páginas que reutilizan sus secciones SHALL omitir contadores/fechas de consulta al pie, Siguiente Tasks y el aviso técnico de Home. MUST ofrecer Load more y New page textuales como filas clickeables de ancho completo, conservar botones con borde, estados de carga y guardias de captura, y acumular páginas sin duplicar IDs. La cobertura sigue disponible en Opciones de vista.

#### Scenario: Cargar desde el extremo de la fila
- **WHEN** hay otra página y se activa Load more cerca del borde derecho
- **THEN** el botón ejecuta una única lectura y acumula elementos sin duplicarlos; el espacio no se reserva para un contador técnico y al terminar las páginas desaparece Load more

#### Scenario: Captura de ancho completo
- **WHEN** se activa New page sin borde desde cualquier punto de su fila
- **THEN** se prepara el borrador mediante las guardias existentes, sin guardarlo; las acciones de creación con borde mantienen tamaño y estilo
