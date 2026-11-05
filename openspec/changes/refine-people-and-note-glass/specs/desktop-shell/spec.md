# Spec Delta

## ADDED Requirements

### Requirement: Tarjetas People con marco coherente

Las tarjetas People SHALL presentar esquinas suaves, borde iluminado y sombra corta sobre contenido sólido, con reflejo tenue en hover/foco. MUST conservar fotos, colores de relaciones, nombres, apertura y densidad de galería/lista/pipeline sin blur por tarjeta.

#### Scenario: Abrir contacto local
- **WHEN** se enfoca o abre una persona desde People en cualquiera de sus presentaciones
- **THEN** el marco identifica la interacción, se abre el mismo registro y sus fotos y datos permanecen íntegros sin escritura al navegar

### Requirement: Cápsulas de relaciones legibles

Las relaciones vinculadas SHALL usar cápsulas compactas con icono/nombre y eliminación independiente. Agregar relación SHALL tener tratamiento secundario con vidrio en hover/foco. MUST conservar apertura, selección por nombre, desvinculación, guardias y estados bloqueados sin alterar entidades.

#### Scenario: Relación con borrador o bloqueo
- **WHEN** se opera una cápsula con cambios pendientes o con nota bloqueada
- **THEN** se conservan las guardias, la eliminación no abre la página y sólo las acciones permitidas permanecen habilitadas; el contenido pendiente no se guarda por un cambio visual

### Requirement: Controles de nota con material compartido

Tag y Type de Note SHALL usar cápsulas de campo tenues con foco visible. URL MUST conservar presentación de lectura y Favorite su checkbox real. Locked, Refresh, estado y menú SHALL compartir una cápsula superior conservando operaciones, semántica y guardado explícito.

#### Scenario: Edición y acciones de nota
- **WHEN** se cambia Type/Favorite o se usa la cabecera con nota guardada, pendiente, bloqueada o temporalmente desbloqueada
- **THEN** los controles y estados existentes funcionan con la nueva agrupación, los datos sólo cambian mediante sus operaciones y el menú conserva posición y teclado

#### Scenario: Tag compacto en columna estrecha y bloqueo
- **WHEN** Tag muestra un destino o está vacío en una columna estrecha, con o sin bloqueo de nota
- **THEN** conserva una sola fila de 32 px y su flecha no salta debajo del valor; el marco comparte material con Type, se atenúan los controles inhabilitados y los destinos navegables siguen legibles y accesibles

### Requirement: Accesibilidad de People y controles de nota

El acabado SHALL ser legible en ambos temas y neutro al cambiar acento. Contraste aumentado o transparencia reducida MUST ofrecer marcos sólidos sin reflejos ni filtros. Movimiento reducido MUST retirar transiciones decorativas y las vistas estrechas MUST conservar acceso sin overflow global.

#### Scenario: Preferencias y contenido largo
- **WHEN** se cambia tema/acento, se activan preferencias accesibles o se muestran nombres largos en una ventana estrecha
- **THEN** permanecen legibles foco, contenido y acciones; las cápsulas se ajustan o envuelven sin recortar controles ni desbordar la ventana
