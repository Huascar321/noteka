# Spec Delta

## ADDED Requirements

### Requirement: Material transparente compartido en popups

Los popups que usaban vidrio SHALL compartir el tinte gris, reflejo tenue y desenfoque corto aprobados en Search. Incluye Settings, confirmaciones, Move project, menús, iconos, relaciones, herramientas flotantes, popovers del editor, calendario Due y tooltips de herramientas. MUST conservar legibilidad y comportamiento sin red ni cambios de datos.

#### Scenario: Consistencia entre superficies
- **WHEN** se abren estas superficies en ambos temas y con distintos acentos
- **THEN** muestran contenido real detrás con material neutro coherente y un solo filtro local, sin recuperar velos blancos ni tinte denso anterior

### Requirement: Preservación y alternativas del vidrio compartido

La propagación SHALL conservar dimensiones, posicionamiento, teclado, foco, cierre, buffers y operaciones de cada popup. MUST mantener texto normal legible con contraste compuesto mínimo 4,5:1 y alternativas sólidas sin filtros/reflejos para contraste aumentado o transparencia reducida. Movimiento reducido MUST conservar presentación inmediata.

#### Scenario: Operación y preferencias
- **WHEN** se busca, se elige un icono o relación, se opera Due o una confirmación y se cambian preferencias o ancho
- **THEN** cada operación y sus estados siguen disponibles, sin overflow global ni pérdida de borrador, con la alternativa accesible correspondiente
