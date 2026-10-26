# Proposal

## Why

Los videos locales `shark_day.mp4` y `shark_night.mp4` muestran cuerpos que se
curvan durante el giro, colas que siguen esa curva y sombreado ASCII con volumen.
La portada vigente gira mallas casi rígidas con desplazamiento lateral de cola;
la referencia nocturna también necesita una paleta propia más fiel.

## What Changes

- Curvatura continua de cuerpo y cola ligada al giro, con ondulación que aumenta
  hacia la cola, aletas que acompañan la postura y cabeza estable.
- Recorridos suaves con impulso/deslizamiento y profundidad/orden coherentes.
- Celdas y glifos más finos, tonos gris azulado para tiburones y oliva/dorados
  para otros peces, con luces y sombra adaptadas a ambos temas.
- Fondo de grilla y bancos pequeños con desplazamiento local discreto.
- Mantener composición horizontal de Home, cuatro animales, recursos locales,
  preferencia ASCII/Foto, movimiento reducido y todas las pausas existentes.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: precisar nado articulado y aspecto diurno/nocturno de la
  portada accesible, conservando requisitos y escenarios anteriores.

## Impact

Sólo geometría/proyección y renderer de presentación TypeScript, documentación y
delta de la portada. Sin dependencias, Rust, contratos, migraciones, operaciones
del workspace ni cambios en otras pantallas. Los videos guían la apariencia;
no se incorporan al producto, ni se copia su interfaz. La comparación visual y
la revisión de pausas se hacen en navegador headless con el componente real.
La aprobación de una pantalla física de 120 Hz queda fuera de esta entrega.
