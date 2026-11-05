# Verificación de herramientas y popovers Liquid Glass

04/11/2026. Cambio `unify-liquid-glass-tools-and-popovers`, autorizado por el
usuario después de aceptar el menú lateral, los controles de navegación y el
acceso Search. Queda local sin commit adicional, push, sync ni archivo.

## Implementación

1. Tasks, Notes, Projects, Tags, Goals, People y Milestones agrupan sus iconos en
   una cápsula neutra de 32 px. Los botones conservan 28 px; estado de lectura y
   New quedan como hermanos. El grupo no tiene blur para conservar las
   coordenadas de menús fijos.
2. El campo del popup Search comparte la forma, borde y reflejos de Search
   lateral, con foco exterior. Conserva input, combobox, resultados y cierre.
   El placeholder usa tinta opaca reforzada; el campo oscuro usa gris al 40%
   para mantener contraste cuando WebKit no compone el blur en la captura.
3. `.glass-menu` comparte radio de 14 px, borde direccional y reflejo interior
   en menús de página/apertura/bloques, pickers de iconos/relaciones, More Tasks
   y paneles flotantes PageTools. El dueño conserva posición, filtros locales,
   foco, scroll y eventos. Los paneles PageTools en el flujo no adoptan el marco.
4. ASCII/Photo reutiliza `slidingTabs`: un indicador se desplaza y adapta a la
   opción activa. Se conserva el anclaje absoluto, visibilidad con hover/foco,
   semántica de botones y preferencia `noteka.home-cover.v1`.

Contraste aumentado/transparencia reducida retiran reflejos y filtros con
superficies sólidas; movimiento reducido elimina la transición del indicador.
No cambian Rust, operaciones, formatos persistentes ni dependencias.

## Evidencia focal

- `npm run build`: aprobado, cero errores/advertencias Svelte, contratos e
  iconos vigentes. Vite conserva su aviso de bundle mayor a 500 kB.
- UI compilada con puente IPC local, CLI real y SQLite sintético, Playwright
  CLI y DPR 2: **60 comprobaciones Chromium / 59 WebKit**, aprobadas.
- Ambos temas y Blue/Violet: seis grupos de Home, siete contextos incluidos
  Milestones, New independiente, altura compacta y material neutro. Refresh y
  opciones continúan operables; las seis páginas de sección conservan su grupo.
- Popup Search: foco inicial, escritura, resultados y Escape. Contraste mínimo
  del placeholder sobre píxeles compuestos de las muestras: **7,54:1 Chromium /
  5,86:1 WebKit**, por encima de 4,5:1. Se ocultó la tinta para capturar el fondo
  compuesto; el color se obtiene tras finalizar las transiciones. El primer
  intento encontró poco contraste y motivó reforzar tinta y tinte oscuro.
- Menús reales de página, apertura, iconos, bloques y panel External resources:
  borde compartido, capas sin eventos y posición dentro del viewport. Escape,
  cierre y lectura conservados; abrir menús no cambia la revisión de la nota.
- Selector de portada: Photo con ratón, ASCII con Enter, indicador alineado y
  preferencia Photo conservada tras navegar fuera de Home y volver. Movimiento
  reducido desactiva su transición; foco revela la cápsula.
- Contraste aumentado y movimiento reducido en ambos motores. Transparencia
  reducida emulada con CDP en Chromium, aprobada; no emulada en WebKit. Vista
  de 600 × 700 sin overflow horizontal global. Sin errores de página ni
  solicitudes externas en el recorrido focal.
- `headless-relation-picker-smoke.mjs`: **33 comprobaciones**, aprobadas en
  Chromium, sobre el flujo existente de búsqueda, selección y relación.
- Ventana Tauri/macOS: observados Home con herramientas agrupadas y New
  independiente, cápsula ASCII/Photo revelada al enfocar y campo Search con
  cierre Escape. Se conservó la preferencia nativa del usuario. El último
  refuerzo de tinta/tinte oscuro se verificó en los dos motores headless.
- `openspec validate unify-liquid-glass-tools-and-popovers --strict` y
  `git diff --check`: aprobados. Revisión del diff limitada a estilos,
  agrupaciones visuales, clases de material y acción de indicador existente;
  se preserva el resto del trabajo local anterior.

## Recorridos heredados y límites

`headless-page-actions-smoke.mjs` llegó a **99 comprobaciones en cada motor**,
incluidos menús de los siete tipos, herramientas flotantes, foco, guardias y
Archive/Restore. Después falló al buscar el botón histórico `Create in Tasks`;
la interfaz vigente usa New. No se declara aprobada la suite completa.

`headless-ascii-side-peek-smoke.mjs` conserva expectativas previas al acabado
ya aceptado: blur exacto de 8 px en breadcrumb y pausa de ASCII con Search.
Un intento temporal con el blur acotado avanzó hasta la expectativa de pausa,
contraria al comportamiento aprobado el 02/11. El test quedó sin cambios;
el selector nuevo se verificó con el recorrido focal anterior.

La ventana es opaca y el acabado es CSS, sin refracción nativa ni promesas de
rendimiento GPU/energía. No se repitió la suite Rust completa ni se modificaron
las operaciones para satisfacer expectativas históricas. OpenSpec valida los
documentos; la evidencia de aplicación es la indicada arriba.

Capturas/logs locales ignorados por Git:
`output/playwright/glass-tools-{chromium,webkit}.log`,
`glass-tools-{toolbar,cover,search}-{chromium,webkit}-{light,dark}.png`,
`glass-tools-page-actions-{chromium,webkit}.log`,
`glass-tools-relations.log`, `glass-tools-ascii.log` y
`glass-tools-build.log`. Recorrido focal temporal:
`/tmp/noteka-glass-tools-check.cjs` y `/tmp/noteka-glass-tools-runner.mjs`.

Sistema de diseño, plan y workflow actualizados. El delta permanece activo y
no publica comportamiento propuesto en las specs principales.
