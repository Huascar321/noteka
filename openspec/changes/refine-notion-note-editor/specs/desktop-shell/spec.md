# Spec Delta

## ADDED Requirements

### Requirement: Superficie de escritura contextual en Notes

Note normal SHALL ofrecer página centrada y herramientas contextuales accesibles,
con menús dentro del viewport, ambos temas y ancho estrecho. MUST conservar
propiedades, relaciones, guardado y herramientas; Task/Goal/Milestone conservan
su composición. El diálogo pendiente SHALL ordenar Cancel/Discard/Save and
continue, con iconos a la izquierda y espacio tras la explicación; Cancel recibe
foco inicial y las operaciones se conservan.

#### Scenario: Escribir en una página
- **WHEN** se abre una Note y se comienza con Enter desde el título o haciendo clic en el cuerpo
- **THEN** se escribe directamente en la columna de contenido, con jerarquía tipográfica, interlineado y espaciado de bloques, sin activar edición aparte ni paneles de formato permanentes delante del cuerpo; los controles aparecen en su contexto con nombres accesibles, foco visible y acceso por teclado, y el guardado sigue explícito

#### Scenario: Temas y ancho estrecho
- **WHEN** se edita y se abren menús en temas claro/oscuro o un viewport de 420 px
- **THEN** texto, selección y foco son legibles, los menús se ajustan al espacio visible y las acciones esenciales siguen accesibles por teclado

#### Scenario: Diálogo de cambios pendientes
- **WHEN** se intenta salir con un borrador pendiente
- **THEN** Cancel inicia la fila y recibe foco, Discard queda en medio y Save and continue al final/derecha; Tab sigue ese orden y las tres acciones conservan su efecto previo

### Requirement: Herramientas de Note en la barra superior

El detalle de Note normal SHALL agrupar Export, External resources, Attachments
y Archive/Restore en un único botón `…` de la barra superior junto al estado de
guardado, sin repetirlos fuera del menú. Cada opción SHALL mostrar icono y texto,
conservar disponibilidad y permitir recorrer acciones habilitadas con flechas,
Home/End, Enter y Escape. SHALL abrir los paneles de herramientas acotados al
viewport desde el menú, con cierre exterior/Escape y retorno de foco al botón.
MUST conservar disponibilidad,
campos y resultados al cerrar, sin ejecutar acciones por abrir/cerrar ni
alterar el contenido. El seguimiento `unify-page-actions-menu` lleva la misma ubicación a otros tipos de detalle.

#### Scenario: Ubicación y apertura
- **WHEN** se abre una Note y se expande el botón `…` de la cabecera
- **THEN** Export, External resources, Attachments y Archive/Restore aparecen como filas con icono/texto; seleccionar una herramienta abre su panel y sólo una acción explícita dentro del panel exporta, vincula o procesa

#### Scenario: Cierre y recuperación del panel
- **WHEN** se escribe un destino de exportación, se cierra con Escape o clic exterior y se reabre
- **THEN** el campo se conserva, Escape devuelve foco a `…` y las condiciones de guardado siguen aplicándose en ambos temas/420 px

#### Scenario: Teclado y acciones de archivo
- **WHEN** se abre el menú con teclado, se recorren sus opciones y se elige Archive o Restore con un borrador pendiente o una entidad guardada
- **THEN** flechas/Home/End omiten acciones deshabilitadas, Escape cierra y retorna el foco; archivar/restaurar conserva la misma guardia Guardar/Descartar/Cancelar y revisión esperada, y abrir/cerrar el menú no muta contenido ni ejecuta acciones
