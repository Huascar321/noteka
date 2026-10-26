# Revisión acotada de la barra superior

Fecha: 24/10/2026. Cambio `integrate-native-window-titlebar`.

## Alcance

Implementación de barra única, lateral plegable, overlay Mac, controles Windows
y variante web. Sin nuevas operaciones Rust, contratos, migraciones o edición
de datos locales. Meeting Notes conserva sus 19 tareas pendientes.

## Compilación y configuración

- `npm run build`: aprobado; 0 errores y 0 advertencias Svelte. Permanece el
  aviso previo de Vite por bundle mayor a 500 kB.
- `cargo check --locked --offline -p noteka-desktop` y
  `cargo build --locked --offline -p noteka-desktop`: aprobados en macOS.
- Configuraciones completas de Mac/Windows combinadas con la base y validadas
  contra el schema instalado de Tauri 2.12.2: ambas válidas; geometría y label
  conservados. Ajv requirió unicodeRegExp=false por una expresión de nombres de
  archivo del schema que JavaScript rechaza con bandera Unicode; no se modificó
  el schema ni se omitió la validación de propiedades.
- Un selector del recorrido headless existente se actualizó al nuevo nombre
  accesible. `node --check` comprueba sintaxis, sin ejecutar ese recorrido.

## Inspección manual

UI compilada → Chromium → puente CLI → copia restaurada del workspace.
El puente admite workspace.open y operaciones terminadas en get/list/query/read;
rechaza mutaciones. No se guardó el borrador usado para observar la presentación.

- Barra a y=0 y altura exterior de 44 px. En la composición Mac existe reserva
  para los botones reales, sin dibujar imitaciones. En Windows hay tres controles
  de 46 px al extremo derecho, fuera de la zona de arrastre.
- Plegar en 1280 px amplió el panel de 1030 a 1280 px y conservó exactamente el
  mismo nodo/ID. Navegación oculta inert=true/aria-hidden=true, sin overflow global.
- Nueva pestaña abrió el buscador existente y Notes como segundo panel. Un título
  de borrador permaneció en el mismo input/panel al plegar/expandir y al alternar
  Home/segunda pestaña; dos paneles montados, uno activo, indicador pendiente.
- Cerrar esa pestaña mostró Guardar y continuar/Descartar/Cancelar. Cancelar
  conservó el título; Descartar retiró sólo ese borrador, dejando Home. Sin guardar.
- A 420 px el panel superpuesto comenzó bajo la barra (y=44), con 240 px de ancho.
  Escape devolvió el foco al control y dejó la navegación inert; viewport/documento
  420/420. Volver a 1280 conservó el borrador y recuperó la disposición de escritorio.
- Composiciones claras/oscuras revisadas; Windows también a su mínimo de 760 px,
  documento/viewport 760/760. Sólo se observó la barra HTML y su consulta simulada
  de maximización: no se ejecutaron minimizar, maximizar, arrastre o cierre nativos.
- Una pestaña nueva sin mocks mostró platform=web, isTauri=false, cero controles
  de ventana y cero reserva Mac. Bienvenida web sin backend de escritorio.
- Sin errores JavaScript ni solicitudes externas observadas durante la carga
  inicial de la copia. Consola final: 0 mensajes, 0 errores y 0 advertencias.

Un intento de recaptura esperó Expandir después de volver a escritorio, donde
el lateral ya estaba expandido: timeout del controlador, corregido leyendo el
estado actual y sin modificar producto ni atribuirle un fallo. Se conserva el log.

## Límites

No se añadieron ni ejecutaron suites. No se usó pantalla ni IPC de la aplicación
nativa del usuario, ni se la cerró manualmente. No hay aceptación de arrastre,
tráfico de eventos, fullscreen o cierre real de Mac/Windows. Las composiciones
Mac/Windows en Chromium no son una ejecución de esos sistemas. Windows real
queda pendiente explícitamente por pedido del usuario. Web conserva su límite
actual de almacenamiento; esta entrega sólo adapta la barra.

## Preservación y cierre

Dos requisitos añadidos contrastados y sincronizados en desktop-shell: 56 → 58.
Los 56 bloques anteriores y su Purpose/prefijo se conservaron byte por byte,
igual que las otras 16 specs y los ocho archivos de planificación de Meeting Notes.
Contratos, API de presentación anterior, host Rust y prefijo CSS preservados.
Cambio archivado 3/3 como `2026-10-24-integrate-native-window-titlebar`;
validación documental estricta final aprobada: 18/18 (17 specs y Meeting Notes).
Las tareas de Meeting Notes permanecen 0/19. La evidencia técnica se guarda fuera de
Git; no se publica contenido personal ni se hace commit/push.
