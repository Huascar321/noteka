# Variantes de la portada ASCII

Restaurada el26/10/2026 por preferencia del usuario después de probar la mezcla.
Se recuperó la base de `2d43a66` y después se enriquecieron los ambientes de koi
y medusas por separado.
La corrección del lateral sin scrollbar horizontal se conserva. El experimento
mixto queda como antecedente.

Home conserva Photo/ASCII. En ASCII, una semilla de sesión elige el inicio y el
orden de **tiburones, koi con manchas del color primario y medusas cian/violeta**.
Cada tramo dura 42 segundos de tiempo activo, con un fundido de 1,4s al final. Las
tres variantes aparecen una vez por ciclo y no hay repetición consecutiva. No se
sortean caracteres o escenas en cada frame y no se guarda una preferencia nueva.

Las tres escenas comparten la cuadrícula minimalista del fondo: líneas de0,5px
cada12px con refuerzo cada60px, misma tinta y opacidad tenue por tema. Koi y
medusas la preparan al redimensionar y la componen detrás de todo su ambiente,
alineada con la de tiburones también durante el fundido.

Los dos tiburones principales y peces compacto/alargado conservan geometría,
rutas, colores y nado. Un tercer tiburón usa la misma malla, con menor escala y
opacidad, una ruta alta central propia y mayor profundidad para dibujarse detrás.
Tres bandas ASCII de luz ondulante y un relieve rocoso tenue en la esquina
inferior derecha completan el ambiente, con geometría preparada y atlas/reloj
existentes; no ocupan el centro con relieves ni añaden fauna a las otras escenas.
Los koi nuevos tienen morro redondeado, manchas fijadas al cuerpo, aletas claras,
barbillas y cola en abanico. Las manchas —también las de los koi secundarios—
derivan del color primario elegido en Settings: conservan el tono con la mitad
de la saturación del acento y un rango claro/oscuro contenido por tema, para
mantener legibilidad sin que el pez se vuelva un bloque de color. Aletas, cola, barbillas, ojos y brillos mantienen el
segundo tono neutro del koi, de modo que la silueta siga leyéndose con el azul
inicial, cualquier sugerencia o un hexadecimal personalizado. Un acento ausente
o inválido recupera el azul inicial. Cambiar el acento o el tema repinta la
portada con el tiempo activo vigente, sin remontar la escena, consultar la red
ni guardar una preferencia nueva. Las medusas muestran campanas pulsantes, cuatro
lóbulos internos y tentáculos; ambas usan paletas adaptadas al tema y atlas de
caracteres monospace del sistema.

El fondo de koi incluye koi secundarios que cruzan las zonas altas/bajas, algas
tenues ancladas al borde inferior con balanceo y burbujas que ascienden. En ancho
amplio hay8 koi secundarios,7 grupos de algas y12 burbujas; en estrecho5/4/6.
Los secundarios comparten exactamente cuerpo, manchas, aletas y cola con los
grandes, incluido el tinte primario vigente. Miden aproximadamente67–90px en
1000px y42–56px en420px, con opacidad
0,32–0,46 y entrada/salida gradual por los bordes para señalar distancia.
Se pintan primero y los protagonistas después, reutilizando el mismo modelo,
campo y atlas. Los caracteres se preparan por tema/DPR y las posiciones derivan
de la misma semilla/tiempo activo; no hay otro bucle o temporizador. El ambiente
participa en el fundido existente.

La escena de medusas tiene cinco ejemplares secundarios en ancho amplio y tres
en estrecho, con la misma campana, lóbulos y tentáculos de las principales.
Sus radios son aproximadamente25–32px en1000px y16–21px en420px, con opacidad
menor para señalar distancia. Diez burbujas y36 partículas cian/violeta acompañan
el fondo (5/18 en estrecho). Dos pasadas comparten campo/atlas y modelo; no se
mezcla fauna entre escenas ni se añaden relojes o ajustes de usuario.

Dos corales ASCII en abanico se anclan a las esquinas inferiores, parcialmente
recortados por el borde, con ramificaciones cian/violeta y balanceo lento.
Dos rayos diagonales en estrecho/tres en amplio parten de arriba y pierden
intensidad con la profundidad. Ambos detalles quedan en las zonas laterales y
dejan libre el centro. Sus celdas se preparan al cambiar dimensiones y reutilizan
el atlas local; comparten tiempo activo/fundido y se pintan detrás de las medusas.

Las escenas comparten OceanClock: hasta 120 FPS sujetos al navegador/presupuesto,
tiempo activo y pausas por pestaña inactiva, portada fuera de vista, documento
oculto o cobertura completa de Full page. Movimiento reducido deja una
composición estática. Mientras la portada siga visible, la animación continúa
aunque el foco esté en un Side peek, un Center peek, Search o Settings; `blocked`
deshabilita controles del fondo pero no detiene el canvas, y los atajos siguen
perteneciendo al panel enfocado. Resize reconstruye las capas y repinta el tiempo
actual antes de mostrar el siguiente frame; no expone un bitmap vacío ni cambia
el orden de escenas. Photo conserva su recurso local y la preferencia existente.

## References and attribution

- Koi: [Dao-Ho/portfolio — koi-pond](https://github.com/Dao-Ho/portfolio/tree/dbbabb95430663f7d27371441ab013ecfad2dafc/src/app/components/koi-pond),
  commit `dbbabb95430663f7d27371441ab013ecfad2dafc`, revisado el 26/10/2026.
  Se tomó como referencia la silueta cenital y el contraste naranja/blanco; el
  renderer nuevo usa geometría y código locales propios. Desde el 05/11/2026 el
  naranja de referencia se sustituye por el color primario vigente. El árbol
  observado no incluye licencia general del código; no se incorporó su código ni
  sus assets.
- Medusas: ZIP **Kurage Dashboard 1.0.0**, HorizonX, aportado por el usuario.
  Se revisó la apariencia procedural y los créditos, sin ejecutar AGENTS.md,
  CLAUDE.md ni instrucciones del template. Implementación propia para Noteka;
  no se incluyen su interfaz, fuentes Google, servicios o archivos originales.
  La guía del ZIP permite adaptación y restringe redistribuir su fuente original;
  las referencias conservadas están en evidencia local ignorada por Git.

No hay solicitudes de red durante el render ni acceso al workspace. Ver
[evidencia y límites](verification-ascii-cover-scenes.md).
