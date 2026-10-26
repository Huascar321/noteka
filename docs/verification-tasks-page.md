# Revisión: Tasks, breadcrumb y acciones de fila

Fecha: 24 de octubre de 2026. Cambio `match-tasks-page-reference`.

## Alcance y compilación

- CLI Rust: `cargo build --locked --offline -p noteka-cli`, completado.
- Contratos: `npm run contracts:generate`, completado; cambios acotados a dos
  vistas Home y el campo opcional project_end_date.
- Frontend: `npm run build`, cero errores y cero advertencias Svelte.
  Persiste el aviso Vite del bundle mayor de 500 kB.
- Bundle final: `index-D15J3qmw.js`, 764,76 kB. No cambian SQLite/payloads ni
  el número de operaciones. No se agregaron ni ejecutaron suites.
- Dieciséis selectores históricos de regreso a Home se adaptaron al breadcrumb;
  su sintaxis se revisó, pero esos recorridos no se ejecutaron ni se declara su
  compatibilidad completa con todos los ajustes anteriores.

## Inspección manual de la UI

UI compilada en Chromium headless → puente local de sólo lectura → CLI Rust real
→ copia del workspace. La copia se obtuvo mediante backup/restauración canónicos
fuera de Git. El puente del navegador rechaza mutaciones de entidad y recursos
externos; abrir una captura sólo prepara memoria del cliente.

Observaciones de este snapshot, no valores fijos de producto:

- Tasks inicia Today con diez filas; Load more desde su extremo derecho carga
  catorce y desaparece al completar la vista. Projects inicia con siete registros,
  grupos Doing/Ongoing y arte local; se observaron 24 imágenes en ambas secciones.
- Completed devuelve un proyecto y Deadlines queda vacío por falta de fechas
  locales de fin. Board presenta Planned/Doing/Ongoing/Done/On Hold. Su guardado
  por arrastre no se volvió a ejecutar en este cambio.
- Breadcrumb de proyecto: tres tramos, logo local y antecesores Home/Projects.
  El enlace Projects abrió su listado; Notes/Tags/People/Tasks y Home mostraron
  sus rutas correspondientes. No se copiaron nombres personales a fixtures.
- Notes carga diez y luego veinte filas al activar el extremo derecho de
  Load more. El ancho de sección, Load more y New page fue 1436 px; la captura
  New Note mantuvo 109,61 px y borde de 1 px.
- New page desde el extremo derecho abrió un borrador vacío de nota; Home mostró
  la guardia y Descartar volvió sin guardar. No se actualizaron entidades reales.
- Cero contadores técnicos al pie, cero aviso final Home y cero Siguiente Tasks.
  Cobertura accesible únicamente al abrir Opciones de vista.
- Claro/oscuro y 420 px inspeccionados. Tanto Tasks como el detalle tuvieron
  clientWidth/scrollWidth de 420/420 px; el breadcrumb largo conserva texto
  completo accesible y el título ajusta altura tras cambiar ancho.
- Inicio final sin errores JavaScript ni solicitudes externas; consola de la
  revisión sin errores/advertencias. No se activaron modelos ni proveedores.

Capturas y logs se conservan en el directorio de evidencia privado y en
`output/playwright/` ignorado. El primer intento del driver usó require no
admitido; otro selector incluyó un nav oculto. Ambos se corrigieron en el driver
sin cambiar las condiciones del producto. Las capturas finales esperan datos,
imágenes y transiciones; se conservan los intentos previos.

## Límites

No se ejecutaron suites, datos inválidos sintéticos, carreras, guardados ni
aceptación de teclado/IPC/cierre Tauri nuevos. La inspección de invalidación,
cursores, filtros, guardias y límites se apoyó en código existente y compilación;
los resultados históricos mantienen su alcance. El snapshot vacío de Deadlines
no acredita todas las combinaciones de fechas. Las vistas pendientes no aparentan
estar implementadas. No hay importación, sincronización ni actualización de
contenido personal en este ajuste.

## Preservación y cierre

Baseline previo de archivos afectados, manifiestos SHA-256 y copia de evidencia
fuera de Git. Se conservan los cambios locales anteriores. El delta desktop-shell
agrega cinco requisitos; la sincronización compara los 42 anteriores y las otras
16 specs. Sólo se publica en specs vigentes el comportamiento implementado y
revisado descrito arriba.
