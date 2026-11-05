# Spec Delta

## ADDED Requirements

### Requirement: Selección lateral con vidrio líquido neutro

La sección activa del menú lateral SHALL mostrar una cápsula de vidrio gris neutro con borde luminoso direccional, reflejos internos y sombra corta en claro y oscuro. Las filas inactivas MUST conservar su presentación sin fondo persistente. La selección MUST conservar tamaño, foco visible, navegación y movimiento existentes, independientemente del color primario.

#### Scenario: Selección y cambio de sección
- **WHEN** se abre el workspace o se navega a otra sección mediante ratón o teclado
- **THEN** una única cápsula de vidrio identifica la sección activa y se desplaza a su fila, sin cambiar dimensiones, iconos, textos ni contenido persistido

#### Scenario: Temas y acentos
- **WHEN** se usa tema claro u oscuro y se cambia el color primario
- **THEN** el material conserva su tinte neutro y el texto de selección mantiene contraste de al menos 4,5:1 sobre el fondo compuesto del sidebar

#### Scenario: Preferencias de accesibilidad
- **WHEN** se solicita transparencia reducida, contraste aumentado o movimiento reducido
- **THEN** las dos primeras preferencias reemplazan el vidrio por una cápsula sólida sin reflejos y con borde definido; movimiento reducido elimina la transición y el foco de teclado sigue visible

#### Scenario: Ancho reducido y operación local
- **WHEN** se estrecha el sidebar o la ventana sin conexión a red
- **THEN** el indicador permanece dentro del menú, no intercepta clics y la navegación sigue disponible sin nuevas solicitudes remotas, filtros persistentes ni cambios de datos
