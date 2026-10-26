# Spec Delta

## ADDED Requirements

### Requirement: Herramientas discretas por grupo de funciones

Los detalles SHALL reemplazar Más opciones por botones de icono que abren paneles, sin ejecutar acciones al abrir. Notes SHALL ofrecer Exportar/Recursos externos/Adjuntos; otros tipos sólo grupos aplicables, con disponibilidad derivada de las condiciones anteriores. MUST preservar acciones, controles, buffers de panel y operaciones iniciadas al cerrar. Los botones SHALL tener tooltip en hover/foco, nombre accesible, aria-controls/expanded, foco visible y Tab/Enter/Espacio/Escape con retorno al disparador; el contenido oculto no recibe foco.

#### Scenario: Abrir y cerrar sin operaciones
- **WHEN** se abre un grupo con clic o teclado y se cierra con Escape
- **THEN** sólo cambia la visibilidad, no se exporta/vincula/procesa nada, el foco vuelve al icono y reabrir conserva los campos del panel

#### Scenario: Disponibilidad y contextos
- **WHEN** hay cambios pendientes, un borrador sin identidad o un documento desconocido
- **THEN** se mantienen las condiciones previas: exportar requiere documento guardado y sin cambios, Markdown requiere formato editable, recursos requieren identidad para abrir y sus escrituras siguen protegidas; Adjuntos aparece sólo para tipo note y Project/Tag/Person no reciben grupos ficticios

### Requirement: Cabecera de Project basada en la referencia local

El detalle Project SHALL presentar portada decorativa local, icono/título editables y Status/Target Deadline/Progress compactos. MUST usar estado/end_date locales y progreso devuelto por Rust, con décimas, carga/error/ausencia reales sin copiar porcentajes de Notion ni calcular reglas de dominio en la UI. Inicio/propósito/procedencia SHALL quedar en View details y las relaciones de contexto abrirán sus originales con guardias existentes.

#### Scenario: Propiedades y progreso acreditado
- **WHEN** se abre un Project con resumen disponible en la consulta local
- **THEN** el porcentaje y barra reflejan el resumen Rust y editar estado/plazo sólo prepara el payload pendiente, conservando revisión y documentos

#### Scenario: Resumen ausente o fallido
- **WHEN** el Project no está en la página de resumen disponible o falla su lectura
- **THEN** se distingue no disponible de cero y de error, sin inventar progreso; la lectura fallida permite reintentar

### Requirement: Contenidos relacionados del Project separados por tipo

Project SHALL presentar Nav a Projects/Dashboard y secciones Tasks/Notes de los originales obtenidos por workflow/context. Tasks SHALL agrupar por estado local reconocido y Notes conservar títulos/tipos reales. MUST conservar paginación acumulativa con cobertura explícita, carga/error/reintento, apertura protegida y captura contextual existente sin writes al preparar. Relaciones de cabecera no duplicarán todas las Tasks/Notes y el editor de relaciones completo seguirá accesible. Temas y 420 px SHALL evitar overflow global.

#### Scenario: Páginas y captura contextual
- **WHEN** se continúa la lectura o se prepara una Task/Note desde el Project
- **THEN** se acumulan originales únicos con el cursor del núcleo o se prepara la captura vinculada sin guardarla; una página parcial no afirma ausencia total de un tipo

#### Scenario: Cambios pendientes y respuesta vieja
- **WHEN** se intenta abrir un registro con cambios pendientes o llega una lectura después de abandonar el Project
- **THEN** se conserva Guardar/Descartar/Cancelar o se descarta la respuesta anterior sin reemplazar el buffer de otra página
