# Spec Delta

## MODIFIED Requirements

### Requirement: Lateral plegable sin reconstruir pestañas

Expandir y contraer SHALL usar una transición breve y reversible que desplace el lateral y ajuste el ancho del contenido en escritorio, y desplace el panel superpuesto en ancho estrecho, sin comprimir sus etiquetas. MUST omitir esa transición con movimiento reducido. El control superior SHALL expandir o contraer el lateral en escritorio y abrirlo/cerrarlo como panel superpuesto en ancho estrecho. MUST reflejar su estado accesible, retirar del foco la navegación oculta y conservar las instancias, buffers, selección y scroll de todas las pestañas. Plegar el lateral o cambiar el ancho SHALL ser una operación de presentación sin lecturas ni escrituras de entidades ni reinicio de la aplicación.

El lateral SHALL ajustar sus controles y borde de resize al ancho disponible sin scrollbar ni desplazamiento horizontal. MUST conservar scroll vertical cuando la altura no alcanza y mantener el ajuste con puntero/teclado sin ampliar su límite de ancho.

#### Scenario: Plegar con una edición pendiente
- **WHEN** se alterna el lateral con una página o borrador abierto
- **THEN** el área principal aprovecha el ancho liberado y el mismo panel conserva su contenido, revisión y estado sin guardar, descartar ni recrear la pestaña

#### Scenario: Panel estrecho y teclado
- **WHEN** se abre la navegación en ancho estrecho y se cierra con su control o Escape
- **THEN** la navegación puede usarse por teclado, el foco vuelve al control al cerrar con Escape y los controles ocultos dejan de participar del orden de tabulación

#### Scenario: Transición bidireccional e inversión
- **WHEN** se expande o contrae el lateral, o se invierte la acción antes de terminar
- **THEN** el movimiento continúa desde la posición visible sin saltos al destino, conserva paneles/buffers y retira inmediatamente del teclado la navegación que se cierra

#### Scenario: Movimiento reducido
- **WHEN** se alterna el lateral con preferencia de movimiento reducido
- **THEN** el estado final se aplica sin transición y mantiene el contenido y las mismas reglas de foco

#### Scenario: Lateral angosto o de poca altura
- **WHEN** se estrecha el menú, se arrastra su borde o la ventana no tiene altura para todas las acciones
- **THEN** no aparece scroll horizontal, navegación/Settings/foco siguen accesibles mediante scroll vertical y el borde ajustable permanece operativo
