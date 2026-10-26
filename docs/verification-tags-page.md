# Revisión: página Tags

Fecha: 24 de octubre de 2026. Cambio `match-tags-page-reference`.

## Compilación y alcance

- `npm run build` completado: iconos y contratos vigentes, cero errores y cero
  advertencias Svelte. Aviso Vite existente de bundle mayor de 500 kB.
- Bundle `index-rlHg44cX.js`, 782,51 kB; CSS `index-CXMyk2hv.css`, 83,49 kB.
- TagsPage/HomeTagSection/Table y conexión de WorkspacePane; sin cambios Rust,
  contrato, migración, payload, documento o relación. HomeReader y HomeTagRow
  permanecen idénticos al baseline, con la consulta/validación Rust existentes.
- Revisión de código: captura/navegación protegidas, cola limitada, cancelación,
  dispose y atajos activos con guardias de diálogo/composición/repetición. Table
  conserva su presentación durante actualización y paginación del lector.
- No se agregaron ni ejecutaron suites, ni una nueva aceptación nativa Tauri.

## Inspección manual

UI compilada en Chromium headless → puente sólo lectura → CLI real → copia
restaurada mediante backup canónico, fuera de Git. El cliente rechaza operaciones
mutantes y solicitudes externas; preparar/descartar una captura sólo cambia
memoria de UI. No se abrió la pantalla nativa del usuario.

Observaciones del snapshot, no cantidades fijas del producto:

- Breadcrumb Home → Tags, icono/título grande, Nav, New Tag, cinco pestañas y
  Recent inicial con diez filas. Últimas notas, tipos e imágenes locales visibles;
  sin cobertura técnica al pie. Inicio sin errores JS ni solicitudes externas.
- Table mostró diez filas con encabezados Tag/Last Note/Type y orden A-Z.
  Load more, activado en el extremo derecho, acumuló trece IDs únicos y retiró
  el botón al agotar el cursor. Actualizar mantuvo Table y volvió a la página
  inicial de diez. A-Z volvió a lista; Fav. mostró dos filas y Types presentó
  Area/Resource/Entity con conteos de la página, incluido el grupo vacío.
- Abrir un tag mostró el mismo registro y Home → Tags → título. Abrir su
  última nota mostró el título correspondiente y Home → Notes → título.
- New page desde el extremo derecho preparó título vacío y Home → Tags →
  Nuevo tag. Navegar al antecesor abrió Guardar y continuar/Descartar/Cancelar;
  Descartar volvió a Tags sin guardar. New page/Load more midieron 1498 px;
  New Tag, 101,81 px con borde de 1 px.
- Nav abrió Tasks, Notes y Projects con sus breadcrumbs. Tags en Notes mantuvo
  cuatro pestañas; Home conservó cuatro pestañas y Types seleccionado.
- Claro/oscuro y 420 px inspeccionados. Documento 420/420 px; sección y acciones
  textuales 386 px. La tabla de 660 px quedó dentro de su región de 386 px con scroll interno,
  sin overflow del documento. Las listas acomodaron títulos largos y metadata.
- Consola al terminar: cero mensajes, errores y advertencias.

Las primeras capturas de cambio de tema y tamaño incluyeron transiciones de CSS;
se repitieron esperando su final y capturando con animaciones desactivadas. El
primer build tenía una advertencia de tabIndex en región no interactiva: se retiró
el tabIndex innecesario y se recompiló. Table usa un icono válido del catálogo.
Logs/capturas quedan en `output/playwright/` ignorado y evidencia privada.

## Preservación y límites

Baseline privado de 32 archivos con SHA-256; cambios previos del checkout
conservados. CSS previo preservado como prefijo íntegro. Delta de dos requisitos
desktop-shell; el cierre contrasta los 51 bloques previos, Purpose y otras 16
specs antes de sincronizar únicamente el comportamiento implementado.

Las lecturas de error/obsolescencia y los atajos se revisaron en código y conservan
su evidencia histórica; no hubo inyección de fallos, carreras, guardados, suites
ni aceptación de IPC/teclado/cierre Tauri. Recent conserva su actividad importada
acotada; no se igualaron datos o fechas para reproducir el orden remoto.
