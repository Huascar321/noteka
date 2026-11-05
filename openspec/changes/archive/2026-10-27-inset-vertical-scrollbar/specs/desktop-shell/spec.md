## MODIFIED Requirements

### Requirement: Barras de desplazamiento discretas

El escritorio SHALL presentar barras de desplazamiento finas y discretas, compatibles con los temas claro y oscuro, sin impedir el desplazamiento ni mostrar barras ocultas deliberadamente.

#### Scenario: Superficie desplazable
- **WHEN** se desplaza una vista vertical u horizontal con contenido desbordado
- **THEN** la barra conserva funcionamiento nativo, con tirador vertical de 7 px de tinta, un margen transparente derecho de 2 px dentro de un canal de 10 px y canal horizontal de 8 px en motores compatibles, pista transparente y contraste adaptado al tema

#### Scenario: Barra oculta intencionalmente
- **WHEN** una superficie oculta su barra para presentar otra navegación
- **THEN** la regla general no vuelve a mostrarla
