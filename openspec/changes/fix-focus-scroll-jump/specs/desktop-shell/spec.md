## ADDED Requirements

### Requirement: Refresco sin salto de scroll artificial

La interfaz SHALL preservar la posición visual durante el refresco de resultados existentes al recuperar el foco, sin desplazar el scroll al restaurar foco. MUST informar actualización y errores, preservar las guardias existentes y permitir el desplazamiento del usuario.

#### Scenario: Recuperar foco con resultados sin cambios

- **WHEN** se refrescan secciones visibles con resultados anteriores sin cambios de contenido
- **THEN** los indicadores transitorios no agregan altura ni desplazan el contenido y el estado de actualización permanece disponible

#### Scenario: Restaurar el foco de un control

- **WHEN** el refresco restaura el foco a un control que quedó fuera del viewport
- **THEN** no desplaza el scroll para mostrarlo
