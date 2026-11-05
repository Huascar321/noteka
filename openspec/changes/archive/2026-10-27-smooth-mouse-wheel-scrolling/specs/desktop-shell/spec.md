## ADDED Requirements

### Requirement: Desplazamiento suave de rueda de mouse

El escritorio SHALL suavizar los pasos verticales reconocibles de la rueda de mouse y conservar su dirección y distancia aproximada, con una preferencia local activada por defecto que permite regresar al comportamiento nativo. MUST respetar movimiento reducido y no interferir con zoom, desplazamiento horizontal, controles que administran rueda ni propagación en límites. Los eventos sin señal suficiente SHALL continuar con el comportamiento nativo; la detección no se presenta como identificación infalible del dispositivo.

#### Scenario: Pasos y reversión
- **WHEN** se desplaza una rueda reconocible o se invierte su dirección sobre un panel desplazable
- **THEN** el contenido se mueve suavemente en la dirección indicada, acumulando pasos sin saltar fuera del panel

#### Scenario: Eventos nativos y límites
- **WHEN** se usa entrada precisa no identificada, zoom, desplazamiento horizontal, un control especial o se llega al límite
- **THEN** se conserva la acción nativa y el panel exterior puede recibir el desplazamiento según sus reglas

#### Scenario: Preferencia y movimiento reducido
- **WHEN** se desactiva la opción o el sistema solicita movimiento reducido
- **THEN** el desplazamiento usa el comportamiento nativo; la preferencia local se conserva y un error de guardado se informa sin descartar la elección de sesión
