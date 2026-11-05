## ADDED Requirements

### Requirement: Vidrio líquido en la búsqueda

El diálogo de búsqueda SHALL presentar una superficie translúcida con blur local, tinte más transparente, reflejo y borde luminoso, y MUST aplicar el mismo material al campo, la fila activa y el pie. MUST conservar la legibilidad y volver a superficies sólidas con transparencia reducida o contraste aumentado.

#### Scenario: Búsqueda abierta

- **WHEN** la persona abre la búsqueda sobre Home o una página
- **THEN** el panel deja ver el contenido detrás con blur y reflejo, sin desenfocar el resto de la ventana

#### Scenario: Transparencia reducida o contraste aumentado

- **WHEN** el entorno solicita menor transparencia o mayor contraste
- **THEN** el panel, el campo, la fila activa y el pie usan fondos sólidos y conservan el foco visible
