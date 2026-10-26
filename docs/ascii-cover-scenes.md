# Composición de la portada ASCII

Home conserva Photo/ASCII. El experimento `combine-ascii-ocean-life` reúne
**tiburones, koi naranjas/blancos y medusas cian/violeta** en una escena continua.
Una semilla por instancia conserva fases y marcas; no se sortean caracteres por
frame ni se añade una preferencia. Hay 11 animales en ancho amplio y nueve en
estrecho, contando los peces compactos/alargados del océano original.

Cada individuo tiene profundidad propia, que controla tamaño, opacidad y orden de
dibujo. Los lejanos quedan más pequeños y discretos; los cercanos, más definidos.
Todas las familias se intercalan: ninguna especie es una capa siempre delante.
Las rutas reparten centros por el ancho y altura, con cruces ocasionales. Los
animales compartidos conservan su recorrido y profundidad al pasar el umbral de
densidad estrecha.

El renderer de tiburones conserva geometría, colores y nado; expone dibujo por
individuo para la composición compartida y mantiene su API anterior.
Los koi nuevos tienen morro redondeado, manchas fijadas al cuerpo, aletas claras,
barbillas y cola en abanico. Las medusas muestran campanas pulsantes, cuatro
lóbulos internos y tentáculos; ambas usan paletas adaptadas al tema y atlas de
caracteres monospace del sistema.

La composición usa OceanClock: hasta 120 FPS sujetos al navegador/presupuesto,
tiempo activo y pausas por pestaña/portada/documento ocultos. Movimiento reducido
deja una composición estática. Resize reconstruye las capas y repinta el tiempo
actual antes de mostrar el siguiente frame, sin exponer un bitmap vacío. Photo
conserva su recurso local y la preferencia existente.

El estado anterior de tres escenas alternadas cada 42s con fundido de 1,4s está
preservado en el commit local `2d43a66`. La rama `codex/combined-ascii-ocean`
contiene la prueba combinada; el historial permite recuperar la versión anterior
sin modificar los datos del workspace. Ver
[decisión](decisions/0053-shared-depth-ascii-composition.md) y
[evidencia actual](verification-mixed-ocean.md).

## References and attribution

- Koi: [Dao-Ho/portfolio — koi-pond](https://github.com/Dao-Ho/portfolio/tree/dbbabb95430663f7d27371441ab013ecfad2dafc/src/app/components/koi-pond),
  commit `dbbabb95430663f7d27371441ab013ecfad2dafc`, revisado el 26/10/2026.
  Se tomó como referencia la silueta cenital y el contraste naranja/blanco; el
  renderer nuevo usa geometría y código locales propios. El árbol observado no
  incluye licencia general del código; no se incorporó su código ni sus assets.
- Medusas: ZIP **Kurage Dashboard 1.0.0**, HorizonX, aportado por el usuario.
  Se revisó la apariencia procedural y los créditos, sin ejecutar AGENTS.md,
  CLAUDE.md ni instrucciones del template. Implementación propia para Noteka;
  no se incluyen su interfaz, fuentes Google, servicios o archivos originales.
  La guía del ZIP permite adaptación y restringe redistribuir su fuente original;
  las referencias conservadas están en evidencia local ignorada por Git.

No hay solicitudes de red durante el render ni acceso al workspace. Ver
[evidencia anterior de variantes](verification-ascii-cover-scenes.md).
