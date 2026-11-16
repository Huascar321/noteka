# Proposal

## Why

El usuario percibe tirones en transiciones, animación y scroll, y distingue explícitamente rendimiento de velocidad configurada. El perfil nativo detecta trabajo repetido de Canvas2D y recomposición; el código de overlays lee geometría por frame y el scroll instala un listener global bloqueante.

## What Changes

- Preparar la geometría de recorte de los overlays al abrir y dejar la reproducción a Web Animations, conservando 400/200 ms, contenido progresivo y salida interrumpida.
- Mantener la clasificación de rueda pasiva globalmente y limitar la cancelación a un scroller con una rueda discreta detectada.
- Acotar invalidaciones de diagnóstico del canvas y aislar su pintura/composición sin bajar velocidades ni apagar la portada detrás de popups.
- Evitar reconstruir los buffers de la portada en cada frame al abrir Side peek; mover el panel con transform y habilitar el documento sin esperar metadatos secundarios. Reutilizar sólo kind/title de la fila para evitar una lectura duplicada del original.
- Medir antes/después con fixtures, registrar el perfil nativo y verificar comportamiento, apariencia y límites.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: planificación de animación/render y scroll sin trabajo global bloqueante innecesario.

## Impact

Presentación TypeScript/Svelte/CSS y pruebas focales. Sin cambios Rust/SQLite, datos, operaciones ni dependencias. Se conservan los cambios locales existentes; el commit de vidrio aceptado es `3115aeb`.
