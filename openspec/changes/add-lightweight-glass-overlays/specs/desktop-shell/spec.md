## ADDED Requirements

### Requirement: Vidrio acotado para búsqueda y menús

La búsqueda global y los menús flotantes SHALL usar superficies legibles con vidrio discreto coherente con Home. MUST limitar el desenfoque a superficies abiertas y conservar búsqueda, navegación y controles existentes, con fondo sólido en transparencia reducida o contraste aumentado.

#### Scenario: Búsqueda abierta sobre Home

- **WHEN** la persona abre búsqueda global sobre Home
- **THEN** el efecto se limita al diálogo y la portada animada deja de generar cuadros hasta que se cierre el bloqueo

#### Scenario: Preferencias de accesibilidad

- **WHEN** el WebView expone transparencia reducida o contraste aumentado
- **THEN** los paneles usan un fondo sólido sin desenfoque y mantienen controles legibles

#### Scenario: Menú flotante

- **WHEN** se abre un menú de acciones, de modo, de relaciones o de editor
- **THEN** conserva su posición, teclado y acciones con el acabado de vidrio compartido
