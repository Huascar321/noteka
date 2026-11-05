# Iconos de interfaz

Noteka usa [Icona.works](https://icona.works/), con **85 claves lógicas** servidas
por SVG locales. Se elige **Fill** cuando existe; los controles sin esa variante
usan trazo **bold2.4**. Fill conserva el stroke1.5 del exportador oficial. No hay
fuentes de iconos, CDN, React ni descargas de iconos en ejecución.

## Presentación y datos

`AppIcon` conserva `currentColor`, tamaño y fallback. Los recortes del relleno
usan máscaras con IDs únicos por instancia, de modo que dejan ver cualquier
fondo y funcionan en ambos temas. Las claves previas siguen funcionando, incluso
cuando Icona requiere un equivalente semántico: por ejemplo, Goals usa una
bandera, Travel un avión de papel y Book un marcador. La correspondencia completa
y sus adaptaciones están en [la procedencia](../third-party/icona/README.md).

Los campos opcionales `icon`/`icon_color` y el selector siguen vigentes. Rust valida
`[a-z0-9-]`, hasta40 bytes, y `#RRGGBB`. Una clave `image-<UUID hexadecimal sin
guiones>` conserva su adjunto local original con [los límites del lector de
imágenes](tasks-reference.md). La migración visual no reescribe entidades, logos,
relaciones ni documentos. Los nombres desconocidos y las imágenes no disponibles
usan el documento de fallback. Los iconos decorativos llevan `aria-hidden`; las
etiquetas de sus botones se conservan. No se acepta SVG de una entidad.

Marca, controles nativos de ventana y marcas de estado de tareas conservan su
presentación; el toggle lateral sí usa AppIcon. El guion pendiente sigue blanco.

## Movimiento

Render, navegación desde otra procedencia y hover permanecen estáticos. Un clic
de ratón o teclado en Home, Tasks, Notes, Projects, Tags, Goals, People, Search
o Collections del lateral, o en una tab de vista de sección, carga el módulo de
movimiento e inicia el mismo **Jelly de Tasks**. La receta CSS oficial de Icona
se comparte entre las nueve figuras del lateral y las tabs, con idénticos
keyframes, intensidad y duración, sin partículas. Las tabs usan un wrapper
genérico para animar el icono propio sin sustituir su figura.

El ciclo dura **1000ms**, tras el pedido de un ritmo más lento y pausado. La curva
y la intensidad del rebote se conservan. A los1050ms se
restauran los mismos nodos estáticos y se eliminan estilos/clases temporales.
Otro clic reinicia un único ciclo global. Movimiento reducido, página oculta o
desmontaje cancelan la reproducción o carga pendiente. Los estilos, keyframes y
IDs de cada reproducción están aislados. Un fallo al cargar decoración no impide
navegar y queda registrado en consola.

## Generación y licencias

- `scripts/icon-catalog.mjs` fija revisión, variantes, adaptaciones y Jelly en los
  nueve accesos; `SIDEBAR_MOTION_DURATION_MS` centraliza su duración.
- `third-party/icona/assets.json` contiene66 variantes oficiales seleccionadas y
  nueve escenas con la misma receta Jelly; el generador deduplica en71 SVG para85
  claves y una definición CSS compartida. Las figuras y el catálogo estático se conservan.
- `npm run icons:generate` genera el catálogo estático y el módulo CSS diferido,
  validando tags/atributos/referencias locales y namespace de máscaras.
- `npm run icons:check`, incluido en build, contrasta recursos y avisos MIT en
  `public/licenses/icona/`. Funciona sin upstream, catálogo npm ni red.
- `scripts/import-icona.mjs` permite auditar/reimportar explícitamente desde un
  checkout local limpio de la revisión fijada usando el exportador oficial.
- AnimateIcons dejó de ser dependencia; sus avisos anteriores se conservan como
  historial. Ya no hace falta la excepción de prebundling para ese paquete.

## Consistencia de navegación

`SectionIcon` y `navigation.ts` comparten forma entre lateral, breadcrumb,
pestañas, Home, encabezados y captura. Las siete claves canónicas permanecen:
home, task, file-text, project, tag, trophy y person. Los encabezados/breadcrumb
siguen estáticos y el lateral recupera la misma forma al terminar el ciclo.

El acento compartido se elige en [Settings → Preferences](settings.md), con ajuste
de contraste por tema. Los colores personales de cada registro se conservan.

[Documentación oficial](https://icona.works/docs),
[licencias y equivalencias](../third-party/icona/README.md),
[ADR0055](decisions/0055-local-icona-fill-bold.md) y
[verificación inicial](verification-icona.md). La unificación posterior a Jelly
se registra en [su verificación focalizada](verification-sidebar-jelly.md).
