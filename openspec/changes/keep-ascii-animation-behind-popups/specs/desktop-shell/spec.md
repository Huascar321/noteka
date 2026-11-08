# Spec Delta

## MODIFIED Requirements

### Requirement: Animación visible junto al Side peek

La portada ASCII de Home SHALL continuar animándose mientras Home y su portada
sigan visibles, aunque el foco esté en un Side peek, un Center peek, Search o
Settings. MUST conservar la pausa cuando la portada quede fuera de vista, la
pestaña o ventana estén ocultas, Home quede cubierto por una presentación Full y
con movimiento reducido. El bloqueo de los controles del fondo MUST NOT
suspender el canvas. El foco, atajos y borradores de los paneles MUST permanecer
independientes.

#### Scenario: Side peek con foco

- **WHEN** se abre y enfoca un Side peek mientras la portada ASCII de Home
  permanece visible
- **THEN** la animación continúa y el foco y atajos pertenecen al panel enfocado

#### Scenario: Popups sobre Home visible

- **WHEN** se abren Settings o Search sobre Home, o un detalle pasa a Center
  peek, mientras la portada ASCII sigue visible
- **THEN** la animación continúa, los controles del fondo siguen bloqueados o
  inertes y los atajos y borradores pertenecen al popup o detalle enfocado

#### Scenario: Portada o pestaña oculta

- **WHEN** la portada sale del área visible o se cambia a otra pestaña
- **THEN** se suspende la animación y vuelve a avanzar cuando Home y su portada
  vuelven a estar visibles

#### Scenario: Movimiento reducido y detalle cubierto

- **WHEN** el sistema pide movimiento reducido o Home queda cubierto por una
  presentación Full
- **THEN** la portada permanece estática o suspendida, sin modificar documentos
  ni buffers

#### Scenario: Cierre del popup

- **WHEN** se cierran Search, Settings o Center peek sin cambiar de pestaña ni
  cubrir Home
- **THEN** la portada conserva el mismo canvas y sigue animando, sin remontar la
  escena
