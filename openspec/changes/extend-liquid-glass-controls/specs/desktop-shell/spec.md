# Spec Delta

## ADDED Requirements

### Requirement: Material de vidrio coherente en navegación y creación

El shell SHALL compartir el borde iluminado y los reflejos del sidebar con la pestaña superior activa, breadcrumb, vista seleccionada y acción New. La pestaña y la vista MUST ser neutras y de menor relieve; el breadcrumb MUST ser más tenue. New MUST usar el color primario con texto blanco legible. Los controles + y × MUST ofrecer una superficie circular al pasar el cursor o enfocar.

#### Scenario: Jerarquía de controles en ambos temas
- **WHEN** se abre Home o una sección en tema claro u oscuro
- **THEN** la pestaña activa y la vista seleccionada muestran vidrio neutro con sombra corta, el breadcrumb muestra un reflejo tenue y New muestra vidrio teñido; los controles inactivos conservan su jerarquía secundaria

#### Scenario: Estados y color primario
- **WHEN** se interactúa con los controles o se cambia el color primario
- **THEN** hover, presión y foco mantienen la óptica compartida, sólo New adopta el tinte de acción y su texto blanco mantiene contraste mínimo de 4,5:1 sobre su fondo compuesto en reposo y hover

### Requirement: Accesibilidad e interacción preservadas en controles de vidrio

Los cuatro controles SHALL conservar sus operaciones, semántica, áreas de interacción y transiciones existentes. La transparencia reducida y el contraste aumentado MUST ofrecer superficies sólidas sin reflejos ni filtros. Movimiento reducido MUST desactivar las transiciones decorativas. El acabado MUST funcionar offline sin cambiar contenido persistido.

#### Scenario: Navegación y creación
- **WHEN** se cambia o cierra una pestaña, se usa +, se navega por una miga, se cambia de vista o se activa New
- **THEN** se ejecuta la operación existente y se conservan las guardias de borradores, el teclado y el estado de selección; las capas de brillo no interceptan eventos

#### Scenario: Preferencias y vista estrecha
- **WHEN** se activan las preferencias de accesibilidad o se estrecha la ventana
- **THEN** los controles siguen alcanzables, sus indicadores no recortan la lectura y las alternativas sólidas o sin movimiento respetan la preferencia solicitada
