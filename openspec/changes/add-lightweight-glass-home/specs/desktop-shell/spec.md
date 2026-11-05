## ADDED Requirements

### Requirement: Marco de Home con vidrio liviano

El shell SHALL presentar una apariencia de vidrio discreta coherente en sidebar, barra superior y controles de Home, en ambos temas y con acento configurable. MUST conservar legibilidad, foco y geometría, y evitar filtros de desenfoque grandes y animaciones continuas adicionales.

#### Scenario: Home en temas claro y oscuro

- **WHEN** la persona abre Home o cambia de tema y color primario
- **THEN** el marco y sus controles mantienen la estética y legibilidad sin alterar datos ni geometría

#### Scenario: Menor transparencia

- **WHEN** el entorno solicita transparencia reducida y el WebView expone esa preferencia
- **THEN** las superficies adoptan fondos sólidos conservando foco y contraste

#### Scenario: Navegación y paneles

- **WHEN** la persona abre búsqueda, navega o redimensiona el sidebar
- **THEN** se conservan las acciones, el foco, las transiciones existentes y el contenido

#### Scenario: Continuidad de la portada ASCII

- **WHEN** Home muestra una escena ASCII en cualquiera de los dos temas
- **THEN** la portada y el contenido comparten el mismo fondo, sin añadir animación ni desenfoque al fondo

#### Scenario: Cambio del color primario

- **WHEN** la persona cambia el color primario en Settings
- **THEN** foco, iconos y acciones se adaptan al color elegido, mientras los fondos neutros conservan la legibilidad en ambos temas
