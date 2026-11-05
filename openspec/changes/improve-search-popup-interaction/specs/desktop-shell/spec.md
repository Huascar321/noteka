## ADDED Requirements

### Requirement: Interacción del popup de búsqueda

El popup de búsqueda SHALL cerrarse al hacer clic fuera del panel y MUST conservar el cierre por Escape y por el botón Close. Mientras esté abierto, la portada ASCII de Home MUST seguir animando, sin habilitar los controles bloqueados del fondo.

#### Scenario: Clic fuera del popup

- **WHEN** la persona hace clic en el backdrop
- **THEN** el popup se cierra

#### Scenario: Fondo animado

- **WHEN** la búsqueda está abierta sobre Home
- **THEN** la animación ASCII continúa y los controles del fondo siguen bloqueados

#### Scenario: Movimiento reducido

- **WHEN** el sistema prefiere movimiento reducido
- **THEN** el canvas permanece en su último cuadro y el popup conserva sus cierres
