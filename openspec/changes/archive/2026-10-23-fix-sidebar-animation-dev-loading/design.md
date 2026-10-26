# Design

## Context

La app nativa usa el Vite ya abierto en 127.0.0.1:1420. En esa sesión el import
`@animateicons/vanilla` se reescribe a un recurso optimizado que devuelve 504,
también después de recargar. La página aislada con la misma acción produce cero
frames activos/transformación en Chromium y WebKit; reduced motion es false.
El build de producción anterior carga correctamente, por lo que esa evidencia no
cubría el servidor de desarrollo usado por el usuario.

## Goals / Non-Goals

**Goals:** recuperar carga diferida en desarrollo y acreditar movimiento visible.
**Non-Goals:** cambiar datos, SVG, duración, accesibilidad, CSP, API o proveedor.

## Decisions

1. `optimizeDeps.exclude: ['@animateicons/vanilla']`. El paquete es ESM
   autocontenido, sin dependencias CommonJS. Vite lo sirve directamente en dev;
   el build conserva su chunk diferido. No se fuerza optimización global ni se
   borra el caché de otras dependencias.
2. Una carga decorativa fallida conserva la navegación y emite diagnóstico de
   consola. No se reintenta en bucle ni se recarga la página automáticamente.
3. La revisión observa matrices/geometría de SVG durante rAF y compara reposo;
   no confunde data-icon-playing con movimiento efectivo. Revisión en Chromium
   y WebKit con la misma acción/import del producto; nativo sólo con autorización.
4. No cambia propiedad de datos ni formatos persistentes. SQLite, adjuntos y
   Rust no intervienen. No hay migración ni nuevas dependencias. Revertir los dos
   cambios de frontend/configuración permite volver al estado anterior.

## Risks / Trade-offs

- La actualización de configuración reinicia Vite/recarga frontend. Se conserva
  el proceso Tauri y no se envían escrituras de entidades.
- Servir ESM directamente puede implicar más recursos JS locales en desarrollo;
  sólo ocho definiciones seleccionadas y el player se piden al clic. Producción
  conserva el mismo empaquetado diferido.
- La automatización headless no demuestra transporte IPC nativo ni cierre de
  ventana. Se documenta cada ámbito sin extender las conclusiones.
