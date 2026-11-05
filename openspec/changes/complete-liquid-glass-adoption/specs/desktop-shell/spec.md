## ADDED Requirements

### Requirement: Adopción coherente de materiales de escritorio

Los controles de escritorio SHALL aplicar las recetas compartidas de campos, acciones, grupos y popups de la guía de diseño. MUST conservar semántica, foco, guardias, estados y contenido sólido; las alternativas de contraste o transparencia reducida SHALL ser sólidas.

#### Scenario: Controles adaptados
- **WHEN** se usan Collections, detalles, búsquedas, herramientas, Settings o bienvenida en temas claro u oscuro
- **THEN** los controles mantienen sus operaciones y estados accesibles con materiales y dimensiones adecuados a su contexto

#### Scenario: Popups y preferencias accesibles
- **WHEN** se abre Move project o el selector flotante de vistas, o se solicita mayor contraste o menor transparencia
- **THEN** el popup conserva selección, foco, cierre y acciones; la alternativa accesible elimina el desenfoque y utiliza una superficie sólida

#### Scenario: Menús de detalle
- **WHEN** se abren menús en side, center o full peek con desplazamiento
- **THEN** permanecen anclados y accesibles y se conservan borradores y guardias existentes

### Requirement: Creación superior única en People y Tags

People y Tags SHALL ofrecer una única acción superior New en la barra de vistas, conservando las acciones de creación contextuales y la alineación compartida de barras.

#### Scenario: Creación desde sección y grupos
- **WHEN** se visita People o Tags
- **THEN** no aparece una captura superior New Person o New Tag duplicada y siguen disponibles New y las acciones contextuales correspondientes

### Requirement: Icono coherente en acciones New

Las acciones New de creación en barras, cabeceras y filas contextuales SHALL mostrar el icono + compartido de 16 px, conservando su texto, nombre accesible y destino.

#### Scenario: Creación desde barras y subsecciones
- **WHEN** se presentan acciones New en barras, cabeceras o filas contextuales
- **THEN** muestran el mismo icono y sus acciones contextuales conservan su función y accesibilidad

### Requirement: Foco visible según modalidad de interacción

El escritorio SHALL ocultar el anillo de foco provocado por puntero y SHALL conservar un indicador visible al navegar por teclado. MUST preservar indicadores de selección y arrastre.

#### Scenario: Foco por puntero y teclado
- **WHEN** se activa un control mediante puntero y después se navega mediante teclado
- **THEN** el primer foco no muestra el anillo y el foco de teclado vuelve a mostrarlo sin alterar la selección
