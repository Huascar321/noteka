# Spec Delta

## ADDED Requirements

### Requirement: Guardia de cambios pendientes con material compartido

El diálogo de cambios sin guardar SHALL usar marco translúcido, borde luminoso y reflejos compartidos con los popups. Cancel/Discard SHALL ser cápsulas neutras y Save and continue una acción teñida. MUST conservar texto, orden, foco, deshabilitados y decisiones existentes sin descartar o guardar implícitamente.

#### Scenario: Elegir cómo resolver el borrador
- **WHEN** se abre la guardia y se usa Cancel, Discard, Save and continue o Escape
- **THEN** cada acción conserva su operación y resultado previo, el estado sigue legible y el material no intercepta eventos

### Requirement: Guardado de cabecera coherente con la acción primaria

Save/Retry de cabecera SHALL usar la misma cápsula de vidrio teñido que la confirmación de guardado, con atajo blanco legible. MUST conservar Unsaved changes, Saving, Saved y error, guardado explícito y protección del borrador ante fallos o conflictos.

#### Scenario: Fallo y reintento
- **WHEN** falla un guardado desde cabecera o guardia y se vuelve a intentar
- **THEN** el borrador continúa disponible y la interfaz sólo confirma Saved o continúa la acción al completarse la operación existente

### Requirement: Accesibilidad del material de guardado

Los tratamientos SHALL conservar lectura, foco y acciones en ambos temas, distintos acentos y ancho estrecho. Contraste aumentado o transparencia reducida MUST usar superficies sólidas sin reflejos/filtros; movimiento reducido MUST eliminar transiciones decorativas.

#### Scenario: Preferencias y ventana estrecha
- **WHEN** se cambian tema/acento, preferencias accesibles o ancho
- **THEN** botones y etiquetas siguen accesibles, las acciones se ajustan sin desbordar la ventana y se aplica la alternativa sólida o sin movimiento correspondiente
