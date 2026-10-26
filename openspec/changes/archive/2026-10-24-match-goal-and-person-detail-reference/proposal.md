# Proposal

## Why

El usuario pidió continuar la adaptación visual de detalles con Goals y People. Se capturaron e inspeccionaron páginas reales de Notion y se acreditaron sus identidades locales. Goal conserva un documento Note; Person sólo admite sus propiedades y relaciones. El alcance explícito exige conservar ambos comportamientos sin ampliar cuerpo libre a entidades core.

## What Changes

- Goal con cabecera compacta Status/Tag/Progress canónico, fechas secundarias en View details, Nav y hitos en tabla con ambas fechas.
- Mantener el mismo editor/documento de Goal y los vínculos originales a proyectos; no interpretar ni eliminar bloques de plantilla para aparentar paridad.
- Person con foto local como portada, icono/título, campos editables en View details y secciones de notas, proyectos y tareas del contexto actual.
- Conservar captura Note/Task, relaciones, condiciones de herramientas, guardias, cursores, errores y teclado. Sin copiar contadores/relaciones remotos ausentes.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: presentación de detalles Goal y Person sobre los modelos y operaciones existentes.

## Impact

Svelte/CSS y documentación. home.query/goals_activity, milestones_all, workflow/context y CRUD/CAS existentes. Sin Rust, contratos, SQLite, dependencias o datos personales modificados. Referencias sólo lectura; revisión UI sobre copia restaurada con escrituras bloqueadas.

## Non-goals

Sin editor/documento de Person ni ampliación del de Goal, migración, jerarquía nueva, recuperación de campos privados, copia de Quick Notes/Gift Ideas, sincronización de Notion, nuevas acciones de dominio o Meeting Notes. Meeting Notes permanece propuesto 0/19. Sin pantalla nativa, publicación, commit/push ni operación sobre Eco Armario o voicebot.
