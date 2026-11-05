## ADDED Requirements

### Requirement: Apariencia discreta de Projects Board

Projects Board SHALL presentar columnas, tarjetas y acciones con superficies y bordes discretos en temas claro y oscuro. MUST conservar etiquetas de estado legibles e indicadores visibles de foco y destino de arrastre.

#### Scenario: Lectura del tablero en ambos temas

- **WHEN** la persona abre Projects Board en tema claro u oscuro
- **THEN** distingue las cinco etiquetas de estado sin superficies ni acciones intensamente saturadas

#### Scenario: Movimiento de una tarjeta

- **WHEN** arrastra una tarjeta hacia una columna
- **THEN** el destino permanece identificable y el movimiento conserva su comportamiento existente
