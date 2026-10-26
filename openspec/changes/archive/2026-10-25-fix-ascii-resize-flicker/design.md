# Design

## Context

ResizeObserver se ejecuta después del RAF y antes del paint. Cambiar canvas.width/height limpia el bitmap; aplazar draw al RAF siguiente permite presentar esa limpieza en cada paso de la transición. measure además cancela el RAF y suspende el reloj en cada resize.

## Decisions

- El renderer conserva el último tiempo dibujado y repinta esa escena dentro de resize, antes de devolver control al navegador. No avanza tiempo ni presenta un estado vacío.
- measure sólo despierta el bucle si no hay RAF pendiente. Las pausas reales siguen usando wake/suspend.
- Atlas nuevo únicamente cuando cambia DPR/tema/color; capas e historial se reconstruyen como antes ante dimensiones distintas. No introducir debounce que estire una escena borrosa ni temporizadores ligados al lateral.

## Validation Plan

Reproducir con el bundle anterior, contar bitmaps transparentes al final de ResizeObserver y repetir sobre el bundle corregido en ambos temas, expansión/contracción/inversión, movimiento reducido y ancho estrecho. Compilar y ejecutar las pruebas existentes del reloj/geometría. Sin pantalla nativa ni workspace personal.
