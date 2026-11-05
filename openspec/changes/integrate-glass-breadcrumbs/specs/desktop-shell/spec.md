## ADDED Requirements

### Requirement: Breadcrumbs integrados con el contenido

El shell SHALL presentar breadcrumbs sin una franja de fondo separada y superpuestos al contenido desplazable en Home y las vistas de sección. MUST mantener navegación, foco, lectura y posición estable del contenido.

#### Scenario: Home con portada ASCII

- **WHEN** la persona abre Home con la portada ASCII
- **THEN** la animación aparece detrás de una miga de vidrio neutro y no hay franja opaca entre navegación y portada

#### Scenario: Vista de sección

- **WHEN** la persona abre una vista de sección
- **THEN** el breadcrumb se integra con el fondo de esa vista y cada miga conserva su identidad e interacción

#### Scenario: Contenido bajo el breadcrumb

- **WHEN** la persona desplaza el contenido de una vista de sección
- **THEN** el contenido pasa por debajo de las cápsulas de vidrio sin una fila reservada y las migas conservan su interacción

#### Scenario: Entrar a una sección desde Home

- **WHEN** la persona abre una sección desde Home
- **THEN** la miga de nivel 2 nace desde la miga Home y se desplaza a su posición con una transición suave

#### Scenario: Volver por el breadcrumb

- **WHEN** la persona activa una miga anterior
- **THEN** las migas de la derecha se desplazan y se funden en la miga elegida con una transición suave
- **AND** con movimiento reducido el cambio es inmediato

#### Scenario: Transparencia reducida

- **WHEN** el entorno expone la preferencia de transparencia reducida o contraste aumentado
- **THEN** las migas usan una superficie sólida legible sin desenfoque

### Requirement: Sección activa del sidebar

El shell SHALL aplicar la cápsula de vidrio compartida sólo a la sección activa del sidebar. MUST conservar la navegación, el foco y la lectura de las demás filas.

#### Scenario: Sección activa

- **WHEN** la persona abre una sección desde el sidebar
- **THEN** sólo la fila activa muestra la cápsula de vidrio y las demás conservan su fila simple

#### Scenario: Cambio de sección

- **WHEN** la persona elige otra sección desde el sidebar
- **THEN** la cápsula activa se desplaza hasta la nueva fila con una transición suave
- **AND** con movimiento reducido el cambio es inmediato

### Requirement: Tabs de vista deslizantes

El shell SHALL indicar la vista activa de las tabs de sección con un único indicador que se desplaza y redimensiona al cambiar de vista. MUST conservar la navegación, el foco y la lectura de las etiquetas.

#### Scenario: Cambio de vista

- **WHEN** la persona elige otra vista en las tabs de sección
- **THEN** el indicador se desplaza desde la vista anterior hasta la nueva y adopta su ancho
- **AND** el icono de la vista elegida reproduce un ciclo Jelly
- **AND** con movimiento reducido el cambio es inmediato

#### Scenario: Transparencia reducida en el sidebar

- **WHEN** el entorno expone la preferencia de transparencia reducida o contraste aumentado
- **THEN** la cápsula activa usa una superficie sólida sin sombra ni desenfoque
