## ADDED Requirements

### Requirement: Animación visible junto al Side peek

La portada ASCII de Home SHALL continuar animándose cuando Home esté visible junto a un Side peek, aunque el foco esté en el detalle. MUST conservar la pausa cuando la portada quede fuera de vista, la pestaña o ventana estén ocultas y la composición estática con movimiento reducido. El foco, atajos y borradores de ambos paneles MUST permanecer independientes.

#### Scenario: Side peek con foco
- **WHEN** se abre y enfoca un Side peek mientras la portada ASCII de Home permanece visible
- **THEN** la animación continúa y el foco y atajos pertenecen al panel enfocado

#### Scenario: Portada o pestaña oculta
- **WHEN** la portada sale del área visible o se cambia a otra pestaña
- **THEN** se suspende la animación y vuelve a avanzar cuando Home y su portada vuelven a estar visibles

#### Scenario: Movimiento reducido y detalle cubierto
- **WHEN** el sistema pide movimiento reducido, Center peek recibe el foco o Home queda cubierto por una presentación Full
- **THEN** la portada permanece estática o suspendida, sin modificar documentos ni buffers
