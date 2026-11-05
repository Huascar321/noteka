# Verificación de barras de desplazamiento

26/10/2026 (histórico): comprobación de estilos computados en Chromium y WebKit, en temas claro y oscuro, con scroll vertical y horizontal; canal 8 px, pista transparente, tirador visible y barra de ScrollOutline oculta. Svelte check sin errores ni advertencias. No se hizo aceptación visual de las barras nativas en macOS/Windows/Linux; el sistema puede usar barras superpuestas.

27/10/2026: el canal vertical se amplió de 8 a 9 px y el horizontal conserva 8 px. Compilación aprobada; revisión de estilos computados en Chromium y WebKit, temas claro/oscuro, desplazamiento en ambos ejes y ocultación de ScrollOutline. Las barras superpuestas y preferencias del sistema pueden afectar la apariencia nativa, no inspeccionada en esta ocasión.

27/10/2026, separación: el canal vertical aumenta a 10 px para reservar un borde transparente derecho de 2 px, manteniendo 7 px de tinta; el horizontal sigue en 8 px. Build aprobado y controles de CSSOM, temas, scroll y ocultación pasaron en Chromium y WebKit headless. La captura headless no mostró la barra superpuesta, por lo que el resultado visual en la ventana nativa sigue pendiente.
