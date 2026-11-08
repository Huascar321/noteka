# Transiciones compartidas de menús y popups

Fecha: 07/11/2026. Cambio activo: `share-liquid-overlay-transitions`.

## Comportamiento y alcance

El usuario acepta Due a 400/200 ms y pide extender la transición. Se comparten
sus tiempos y curvas mediante `liquid-motion.ts`. `liquid-overlay.ts` conecta
explícitamente las otras superficies; no hay descubrimiento global por clases,
dependencias nuevas, cambios Rust/SQLite, escrituras ni solicitudes remotas por
animar. El efecto es una aproximación web y no refracción nativa de Apple.

| Superficies | Origen / preservación |
| --- | --- |
| Due | Conserva capas y cierre existentes; sólo importa la receta compartida. |
| Page actions / Peek mode | Botón original; roving focus, selección y opciones conservados. |
| Iconos y relaciones | Control que los abre; elección, búsquedas y vínculos conservados. |
| PageTools | Botón original; paneles permanecen montados con buffers. La salida visual no los hace interactivos ni altera consultas por `hidden`. |
| Editor | Caret/selección o grip; formato, comandos, acciones, link/attachment, view picker y slash convencional. El texto nunca hereda la compresión del vidrio. |
| Nav / opciones de vista | Secciones Tasks, Notes, Projects, Tags, Goals y People; Milestones comparte opciones. Conservan posición en flujo y navegación. |
| Search / Settings | Botón al abrir por puntero; expansión central al usar atajo o faltar ancla visible. Foco y modalidad conservados. |
| Move project | Grip original, Cancel y persistencia de destino existentes. |
| Cambios pendientes | Expansión central, Cancel/Discard/Save originales y contenido pendiente conservado. |
| Center peek | Origen conectado o expansión central; no remonta el editor al cambiar modo. |

Selects, file dialogs y otros controles del sistema, tooltips de ayuda,
disclosures de contenido, barras expansibles del editor convencional y Side/Full
peek conservan su comportamiento propio. No se presentan como popups animados.

## Decisiones y casos delicados

- El contenedor real conserva estructura, controles, layout y semántica. Una
  capa temporal inerte/aria-hidden copia sólo material y borde; no copia datos
  ni controles. Menús/pickers usan top layer para evitar recortes; diálogos y
  Center peek usan una capa local detrás del contenido.
  El dueño suspende su material mientras esa capa existe: un solo blur local.
- Contenido por opacidad y material con transformación independiente. El sheen
  es el mismo reflejo breve de Due. Observadores de tamaño, scroll y resize
  existen únicamente durante la transición y se liberan junto con los efectos.
- La salida captura el frame actual; un menú que se reabre durante el cierre
  reutiliza la geometría presente. Las salidas son inertes y no repiten acciones.
- Search y guardias esperan la salida/desmontaje antes de navegar o ejecutar la
  continuación ya aprobada. Se evita que el cleanup del modal quite el foco al
  destino. Escape reconoce todos los overlays activos del detalle: el selector
  de iconos ya no deja cerrar simultáneamente su peek.
- `Reduce Motion` omite la transición y cancela la activa al cambiar. Contraste
  alto/transparencia reducida mantienen sus superficies sólidas. Sin soporte de
  popover el fallback es presentación inmediata. No se añade un renderer/RAF
  permanente: las transiciones finitas siguen el ciclo de Svelte/WAAPI.

## Ajuste de contenido progresivo

La revisión del usuario detectó una fase vacía seguida de contenido repentino.
El vidrio decorativo en top layer quedaba encima del texto y lo velaba, además
de existir un tramo inicial de opacidad cero. Para diálogos y Center peek se
coloca detrás del texto, en el mismo contexto de apilamiento. El contenido real
arranca al 24% de su opacidad original y alcanza su valor original a los 220 ms;
se recorta a la silueta animada sin heredar escala ni duplicar controles.
Se restauran estilos de recorte al terminar, cancelar o cambiar preferencias.
Menús/pickers y Due mantienen su receta. Entrada/salida siguen en 400/200 ms.

Los dos frames de cada motor pausan el reloj de Svelte en el primer frame
observable y después de alcanzar 100 ms; no representan tiempos exactos de
pantalla. Verifican texto visible, aumento de opacidad, ausencia de escala,
un único contenido y vidrio detrás, además de limpieza al asentarse. Se revisaron
las cuatro capturas. El reloj de presencia persistente es finito y actualiza
material y recorte juntos, sin renderer permanente.

El build nativo debug se recompiló y Noteka quedó reiniciada con estos assets.
Settings se observó visible y asentado, y cerró con Escape desde Search. Durante
la automatización hubo una captura con la app en segundo plano todavía en su
frame inicial; al reabrir Settings con la ventana activa se mostró completo.
La aceptación nativa aquí
es de presentación/cierre en primer plano, no una medición de frames intermedios.

[Frame inicial WebKit](design/liquid-overlays-preview/webkit/guard-preview-0.png) ·
[Frame intermedio WebKit](design/liquid-overlays-preview/webkit/guard-preview-100.png).

## Verificación reproducible

Resultados de esta implementación:

- Build frontend: cero errores/advertencias Svelte; se conserva el aviso Vite de
  chunk mayor a 500 kB. Build nativo debug con assets compilados aprobado.
- Due después de extraer la receta: **56 checks Chromium + 56 WebKit**.
- Recorrido actualizado para contenido progresivo: **103 checks Chromium +
  103 WebKit**, incluido foco después de Search, salida modal inerte, cierre
  interrumpido, buffers, frames iniciales/intermedios y condiciones accesibles.
- Editor de tareas: **18 checks**. Adopción Liquid Glass: **569 checks**, incluidos
  Move project/Cancel/persistencia, menús, materiales y controles existentes.
- Atlas de la extensión inicial: **71 PNG por motor**. Sus capturas usan
  fixtures y movimiento reducido; no acreditan el ajuste posterior de entrada.
  Para éste se revisaron y conservaron dos frames de guardia por motor en
  `docs/design/liquid-overlays-preview/`, sin datos personales.
- Noteka nativa reiniciada con assets compilados: Settings abre y cierra con
  Escape desde Search, sin cambiar preferencias ni datos. La inspección no es
  una medición de fotogramas ni una aceptación completa de todos los flujos nativos.
- `openspec validate share-liquid-overlay-transitions --strict` y diff whitespace
  aprobados. El cambio permanece activo.

```sh
npm run build
npm run test:headless-due-picker
NOTEKA_TEST_BROWSER=webkit npm run test:headless-due-picker
npm run test:headless-liquid-overlays
NOTEKA_TEST_BROWSER=webkit npm run test:headless-liquid-overlays
node scripts/headless-task-editor-smoke.mjs
node scripts/headless-liquid-adoption-smoke.mjs
npm run design:screenshots
NOTEKA_TEST_BROWSER=webkit npm run design:screenshots
openspec validate share-liquid-overlay-transitions --strict
```

El recorrido nuevo utiliza CLI Rust/SQLite temporales y fixtures sintéticas;
verifica tiempo y top layer, texto sin escala, liberación de capas, cierre
interrumpido, modalidad, foco destino, conservación de instancia/borrador,
opciones/Nav, 420 px, dos temas/acentos y preferencias. Bloquea solicitudes
externas del frontend. Las capturas específicas se guardan en
`output/playwright/liquid-overlays/<motor>/`; el atlas general en
`docs/design/liquid-glass/<motor>/` conserva su manifest con hash de fuentes,
incluidas ambas utilidades nuevas al generar la extensión inicial. Ese manifest
es anterior al ajuste de contenido progresivo; no se presenta como snapshot
actual del código de animación.

Los primeros fallos de preparación del recorrido (labels específicos de Note,
selector de Settings oculto al estrechar) se corrigieron en la fixture. La
prueba de asentamiento espera el fin observable del efecto, con límite, en vez
de asumir que 500 ms de reloj externo equivalen a 400 ms del browser bajo carga.
La cadena Discard → cerrar Center peek espera ambas salidas, en lugar de comprobar
el desmontaje a los 300 ms de dos transiciones sucesivas. Escape de los popups
modales se maneja también explícitamente, sin depender sólo de `cancel` nativo.

El primer arranque nativo parecía vacío mientras la ventana quedaba en segundo
plano. El proceso estaba vivo y el frontend cargaba correctamente en navegador;
no se demostró un fallo del código de transición. Se verificó un arranque debug
con assets incorporados y activación de la app mediante Launch Services, donde
la interfaz y Escape funcionaron. No se modificó Rust, CSP ni datos para resolverlo.
La instancia dejada abierta usa ese build (sin dependencia de Vite):

```sh
npm run tauri build -- --debug --no-bundle
./target/debug/noteka-desktop
```

## Límites

La galería con movimiento reducido verifica reposo/materiales y no acredita
el movimiento por sí sola; eso se comprueba en el recorrido con movimiento.
No se midieron GPU, pantalla física o 120 Hz ni Windows. No se afirma igualdad
óptica entre Chromium, WebKit headless y WKWebView. La rama de transparencia
reducida comparte el fallback comprobado con contraste alto; no se emuló por
separado esa preferencia. Los datos personales no entran en capturas versionadas.

No se ejecuta la suite general ni se declara toda la aplicación verificada.
Sin commit, push, publicación, sync o archivo; cambios anteriores conservados.
