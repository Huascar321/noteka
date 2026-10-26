# Revisión del popup Settings

Fecha: 25/10/2026. Cambio `add-sidebar-settings-popup`.

## Alcance final

Se implementó la petición inicial de agrupar Tema/ruta/Cambiar workspace y
retirar texto/punto azul. Durante la implementación el usuario añadió una
referencia de Notion para un panel que admita más ajustes. El mismo cambio se
actualizó con navegación Personal/Preferencias y Workspace/General, búsqueda
local con vacío/limpieza y composición amplia adaptable al ancho estrecho.
Sólo se muestran los tres controles existentes; no se copian cuentas, pagos,
notificaciones o capacidades futuras de la referencia.

App sigue siendo dueño del tema/preferencia y del cambio de workspace;
SettingsDialog presenta props/callbacks. `leaveWorkspace` cierra el popup antes
de preparar todas las pestañas. Se conserva la operación original y sus guardias.
No hay Rust, contrato, dependencia, migración, datos personales o proveedor nuevos.

## Compilación y catálogo

`npm run build` aprobado: iconos/contratos vigentes, Svelte cero errores y
advertencias; bundle `index-D0Du4Vci.js`, CSS `index-CCzCPSs_.css`. Permanece el
aviso anterior de bundle mayor a 500 kB.

Se añade un engranaje SVG estático Lucide/Settings del paquete instalado
AnimateIcons 0.1.0: 77 iconos, nueve definiciones laterales diferidas sin cambios.
Los 76 iconos anteriores y sus licencias se contrastan con el baseline.

Diez scripts UI existentes adaptan sólo su acceso a Tema/Cambiar workspace al
nuevo Settings y a General. Su sintaxis se comprobó con `node --check`; no se
crearon suites nuevas ni se ejecutaron las suites históricas o el host nativo.

## Revisión del producto compilado

Playwright/Chromium headless operó App real compilado mediante el puente de
revisión existente, CLI Rust y SQLite sintéticos. No se abrió el workspace
personal ni se usó pantalla del usuario. El transporte IPC/evento de cierre está
simulado; no constituye aceptación de ventana Tauri.

Treinta y tres comprobaciones aprobadas, registradas en `review.json`:

- Botón Settings/engranaje, pie limpio y ausencia del punto/texto solicitados.
- Navegación/filtro, coincidencia con mayúsculas/acentos, vacío y limpieza.
- Foco con Tab/Shift+Tab, Escape/restauración y atajos de fondo bloqueados.
- Tema aplicado/persistido, fallo de almacenamiento recuperable y reintento.
- Buffers activos/inactivos conservados al cambiar tema y entidades/documentos
  iguales en CLI antes/después de Settings, tema y descartes de revisión.
- Cambio de workspace considera ambos buffers y cierra Settings antes de la
  guardia. Cancelar o simular fallo de guardado conserva el contenido. El evento
  simulado de cierre del host mantiene la misma protección.
- Cambio confirmado vuelve al selector de workspace y conserva preferencia;
  reapertura funcional, ambos temas y 420 px sin desbordamiento de ruta/popup.

En 420 px el popup ocupa x=12…408 y el documento mide 420 px. Capturas de
Preferencias, General y lateral revisadas; cero errores de navegador y cero
solicitudes externas durante el recorrido.

El primer diseño simple dejó que Tab alcanzara el foco del navegador; el panel
final añade ciclo explícito además de dialog nativo y pasó ambos sentidos. Otra
revisión intermedia buscó «Guardar» en vez del botón existente «Guardar y
continuar»; se corrigió sólo el script de revisión y se repitió el recorrido.
Los logs intermedios se conservan, no se presentan como aprobados.

## Conservación y cierre

Baseline de 1.081 archivos y evidencia privada/ignorada en
`output/playwright/settings-popup/`. Se contrastan núcleo, contratos, locks,
portada/animación y archivos ajenos, además de dieciséis specs ajenas y ocho
artefactos Meeting Notes propuesto 0/19. Las diez adaptaciones UI se revierten
en memoria al texto original exacto del baseline al quitar sólo los nuevos
pasos Settings/General; los 76 iconos existentes también coinciden íntegros.

Se añade un único requisito desktop-shell con cinco escenarios; los 67 requisitos
anteriores se conservan completos, 68 en total. Archivado **3/3** como
[`2026-10-25-add-sidebar-settings-popup`](../openspec/changes/archive/2026-10-25-add-sidebar-settings-popup/proposal.md).
Validación estricta final 18/18. Sólo se cierran navegador y servidor propios de
la revisión; las capturas y evidencia permanecen como archivos locales.
