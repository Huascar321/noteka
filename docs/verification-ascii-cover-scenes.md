# Verificación de las tres variantes ASCII

Esta revisión documenta la versión de tres escenas preservada en `2d43a66`.
El experimento posterior combina la fauna: ver
[evidencia actual](verification-mixed-ocean.md). Los resultados siguientes son
históricos y no se vuelven a contar como pruebas nuevas.

Fecha: 26/10/2026. Cambio: `add-random-ascii-cover-scenes`. Implementado y
verificado localmente; deltas pendientes de sincronización/archivo.

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

## Limits

Las cifras miden Canvas en este navegador/máquina; no garantizan 120 FPS físicos,
cadencia nativa, otros equipos o consumos de GPU. AsciiShark conserva sus
observadores, limpieza y reloj, y los renderizadores originales de tiburones no
se reescribieron. No se inspeccionó pantalla nativa ni datos personales. No se
cambiaron Rust, SQLite, contratos, dependencias o preferencia Photo/ASCII. Ver
[guía y atribución](ascii-cover-scenes.md) y
[ADR0052](decisions/0052-local-ascii-scene-composition.md).
