# Verificación del océano ASCII combinado y lateral

Fecha: 26/10/2026. Cambio: `combine-ascii-ocean-life`. Registro histórico del
experimento, descartado para la portada por decisión del usuario. Se restauraron
[las tres variantes](verification-ascii-cover-scenes.md) y se conserva el arreglo
del lateral. Su delta final sólo modifica el lateral; no retirar las variantes.

## Reduced density follow-up

El usuario aprobó reducir saturación después de revisar la escena original.
La versión probada mostraba dos tiburones, dos koi y dos medusas principales,
más dos peces ambientales muy pequeños/tenues. Figuras cercanas alrededor del20%
menores, profundidad de fondo más discreta y centros de recorrido separados.

Seis pruebas focalizadas de `tests/mixed-ocean-scene.mjs` aprobaron: cantidades,
fondo tenue, geometría/orden/semilla, escala estrecha, continuidad y muestreo de
diez minutos con tres semillas sin ternas de centros agrupados. El criterio de
agrupación usa diferencias menores a18% del ancho y25% de la altura entre los
tres pares; no afirma ausencia de colisiones de siluetas.

Veinte casos visuales nuevos aprobaron en claro/oscuro,1000/420px y tiempos
0/8/24/42/80: bitmap repetible y no vacío, paletas presentes, repintado al resize
y orden global real de pintura. Capturas amplias/estrechas revisadas. p95 aislado
de dibujo:2,1ms a1000×288/DPR1,2,0ms/DPR2;1,4/1,5ms a420×230. Build aprobado con
cero errores/advertencias Svelte; aviso previo del tamaño del bundle conservado.
Sin errores de navegador o requests externos. No se repitieron las suites
históricas de App/ciclo de vida, cuyo código permanece intacto. Evidencia nueva
en `output/playwright/relaxed-ocean-20261026/`. Lo que sigue registra la primera
composición de11/9 animales, anterior a este seguimiento.

Noteka ya estaba ejecutándose con Vite; se volvió a mostrar su instancia nativa
existente al frente, conservando la sesión. La activación se confirmó por PID y
aplicación en primer plano, sin inspeccionar pantalla ni afirmar aceptación
visual nativa.

## Checkpoint and method

Antes de editar se creó el commit local **2d43a66**, que conserva todos los
avances de código/documentación/planificación y las tres escenas alternadas.
La prueba parte de él en **codex/combined-ascii-ocean**. El checkpoint no afirma
una nueva validación de todos los cambios históricos que contiene.

Se usaron Chromium headless aislado, módulos servidos por Vite y la App compilada
con CLI Rust/SQLite sintético. Sin pantalla nativa ni workspace personal. El
evento de visibilidad del documento se simuló en DOM; no acredita ocultar una
ventana física. Evidencia privada/ignorada:
`output/playwright/mixed-ocean-20261026/`.

## Sidebar results

La reproducción anterior midió clientWidth249/scrollWidth252 en el aside y un
handle que sobresalía 2px del borde. El cuerpo con overflow:auto lo convertía en
scrollbar horizontal. La solución separa cuerpo vertical y borde, coloca el
handle dentro y ajusta mínimos del grid de Settings. La revisión a320px detectó
también cuatro píxeles de overflow del botón, corregidos antes de aceptar.

`ui.cjs` aprobó **37 comprobaciones**. Incluyen 16 combinaciones de tema,
viewport1440/1000/420/320 y ancho mínimo/máximo del menú: tanto cuerpo como
contenedor tienen scrollWidth igual a clientWidth, scrollLeft queda en0, borde
interior operativo, máximo35% y ausencia de overflow global. También aprobaron
arrastre real, Shift/Alt/flechas, Escape, scroll vertical en ventana de260px de
alto, Settings alcanzable, foco de vuelta en Home y borde fijo al desplazar el
contenido. La revisión espera que se apliquen los eventos del media query antes
de medir el panel estrecho.

## ASCII results

- Cinco pruebas nuevas en `tests/mixed-ocean-scene.mjs`: familias presentes,
  orden global, individuos cerca/lejos de cada familia, escala/contraste,
  geometría/semilla, menor densidad y continuidad en42s y en el umbral de600px.
- Las once pruebas previas de `tests/ascii-ocean.mjs` y
  `tests/ascii-cover-scenes.mjs` aprobaron; reloj, cadencia, pausas, geometría y
  helpers de la versión anterior permanecen. **16 pruebas puras** en total.
- `preview.cjs` aprobó **20 casos visuales**: claro/oscuro ×1000×288/420×230
  ×tiempos0/8/24/42/80. Bitmap visible/repetible al mismo tiempo, koi naranja,
  medusas cian/violeta y repintado inmediato tras resize. Se revisaron capturas
  amplias y estrechas. Los cruces ocasionales forman parte de la composición.
- Se observó el orden real de llamadas de dibujo en una instancia, que coincide
  con la lista global de profundidad e intercala familias. El intento inicial de
  observar prototipos externos no veía las clases con identidades de módulo HMR
  distintas; la observación final usa los renderizadores de la instancia real.
- `legacy.cjs` comparó la API original del océano con el archivo de `2d43a66`:
  **12 bitmaps idénticos**, dos temas ×dos tamaños ×tiempos0/8/42. La geometría
  de tiburones no se modificó; el refactor mantiene el resultado anterior.
- Las 37 comprobaciones compartidas de App también cubren11 animales en Home,
  reducido estático, actividad/pausa al enfocar un detalle, reanudación al
  suprimirlo, el mismo borrador/editor intacto, portada fuera de pantalla,
  señales de documento oculto/visible, desmontaje a Photo y vuelta a ASCII.
  Estas observaciones pertenecen al mismo script, no son otra suite.

No se registraron errores de navegador, requests externos o writes implícitos
de entidades. El borrador sintético se descartó explícitamente al terminar.
`npm run build` aprobó iconos/contratos y cero errores/advertencias Svelte;
permanece el aviso previo de tamaño del bundle. OpenSpec estricto se valida
separado de la aplicación.

Medición aislada de80 dibujos por caso; milisegundos:

| Tamaño | DPR | p50 | p95 |
| --- | ---: | ---: | ---: |
| 1000×288 | 1 | 2,5 | 2,9 |
| 1000×288 | 2 | 2,5 | 2,7 |
| 420×230 | 1 | 1,5 | 1,7 |
| 420×230 | 2 | 1,4 | 1,7 |

## Limits

Estos tiempos miden Canvas en el navegador/máquina de prueba; no garantizan
cadencia física120Hz, GPU o rendimiento en otros equipos. La escena no simula
colisiones. No se repitieron todas las suites históricas del núcleo ni se
verificó IPC físico de Tauri. No hay cambios de Rust, SQLite, contratos,
dependencias o preferencias persistentes. El delta anterior de variantes debe
sincronizarse antes del nuevo: éste retira su requisito temporal y lo sustituye
por composición mixta. Ver [guía](ascii-cover-scenes.md) y
[ADR0053](decisions/0053-shared-depth-ascii-composition.md).
