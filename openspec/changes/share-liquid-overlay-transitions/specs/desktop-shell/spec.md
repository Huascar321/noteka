## ADDED Requirements

### Requirement: Movimiento compartido de menús y popups

Las superficies interactivas transitorias de Noteka SHALL usar expansión elástica de 400 ms y cierre de 200 ms, con contenido legible sin compresión. MUST conectar al ancla visible o usar expansión central contenida si no existe. Incluye acciones, modos peek, relaciones, iconos, editor, herramientas, Nav, opciones de vista, Search, Settings, confirmaciones y Center peek; los controles nativos del sistema conservan su presentación.

#### Scenario: Apertura contextual
- **WHEN** se abre un menú, picker o herramienta desde un control o selección
- **THEN** su material se expande desde ese origen y el contenido aparece sin deformación ni cambio de operaciones

#### Scenario: Presentación sin ancla
- **WHEN** se abre Search por atajo, una confirmación o un Center peek sin control visible de origen
- **THEN** usa expansión central contenida con el mismo ritmo sin desplazarse desde una posición ficticia

#### Scenario: Contenido progresivo de diálogos
- **WHEN** se abre una confirmación, Search, Settings, Move project o Center peek
- **THEN** el contenido real se revela progresivamente dentro del vidrio desde el inicio, sin una fase deliberadamente vacía, sin duplicados interactivos ni aparición repentina al desmontar el material animado

### Requirement: Ciclo de vida accesible de superficies transitorias

Las transiciones MUST conservar foco, teclado, posición y contenido de cada superficie, respetar movimiento reducido y los materiales sólidos accesibles. El cierre interrumpido MUST comenzar en el frame actual y las superficies salientes MUST quedar inertes. MUST liberar efectos y capas decorativas al terminar o desmontarse, sin red ni escrituras por animar.

#### Scenario: Cierre e interrupción
- **WHEN** se cierra durante la expansión, se reabre rápidamente o se cambia a un menú relacionado
- **THEN** no salta primero al estado expandido, no se repiten acciones y el foco sigue el contrato del componente

#### Scenario: Preferencias y temas
- **WHEN** se usan temas claro u oscuro, dos acentos, movimiento reducido o contraste aumentado
- **THEN** se conserva legibilidad, el vidrio sigue neutro y las alternativas accesibles eliminan movimiento o transparencia según corresponda

#### Scenario: Panel persistente
- **WHEN** se cierra y reabre una herramienta con un borrador local o se cambia de presentación de detalle
- **THEN** el movimiento no remonta el editor ni descarta datos, búsquedas, borradores o propiedades desconocidas
