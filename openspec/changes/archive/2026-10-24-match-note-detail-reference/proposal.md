# Proposal

## Why

El usuario pidió llevar al detalle de Notes la composición de su Notion, igual que en Tasks. La nota de referencia muestra propiedades compactas y el cuerpo inmediatamente debajo; Noteka mezcla propiedades de origen de sólo lectura con campos locales separados y no ofrece entrada visible cuando el cuerpo está vacío.

## What Changes

- Composición de Note normal con icono/título, Tag/URL/Favorite/Type, relaciones Project/People y editor visible.
- Tipo/favorito editables desde el payload local; tags/proyectos/personas reales navegan mediante guardias compartidas. URL importada se muestra como lectura, sin añadir un campo persistente ficticio.
- Inbox, procedencia y snapshot completo de propiedades en View details; prefijo importado intacto al editar.
- Enter desde título, placeholder y estructuras iniciales opcionales para Note vacía, conservando el editor, guardado explícito y defaults de Task/Goal/Milestone.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: detalle Note con propiedades locales y snapshot de origen separados.
- `block-editor`: inicio visible y estructuras de Note sólo sobre un párrafo vacío.

## Impact

Interfaz Svelte y CSS, guías y dos deltas. Sin cambios Rust/contratos/SQLite/documentos/dependencias ni recuperación de datos. Captura Notion privada con Playwright y revisión manual del borrador sobre copia restaurada; sin suites, escrituras personales ni aceptación IPC nativa.

## Non-goals

Sin autoguardado, edición del snapshot URL, comentarios, Content, ASR/Meeting Notes, nuevos tipos de bloque, sincronización de Notion, publicación o cambios de biblioteca/Home. Meeting Notes permanece propuesto 0/19.
