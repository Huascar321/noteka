# Spec Delta

## ADDED Requirements

### Requirement: Popups de Settings y Search con vidrio perceptible

Settings y Search SHALL compartir marco luminoso y superficie translúcida que deje percibir el contenido real detrás, con un filtro local por popup abierto. Settings MUST usar navegación de vidrio y velo de lectura más denso en su contenido. MUST conservar funciones, scroll, dimensiones y datos sin depender de red.

#### Scenario: Fondo real detrás del popup
- **WHEN** se abre Settings o Search sobre contenido con variación de color
- **THEN** esa variación se percibe a través de la superficie con desenfoque local; el fondo de ventana completo no recibe filtro y los textos permanecen legibles

### Requirement: Campos Search coherentes entre superficies

Los campos de Settings y Search SHALL compartir forma, altura compacta de 32 px, tipografía y reflejos del acceso Search lateral. MUST usar placeholder Search conservando nombres accesibles específicos, foco visible y operaciones de búsqueda/filtro existentes.

#### Scenario: Buscar en ambos popups
- **WHEN** se escribe y se opera con teclado en cualquiera de los campos
- **THEN** se conserva su búsqueda, resultados, navegación y cierre, con campo compacto y foco visible sin cambiar datos de dominio

### Requirement: Alternativas accesibles de los popups de vidrio

Los popups SHALL conservar legibilidad, foco y acceso en ambos temas, distintos acentos y ancho estrecho. Contraste aumentado o transparencia reducida MUST restituir superficies sólidas y retirar filtros/reflejos; movimiento reducido MUST retirar transiciones decorativas.

#### Scenario: Preferencias y ancho estrecho
- **WHEN** cambian tema/acento, preferencias accesibles o ancho de ventana
- **THEN** los campos y controles siguen alcanzables, la apariencia conserva neutralidad y se aplica la alternativa sólida o sin movimiento correspondiente
