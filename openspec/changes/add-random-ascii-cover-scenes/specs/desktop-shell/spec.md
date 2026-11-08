# Spec Delta

## MODIFIED Requirements

### Requirement: Portada local y animación ASCII accesible

Home SHALL ofrecer la portada local recuperada de la referencia autorizada y una alternativa ASCII con tres variantes aleatorias: tiburones, koi con manchas del color primario y medusas. La variante de tiburones conserva nado y giros mediante cambios continuos de forma, orientación y profundidad. MUST funcionar offline en claro/oscuro, conservar la preferencia de portada separada de los datos y detener movimiento cuando se pide movimiento reducido, la portada no es visible, el documento está oculto o la pestaña está inactiva.

La variante de tiburones SHALL distinguir morro/aletas/cola, coordinar giros y ondulación posterior con impulso/deslizamiento y estabilizar caracteres sin ruido temporal ni estelas de celdas vacías. MUST reutilizar buffers y caracteres preparados, conservar velocidad basada en tiempo y programar dibujo con requestAnimationFrame hasta120 FPS, sujeto a la cadencia/presupuesto reales. Partículas, luz y bancos de peces ASCII SHALL ser ambientales, acotados y locales; sin consultas de datos, recursos remotos ni nuevos ajustes persistentes.

La presentación de tiburones SHALL seguir la referencia de video autorizada mediante vista cenital, celdas sombreadas con letras contrastantes y acentos deterministas discretos, y grilla fina de fondo. MUST incluir tiburones y peces de cuerpo compacto/alargado con geometrías y cadencias propias, conservando render procedural, estabilidad temporal y las pausas/cadencia existentes. La interfaz del video no se incorpora a Home.

La escena de tiburones SHALL seguir ambas referencias diurna y nocturna con curvatura visible
que se distribuye por el cuerpo durante los giros y llega progresivamente a la
cola; las aletas y el sombreado MUST acompañar esa postura. Los animales SHALL
conservar ritmos y trayectorias suaves propios, con tonos gris azulados y
oliva/dorados legibles según el tema, y solapamiento coherente con la profundidad.

La composición de tiburones SHALL repartir los animales por distintas zonas del ancho y
alturas de la portada, conservando posiciones variables y recorridos continuos.
MUST reducir la acumulación frecuente de tres o cuatro animales en un mismo
lugar sin reemplazar el nado por saltos o posiciones estáticas; los cruces
ocasionales siguen siendo parte de la escena.

La escena SHALL conservar los dos tiburones originales y los peces compacto/alargado e incorporar un tercer tiburón de fondo con la misma geometría/articulación. MUST tener menor escala y contraste, mayor profundidad y recorrido propio continuo, dibujado detrás de los otros animales. Bandas ASCII de luz ondulante y un relieve rocoso tenue en un solo borde SHALL completar el fondo sin recargarlo, reutilizando atlas/tiempo activo/pausas/fundido y sin cambiar koi/medusas, preferencias o datos.

#### Scenario: Home visible junto a Side peek
- **WHEN** la portada de Home permanece visible junto a un Side peek enfocado
- **THEN** la animación continúa sin transferir el foco ni los atajos a Home

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
- **WHEN** se dibuja o redimensiona la variante de tiburones en cualquiera de los temas
- **THEN** tiburones y detalle ambiental conservan geometría finita y caracteres preparados con contraste del tema, el historial se reinicia al cambiar dimensiones y no se añaden requests ni escrituras del workspace

#### Scenario: Diversidad y textura de la referencia
- **WHEN** se observa la variante de tiburones en movimiento o como composición estática
- **THEN** se distinguen las siluetas de tiburón, pez compacto y pez alargado, con sus propias aletas/colas y nado, mientras celdas/letras mantienen sombreado y acentos estables sin usar fotogramas como recurso

#### Scenario: Nado curvo y referencias de día y noche
- **WHEN** los animales giran, se cruzan o se alterna el tema durante el nado
- **THEN** cuerpo, aletas y cola forman una curva continua sin estiramientos laterales de cola ni giros rígidos, con sombreado que acompaña la postura y paleta del tema; la animación sigue siendo procedural y local

#### Scenario: Recorridos más dispersos
- **WHEN** se observa la variante de tiburones durante distintos tramos del nado, también en ancho estrecho
- **THEN** los tiburones conservan recorridos separados por el ancho y los peces pequeños alternan zonas altas/bajas con posiciones cambiantes, reduciendo agrupaciones prolongadas y conservando el estilo, nado y pausas existentes

#### Scenario: Tiburón lejano y ambiente
- **WHEN** se dibuja la variante tiburones en ambos temas, ancho estrecho, resize, fundido o movimiento reducido
- **THEN** un tercer tiburón reconocible, menor y tenue, nada detrás de los animales originales con formas finitas y ruta continua, mientras luz ASCII lenta y un relieve rocoso discreto acompañan el fondo con el mismo tiempo activo

## ADDED Requirements

### Requirement: Variantes ASCII aleatorias y locales

La portada ASCII SHALL alternar entre tiburones, koi con manchas del color primario y medusas cian/violeta con selección inicial aleatoria estable y transición gradual. MUST conservar cada escena durante un tramo visible de tiempo activo, sin repetir consecutivamente ni sortear caracteres por frame. Las tres variantes SHALL respetar temas, ancho estrecho, pausas, movimiento reducido y funcionamiento offline sin nuevos ajustes ni writes.

La variante koi SHALL incluir pececitos pequeños de fondo, algas discretas y burbujas suaves detrás de sus peces principales. MUST conservar contraste secundario, densidad acotada/adaptada al ancho y animación basada en el mismo tiempo activo, sin extender ese ambiente a tiburones.

Las manchas de los koi principales y secundarios SHALL derivar del color primario elegido en Settings, conservando su tono con una saturación atenuada (alrededor de la mitad del acento) y extremos de luminosidad legibles por tema. MUST recuperar el azul inicial ante un acento ausente o inválido y conservar el segundo tono neutro en aletas, cola, ojos y brillos. Un cambio de acento o tema SHALL repintar la portada con el tiempo activo vigente, sin datos, preferencias ni solicitudes nuevas.

Los peces del fondo SHALL compartir el diseño reconocible de los koi grandes, con manchas del color primario vigente y blanco, aletas y cola. MUST tener un tamaño secundario legible y menor contraste que indique distancia, sin reemplazarlos por símbolos mínimos ni competir con los protagonistas.

La variante medusas SHALL incluir ejemplares secundarios con el mismo diseño de campana/lóbulos/tentáculos, más burbujas y partículas tenues cian/violeta. MUST distinguir profundidad mediante escala/contraste menores, mantener densidad acotada/adaptable y compartir reloj/pausas/fundido sin alterar la apariencia de koi o tiburones.

El fondo de medusas SHALL incorporar corales ASCII en abanico en las esquinas inferiores y rayos diagonales tenues desde arriba. MUST mantener contraste secundario, centro despejado y detalle adaptado al ancho, con formas estables y balanceo/luz derivados del tiempo activo existente; sin relojes, recursos externos ni ajustes nuevos.

Koi y medusas SHALL mostrar la misma cuadrícula minimalista de fondo que tiburones. MUST conservar espaciado/origen, tinta y contraste por tema, prepararla al cambiar dimensiones/DPR/tema/tinta y dibujarla detrás de fauna/ambiente, sin nuevos relojes, recursos externos ni ajustes.

#### Scenario: Tres variantes reconocibles
- **WHEN** se observa la portada durante un ciclo de tiempo activo
- **THEN** aparecen las tres variantes, con siluetas propias y fundido entre ellas; los tiburones conservan su nado actual, los koi muestran manchas del color primario y blanco, y las medusas campanas y tentáculos

#### Scenario: Pausa y resize
- **WHEN** se oculta o pausa la portada, cambia su tamaño o se pide movimiento reducido
- **THEN** el tiempo pausado no adelanta escenas, resize conserva composición/bitmap y movimiento reducido no deja un bucle activo


#### Scenario: Koi con el color primario
- **WHEN** se elige un color primario en Settings o se alterna el tema con la escena koi visible
- **THEN** las manchas de koi principales y secundarios adoptan el tono vigente con contraste legible en claro/oscuro, conservan el segundo tono neutro y la portada se repinta sin escribir datos ni preferencias; un acento inválido recupera el azul inicial

#### Scenario: Ambiente de koi
- **WHEN** se muestra la escena koi en claro/oscuro o ancho estrecho, también durante el fundido
- **THEN** pequeños peces lejanos, algas y burbujas acompañan el fondo con formas ASCII estables y movimiento suave, sin tapar a los koi principales ni alterar las otras escenas

#### Scenario: Independencia de datos y recursos
- **WHEN** se alterna photo/ascii o se dibuja cualquiera de las escenas en ambos temas y 420px
- **THEN** se usan geometrías y fuentes locales sin solicitudes externas, y contenido, buffers y preferencia de portada permanecen intactos

#### Scenario: Profundidad de medusas
- **WHEN** se observa la variante medusas en claro/oscuro, ancho estrecho o movimiento reducido
- **THEN** medusas pequeñas reconocibles, burbujas y partículas acompañan el fondo sin competir con las principales, con formas estables y el mismo tiempo activo

#### Scenario: Corales y luz de medusas
- **WHEN** se dibuja la variante medusas en ambos temas, ancho estrecho o durante resize/fundido
- **THEN** corales en abanico y rayos tenues aportan detalle en los bordes sin ocupar el centro, conservan formas estables y el mismo tiempo activo, y no alteran las otras escenas ni los modelos de medusas

#### Scenario: Cuadrícula común a las tres escenas
- **WHEN** se muestran koi o medusas en ambos temas, ancho estrecho, resize o fundido entre variantes
- **THEN** la cuadrícula tenue coincide con la referencia de tiburones en origen/espaciado/contraste, queda detrás del ambiente y conserva el render estable y los modelos existentes
