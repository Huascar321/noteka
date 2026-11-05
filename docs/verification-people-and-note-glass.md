# Verificación de People y controles de nota

04/11/2026. Cambio `refine-people-and-note-glass`, autorizado después de las
cuatro recomendaciones visuales. Se comenzó por tarjetas People y relaciones;
el trabajo queda local sin commit adicional, push, sync ni archivo.

## Implementación

- People: marco iluminado tenue, base sólida, sombra corta y radio de 16 px en
  galería/pipeline y 12 px en lista. Hover/foco refuerzan el marco y añaden un
  reflejo estático detrás del contenido. No hay blur por tarjeta ni traslación.
  Foto, posición de foto, etiquetas semánticas, orden y apertura se conservan.
  El anillo de foco queda fuera del recorte de la tarjeta.
- Relaciones: cápsulas con icono y nombre envolvente, apertura y × como botones
  independientes. El botón × tiene área de 26 px; Add muestra material de campo
  en hover/foco de teclado. Las operaciones y restricciones existentes siguen
  gobernando vinculación, desvinculación, apertura y borradores.
- Propiedades Note: Tag con marco de campo y destinos internos planos; Type
  con select HTML y desplegable nativo. `appearance: none` y una flecha sin
  eventos normalizan el marco en WebKit, que ignoraba altura/radio/sombra con
  apariencia automática. URL sigue como texto de lectura y Favorite como
  checkbox real; el control Type bloqueado conserva una señal visual atenuada.
- Cabecera Note: bloqueo temporal/re-lock, refresco, estado, Save/Retry y menú
  reunidos en una cápsula de 18 px. Sólo cambia la agrupación de Note; las otras
  páginas conservan su estructura. No hay filtro ni transform en el ancestro
  de menús fixed. Se mantienen nombres, handlers y restauración de foco.

Tarjetas y cápsulas reutilizan bordes/reflejos del sidebar y tokens de contenido
más tenues; cuerpos y código continúan sólidos. Contraste aumentado y menor
transparencia retiran gradientes, sombras y marcos decorativos. No se agregan
transiciones, relojes, dependencias, operaciones Rust ni formatos persistentes.

## Verificación

- `npm run build`: aprobado, cero errores/advertencias Svelte, contratos e
  iconos vigentes. Permanece el aviso previo de Vite por bundle mayor a 500 kB.
- `node scripts/headless-relation-picker-smoke.mjs`: 33 comprobaciones aprobadas
  en Chromium, con UI compilada, CLI Rust y SQLite sintético. Incluye conflicto,
  resultados tardíos, borrador pendiente, foco, vinculación y desvinculación.
- Recorrido focal con Playwright CLI, DPR 2, misma UI compilada y operaciones
  reales sobre workspace sintético: **72 comprobaciones Chromium / 71 WebKit**
  aprobadas. Tarjetas con foto local en las tres
  presentaciones; tema claro/oscuro y Blue/Violet; controles de Note y cápsulas
  de relaciones; guardado explícito, bloqueo temporal/re-lock, guardias de
  apertura, desvinculación con borrador y menús en side/center/full.

- Contraste sobre píxeles compuestos, ocultando sólo la tinta y los adornos:
  mínimo **4,93:1 Chromium / 4,97:1 WebKit**. Se midieron título People,
  relación, Tag, Type y texto secundario de cabecera en ambos temas; los mínimos
  corresponden al texto secundario de cabecera, por encima de 4,5:1. Tinta y
  fondo se midieron tras retirar transiciones durante la captura.
- Los colores semánticos y la fuente/posición de foto permanecen iguales al
  navegar entre vistas. La apertura de la tarjeta no cambia la revisión de la
  persona; Type/Favorite sólo se guardan con Save. Abrir una relación con
  borrador activa la guardia; cancelar conserva el borrador. Desvincular no
  abre el destino ni descarta el contenido pendiente.
- Menús dentro del viewport y retorno de foco con Escape en side/center/full;
  opciones deshabilitadas con nota bloqueada, desbloqueo temporal y re-lock.
  Alternativas sólidas con contraste aumentado y movimiento reducido en ambos
  motores. Menor transparencia emulada con CDP en Chromium, no en WebKit.
- A 420 × 740, People y Note no desbordan horizontalmente la ventana; los
  nombres largos de relaciones envuelven sin recortar botones. Sin errores
  de página ni solicitudes externas durante el recorrido focal.
- `openspec validate refine-people-and-note-glass --strict` y `git diff --check`
  aprobados. Se revisó el diff propio contra una copia previa al pedido para
  separar estos ajustes de los cambios locales anteriores.

## Límites y evidencia

Las capturas usan contenido sintético y una imagen local de prueba. Acreditan
el acabado CSS y las operaciones del puente CLI, no IPC o aceptación nativa
Tauri/macOS. No se accedió a la ventana nativa en este cambio. La ventana
permanece opaca; no se afirma refracción nativa ni rendimiento GPU/energía.
No se repitieron suites amplias del núcleo o recorridos heredados con
expectativas obsoletas, documentados en la verificación del cambio anterior.

Los primeros intentos del recorrido detectaron el marco nativo de Type en
WebKit y diferencias de foco de teclado, corregidas en producto. También se
ajustó la instrumentación: esperar la lectura de relaciones, aislar transiciones
al medir tinta, ocultar la flecha decorativa al muestrear el fondo y activar
foco de teclado después de que el control se habilite. No se alteraron reglas
del núcleo ni pruebas existentes para acomodar el acabado.

Capturas/logs locales ignorados por Git:
`output/playwright/people-note-glass-{card,note,header}-{chromium,webkit}-{light,dark}.png`,
`people-note-glass-{chromium,webkit}.log`, `people-note-glass-relations.log` y
`people-note-glass-build.log`. Scripts focales temporales:
`/tmp/noteka-people-note-glass-check.cjs` y
`/tmp/noteka-people-note-glass-runner.mjs`.

Sistema de diseño, plan y workflow actualizados; el delta continúa activo sin
publicar propuestas en las specs principales.

## Corrección de Tag después de la revisión visual

El usuario detectó que una sola etiqueta podía empujar la flecha a otra fila
y que Type bloqueado atenuaba también el marco, a diferencia de Tag. La lista
de destinos ahora envuelve dentro de su propia zona flexible, reservando la
flecha fuera de esa lista. Con un destino o vacío, Tag ocupa 32 px como Type;
el nombre individual se trunca con tooltip y varias etiquetas pueden añadir
filas. Se mantiene el acceso al destino y el bloqueo de edición.

Type bloqueado conserva opacidad y material del campo; sólo se atenúa su tinta.
Las flechas de Tag/Type usan la misma tinta secundaria al estar inhabilitadas.
Esta diferencia funcional sigue siendo deliberada: el destino Tag se puede
abrir, mientras que el valor Type no se puede editar con la nota bloqueada.

Se amplió la verificación a columnas de 104/120/140/180/260 px, con etiqueta
larga, vacío y varias etiquetas, además de ambos temas, bloqueo, navegación,
picker, fallback sólido y ventana de 420 px. Evidencia específica en
`output/playwright/tag-field-{chromium,webkit}.log`,
`tag-field-{chromium,webkit}-{light,dark}-{locked,editable}.png` y
`tag-field-build.log`; scripts temporales `/tmp/noteka-tag-field-check.cjs` y
`/tmp/noteka-tag-field-runner.mjs`.

Resultado: **42 comprobaciones Chromium / 42 WebKit**, aprobadas, con build,
OpenSpec estricto y `git diff --check` aprobados. Las capturas confirman
flechas alineadas y marcos equivalentes en claro/oscuro, también bloqueados.
No se repitió el recorrido amplio anterior: la corrección se verificó con esta
matriz focal de geometría, estados y operaciones de Tag.
