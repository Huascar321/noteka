# Verificación de Jelly uniforme

Fecha: 26/10/2026. Cambio `unify-sidebar-jelly-motion`. Decisión del usuario:
usar el mismo rebote Jelly de Tasks en los nueve accesos laterales. Verificado
localmente; delta pendiente de sincronización/archivo.

El usuario posteriormente pidió un ritmo más lento: el ciclo vigente es de
1000ms y la limpieza ocurre a1050ms. Las secciones iniciales siguientes conservan
la evidencia de la unificación a700ms; el ajuste posterior se registra al final.

## Implementación

El importador reutiliza la exportación oficial checklist/Jelly de la revisión
Icona ya fijada y envuelve cada figura estática en su grupo .fx. Los nueve
accesos comparten el mismo CSS; la generación valida claves, recursos, atributos
y referencias locales como antes. No cambia el reproductor ni la acción Svelte.
Mismos 700ms, limpieza a 750ms, carga al primer clic y cancelaciones. Sin nueva
dependencia, preferencia, persistencia, operación, Rust o red remota.

## Preservación y generación

Seis verificaciones contra snapshot anterior aprobadas:

- La exportación de Tasks permanece idéntica, incluidos body/CSS/duración.
- `icons.generated.ts` es idéntico byte a byte; SHA256
  `d424e8c662b4917e32c727a66214d11d4616637328a7e5853629d74b05f7b251`.
- Las 66 variantes SVG fuente permanecen idénticas estructuralmente.
- Los nueve accesos declaran Jelly.
- Una sola definición CSS compartida.
- Reimportar desde checkout limpio fijado y regenerar produce archivos idénticos.

Los avisos/procedencia se actualizaron para distinguir una receta oficial
compartida de nueve variantes curadas diferentes. SHA256 actual de assets:
`10bdf01afcccbf150b4e4e2c7ab58ff0a3d62d94a944de5948657b5b205922ca`.
`npm run icons:check` comprueba también los avisos distribuidos offline.

## Build y navegador

`npm run build` aprobado: iconos/contratos vigentes, Svelte con 0 errores/advertencias,
Vite con 288 módulos. Se conserva el aviso previo del bundle principal mayor a 500kB
sin modificar umbrales. Módulo de movimiento de 8,47kB/2,11kB gzip, con una copia CSS.

**70 checks headless por motor (Chromium y WebKit), 140 en total:**

- 60 por motor sobre los nueve accesos reales del harness Svelte con AppIcon y
  la misma acción lateral: carga diferida, clic, Enter/Space, ciclo acotado,
  máscaras/accesibilidad, SVG y nodos originales exactos al terminar, repetición,
  cancelación global, movimiento reducido antes/durante, visibilitychange y
  desmontaje durante carga/reproducción. Render/hover siguen estáticos.
- 10 por motor comparan el Jelly de las nueve figuras a 0/70/140/210/350/550/700ms:
  igual matriz de transformación, origen 12px12px/view-box, duración e iteraciones
  que Tasks en todas las muestras. Tasks cambia geometría en el tiempo;
  a 70ms los dos motores devuelven matrix(1.25,0,0,0.75,0,0).

## Evidencia y límites

`output/playwright/jelly-20261026/` (ignorado) conserva snapshot previo,
generación/hashes, build, driver y JSON de movimiento/matrices por motor.
La visibilidad y los tiempos de muestreo se controlan en el navegador de prueba;
no acreditan percepción en pantalla física, IPC o cierre de Tauri. No se accedió
al workspace personal ni se alteraron datos para probar el movimiento.

No se repiten suites Rust o revisiones de todas las vistas por un cambio de
receta/generación sin cambios en reproductor o navegación. La
[verificación inicial de Icona](verification-icona.md) conserva su alcance
histórico de geometría/imágenes/UI; este documento verifica la nueva decisión.

## Ajuste posterior a1000ms

La duración queda centralizada en `SIDEBAR_MOTION_DURATION_MS`. Importador y
generador comparten ese valor; el reproductor ya programa la limpieza50ms después
de la duración de la escena. No se modifican reproductor, navegación ni datos.

La comparación contra el estado de700ms verifica que el CSS de Tasks difiere
únicamente en la duración de la declaración animation (0.7s →1s). Keyframes,
porcentajes, intensidad, easing, geometría y catálogo estático se conservan.
Siete checks de generación/preservación/reproducibilidad aprobados; los nueve
comparten el mismo CSS y1000ms. SHA256 actual de assets:
`31329e6519c9aad5cb114cff271a7c4041d4ef4583e94ffb2ff700e4f3aa09e1`.

Build aprobado con0 errores/advertencias Svelte, contratos/iconos vigentes y el
mismo aviso previo de tamaño del bundle. Evidencia del ajuste bajo
`output/playwright/jelly-slower-20261026/` (ignorado). Sin suites Rust repetidas,
dependencias nuevas o aceptación nativa.

Nueva revisión headless:73 checks por motor (Chromium/WebKit),146 en total.
Se conservan60 checks de clic/teclado/reposo/cancelaciones y se añaden13 de
muestras/duración: los nueve coinciden con Tasks a0/100/200/300/500/800/1000ms,
el ciclo CSS dura1000ms con una iteración y sigue activo a800ms, superando el
antiguo corte a750ms. Al finalizar se retiran los estilos y vuelve el SVG original.
