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

El detalle de Note normal SHALL ubicar Exportar, Recursos externos y Adjuntos
en la barra superior junto al estado de guardado, sin repetirlos debajo del
contenido. SHALL abrir paneles acotados al viewport desde esos iconos, con
tooltip, teclado, Escape y retorno de foco. MUST conservar disponibilidad,
campos y resultados al cerrar, sin ejecutar acciones por abrir/cerrar ni
alterar el contenido o la ubicación de herramientas en otros tipos de detalle.

#### Scenario: Ubicación y apertura
- **WHEN** se abre una Note y se pulsa uno de los tres iconos de la barra superior
- **THEN** su panel aparece desde la cabecera y no al pie; sólo la acción explícita dentro del panel exporta, vincula o procesa

#### Scenario: Cierre y recuperación del panel
- **WHEN** se escribe un destino de exportación, se cierra con Escape o clic exterior y se reabre
- **THEN** el campo se conserva, Escape devuelve foco al disparador y las condiciones de guardado siguen aplicándose en ambos temas/420 px
