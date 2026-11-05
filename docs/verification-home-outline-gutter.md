# Verificación del margen del indicador de Home

El 27/10/2026 se redujo de 68 a 36 px la reserva final del contenido de Home, recuperando 32 px en el ancho útil cuando el contenido no está limitado por su ancho máximo. El rail de 24 px, scrollbar y borde ajustable conservan sus dimensiones; el detalle de Note mantiene su margen anterior. Para un panel de Home de hasta 300 px se adaptan cabeceras y acciones al contenedor.

`npm run build` pasó con cero errores y advertencias de Svelte; se mantiene el aviso previo de chunk superior a 500 kB. Una revisión en Chromium y WebKit headless, con UI compilada y workspace sintético, aprobó 42 comprobaciones por motor: geometría del tablero y del rail con Side peek, separación y ausencia de overflow global, clic y teclado del indicador, resize con teclado y puntero, conservación de borrador y documento, ambos temas y ventana de 420 px. Se revisaron capturas sintéticas a ancho normal y estrecho.

En el primer control estrecho se detectó que las acciones del tablero quedaban apretadas; se adaptaron al ancho real del panel y se comprobó que el botón es alcanzable. Las pruebas no acreditan interacción física ni composición de la ventana nativa de macOS/Windows/Linux.
