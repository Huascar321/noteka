# Proposal

## Why

Después de probar la composición mixta y reducir su densidad, el usuario prefiere las tres escenas que transicionan periódicamente. Se restaura esa portada desde `2d43a66` y se conserva la corrección del scrollbar horizontal desarrollada durante el experimento.

## What Changes

- Restaurar tiburones, koi y medusas como escenas separadas, con inicio/orden aleatorios,42s activos y fundido1,4s, exactamente como en el checkpoint.
- Retirar el módulo mixto y sus pruebas específicas de la ruta activa; el primer experimento permanece en el historial `fd4f436`.
- Conservar el cuerpo vertical separado del borde interior del lateral, sus anchos recordados y todas las mejoras de editor/peeks/indicador.
- Mantener reloj, pausas, Photo/ASCII, formato de preferencia y datos.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: lateral sin overflow horizontal, con scroll vertical y resize operativos. El ASCII vuelve al comportamiento del cambio `add-random-ascii-cover-scenes`; este delta no lo sustituye ni retira sus requisitos.

## Impact

Restauración acotada de los tres renderizadores y retiro del módulo mixto inactivo, pruebas/documentación coherentes y corrección del lateral preservada. Sin Rust, SQLite, contratos, dependencias o preferencias nuevas. Los pasos anteriores de tareas/evidencia registran el experimento realizado; el alcance final conserva sólo la corrección del lateral y el retorno explícito de la portada.
