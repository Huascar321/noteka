# Spec Delta

## ADDED Requirements

### Requirement: Claridad temprana de contenido emergente

Los menús y pickers compartidos SHALL completar su entrada y retirar el material temporal en 160 ms. Su contenido, el de diálogos y el de Due MUST alcanzar opacidad completa en 100 ms sin espera inicial deliberada. Diálogos y Due SHALL conservar material de entrada de 400 ms y cierre de 200 ms; las operaciones, vidrio en reposo, foco, borradores, interrupción y preferencias MUST conservarse.

#### Scenario: Abrir un selector
- **WHEN** se abre un menú o selector de proyectos con movimiento habilitado
- **THEN** el contenido no espera 120 ms para comenzar a aparecer y la capa temporal deja de cubrirlo al finalizar 160 ms

#### Scenario: Diálogo y accesibilidad
- **WHEN** se abre un diálogo o Due, o se solicita movimiento reducido
- **THEN** el contenido gana claridad en 100 ms con geometría y comportamiento accesible conservados, omitiendo animación cuando corresponde
