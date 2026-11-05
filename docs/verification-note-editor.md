# Revisión del editor contextual de Notes — 26/10/2026

Cambio `refine-notion-note-editor`: implementado y verificado localmente, con
deltas pendientes de sincronización/archivo. La indicación posterior del usuario
reubicó los tres grupos de herramientas del pie en la barra superior de Note.

## Alcance y entorno

Svelte/Tiptap/CSS, sin migraciones, nuevas dependencias, operaciones ni cambios de
contratos Rust. Noteka permanece ejecutándose en desarrollo. La revisión usó una
sesión Chromium headless propia con el puente existente UI → CLI Rust real → SQLite
sintético; el CLI se compiló con `cargo build --offline --locked -p noteka-cli`.
Ningún workspace personal se abrió o editó para verificar.

Evidencia privada/ignorada en `output/playwright/note-editor-20261026/`:
`results.json`, `setup.cjs`, `review.cjs`, `extended.cjs`, capturas por tema/ancho y
copias iniciales de las fuentes. El puente creó su corpus en
`/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/noteka-headless-qjUKnp`.
Los registros de la sesión no se versionan.

## Resultado

La última revisión aprobó **59 comprobaciones**: 23 del editor y 36 del flujo ampliado.

| Límite revisado | Evidencia observada |
| --- | --- |
| Apertura y formato | Abrir no reescribe el documento; selección y negrita exactas; marcas guardadas en SQLite |
| Enlaces | Esquema no permitido y URL incompleta permanecen en el formulario con error; enlace válido conserva su selección |
| Comandos | Tres niveles, filtrado, alias h2, flechas/Enter, Escape literal y vínculo ARIA entre superficie y menú |
| Bloques | Duplicación con IDs únicos en descendientes, undo/redo, borrado completo, movimiento por botón y arrastre con identidad/marcas conservadas |
| Inserción | `+` y teclado, imagen local administrada guardada; URL/código conservan barras; cancelar adjunto/vista conserva el comando |
| Cabecera | Tres iconos arriba y cero duplicados al pie; paneles acotados, Escape/foco y destino de exportación conservados |
| Temas y ancho | Claro/oscuro a 420 px: selección, slash y panel superior dentro del viewport, sin overflow horizontal global |
| Integridad | Prefijo importado íntegro, error de almacenamiento, cancelación de navegación y conflicto de revisión conservan el buffer |
| Compatibilidad | Task y Goal conservan editor/ubicación anteriores; documento desconocido recuperable; Note reabierta conserva bloques e imagen |
| Runtime/red | Cero errores de navegador y cero solicitudes externas durante ambos recorridos finales |

`npm run build` aprobó catálogo de 77 iconos, contratos, TypeScript/Svelte
(**0 errores y 0 advertencias de Svelte**) y bundle. Vite conserva la advertencia
de chunk mayor que 500 kB; no se modificaron el umbral ni la distribución del bundle.
La revisión de whitespace de los archivos de esta entrega pasó. El diff global
ya tenía una línea final extra en `productivity-workflows/spec.md`, ajena al cambio.

## Correcciones durante la revisión

Se corrigieron el seguimiento del bloque al cruzar hacia el control lateral, el
foco inmediato tras comandos/inserciones y la acotación vertical de popovers con
selecciones fuera de pantalla. Los ensayos previos fallidos no se cuentan como
aceptación. También se corrigieron fixtures: tipo de Note inexistente, conteo de
opciones nativas, listeners repetidos de beforeunload, readiness de la UI y espera
del lateral responsive. El ensayo de arrastre final mantiene origen/destino visibles.

## Límites

No se repitieron suites históricas ni aceptación IPC/foco/cierre nativa o Windows.
El guardado/reapertura está acreditado por el puente y SQLite, no por una nueva
aceptación del transporte Tauri. El selector de vistas se verificó al abrir/cancelar;
la persistencia de referencias v3 conserva su implementación previa. Milestone
comparte el camino anterior inspeccionado; no tuvo un recorrido UI nuevo.
No se acreditan autoguardado, colaboración, descarga remota ni Meeting Notes.

## Seguimientos visuales del 26/10/2026

Por pedido del usuario se retiraron la ayuda inferior de atajos y Add paragraph
de Note; los otros contextos conservan su variante anterior. El diálogo compartido
ahora ordena Cancel/Discard/Save and continue, deja Discard en medio y el primario
al final/derecha, con 24px entre explicación y acciones e iconos close/Trash2/Save.
El catálogo suma dos SVG locales y conserva los 77 anteriores: 79 en total.

`npm run build` volvió a aprobar iconos, contratos, Svelte (0 errores/advertencias)
y bundle, con la advertencia de tamaño anterior. Se aprobaron **27 comprobaciones
del diálogo y 2 de los controles retirados**, en sesión headless propia sobre
CLI/SQLite sintético. Se verificaron orden DOM/visual/Tab, iconos a la izquierda,
espaciado, foco inicial en Cancel, guardar antes de navegar, descartar sin pisar
la revisión guardada, cancelar/Escape conservando buffer, ambos temas y 420px.
Cero errores de navegador/solicitudes externas. No se repitió el recorrido original
de 59 checks ni la aceptación nativa; esos resultados conservan su alcance anterior.

Evidencia ignorada en `output/playwright/unsaved-dialog-20261026/`, incluidos
`review.cjs`, resultados y capturas; `dark-dialog.png` muestra el diálogo final.

Seguimiento de hover: los controles laterales ahora siguen el bloque bajo el
puntero y se ocultan al salir, incluso con el editor enfocado. No se activan por
escritura ni por el espacio vacío dentro de la altura mínima. Se conservan cruce
al control, menú, foco explícito/atajo, arrastre y undo. Build aprobado y **25
comprobaciones headless** adicionales, incluidos ambos temas, Note vacía y entrada
desde el título, sin errores de navegador ni solicitudes externas. Evidencia
ignorada en `output/playwright/hover-controls-20261026/`; no cambia el alcance
histórico de las revisiones anteriores ni acredita aceptación nativa nueva.

## Hover extendido a los márgenes — 26/10/2026

El seguimiento posterior reemplaza el límite horizontal del bloque por los
márgenes visibles de su propio panel de Note. La coordenada horizontal se
proyecta sobre Tiptap; se conserva la altura real del bloque y se valida la
superficie bajo el puntero. Así puede entrarse directamente desde un margen,
cambiar de fila y alcanzar los controles sin extender el hover a huecos,
popovers, lateral u otros paneles.

`npm run build` aprobó iconos (81), contratos, Svelte/TypeScript con cero errores
y advertencias, y bundle; permanece la advertencia de Vite sobre tamaño de chunk.
La sesión Chromium headless propia aprobó **82 comprobaciones focalizadas** con
UI de producción → CLI Rust real → SQLite sintético: párrafos, encabezado, listas,
cita, checklist y código en ambos márgenes; entrada directa, padding superior e
inferior del código, cambio de fila, hueco vertical, salida hacia el lateral,
cruce a controles/menú, Escape/atajo, scroll y resize bajo puntero quieto. Temas
claro/oscuro a 1280 y 420 px conservaron el hover y ocultaron controles bajo la
última fila. No hubo estado pendiente, cambio de documento/IDs/revisión, errores
de navegador ni solicitudes externas.

Evidencia ignorada en `output/playwright/hover-row-20261026/`: `setup.cjs`,
`review.cjs`, `results.json`, salida completa y capturas por tema/ancho, incluida
`code-right-margin.png`. Los intentos anteriores corrigieron el fixture de código,
selectores del DOM y preparación de pestaña/lateral responsive; no se cuentan
como aceptación. No se repitieron las suites históricas ni la aceptación nativa.
El cambio sigue activo con sus deltas pendientes de sincronización/archivo.

## Continuar desde el espacio inferior — 26/10/2026

Un clic primario sin modificadores bajo el último bloque crea un párrafo y
coloca el cursor dentro; un párrafo final vacío se reutiliza. Incluye el vacío
dentro de la altura mínima y el fondo inferior de la página fuera del editor.
La pulsación debe empezar y terminar en contenedores vacíos de la propia Note,
con movimiento máximo de 5 px; no intercepta selección arrastrada, controles,+huecos entre bloques, clic secundario/modificado ni el estado deshabilitado.
Los listeners se retiran al desmontar la instancia. El guardado sigue explícito.

`npm run build` aprobó iconos/contratos, Svelte/TypeScript con cero errores y
advertencias, y bundle; conserva la advertencia de tamaño de Vite. La sesión
Chromium headless propia aprobó **70 comprobaciones principales y 4 adicionales**
sobre UI de producción → CLI Rust real → SQLite sintético:

- Finales de párrafo, código SQL, H2, lista y divisor: párrafo con ID propio,
  foco/escritura sin heredar tipo, clics repetidos, undo/redo, IDs/contenido
  previos exactos y guardado/reapertura.
- Note vacía y final vacío existente: foco sin acumular bloques ni cambiar
  documento/revisión guardada o estado pendiente sólo por enfocar.
- Clic secundario/modificado, arrastre desde texto o dentro del vacío, hueco
  intermedio, bloque existente y herramientas de página no insertan contenido.
  El editor deshabilitado durante un guardado demorado tampoco inserta.
- Claro/oscuro a 1280/420 px: crear, escribir y guardar después de código.
  El fondo de página fuera del editor permite continuar y un pointercancel
  impide la inserción. La revisión principal no registró errores de navegador
  ni solicitudes externas.

Evidencia ignorada en `output/playwright/append-paragraph-20261026/`: `setup.cjs`,
`review.cjs`, `lower-space.cjs`, resultados JSON/salidas completas y capturas por
tema/ancho. La preparación se ajustó para esperar el editor habilitado, comparar
el documento canónico del CLI, excluir widgets GapCursor del conteo y ubicar el
clic vacío fuera de los controles iniciales de una Note vacía. Los intentos
previos no se cuentan como aceptación. No se repitieron suites históricas ni
aceptación nativa; `refine-notion-note-editor` queda 8/8, pendiente de sync/archivo.

## Acciones agrupadas en la cabecera — 26/10/2026

El botón `…` reemplaza los tres iconos independientes y Archive de Note normal.
El menú muestra Export, External resources, Attachments y Archive/Restore con
icono/texto; conserva las condiciones de disponibilidad y explica Export cuando
requiere guardar. Los paneles anteriores siguen montados y sus campos se
conservan. Flechas/Home/End recorren acciones habilitadas, Enter abre el panel,
Escape retorna a `…` y el clic exterior cierra. Archive/Restore usa el handler,
guardia del borrador y revisión esperada existentes. Footer y Archive de Task
conservan su variante previa.

`npm run build` aprobó contratos, catálogo de **83 SVG locales** (se agregaron
Ellipsis y Undo), Svelte/TypeScript con cero errores/advertencias y bundle, con la
advertencia de tamaño de Vite anterior. La sesión Chromium headless propia
aprobó **43 comprobaciones principales y 6 de compatibilidad**, usando UI de
producción → CLI Rust real → SQLite sintético:

- Un solo disparador, cuatro filas con nombres/iconos, teclado, foco y cierre.
- Exportación JSON real dentro del corpus sintético, destino conservado al
  reabrir, apertura de recursos/adjuntos sin acciones por abrir/cerrar y Note
  intacta por el uso del menú. Cero errores de navegador/solicitudes externas.
- Export deshabilitado con motivo visible y omisión por teclado; Cancel conserva
  borrador; Discard archiva la versión guardada; Save and continue guarda antes
  de archivar. Restore conserva documento y cada operación respeta su revisión.
- Menú/panel/foco dentro del viewport claro/oscuro a 1280 y 420 px; revisión de
  capturas. Note nueva no ofrece Archive y conserva requisitos de guardado;
  Task mantiene Export al pie y Archive en su cabecera.

Evidencia ignorada en `output/playwright/note-actions-20261026/`: `setup.cjs`,
`review.cjs`, `compatibility.cjs`, JSON/salidas y cuatro capturas del menú. La
preparación final usa título/destino de exportación únicos y espera la respuesta
real de archivo/restauración antes de consultar SQLite; los intentos previos
fallidos por esas condiciones del harness no se cuentan como aceptación.
No se repitieron suites históricas ni aceptación nativa. El cambio queda 9/9,
con deltas pendientes de sincronización/archivo.
