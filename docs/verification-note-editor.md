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
