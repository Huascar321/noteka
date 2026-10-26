# Spec Delta

## MODIFIED Requirements

### Requirement: Portada local y animación ASCII accesible

Home SHALL ofrecer la portada local recuperada de la referencia autorizada y una alternativa ASCII que reúne tiburones, koi naranja/blanco y medusas cian/violeta en una composición continua de profundidades compartidas. Los tiburones conservan nado y giros mediante cambios continuos de forma, orientación y profundidad. MUST funcionar offline en claro/oscuro, conservar la preferencia de portada separada de los datos y detener movimiento cuando se pide movimiento reducido, la portada no es visible, el documento está oculto o la pestaña está inactiva.

La alternativa ASCII SHALL distinguir morro/aletas/cola, coordinar giros y ondulación posterior con impulso/deslizamiento y estabilizar caracteres sin ruido temporal ni estelas de celdas vacías. MUST reutilizar buffers y caracteres preparados, conservar velocidad basada en tiempo y programar dibujo con requestAnimationFrame hasta120 FPS, sujeto a la cadencia/presupuesto reales. Partículas, luz y bancos de peces ASCII SHALL ser ambientales, acotados y locales; sin consultas de datos, recursos remotos ni nuevos ajustes persistentes.

La presentación SHALL seguir la referencia de video autorizada mediante vista cenital, celdas sombreadas con letras contrastantes y acentos deterministas discretos, y grilla fina de fondo. MUST incluir tiburones y peces de cuerpo compacto/alargado con geometrías y cadencias propias, conservando render procedural, estabilidad temporal y las pausas/cadencia existentes. La interfaz del video no se incorpora a Home.

La escena SHALL seguir ambas referencias diurna y nocturna con curvatura visible
que se distribuye por el cuerpo durante los giros y llega progresivamente a la
cola; las aletas y el sombreado MUST acompañar esa postura. Los animales SHALL
conservar ritmos y trayectorias suaves propios, con tonos gris azulados y
oliva/dorados legibles según el tema, y solapamiento coherente con la profundidad.

La composición SHALL repartir los animales por distintas zonas del ancho y
alturas de la portada, conservando posiciones variables y recorridos continuos.
MUST reducir la acumulación frecuente de tres o cuatro animales en un mismo
lugar sin reemplazar el nado por saltos o posiciones estáticas; los cruces
ocasionales siguen siendo parte de la escena.

#### Scenario: Foto y ASCII en ambos temas
- **WHEN** se alterna foto/ASCII y tema con edición pendiente en otra pestaña
- **THEN** la portada elegida se muestra con recursos empaquetados y contraste propio del tema, y no se escriben ni descartan entidades ni buffers

#### Scenario: Movimiento reducido y desmontaje
- **WHEN** se activa movimiento reducido, se oculta la portada o se cierra su pestaña
- **THEN** queda una composición estática o se suspende el render, sin un bucle de animación activo para contenido oculto ni listeners retenidos tras desmontar

#### Scenario: Cadencia y reanudación
- **WHEN** el navegador entrega callbacks a distintas frecuencias o la animación se reanuda tras una pausa
- **THEN** se dibuja como máximo a120 FPS, el desplazamiento depende del tiempo activo y no de la cantidad de cuadros, y la pausa no produce saltos de recuperación

#### Scenario: Escena ambiental estable
- **WHEN** se dibuja o redimensiona la portada en cualquiera de los temas
- **THEN** tiburones y detalle ambiental conservan geometría finita y caracteres preparados con contraste del tema, el historial se reinicia al cambiar dimensiones y no se añaden requests ni escrituras del workspace

#### Scenario: Diversidad y textura de la referencia
- **WHEN** se observa la portada en movimiento o como composición estática
- **THEN** se distinguen las siluetas de tiburón, pez compacto y pez alargado, con sus propias aletas/colas y nado, mientras celdas/letras mantienen sombreado y acentos estables sin usar fotogramas como recurso

#### Scenario: Nado curvo y referencias de día y noche
- **WHEN** los animales giran, se cruzan o se alterna el tema durante el nado
- **THEN** cuerpo, aletas y cola forman una curva continua sin estiramientos laterales de cola ni giros rígidos, con sombreado que acompaña la postura y paleta del tema; la animación sigue siendo procedural y local

#### Scenario: Recorridos más dispersos
- **WHEN** se observa la portada durante distintos tramos del nado, también en ancho estrecho
- **THEN** los tiburones conservan recorridos separados por el ancho y los peces pequeños alternan zonas altas/bajas con posiciones cambiantes, reduciendo agrupaciones prolongadas y conservando el estilo, nado y pausas existentes

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


## ADDED Requirements

### Requirement: Composición mixta y profundidad ASCII

La portada ASCII SHALL mostrar juntos tiburones, koi y medusas, con varios tamaños y contraste según profundidad. El experimento SHALL sustituir la alternancia temporal anterior por una escena continua con inicio/composición estables por instancia. MUST ordenar animales de todas las familias del fondo al frente, conservar geometría y trayectorias continuas y reducir la densidad en anchos estrechos sin perder ninguna familia.

#### Scenario: Fauna compartida
- **WHEN** se observa la portada en movimiento o con movimiento reducido
- **THEN** las tres familias están presentes simultáneamente y se distinguen siluetas/tamaños/colores, con animales discretos detrás y otros definidos delante

#### Scenario: Cruces y estabilidad
- **WHEN** se cruzan animales de distintas familias o se redimensiona/cambia tema
- **THEN** el orden responde a profundidad compartida, conserva composición y bitmap sin escenas alternadas, ruido sorteado por frame ni estelas

#### Scenario: Independencia y pausas
- **WHEN** se alterna Photo/ASCII, se oculta la portada/documento o la pestaña queda inactiva
- **THEN** se conservan preferencia, pausas, reloj activo y borradores sin requests externos, writes de entidades o nuevos ajustes persistentes

## REMOVED Requirements

### Requirement: Variantes ASCII aleatorias y locales

**Reason**: El usuario reemplazó las tres escenas temporales del cambio previo por una composición compartida continua.

**Migration**: Sin cambios de datos o preferencias. Aplicar el delta anterior `add-random-ascii-cover-scenes` antes de este al sincronizar, porque allí se añadió este requisito; este cambio lo retira y lo sustituye por Composición mixta y profundidad ASCII. El comportamiento anterior sigue recuperable en `2d43a66`.
