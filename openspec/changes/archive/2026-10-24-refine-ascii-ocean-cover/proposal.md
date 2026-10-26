# Proposal

## Why

El usuario pidió mejorar silueta, nado y estabilidad de los tiburones ASCII, elevar el límite de24 a120 FPS y añadir detalle ambiental. El render actual proyecta la malla con trigonometría por punto, crea tres buffers cada cuadro y aplica ruido por celda. Subir sólo el límite multiplicaría trabajo sin resolver legibilidad.

## What Changes

- Afinar cuerpo, morro, aletas, cola, ojos y branquias; nado con ondulación progresiva, impulso/deslizamiento y giros coordinados.
- Proyección preparada por pose, buffers reutilizados y atlas de caracteres local; estabilizar sombreado/selección sin ruido de celda ni estelas.
- Cadencia requestAnimationFrame hasta120 FPS, velocidad basada en tiempo y conservación de pausas/movimiento reducido.
- Ambiente sutil con partículas, luz submarina y pequeños bancos de peces ASCII; misma portada y controles existentes.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: portada ASCII local más legible, ambiental y con cadencia máxima120.

## Impact

Sólo presentación Svelte/TypeScript y documentación; Canvas2D sin dependencias ni recursos remotos nuevos. Se conserva HomePanel, preferencia ASCII/Foto, datos y funciones. Validación de geometría/reloj, compilación y revisión visual/rendimiento de Canvas con Playwright, sin abrir el workspace personal.120 FPS es límite de programación, sujeto a pantalla y presupuesto real; no se presenta una simulación como aceptación de pantalla120Hz.

## Non-goals

Sin cambios Rust, contratos, SQLite, edición de contenido, fondos descargados, WebGL/dependencias, controles de calidad nuevos o funciones de Meeting Notes. No operar otras apps/repos ni reiniciar/capturar la ventana nativa. Sin commit/push/publicación.
