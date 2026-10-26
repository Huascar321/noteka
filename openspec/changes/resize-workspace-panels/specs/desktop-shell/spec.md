# Spec Delta

## ADDED Requirements

### Requirement: Anchos ajustables y persistentes de paneles

El menú izquierdo y Side peek SHALL permitir ajustar su ancho desde el borde con puntero y teclado, conservar el ancho local elegido y recuperarlo en la próxima apertura o inicio. MUST limitar el menú a 35% y Side peek a 65% del ancho de la ventana, también al redimensionarla. Preferencias inválidas usan defaults seguros; fallos de escritura SHALL ser recuperables sin modificar documentos.

#### Scenario: Ajuste y reapertura
- **WHEN** se arrastra un borde o se ajusta por teclado, se cierra y vuelve a abrir el panel o se reinicia la aplicación
- **THEN** se recupera el ancho confirmado dentro de sus límites, independientemente del otro panel y sin crear revisiones de entidades

#### Scenario: Límites y ventana menor
- **WHEN** se arrastra más allá del máximo o se reduce la ventana con una preferencia mayor guardada
- **THEN** el menú no supera 35% y Side peek no supera 65%; el ancho preferido permanece recuperable al ampliar la ventana y no hay overflow global

#### Scenario: Cancelación o preferencia inaccesible
- **WHEN** se cancela el arrastre con Escape, se pierde el puntero o falla el almacenamiento local
- **THEN** cancelar recupera las dimensiones anteriores; un valor inválido usa fallback, un fallo de escritura se informa y la sesión sigue operable con sus borradores intactos

### Requirement: Exclusión de navegación y Side peek ancho

Un Side peek visible con ancho mayor al 35% de la ventana MUST ocultar el menú izquierdo. Abrir el menú en ese estado SHALL ocultar el peek sin cerrar, guardar, descartar ni recrear su editor. Reabrir ese mismo detalle SHALL recuperar el buffer y cerrar el menú. Abrir otro destino o abandonar la sesión MUST incluir los peeks ocultos en las guardias existentes.

#### Scenario: Cruzar el umbral
- **WHEN** el peek se amplía desde un ancho menor o igual a 35% hasta uno mayor con el menú abierto
- **THEN** el menú se contrae; a 35% exacto se permite coexistencia y reducir después no abre el menú automáticamente

#### Scenario: Alternancia conservadora
- **WHEN** se abre el menú con un peek ancho visible y luego se elige ese mismo detalle desde el origen
- **THEN** el peek se oculta y reaparece con el mismo editor, ancho, buffer e historial; al reaparecer se oculta el menú y cada panel conserva su scroll

#### Scenario: Otro destino y cierre agregado
- **WHEN** un peek oculto tiene ediciones y se elige otra entidad o se solicita cerrar pestaña, workspace o host
- **THEN** su guardia se hace visible antes de continuar; cancelar o fallar guardar conserva el buffer y no confirma el cierre

#### Scenario: Ventana o pestaña cambian
- **WHEN** cambia el viewport o se activa una pestaña con Side peek ancho
- **THEN** se vuelven a aplicar los límites y la exclusión sin intercambiar buffers ni persistir ocultamiento como contenido

### Requirement: Movimiento reversible de Side peek

Side peek SHALL expandirse y contraerse con transición breve y reversible equivalente al lateral, ajustando el espacio del origen sin escalar texto. Ocultar o animar MUST conservar la instancia del editor y retirar inmediatamente foco/atajos del contenido que se cierra. MUST omitir transición con movimiento reducido y durante el ajuste por arrastre.

#### Scenario: Abrir, ocultar e invertir
- **WHEN** se abre u oculta el peek o se invierte esa acción antes de finalizar
- **THEN** el movimiento continúa desde la posición visible, el origen se ajusta y contenido/historial/revisiones permanecen sin writes ni lecturas por animar

#### Scenario: Cierre explícito
- **WHEN** se confirma guardar o descartar y cerrar el peek
- **THEN** primero se resuelve la guardia, después se contrae y finalmente se libera su instancia; Cancel conserva el detalle visible

#### Scenario: Movimiento reducido y arrastre
- **WHEN** se usa movimiento reducido o se arrastra un borde
- **THEN** el layout sigue el destino directamente sin interpolación ni pérdida de foco, selección o contenido
