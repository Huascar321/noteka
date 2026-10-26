# ADR 0053: profundidad compartida entre animales ASCII

Fecha: 26/10/2026. Estado: experimento local implementado.

## Context

El usuario pidió guardar todo el avance antes de probar tiburones, koi y medusas
juntos con tamaños/contraste/profundidad variados. El checkpoint local `2d43a66`
preserva la alternancia anterior y los demás avances. La prueba vive en
`codex/combined-ascii-ocean`, sin publicación.

## Decision

Una lista pura de poses por individuo comparte profundidad entre familias. Se
dibuja del fondo al frente, con escala y opacidad ligadas a profundidad, sin
blur ni ruido aleatorio por frame. La población se reduce de 11 a nueve animales
en ancho estrecho y conserva las rutas/profundidad de los individuos compartidos.

El renderer del océano expone ambiente y dibujo por actor, manteniendo API y
geometría anteriores. Koi/medusas reutilizan atlas y una superficie de señal con
lectura acotada a la región de cada individuo. El compositor pinta directamente
en el canvas final, con repintado sincrónico al resize y el reloj/pausas existentes.
El selector temporal anterior queda como referencia fuera de la ruta activa.

## Alternatives

- Superponer tres escenas completas: no permite intercalar familias al cruzarse
  y acumula densidad/coste innecesarios.
- Sustituir el océano por geometría nueva: arriesga el nado que se pidió conservar.
- Esconder una especie entera en el fondo: pierde variedad de profundidad dentro
  de cada familia.

## Consequences

La comparación visual requiere ambos temas, varios tiempos y ancho estrecho,
además de medir coste y revisar el orden real de pintura. No es un simulador de
colisiones. El checkpoint conserva la composición anterior; recuperar código no
requiere migraciones, acceso remoto o tocar SQLite. Photo/ASCII y las referencias
atribuidas permanecen. La aceptación técnica headless se documenta separada de
la apariencia en una pantalla física de 120Hz.
