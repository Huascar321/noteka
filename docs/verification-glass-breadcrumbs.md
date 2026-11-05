# Verificación de breadcrumbs de vidrio

29/10/2026. En la ventana nativa oscura se revisó Home con ASCII y Tasks:
el breadcrumb de Home se superpone a la portada, sin franja opaca, y el de
Tasks se integra con el fondo de la sección. La miga actual conserva su
semántica `aria-current`; las migas anteriores siguen siendo botones.

La compilación pasó con cero diagnósticos Svelte. En WebKit, el recorrido
focalizado comprobó superposición sobre el canvas, fondo transparente de barra,
fallback de contraste aumentado, navegación Tasks → Home, pausa/reanudación
del ASCII y estabilidad con side peek. La prueba de temas también pasó. Una
prueba de navegación más antigua se detuvo antes de sus comprobaciones porque
espera exactamente cinco elementos `.home-coverage`; Home actual no cumple esa
premisa. El recorrido focalizado cubrió la navegación que cambió aquí.

También se retiró la franja de las barras de detalle que incluyen breadcrumb;
sus acciones permanecen legibles en una cápsula local sin filtro en el ancestro
de menús. El recorrido focalizado final pasó 27 comprobaciones, incluidas
Tasks → Home y side peek; el de acciones de página pasó 106, con menús dentro
del viewport. Se inspeccionó de nuevo Home nativo tras la corrección.

El filtro afecta solo las cápsulas (8 px); el canvas conserva su tamaño y no se
duplica. No se midió CPU/GPU. El CSS suaviza y ajusta color/contraste, pero no
produce desplazamiento refractivo nativo de Liquid Glass.

## Seguimiento: vidrio más claro y breadcrumb superpuesto

29/10/2026. A pedido del usuario, las cápsulas se acercan al vidrio casi
transparente de la referencia. El tinte base pasó a `#f7f7f7a3` / `#2828284d`
(64 % / 30 %) y la portada ASCII a `#f7f7f799` / `#2828283d` (60 % / 24 %). Se
agregaron un borde `#ffffffd9` / `#ffffff7a`, un brillo superior y una sombra de
texto de 1 px; se conserva `blur(8px) saturate(1.2) contrast(1.08)` local. El
material anterior era 72 % / 62 % y 68 % / 50 %.

La barra dejó de reservar una fila en las vistas de sección: el pane usa una sola
fila de grid, la barra se apila con el contenido, recibe `z-index:11` y solo las
migas capturan punteros. El contenido arranca con inset propio (56 px en
`.entity-list`; 56 px y 48 px en `.tasks-page-content` angosto) y al scrollear
pasa por debajo de las cápsulas. En detalle la barra ya era sticky y
transparente; su disposición no cambió y la cápsula de acciones conserva el
material de overlay para no perder legibilidad.

La compilación pasó sin diagnósticos Svelte. En Chromium y WebKit headless, un
recorrido focalizado de 20 comprobaciones por navegador verificó en oscuro y
claro el alpha computado, la igualdad del borde superior de la barra y el scroll
container, el scroll de Goals, el solape del emblema con la cápsula y el
hit-test dentro del breadcrumb; también Home con ASCII y el fallback sólido sin
desenfoque con contraste aumentado. Las capturas temporales muestran el fondo a
través de la cápsula. No se repitió la inspección de la ventana nativa ni se
midió CPU/GPU.

Contraste compuesto sobre el fondo de página: 13,7:1 en oscuro y 11,8:1 en claro.
Sobre extremos, el tinte oscuro al 30 % sobre blanco cae a 1,5:1; el claro al
64 % sobre negro queda en 4,6:1 y el de ASCII al 60 % en 4,0:1. Por eso el
contenido que pasa por debajo puede reducir el contraste local: es el costo
explícito de la transparencia solicitada. `prefers-reduced-transparency` y
`prefers-contrast` restituyen fondo sólido sin filtro. El CSS no reproduce la
refracción nativa de Liquid Glass.

## Seguimiento: vidrio casi nítido

29/10/2026. El usuario pidió más transparencia y cuestionó el difuminado. Este
ajuste reemplaza el tinte y el filtro del seguimiento anterior: tinte base
`#f7f7f785` / `#2828282e` (52 % / 18 %) y ASCII `#f7f7f77a` / `#2828281f`
(48 % / 12 %). El filtro local pasó de `blur(8px) saturate(1.2) contrast(1.08)`
a `blur(3px) saturate(1.45) brightness(1.03)`; la sombra de texto quedó en 2 px
y el brillo superior en un velo apenas visible.

Se probó un rim direccional con `conic-gradient` y bisel interno, pero resultó
demasiado brillante y se descartó. El estado final no lleva borde blanco: la
cápsula se sostiene con el tinte, el brillo superior tenue y una sombra exterior
suave. El borde sólido `#ffffffe6` / `#ffffff40` queda sólo como fallback y para
contraste aumentado.

La misma cápsula se aplicó al sidebar únicamente en la sección activa; las demás
filas quedan simples y el hover usa `--hover`. La activa usa un tinte propio más
presente (`--glass-sidebar-active`, blanco 90 % / gris 85 %) y la misma sombra
y text-shadow que el breadcrumb, para que la selección se vea sin volver al
borde blanco. No se usa `backdrop-filter` allí porque el fondo del
sidebar es estático. La cápsula activa es un indicador único detrás de las filas;
se desplaza con `transform 280ms cubic-bezier(.3,1.45,.5,1)`. El recorrido
focalizado comprobó la alineación del indicador con el ítem activo y la
transición declarada en Chromium y WebKit; no se midió el overshoot cuadro por
cuadro. El test de temas ahora mide el contraste contra el fondo del indicador,
no contra el botón. Ambos recorridos pasaron.

Al activar una miga anterior, las migas de la derecha se desplazan hacia la
elegida con `cubicOut` (280 ms) y se funden con opacidad; la miga destino hace un
`pop` de escala sin overshoot. El recorrido focalizó un clic en Home desde Goals: a los 80 ms
la miga de sección tenía `tx` negativo y se había movido a la izquierda; a los
400 ms ya no existía y Home era la miga actual. Pasó en Chromium y WebKit. No se
midió el overshoot cuadro por cuadro. Con movimiento reducido las transiciones
se desactivan por la regla global.

La animación inversa también se comprobó: al entrar desde Home a Goals, en el
momento en que aparece la miga de sección tenía `tx` negativo y opacidad menor
que 1, es decir, nacía desde Home y viajaba a su posición. Pasó en Chromium y
WebKit; tampoco se midió el overshoot cuadro por cuadro.

La primera versión de la salida estaba invertida: Svelte reproduce las
transiciones `out` al revés y el CSS estaba escrito como si fueran `in`; la miga
volvía a aparecer y se desvanecía al final. Se corrigió la fórmula. La medición
temporal ahora muestra opacidad decreciente (0,35 → 0,001 entre 80 y 230 ms) y
`tx` alejándose (-65 → -100 px) antes de desaparecer a los 280 ms; se añadió una
comprobación de que la última muestra visible tiene menos opacidad que la
primera. Pasó en Chromium y WebKit.

Las tabs de vista usan una acción compartida (`sliding-tabs.ts`) que mide la tab
activa y mueve un único indicador con `transform` y ancho, ambos con
`280ms cubic-bezier(.3,1.12,.5,1)` (un overshoot más contenido que el sidebar). Se aplicó a Tasks, Notes, Projects, Tags,
Goals, People y Milestones. El recorrido cambió Goals de By Activity a By
Deadline: el indicador arrancó alineado, a los 80 ms ya se había desplazado a la
derecha y a los 480 ms coincidía con la nueva tab activa. Pasó en Chromium y
WebKit. El indicador usa un relleno gris (`--hover`) con sheen y sombra, sin
`backdrop-filter`; con transparencia reducida vuelve a `--hover` sólido. Para que
la sombra no se recorte, el indicador vive en el toolbar y no dentro del
contenedor con `overflow-x:auto`; se reposiciona al scrollear y se oculta si la
tab activa queda fuera del área visible. El recorrido confirmó la alineación en
ambas posiciones. No se midió CPU/GPU.

La tab clickeada reproduce el Jelly del sidebar sobre su propio icono: la acción
envuelve el SVG actual en un grupo `.fx` y le aplica la receta compartida de
1000ms, sin sustituir la figura. A los 80ms el icono tenía
`data-icon-playing="true"`, el grupo `.fx` y el estilo inyectado; el recorrido
pasó en Chromium y WebKit. El Jelly y el desplazamiento del indicador son
independientes.

La compilación pasó sin diagnósticos Svelte. El mismo recorrido focalizado de 20
comprobaciones por navegador volvió a pasar en Chromium y WebKit: alphas
computados (18 % / 12 % en oscuro; 52 % / 48 % en claro), filtro de 3 px,
barra superpuesta al scroll, solape del emblema y fallback sólido sin
desenfoque. Las capturas muestran el emblema casi nítido detrás de la cápsula.
No se repitió la inspección nativa ni se midió CPU/GPU.

El contraste sobre el fondo de página es 14,0:1 en oscuro y 11,9:1 en claro; el
emblema azul detrás de la cápsula queda en 5,2–5,8:1. Los extremos empeoran:
18 % oscuro sobre blanco cae a 1,2:1 y 52 % claro sobre negro a 3,1:1 (48 % ASCII
a 2,7:1). Es el costo de priorizar el paso del contenido; la preferencia de
transparencia reducida sigue restituyendo la superficie sólida.
