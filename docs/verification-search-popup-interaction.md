# Verificación de interacción del popup de búsqueda

02/11/2026. La búsqueda abierta ya no detiene la portada ASCII de Home y el
diálogo se cierra al hacer clic en el backdrop. El botón Close y Escape siguen
funcionando.

Verificación headless en Chromium y WebKit (1440x900):

- Animación: con el diálogo abierto, el canvas pasó de `data-frame` 24 a 66 en
  Chromium y de 18 a 60 en WebKit.
- Clic dentro: hacer clic en el campo de búsqueda mantiene el diálogo abierto.
- Clic fuera: un clic en el backdrop en `(10,10)` elimina el diálogo.
- Escape: cierra y elimina el diálogo.
- Búsqueda: escribir `home` devuelve la sección Home.
- Sin errores de página.

La ventana nativa se revisó: Cmd+K abre el diálogo y un clic fuera, en la barra
lateral, lo cierra; la captura posterior conserva la portada ASCII. La
animación con el diálogo abierto y el cierre por backdrop se midieron en
Chromium y WebKit headless. Build y OpenSpec `--strict` pasan.

Límites: la animación se detiene con `prefers-reduced-motion: reduce`, como antes;
no se midió GPU. El clic sobre el borde de 1px del diálogo se considera externo.
