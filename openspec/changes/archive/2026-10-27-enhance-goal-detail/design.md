## Context

Goals son Notes tipadas cuyos documentos pueden contener contenido importado y bloques ajenos a la plantilla. El editor es la fuente de verdad y la escritura usa CAS y guardias existentes.

## Goals / Non-Goals

Ofrecer Overview, un Journal útil y proyectos legibles con contenido auténtico. No convertir automáticamente contenido importado ambiguo, ni modificar documentos al abrir, ni inferir progreso o datos no disponibles.

## Decisions

Tres secciones del documento se delimitan mediante encabezados de bloque con identificadores reservados y reconocibles; sus editores mantienen texto enriquecido. Se inicializan en memoria y sólo se agregan con una edición explícita; los bloques anteriores y prefijos importados se conservan y permanecen editables en contenido existente. La separación exige marcadores íntegros y únicos; documentos ambiguos se dejan como contenido anterior. Cambios de sección recomponen el documento completo, mantienen IDs/versiones, referencias, guardias, errores y la revisión esperada. La tarjeta de Overview y su sección Additional Details reproducen la jerarquía visual de la referencia; Journal es desplegable. El diario agrega un encabezado con fecha y un párrafo mediante el editor, sin crear entidades. La tabla de proyectos usa estado y fecha de las entidades relacionadas, sin métricas calculadas en UI; permite abrir originales y vincular uno guardado.

## Risks / Trade-offs

Las plantillas heredadas pueden contener su propio Overview o Journal: se conservan como contenido existente y no se convierten automáticamente. Clientes previos ven los encabezados y bloques ordinarios; si modifican los marcadores la UI no interpreta la estructura y protege el contenido como anterior. Validar ida/vuelta, errores y formatos desconocidos, guardias, anchos y temas en workspace sintético; no afirmar aceptación nativa.
