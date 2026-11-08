# Tasks

## 1. Implementation

- [x] 1.1 Separar la animación de la portada del foco y del bloqueo de entrada;
  conservar pausas por visibilidad, documento oculto, movimiento reducido y Full
  page; retirar los props de excepción.
- [x] 1.2 Auditar Settings, Search, Center peek, Full page, guardia de cambios
  pendientes, diálogos de sección, menús, pickers y paneles plegables;
  documentar cuáles suspenden.
- [x] 1.3 Actualizar la guía ASCII, la guía de diseño, el plan y la evidencia;
  registrar la traza OpenSpec.

## 2. Verification

- [x] 2.1 `npm run build` y `npm run check` sin errores ni advertencias Svelte.
- [x] 2.2 `npm run test:headless-ascii-popups` en Chromium y WebKit con
  workspace sintético, sin red externa; comprobar Settings, Search, Side y
  Center peek animando y Full page suspendido.
- [x] 2.3 Validar el cambio con OpenSpec estricto y revisar que el delta no
  contradiga Search, Full page ni movimiento reducido.
