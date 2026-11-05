# Verificación: ASCII junto a Side peek

Fecha: 27/10/2026. Cambio: `animate-ascii-beside-side-peek`.

Home mantiene el reloj de su portada activo cuando comparte pantalla con un Side
peek visible, aunque el foco esté en el detalle. La señal de foco permanece
separada para el manejo de atajos y la edición; la portada conserva su canvas y
su escena durante la transición y el redimensionamiento.

## Comprobaciones

- `npm run build`: Svelte sin errores ni advertencias; Vite compiló con su aviso
  habitual sobre tamaño de bundle.
- `node --experimental-strip-types --test tests/ascii-ocean.mjs tests/ascii-cover-scenes.mjs`:
  12 pruebas aprobadas.
- `node scripts/headless-ascii-side-peek-smoke.mjs` y la misma ejecución con
  `NOTEKA_TEST_BROWSER=webkit`: 18 comprobaciones aprobadas en cada navegador,
  con workspace sintético y sin solicitudes externas. Se comprobó avance con
  nota enfocada, conservación del canvas y borrador sin guardar, atajo de Home,
  resize, portada fuera de vista, pestaña inactiva, modos Center y Full y
  movimiento reducido. La ocultación del documento se simuló con un evento de
  visibilidad.

La ejecución en Chromium/WebKit es headless y no sustituye una aceptación visual
ni una medición de fluidez en la ventana Tauri nativa o en hardware de 120 Hz.
