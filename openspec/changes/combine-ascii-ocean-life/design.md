# Design

## Context

Ver propuesta. El checkpoint `2d43a66` conserva los tres renderizadores y la alternancia anterior. El compositor actual pinta una o dos escenas completas; eso impide intercalar individuos de distintas familias. El borde del lateral termina 3px fuera de un aside con overflow:auto.

## Goals / Non-Goals

**Goals:** escena única con profundidad entre especies, tamaños coherentes y coste acotado; lateral ajustable sin scroll horizontal.

**Non-Goals:** colisiones físicas, interacción con animales, fuentes/remotos, preferencias nuevas o cambios de contenido/Rust/SQLite. Se prueban cambios en navegador aislado, sin pantalla personal.

## Decisions

- Mantener reloj y ciclo de vida de AsciiShark. Una semilla por instancia configura el elenco y sus fases; no hay temporizador de alternancia ni sorteos por frame. El helper anterior de selección queda disponible como referencia, fuera de la ruta activa.
- Calcular poses puras y una lista ordenada de mayor a menor profundidad entre todas las familias. Profundidad controla escala y opacidad/contraste; near/far no se asigna a una especie entera. Población acotada y menor en 420px, con rutas repartidas por altura/ancho.
- Exponer dibujo ambiental y por individuo del renderer de tiburones, conservando su API anterior y geometría/nado. Koi/medusas comparten atlas y rasterizan solamente la región de cada actor en una superficie reutilizada; el compositor pinta cada individuo en el orden global. Evitar ordenar tres canvases completos porque dejaría especies enteras siempre delante.
- Conservar repintado sincrónico durante resize y pausas/reducido; limitar DPR a2. Medir el coste combinado, también en claro/oscuro, antes de aceptar el experimento.
- Separar el cuerpo scrolleable del menú y su borde: aside contiene un cuerpo con overflow-y:auto y el handle como hermano fijo al contorno interior. Así el cuerpo no tiene overflow-x y el borde tampoco se desplaza al hacer scroll vertical. Los controles se ajustan a su ancho mínimo, con foco visible y etiqueta truncada cuando hace falta.
- El cambio depende del delta anterior de escenas: al sincronizar, aplicar primero `add-random-ascii-cover-scenes` y después este. Se reemplaza Portada local y se retira el requisito Variantes ASCII aleatorias y locales agregado por el anterior, para que la spec final no exija también la alternancia.

## Risks / Trade-offs

- [Saturación visual] → tamaños/opacidades escalonados y menos individuos en ancho estrecho; revisión de capturas.
- [Coste de ordenar/dibujar más actores] → atlas/superficies reutilizados y lectura de regiones acotadas; medición separada de120Hz físicos.
- [Clipping del borde/foco] → hit area interior, teclado y prueba de arrastre tras corregir overflow.
- [Regresión del océano anterior] → checkpoint recuperable, API original conservada y pruebas de geometría/reloj.

## Migration Plan

Sin migración ni formato nuevo. La rama `codex/combined-ascii-ocean` parte del checkpoint local; se recuperan los archivos ASCII anteriores desde `2d43a66` si se rechaza la prueba. No se publica ni reinicia la aplicación nativa.
