## ADDED Requirements

### Requirement: Material de vidrio neutro

El shell SHALL presentar fondos neutros en Home, sidebar, barra superior, búsqueda y menús flotantes independientemente del color primario configurado. MUST conservar legibilidad, foco, navegación y preferencias de accesibilidad.

#### Scenario: Cambio del color primario

- **WHEN** la persona cambia el color primario desde Settings
- **THEN** los fondos de vidrio y de contenido conservan su tono neutro mientras foco, iconos y acciones usan el color elegido

#### Scenario: Búsqueda y menú abiertos

- **WHEN** la persona abre búsqueda o un menú flotante
- **THEN** el panel muestra el contenido detrás de una superficie neutra translúcida con borde visible y el desenfoque queda limitado al panel

#### Scenario: Menor transparencia o mayor contraste

- **WHEN** el entorno solicita menor transparencia o mayor contraste y el WebView expone esa preferencia
- **THEN** los paneles de vidrio usan fondos sólidos legibles sin desenfoque

#### Scenario: Navegación contextual en otra pantalla

- **WHEN** la persona abre un menú, tooltip o diálogo pequeño desde Tasks, Projects, editor o Settings
- **THEN** la superficie usa el mismo material neutro, conserva foco y lectura y no desenfoca el cuerpo de la página
