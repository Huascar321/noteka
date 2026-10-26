# Variantes de la portada ASCII

Home conserva Photo/ASCII. En ASCII, una semilla de sesión elige el inicio y el
orden de **tiburones, koi naranjas/blancos y medusas cian/violeta**. Cada tramo
dura 42 segundos de tiempo activo, con un fundido de 1,4s al final. Las tres
variantes aparecen una vez por ciclo y no hay repetición consecutiva. No se
sortean caracteres o escenas en cada frame y no se guarda una preferencia nueva.

El renderer original de tiburones conserva geometría, rutas, colores y nado.
Los koi nuevos tienen morro redondeado, manchas fijadas al cuerpo, aletas claras,
barbillas y cola en abanico. Las medusas muestran campanas pulsantes, cuatro
lóbulos internos y tentáculos; ambas usan paletas adaptadas al tema y atlas de
caracteres monospace del sistema.

Las escenas comparten OceanClock: hasta 120 FPS sujetos al navegador/presupuesto,
tiempo activo y pausas por pestaña/portada/documento ocultos. Movimiento reducido
deja una composición estática. Resize reconstruye las capas y repinta el tiempo
actual antes de mostrar el siguiente frame; no expone un bitmap vacío ni cambia
el orden de escenas. Photo conserva su recurso local y la preferencia existente.

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
[evidencia y límites](verification-ascii-cover-scenes.md).
