# Verificación de las tres variantes ASCII

La pausa histórica por pérdida de foco del panel fue modificada para Home visible
junto a Side peek; ver [verificación del seguimiento](verification-ascii-side-peek.md).

La versión de tres escenas de `2d43a66` volvió a estar activa por preferencia
del usuario el26/10/2026. El experimento mixto se descartó; su
[evidencia](verification-mixed-ocean.md) se conserva como antecedente.

Fecha: 26/10/2026. Cambio: `add-random-ascii-cover-scenes`. Implementado y
verificado localmente; deltas pendientes de sincronización/archivo.

## Shark depth and habitat follow-up

Seguimiento26/10/2026: un tercer tiburón usa la misma geometría/articulación,
con escala base46 frente a84/65, opacidad aproximada0,38–0,44, mayor profundidad
y ruta propia lenta en la zona alta central. Conserva los índices/poses de los
cuatro animales originales. El fondo añade tres bandas ASCII de luz ondulante
y un relieve rocoso tenue recortado en la esquina inferior derecha; celdas
preparadas por tamaño y atlas/tiempo activo existentes, sin otro reloj o recurso.

Doce pruebas puras aprobaron: las once existentes y una sobre el tercer tiburón,
menor/tenue/más profundo y con posición continua, en0–180s y1000/420px. Build
aprobado con0 diagnósticos Svelte y aviso previo de bundle conservado. Doce
casos visuales de escenas ×temas ×anchos aprobaron estabilidad, paletas y resize
con repintado inmediato; capturas amplias/estrechas revisadas.

Diecinueve observaciones adicionales: diez comparaciones exactas de las cuatro
poses originales en0/8/24/42/180s; ocho combinaciones temas/anchos/DPR1/2 con
malla compartida, tiburón nuevo dibujado primero, grilla idéntica, relieve
confinado al borde y render original exacto al retirar sólo las adiciones;
una de luz animada, composición estable a tiempo fijo y geometría cacheada.
Siete módulos de koi/medusas/selector/reloj/componente conservan su hash y las
ocho capturas koi/medusas son idénticas al seguimiento anterior de cuadrícula.

Nueve muestras de fundido no vacías y seis direcciones medidas por separado:
Canvas aislado1000×288/DPR1,80 muestras, tiburones p50=3,1ms/p95=3,3ms;
p95 máximo de fundido=6,1ms. No acredita120Hz físicos. Sin errores de navegador,
requests externos o aceptación nativa nueva; no se repite la regresión histórica
de App/Rust. Evidencia local ignorada: `output/playwright/shark-habitat-20261026/`.

## Shared grid follow-up

Seguimiento26/10/2026: koi y medusas reciben la misma cuadrícula de tiburones:
líneas de0,5px cada12px/refuerzo60px, tinta de la portada y opacidad por tema.
Canvas preparado por tamaño/DPR/tema/tinta, compuesto detrás de fauna/ambiente;
sin trazado por frame o nuevos relojes. Tiburones conserva su implementación.

Build aprobado con0 diagnósticos Svelte y el aviso previo de bundle. Once
pruebas existentes aprobaron; doce combinaciones de escenas ×temas ×1000/420px
aprobaron estabilidad, paletas y repintado inmediato durante resize; capturas
revisadas. Dieciséis combinaciones koi/medusas ×temas ×anchos ×DPR1/2 verifican
grillas pixel idénticas a la de tiburones, respuesta a tinta distinta y bitmap
estable; al ocultar solamente la grilla, fauna/ambiente coinciden píxel a píxel
con el baseline inmediato anterior. Nueve muestras de fundido no vacías.
Sin errores de navegador o requests externos; sin aceptación nativa nueva.
Canvas aislado1000×288/DPR1: p95 koi2,2ms/medusas3,1ms; no acredita120Hz físicos
ni vuelve a medir todas las direcciones de fundido. Evidencia local ignorada:
`output/playwright/ascii-grid-20261026/`.

## Coral and light follow-up

Seguimiento26/10/2026: dos corales ASCII en abanico en las esquinas inferiores,
ramificados y con balanceo lento; dos rayos tenues en estrecho/tres en amplio,
diagonales y con pérdida gradual de intensidad. Geometría preparada por tamaño,
atlas por tema/DPR, mismo tiempo activo, sin reloj/listener o recurso nuevo.

Build aprobado con0 diagnósticos Svelte y aviso previo de bundle conservado.
Once pruebas existentes y doce casos visuales de tres escenas ×dos temas ×
1000/420px aprobaron estabilidad, resize no vacío y paletas; capturas revisadas.
Veintiuna observaciones: ocho koi pixel idénticas al baseline inmediato, ocho
medusas con píxeles ambientales añadidos y cobertura alfa conservada (tolerancia
1/255), una de estabilidad/movimiento del ambiente y cuatro de geometría finita,
contraste secundario, caché estable entre frames y centro despejado.
Nueve muestras de transición no vacías y seis direcciones de fundido aprobadas.
Canvas aislado1000×288/DPR1,80 muestras: medusas p50=2,8ms/p95=3,1ms;
p95 máximo de fundido=6,3ms. No demuestra cadencia física120Hz. Sin errores de
navegador ni requests externos; modelos de fauna, selector/reloj y preferencias
conservados. Sin nueva aceptación nativa ni regresión histórica de App/Rust.
Evidencia local ignorada: `output/playwright/jelly-coral-20261026/`.

## Jellyfish habitat follow-up

Seguimiento26/10/2026: cinco medusas secundarias en ancho amplio/tres en estrecho,
radio aproximado25–32px/16–21px y opacidad0,30–0,42. El mismo método de campana,
lóbulos y tentáculos dibuja primero las secundarias y después las principales.
Diez burbujas y36 partículas cian/violeta (5/18 en estrecho) usan un atlas local
y el mismo tiempo activo; no hay otro RAF, temporizador o listener.

Once pruebas existentes aprobaron. Doce combinaciones visuales de las tres
escenas, dos temas y1000/420px aprobaron bitmap estable, resize y paletas; se
revisaron capturas amplias/estrechas. Nueve muestras del fundido aprobaron.

Diecinueve observaciones adicionales: ocho comparaciones de koi idénticas a la
versión que gustó al usuario; ocho de medusas con fondo añadido y cobertura de
las principales conservada (tolerancia alfa1/255); dos trazas de instancia real
que verifican el mismo método de dibujo para5/3 secundarias menores/tenues y
4/3 principales con opacidad1, más geometría finita a0/8/42/180s; una observación
de ambiente estable al mismo tiempo y animado al avanzar tiempo.

Dibujo aislado de80 muestras a1000×288/DPR1: medusas p50=2,2ms/p95=2,5ms. Las
seis direcciones de fundido se midieron separadamente; p95 máximo=4,6ms en
koi→medusas. Son tiempos Canvas aislados, no aceptación física120Hz. Build
aprobado con0 diagnósticos Svelte y aviso previo de bundle conservado. Sin
errores de navegador o requests externos. Koi, tiburones, selector/reloj y
preferencias se conservan; no se repitió la suite histórica de App o pantalla
nativa. Evidencia privada/ignorada en `output/playwright/jelly-habitat-20261026/`.

## Readable background koi follow-up

Seguimiento26/10/2026: los peces de fondo usan el mismo método de geometría,
manchas, aletas y cola que los protagonistas. Ocho secundarios en ancho amplio/
cinco en estrecho, longitudes aproximadas67–90px/42–56px y opacidad0,32–0,46,
más el fundido de bordes. El grupo secundario y el principal comparten campo y
atlas, pintados en dos pasadas; no hay otro renderer por pez ni reloj.

Once pruebas existentes y12 casos visuales actuales aprobaron estabilidad,
paletas, resize y ambos temas/1000/420px. Se revisaron capturas amplias y
estrechas para comprobar legibilidad y profundidad. Diecisiete observaciones de
preservación/ambiente y nueve muestras del fundido aprobaron de nuevo. Dos
observaciones de la instancia real verifican que el mismo método koi recibe
primero8/5 poses pequeñas con opacidad reducida y luego3/2 principales con
opacidad1; las poses muestreadas a0/8/42/180s son finitas.

Las ocho comparaciones de medusas siguen idénticas al baseline. Los casos koi
conservan la cobertura alfa principal con tolerancia1/255. Algas/burbujas,
selector, reloj, tiburones y preferencias se conservan. No hubo errores de
navegador o requests externos. Build aprobado con0 diagnósticos Svelte y aviso
previo de tamaño del bundle. Sin suites históricas de App o inspección nativa.

Dibujo aislado a1000×288/DPR1: koi p50=1,8ms/p95=2,1ms; fundido
p50=3,7ms/p95=4,1ms. No acredita120Hz físicos. Evidencia privada/ignorada en
`output/playwright/background-koi-20261026/`. El apartado siguiente registra
la primera versión del ambiente, con peces de tres caracteres, ya reemplazada.

## Koi habitat follow-up

Seguimiento26/10/2026: sólo koi recibe peces pequeños lejanos, algas ASCII
ondulantes y burbujas ascendentes. Densidad amplia14/7/12 y estrecha8/4/6,
paleta secundaria y dibujo anterior a los peces principales. Usa semilla y
tiempo activo existentes; no añade listeners, RAF o temporizadores propios.

- Once pruebas existentes de escenas/océano aprobaron, sin añadir una suite
  unitaria para esta presentación reversible.
- Doce combinaciones visuales de tres escenas ×dos temas ×1000/420px aprobaron
  bitmap estable/no vacío y resize con repintado inmediato. Capturas revisadas.
- Nueve muestras del fundido aprobaron, con el ambiente dentro de la capa koi.
- Diecisiete observaciones adicionales: ocho comparaciones de medusas idénticas
  a la versión previa; ocho de koi con3.037–6.258 píxeles de fondo añadidos sin
  reducir cobertura alfa (tolerancia1/255); una de estabilidad al mismo tiempo
  y movimiento del ambiente al avanzar tiempo.
- Sin errores de navegador o requests externos. Build aprobado,0 diagnósticos
  Svelte, aviso previo de bundle conservado. Tiburones/selector/reloj/componente
  permanecen sin cambios en este seguimiento.

Dibujo aislado de80 muestras a1000×288/DPR1: koi p50=1,2ms/p95=1,5ms y fundido
p50=3,3ms/p95=3,6ms. No acredita120Hz físicos. Evidencia privada/ignorada en
`output/playwright/koi-habitat-20261026/`. No se repitió la suite histórica de App
ni se inspeccionó pantalla nativa o datos personales.

## Restoration follow-up

Este apartado registra el retorno previo al enriquecimiento del fondo koi.

Se restauraron los tres renderizadores desde `2d43a66` y se retiraron el módulo
mixto y su prueba específica. Los siete archivos de renderer/selector/geometría/
reloj/componente coinciden byte a byte con el checkpoint. App/style siguen
idénticos a `fd4f436`: se conserva la corrección del scrollbar horizontal y los
demás ajustes del shell.

Las once pruebas existentes de escenas/océano aprobaron. Revisión headless
actual:12 combinaciones (tres escenas, dos temas,1000/420px),9 muestras del
fundido y bitmap estable/no vacío durante resize, sin errores de navegador o
requests externos. Capturas amplias/estrechas y fundido revisados. Build aprobado
con0 errores/advertencias Svelte; el aviso anterior de tamaño del bundle sigue.
Evidencia nueva en `output/playwright/restored-ascii-20261026/`.

No se repitieron las suites históricas de App ni se inspeccionó pantalla nativa.
Las cifras/casos siguientes corresponden a la implementación inicial de variantes.

## Method

Chromium headless aislado con módulos compilados por Vite y la App sobre corpus
SQLite sintético. La referencia pública de koi se fijó al commit documentado y
el ZIP de Kurage se leyó como material visual; no se ejecutaron sus instrucciones
ni se distribuyeron archivos originales. Capturas/mediciones permanecen privadas
en `output/playwright/orange-koi-20261026/` y la integración compartida en
`output/playwright/panel-resize-20261026/`.

## Results

Cuatro pruebas nuevas de `tests/ascii-cover-scenes.mjs` aprobaron: 48 semillas
cubren las tres variantes sin repetición consecutiva, selección estable,
continuidad del fundido y geometría finita/estable del koi y medusas a distintos
tiempos y tamaños. Las siete pruebas existentes de `tests/ascii-ocean.mjs`
aprobaron, incluidos reloj, límite de cadencia, pausa/reanudación y geometría
articulada de tiburones. La ejecución conjunta también incluyó las cinco pruebas
de paneles: **16 pruebas aprobadas**, sin fallos.

La matriz visual `preview.cjs` aprobó **12 combinaciones**: tres variantes ×
claro/oscuro × 1000×288/420×230. En cada una se comprobó bitmap no vacío,
repetibilidad al mismo tiempo, resize con repintado inmediato y familias de color
esperadas. Las capturas revisadas muestran koi naranja/blanco y medusas
cian/violeta reconocibles, incluidas sus composiciones estrechas. Se redujeron
escala/recorrido de koi para limitar cruces y se reforzó la paleta de medusas en
claro. Los módulos importados manualmente se recargaron antes de la aceptación
final para evitar que la caché del navegador mostrara la versión anterior.

Se revisó una lámina de **nueve muestras de transición**, al inicio, mitad y
final del fundido desde cada variante; no hubo bitmap vacío. `final.cjs` aprobó
**18 comprobaciones compartidas** de App: selección válida, movimiento reducido,
pausa por pane enfocado y portada fuera de pantalla, reanudación, resize sin
cambiar selección, Photo/ASCII, semántica decorativa, modos de peek y ausencia de
writes de entidades, errores de navegador o requests externos. Son las mismas
18 citadas en la revisión de paneles, no otro conjunto independiente. El ciclo
de selección/fundido se verifica con tiempos controlados; no se presenta como
observación de un ciclo completo en una pantalla física.

Medición aislada de 80 dibujos por caso a 1000×288 y DPR1, en milisegundos:

| Caso | p50 | p95 |
| --- | ---: | ---: |
| Tiburones | 2,0 | 2,2 |
| Koi | 0,9 | 1,0 |
| Medusas | 1,5 | 1,6 |
| Fundido con dos capas | 3,0 | 3,3 |

No se registraron errores de navegador ni red externa. `npm run build` pasó con
contratos/iconos vigentes y cero errores/advertencias Svelte; queda el aviso
anterior de tamaño del bundle. OpenSpec estricto aprobado de forma independiente.

## Muted primary color follow-up (05/11/2026)

Cambio: `add-random-ascii-cover-scenes`, tarea2.8. Las manchas de los koi
principales y secundarios usan el tono del color primario elegido con la mitad
de su saturación y un rango claro/oscuro contenido: 26–66% en oscuro y 36–64%
en claro. Aletas, cola, barbillas, ojos y brillos mantienen el segundo tono
neutro, y un acento ausente o inválido recupera el azul inicial. `AsciiShark`
lee `--primary-color` junto a `--text` y observa `data-theme` y `style` en la
raíz: un cambio de acento reutiliza la instancia, reconstruye sólo el atlas koi
y repinta el tiempo activo vigente; un cambio de tema rehace las capas porque
cambian tinta y paleta. Tiburones y medusas no reciben el color.

Pruebas ejecutadas:

- `node --experimental-strip-types --test tests/ascii-koi-palette.mjs tests/ascii-cover-scenes.mjs tests/ascii-ocean.mjs`: **18 aprobadas**, 0 fallos. Las seis nuevas cubren tono primario en ambos temas para las seis sugerencias, determinismo, diferencias por acento/tema, fallback inválido o ausente, primarios acromáticos y saturación atenuada (entre35% y65% del acento y nunca superior a36%).
- `npm run check`: 0 errores y 0 advertencias Svelte.
- `npm run build`: aprobado con iconos/contratos vigentes; queda el aviso previo de tamaño del bundle.
- `openspec validate add-random-ascii-cover-scenes`: válido. `--strict` informa la advertencia previa de requisito largo en `Variantes ASCII aleatorias y locales`; la misma advertencia y salida 1 existen en la versión de `HEAD`, por lo que no la introduce esta tarea.

Matriz visual con Chromium headless y los módulos servidos por Vite, cuatro
combinaciones claro/oscuro × 1000×288/420×230 y tres acentos:

| Acento | Tono dominante / esperado | Distancia | Saturación media |
| --- | --- | ---: | ---: |
| Azul `#3183a6` | 195 / 197,9 | 2,9° | 0,174–0,209 |
| Naranja `#c97927` | 35 / 30,4 | 4,6° | 0,217–0,242 |
| Violeta `#8254d4` | 265 / 261,6 | 3,4° | 0,186–0,220 |

Extremos de paleta (oscuro y claro): azul `#304a54`→`#91b2c0` y
`#436675`→`#8aadbc`; naranja `#59432c`→`#c6a98b` y `#7b5c3d`→`#c2a484`;
violeta `#3d2e56`→`#a18ec2` y `#544077`→`#9b88bf`. La saturación del tinte
ronda la mitad del acento (54–67%) y no llega a extremos de brillo.

En cada caso el bitmap fue no vacío (33.605–33.757 píxeles visibles a1000px;
7.050–7.125 a420px) y estable al repetir el mismo tiempo. Un `resize` de azul a
naranja sobre la misma instancia cambió19.999 píxeles y reprodujo las métricas
del naranja, sin remontar. Los compuestos de tiburones y medusas resultaron
idénticos byte a byte entre azul y naranja para la misma semilla/tiempo. No hubo
errores de página ni requests externos.

Capturas revisadas en `output/playwright/koi-muted-20261105/`: cuatro de la
matriz azul y ocho de naranja/violeta/verde/gris en ambos temas a1000×288. El
naranja conserva la calidez de la referencia, el gris neutraliza los koi y el
azul/violeta quedan atenuados sin perder la silueta. La carpeta está ignorada
por Git; las capturas de `koi-primary-20261105` corresponden a la versión
saturada anterior y permanecen como referencia histórica.

También se montó `AsciiShark` con movimiento reducido en la página de Vite hasta
obtener la escena koi (1–7 intentos por la semilla aleatoria) y se cambió
`--primary-color` en la raíz: el tono dominante pasó de195 (azul) a35 (naranja)
y volvió a195, sin remontar el componente. La restauración azul difirió del
primer bitmap en1.772 de ~288.000 píxeles con delta máximo1/255, compatible con
el rasterizado de texto del atlas; tras desmontar no quedó ningún canvas ni
bucle. Esto cubre la reacción al acento dentro del componente, no el recorrido
completo de Settings.

## Limits

Las cifras miden Canvas en este navegador/máquina; no garantizan 120 FPS físicos,
cadencia nativa, otros equipos o consumos de GPU. AsciiShark conserva sus
observadores, limpieza y reloj, y los renderizadores originales de tiburones no
se reescribieron. No se inspeccionó pantalla nativa ni datos personales. No se
cambiaron Rust, SQLite, contratos, dependencias o preferencia Photo/ASCII. Ver
[guía y atribución](ascii-cover-scenes.md) y
[ADR0052](decisions/0052-local-ascii-scene-composition.md).

El seguimiento del color primario no ejecutó la App completa, Settings ni la
ventana nativa. Las capturas naranja del26/10 permanecen como referencia
histórica.
