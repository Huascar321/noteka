# Design system de Noteka

Guía obligatoria para crear o modificar pantallas. La implementación vigente está
en `src/style.css` y en los componentes de `src/lib/`; este documento define las
decisiones compartidas. Si una pantalla necesita apartarse de ellas, registrá
el motivo en su cambio de OpenSpec y actualizá esta guía junto con el código.

## Carácter visual

Noteka usa una interfaz de contenido sobria con controles inspirados en vidrio
de macOS. El contenido manda: notas, listas, formularios, tarjetas y editores
mantienen fondos sólidos para leer y escribir. El vidrio se reserva para la
navegación y las superficies que flotan sobre contenido. Los fondos son neutros;
el color primario elegido en Settings identifica foco, iconos y acciones. Los
colores semánticos (error, advertencia, estados de proyectos) conservan su
significado y no se usan como decoración del vidrio.

La ventana Tauri actual es opaca. El material CSS no muestra el escritorio ni
reproduce la refracción de Liquid Glass nativo. No agregues una imagen de fondo
solo para que el vidrio parezca más visible.

## Superficies y tokens

`src/style.css` define `--bg`, `--surface`, `--text`, `--muted`, `--border`,
`--accent`, `--focus` y `--selected` por tema. El acento personalizado se calcula
en `src/lib/primary-color.ts`. Para overlays, reutilizá `--glass-surface`,
`--glass-search-surface`, `--glass-breadcrumb-surface`, `--glass-breadcrumb-rim`,
`--glass-breadcrumb-sheen`, `--glass-stroke` y `--glass-blur`. No copies colores
hexadecimales del material a componentes nuevos.

| Nivel | Uso | Tratamiento |
| --- | --- | --- |
| Contenido | Notas, Tasks, Projects, Boards, formularios, tablas y tarjetas | Fondo sólido `--bg` o `--surface`; texto con `--text` y `--muted`. |
| Navegación persistente | Barra superior, sidebar y Settings | Gradientes neutros estáticos, borde fino y selección neutra; sin `backdrop-filter` grande. |
| Superficie flotante | Búsqueda, menús, picker, tooltip visible y diálogo pequeño | Material translúcido `--glass-surface`, borde `--glass-stroke`, sombra exterior e interior; filtro local `--glass-blur` si el WebView lo admite. |
| Diálogo grande y side peek | Settings y lectura/edición en paneles | Marco y navegación con acabado neutro; cuerpo sólido. |

La barra de formato abierta del editor usa el tratamiento estático de
navegación; las herramientas de selección y los menús de bloques flotantes usan
el material de overlay.

La búsqueda usa `--glass-search-surface` con un tinte más translúcido, un velo
radial y el sheen compartido, `blur(22px) saturate(1.45)`, borde luminoso y
sombras interiores; el campo, la fila activa y el pie repiten el vidrio. El panel
se mantiene compacto. Con transparencia reducida o contraste aumentado, el panel
y sus capas vuelven a fondos sólidos. La búsqueda se abre como diálogo modal: el
clic en el backdrop la cierra y los clics dentro del panel no. Mientras está
abierta, la portada ASCII de Home sigue animando; `blocked` desactiva los
controles del fondo, no el canvas. La portada ASCII comparte el fondo de Home
para evitar una línea de color al terminar el canvas; la portada fotográfica
conserva su propia imagen.

Los breadcrumbs no llevan una barra opaca propia ni reservan una fila: la barra
flota sobre el contenedor de scroll de Home y de las vistas de sección, de modo
que el contenido pasa por debajo de las cápsulas. Las migas navegables son
botones; la miga actual conserva `aria-current="page"` y no se convierte en un
control sin acción. Las cápsulas usan vidrio claro: tinte neutro mínimo, brillo
superior y desenfoque local corto de 3 px; no llevan un borde blanco. La forma se
define por el tinte, la sombra exterior y el reflejo, no por un contorno. Con
contraste aumentado se restituye un borde sólido para mantener la legibilidad. La portada ASCII admite una
cápsula un paso más transparente; la fotográfica usa el vidrio claro general.
El desenfoque y el ajuste de contraste se limitan a cada cápsula; con menor
transparencia o mayor contraste se usa un fondo sólido. El texto conserva una
sombra local para separarse del contenido que pasa por debajo. El contraste se
garantiza sobre el fondo de página medido; sobre contenido claro u oscuro en
movimiento el contraste local puede bajar, y la preferencia de transparencia
reducida restituye la superficie sólida. El efecto CSS no desplaza píxeles como
el material nativo. En páginas de detalle, la barra del breadcrumb también es
transparente y sus acciones se agrupan en una cápsula de vidrio local con el
material de overlay para mantener la legibilidad de los controles. Al volver por
una miga anterior, las migas de la derecha se desplazan y se funden en la elegida
con una transición suave; la miga destino hace un pop. Al entrar desde Home a una
sección, la miga de nivel 2 nace desde la miga Home y se desliza hasta su lugar
con la misma transición. Con movimiento reducido la animación se desactiva.

En el sidebar, sólo la sección activa usa la cápsula de vidrio; las demás filas
no llevan fondo persistente y el hover se limita a `--hover`. La cápsula activa
usa `--glass-sidebar-active`, un tinte más presente que el breadcrumb, sin
`backdrop-filter`, porque el fondo del sidebar es estático. Es un indicador
único que se desplaza entre filas con una transición elástica; con movimiento
reducido el cambio es inmediato.

Las tabs de vista de las páginas de sección (Tasks, Notes, Projects, Tags, Goals,
People y Milestones) comparten un único indicador deslizante con una curva
elástica más contenida que la del sidebar. La tab activa no pinta su propio
fondo: el indicador usa un relleno gris (`--hover`) con el brillo y la sombra
del vidrio, se desplaza y se redimensiona con el ancho de la nueva tab. El
indicador vive en el toolbar, fuera del contenedor con scroll, para que la sombra
no se recorte; sigue a la tab activa y se oculta si queda fuera del área visible.
El icono de la tab clickeada reproduce el mismo Jelly de un ciclo que el sidebar,
con un wrapper genérico que conserva su figura. Con movimiento reducido el cambio
es inmediato y con transparencia reducida el indicador vuelve a una superficie
sólida.

## Componentes e interacción

- Reutilizá los menús y pickers existentes antes de crear otro patrón de
  posición, foco o cierre. Un menú montado recibe vidrio; al cerrarse, el filtro
  desaparece. No filtres el backdrop ni el cuerpo entero de la página.
- Mantené el foco visible con `--focus` en teclado. No uses solo transparencia o
  color para señalar selección, destino de arrastre o estado; conservá borde,
  posición o etiqueta según el control.
- Botones primarios usan `--accent` y texto con contraste; botones secundarios
  y superficies de navegación son neutros. No uses el acento como fondo de
  sidebar, Home o diálogos. La acción `New` de las barras de sección usa un
  vidrio teñido con el color primario elegido (`--accent`, azul por defecto),
  con sheen, sombra y texto blanco; el fallback sólido usa `--accent`. El resto
  de botones primarios conserva `--accent`.
- Conservá la jerarquía, densidad y anchos de los componentes existentes.
  Los cambios de apariencia no deben mover el scroll, perder borradores ni
  alterar el orden de teclado.
- Los textos propios de la interfaz se escriben en inglés, según
  `docs/plan-noteka.md`. La documentación y los artefactos OpenSpec van en
  español con sus headings estructurales en inglés.
- Las barras de vistas de las páginas de sección comparten el ritmo de Tags:
  sin margen superior, `padding-bottom: 13px` y `gap: 8px`. Las secciones sin
  bloque `.task-capture` reservan además `padding-top: 44px` para que la fila de
  vistas quede a la misma altura que Tags y People. No agregues márgenes
  verticales por sección; la subsección Projects y Milestones usan el ritmo, sin
  la reserva de 44px.
- En Home, todas las barras de vistas comparten el espaciado de Tags:
  `padding: 10px 0 8px`, `margin: 0 0 5px` y `gap: 8px`. La fila de vistas
  queda 10px debajo de la tarjeta de sección y el contenido empieza 13px
  después. No uses el `padding-top: 2px` base ni márgenes particulares.
- Cada sección de listas expone una sola acción de creación contextual (`New`)
  en la barra de vistas; no dupliques botones de captura sobre la lista. Las
  filas `New page`/`New task` dentro de grupos siguen siendo acciones contextuales.
- Las tareas de las listas de Home, Tasks y el detalle de Project usan un checkbox real
  (`role="checkbox"`, `aria-checked`) separado del botón que abre la tarea. El
  pendiente es un cuadrado vacío con borde neutro y el completado un check verde;
  la diferencia no depende solo del color. El toggle actualiza `state` mediante
  `entity.update`, refresca la vista y se deshabilita mientras hay una operación
  en curso. No anides el checkbox dentro del botón de apertura.
- El Board de tareas del detalle de Project usa columnas con tinte semántico
  To Do/Doing/Done y tarjetas sólidas. El selector de estado de cada tarjeta es la
  alternativa de teclado al arrastre; ambos escriben el mismo `state` con CAS.
  El orden dentro de la columna es determinista y no se presenta como orden
  manual persistido. Calendar, comentarios y filtros no se simulan si no existen.

## Temas, accesibilidad y rendimiento

Comprobá cada superficie en temas claro y oscuro, con al menos dos colores
primarios distintos y en anchuras estrechas. El fondo de vidrio debe seguir
neutro al cambiar el color primario. Con `prefers-reduced-transparency: reduce`
o `prefers-contrast: more`, los overlays adoptan fondo sólido y se elimina su
desenfoque. Respetá `prefers-reduced-motion` y la pausa de la portada ASCII
cuando un modal bloquea Home.

Para texto normal buscá contraste de al menos 4,5:1 sobre el fondo compuesto.
Medí el resultado real o probá extremos claros y oscuros detrás de un overlay;
un valor RGBA aislado no demuestra contraste. Mantené el desenfoque acotado a
superficies abiertas y evitá filtros permanentes, animaciones decorativas y
dependencias para simular vidrio. Un cambio de CSS no prueba por sí solo que el
consumo de CPU o GPU sea igual; medilo si se hace una afirmación de rendimiento.

## Verificación al crear una pantalla

1. Elegí el nivel de superficie de la tabla y reutilizá los tokens y patrones
   existentes.
2. Revisá el tema claro y oscuro, cambio de acento, foco por teclado, lectura,
   scroll, vista estrecha y preferencias de transparencia y movimiento.
3. Ejecutá la compilación y los recorridos pertinentes. Para la navegación y
   overlays actuales: `headless-system-theme`, `headless-page-actions`,
   `headless-ascii-side-peek` y el flujo de la pantalla modificada.
4. Inspeccioná la ventana nativa cuando el acabado dependa de composición o
   de la posición real del panel; anotá lo que no se haya verificado.

El alcance y las limitaciones del rediseño inicial están en
`docs/verification-neutral-glass.md` y en el cambio OpenSpec
`refine-neutral-glass`.
