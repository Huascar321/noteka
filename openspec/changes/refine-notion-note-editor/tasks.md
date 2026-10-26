# Tasks

## 1. Contextual editor

- [x] 1.1 Implementar controles contextuales de Note: selección/enlaces, slash filtrable/teclado, inserción/transformación y acciones/reordenación de bloques con IDs e historial; verificar compilación e interacción en documento sintético (`block-editor`).
- [x] 1.2 Activar la presentación sólo en Note normal y ajustar columna, tipografía, controles, temas y ancho estrecho; mover los tres grupos de herramientas a la cabecera con paneles y campos conservados; revisar capturas/foco sin cambiar Task/Goal/Milestone (`desktop-shell`).

## 2. Verification and documentation

- [x] 2.1 Completar revisión headless aislada: comandos, formato, enlaces, mover/copiar/quitar/undo, save/reopen, prefijo importado, error/conflicto/guardias, ambos temas/420 px; compilar UI y registrar evidencia y límites (`block-editor`, `desktop-shell`).
- [x] 2.2 Actualizar guía/plan con comportamiento verificado y contrastar deltas contra implementación; validar OpenSpec y dejar listo para sincronización/archivo sin tocar Meeting Notes.

## 3. Requested refinements

- [x] 3.1 Retirar ayuda inferior y Add paragraph de Note; reordenar el diálogo compartido, separar sus acciones e incorporar iconos locales. Verificar build y revisión headless de orden, teclado, ambos temas/420px y efectos de Cancel/Discard/Save and continue sobre SQLite sintético.
- [x] 3.2 Mostrar controles laterales sólo con hover del bloque, conservando acceso explícito por teclado y arrastre; verificar que foco/escritura no los activen, salida/espacio vacío los oculten y el cruce al control sea utilizable, en ambos temas y sobre Note vacía.
