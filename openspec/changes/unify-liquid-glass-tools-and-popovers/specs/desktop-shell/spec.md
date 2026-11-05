# Spec Delta

## ADDED Requirements

### Requirement: Herramientas de sección agrupadas en vidrio

Las barras de Tasks, Notes, Projects, Tags, Goals, People y Milestones SHALL agrupar sus acciones de icono en una cápsula neutra de vidrio. New MUST conservar una superficie independiente. La agrupación MUST mantener etiquetas, foco, estados y operaciones, con tamaño compacto y sin filtros que desplacen menús flotantes.

#### Scenario: Herramientas y creación
- **WHEN** se abre Home o una página de sección y se usa una herramienta o New
- **THEN** las acciones de icono comparten una cápsula y New permanece fuera de ella; cada acción ejecuta su operación existente y conserva su estado habilitado

### Requirement: Campo de búsqueda y superficies flotantes coherentes

El campo del buscador SHALL compartir el material de campo de Search lateral con foco visible. Los menús de página/apertura/bloques, pickers de iconos/relaciones y paneles flotantes de herramientas SHALL compartir borde iluminado, reflejos y radios. MUST conservar entrada, resultados, selección, posición y cierre existentes.

#### Scenario: Búsqueda y selección
- **WHEN** se escribe en el buscador o se abre un menú o selector
- **THEN** el control presenta el material compartido y conserva navegación de teclado, foco, resultados, opciones y cierre; las capas decorativas no reciben eventos

### Requirement: Portada con selector segmentado de vidrio

El selector ASCII/Photo SHALL presentarse como una cápsula segmentada con un indicador único que se desplaza y adapta a la opción activa. MUST conservar la elección y persistencia existentes, etiquetas, teclado y visibilidad al enfocar.

#### Scenario: Cambio de portada
- **WHEN** se elige ASCII o Photo con ratón o teclado
- **THEN** la portada cambia mediante la operación existente, el indicador identifica la opción activa y la elección se conserva al volver a abrir

### Requirement: Accesibilidad del material de herramientas y popovers

Los tratamientos SHALL funcionar en claro y oscuro y seguir neutros al cambiar el acento. Transparencia reducida o contraste aumentado MUST ofrecer superficies sólidas sin reflejos/filtros; movimiento reducido MUST eliminar transiciones decorativas. En vista estrecha los controles MUST permanecer alcanzables sin overflow global ni alteración de datos.

#### Scenario: Preferencias y ancho reducido
- **WHEN** se cambian tema/acento, se activan preferencias accesibles o se estrecha la ventana
- **THEN** se conserva lectura y foco, los menús se mantienen dentro de sus límites y se aplica la alternativa sólida o sin movimiento correspondiente
