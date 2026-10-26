# ADR 0047: barra integrada por plataforma y lateral de sesión

- Estado: adoptado en código; aceptación nativa por plataforma pendiente
- Fecha: 24/10/2026

## Contexto

El usuario pidió trasladar el lateral y las pestañas a la franja antes ocupada
por el título Noteka, junto a los controles del sistema Mac, con equivalente
Windows y adaptación web. Autorizó dejar Windows implementado hasta disponer
de una máquina real. La autoridad de cierre agregada ya pertenece a App.

## Decisión

WindowTitlebar ocupa una sola fila de 44 px sobre la shell y recibe las mismas
pestañas como snippet. WorkspacePane continúa montado bajo el mismo ID; plegar
el lateral sólo cambia presentación. El panel estrecho y la preferencia de
plegado de escritorio usan estados separados de sesión, sin persistirlos.

Mac usa Overlay, hiddenTitle y decorations=true, con trafficLightPosition
explícita y reserva de espacio. Conserva los tres controles del sistema.
Windows usa decorations=false, APIs de minimizar/maximizar/consultar estado y
un cierre que llama a leaveWorkspace(true). El cierre nativo de Mac, los atajos
y la protección Rust Pending permanecen en su autoridad existente.

La selección de presentación depende de isTauri y navigator.platform del
WebView; web omite la barra nativa incluso sobre Mac/Windows. Esta detección
no autoriza operaciones. Tauri limita los permisos individuales a la ventana
main. No se añaden permisos de filesystem, shell ni conexiones remotas.
El estado maximizado se actualiza por resize, con una lectura activa y una
pendiente coalescida; al desmontar se invalidan resultados y listeners.

Los archivos de plataforma repiten label/geometría completos porque los arrays
se reemplazan al combinar configuración. Las tres definiciones mantienen las
mismas dimensiones y límites; el arte y los sidecars Mac existentes se conservan.
Los espacios libres usan data-tauri-drag-region; sus hijos interactivos no.

## Alternativas consideradas

Mantener las dos filas no aprovecha la franja solicitada. Quitar decoración
Mac requeriría sustituir controles y perder comportamiento nativo. Construir
una barra AppKit/Win32 introduce código específico y dependencias innecesarios
para este ajuste. Se usa la integración existente de Tauri.

## Consecuencias

No hay cambios de SQLite, contratos, documentos, relaciones o modelos.
Las configuraciones nuevas requieren recrear la ventana; no se fuerza el cierre
de una sesión del usuario. Windows real, comportamiento de botones/arrastre
Mac, fullscreen, escalado y Snap requieren aceptación propia. La inspección
Chromium sólo acredita composición y flujo de presentación, no controles del SO.
Meeting Notes permanece propuesto 0/19.

Ver [guía](../window-titlebar.md) y [revisión](../verification-window-titlebar.md).
